use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use gband_client::animation::Animations;
use gband_client::{Controls, Display, Step};
use gband_core::geometry::Size;
use gband_core::input::Key;
use gband_core::layout::{Direction, Layout, LayoutOptions, PaneId, SessionAction};
use gband_lua::keys::parse_key;
use gband_lua::{Config, ConfigError, DEFAULTS, LoadOptions, Locations};
use gband_protocol::{ClientMessage, ServerMessage};

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("gband-client-events-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn locations(&self) -> Locations {
        Locations {
            config: self.0.join("config"),
            plugins: Some(self.0.join("plugins")),
        }
    }

    fn plugin_file(&self, plugin: &str, relative: &str, source: &str) -> PathBuf {
        write(&self.0.join("plugins").join(plugin).join(relative), source)
    }

    fn load_with_budget(&self, source: &str, budget: u64) -> Result<Config, ConfigError> {
        write(&gband_lua::user_file(&self.0.join("config")), source);
        gband_lua::load(&self.locations(), &LoadOptions { budget })
    }

    fn load(&self, source: &str) -> Result<Config, ConfigError> {
        self.load_with_budget(source, gband_lua::BUDGET)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write(path: &Path, source: &str) -> PathBuf {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
    path.to_path_buf()
}

fn key(name: &str) -> Key {
    parse_key(name).unwrap()
}

fn layout_of(count: usize) -> (Layout, Vec<PaneId>) {
    let mut layout = Layout::new();
    let band = layout.bands()[0].id;
    let mut panes = Vec::new();
    for _ in 0..count {
        let pane = layout.allocate_pane();
        layout.open(
            pane,
            band,
            panes.last().copied(),
            None,
            &LayoutOptions::default(),
        );
        panes.push(pane);
    }
    (layout, panes)
}

fn shown(layout: &Layout) -> ServerMessage {
    ServerMessage::Layout {
        cols: 80,
        rows: 24,
        layout: layout.clone(),
    }
}

const RECORD: &str = "log = {}
local function record(name)
  return function(e)
    local keys = {}
    for k, v in pairs(e) do keys[#keys + 1] = k .. '=' .. tostring(v) end
    table.sort(keys)
    log[#log + 1] = name .. ' ' .. table.concat(keys, ',')
  end
end
for _, name in ipairs({ 'Attached', 'FocusChanged', 'BandChanged', 'PaneOpened', 'PaneClosed', 'LayoutChanged', 'TerminalResized', 'ConfigReloaded', 'KeyTableChanged' }) do
  gband.on(name, record(name))
end
";

struct Client {
    display: Display,
    controls: Controls,
}

impl Client {
    fn new(config: Config) -> Self {
        let mut display = Display::new(Size::new(80, 24), Animations::Off);
        let controls = Controls::new(config, &mut display);
        Self { display, controls }
    }

    fn receive(&mut self, messages: impl IntoIterator<Item = ServerMessage>) -> Vec<Step> {
        let received = self.controls.receive(&mut self.display, messages);
        assert_eq!(received.outcome, None);
        received.steps
    }

    fn press(&mut self, name: &str) -> Vec<Step> {
        self.controls.press(&mut self.display, key(name))
    }

    fn log(&self) -> Vec<String> {
        self.controls.runtime().lua().globals().get("log").unwrap()
    }

    fn clear(&self) {
        self.controls
            .runtime()
            .lua()
            .load("log = {}")
            .exec()
            .unwrap();
    }
}

fn recording(name: &str, extra: &str) -> (Scratch, Client) {
    let scratch = Scratch::new(name);
    let config = scratch.load(&format!("{RECORD}{extra}")).unwrap();
    (scratch, Client::new(config))
}

#[test]
fn focus_change() {
    let (_scratch, mut client) = recording(
        "focus",
        "gband.bind('alt+h', gband.action.focus_column_left)",
    );
    let (layout, panes) = layout_of(2);
    client.receive([shown(&layout), ServerMessage::Focus(panes[1])]);
    client.clear();
    client.press("alt+h");
    assert_eq!(client.log(), ["FocusChanged pane=1,previous=2"]);
}

#[test]
fn band_change() {
    let (_scratch, mut client) =
        recording("band", "gband.bind('alt+u', gband.action.focus_band_down)");
    let (layout, _) = layout_of(1);
    client.receive([shown(&layout)]);
    client.press("alt+u");
    let log = client.log();
    assert!(
        log.contains(&"BandChanged band=2,previous=1".to_owned()),
        "{log:?}"
    );
}

#[test]
fn pane_opened_and_closed() {
    let (_scratch, mut client) = recording("opened", "");
    let (one, _) = layout_of(1);
    let (two, panes) = layout_of(2);
    client.receive([shown(&one)]);
    client.receive([shown(&two)]);
    client.receive([shown(&one)]);
    let log: Vec<String> = client
        .log()
        .into_iter()
        .filter(|entry| entry.starts_with("Pane"))
        .collect();
    let second = panes[1].0;
    assert_eq!(
        log,
        [
            format!("PaneOpened band=1,pane={second}"),
            format!("PaneClosed band=1,pane={second}"),
        ]
    );
}

#[test]
fn layout_changed_follows_pane_events() {
    let (_scratch, mut client) = recording("layout-after-panes", "");
    let (one, _) = layout_of(1);
    let (two, panes) = layout_of(2);
    client.receive([shown(&one)]);
    client.receive([shown(&two)]);
    let log: Vec<String> = client
        .log()
        .into_iter()
        .filter(|entry| entry.starts_with("Pane") || entry.starts_with("Layout"))
        .collect();
    assert_eq!(
        log,
        [
            format!("PaneOpened band=1,pane={}", panes[1].0),
            "LayoutChanged ".to_owned(),
        ]
    );
}

#[test]
fn layout_change_without_a_pane_change() {
    let (_scratch, mut client) = recording("layout-change", "");
    let (mut layout, panes) = layout_of(2);
    client.receive([shown(&layout), ServerMessage::Focus(panes[1])]);
    client.clear();
    layout.apply(
        SessionAction::ConsumeOrExpel {
            pane: panes[1],
            direction: Direction::Left,
        },
        Size::new(80, 24),
        &LayoutOptions::default(),
    );
    client.receive([shown(&layout)]);
    assert_eq!(client.log(), ["LayoutChanged "]);
}

#[test]
fn heights_alone_are_no_layout_change() {
    let (_scratch, mut client) = recording("layout-heights", "");
    let (mut layout, panes) = layout_of(2);
    layout.apply(
        SessionAction::ConsumeOrExpel {
            pane: panes[1],
            direction: Direction::Left,
        },
        Size::new(80, 24),
        &LayoutOptions::default(),
    );
    client.receive([shown(&layout)]);
    client.clear();
    layout.apply(
        SessionAction::StepHeight {
            pane: panes[0],
            step: gband_core::layout::Step::Grow,
        },
        Size::new(80, 24),
        &LayoutOptions::default(),
    );
    client.receive([shown(&layout)]);
    assert_eq!(client.log(), Vec::<String>::new());
}

#[test]
fn nothing_at_attach() {
    let (_scratch, mut client) = recording("attach", "");
    client.controls.attached(&mut client.display, "main");
    let (layout, panes) = layout_of(3);
    client.receive([shown(&layout), ServerMessage::Focus(panes[0])]);
    assert_eq!(client.log(), ["Attached session=main"]);
}

#[test]
fn terminal_resized() {
    let (_scratch, mut client) = recording("resized", "");
    let (layout, _) = layout_of(1);
    client.receive([shown(&layout)]);
    client
        .controls
        .resize(&mut client.display, Size::new(100, 30));
    assert_eq!(client.log(), ["TerminalResized cols=100,rows=30"]);
}

#[test]
fn handler_dispatches_an_action() {
    let scratch = Scratch::new("handler-action");
    let config = scratch
        .load("gband.on('PaneOpened', function() gband.action.focus_column_left() end)")
        .unwrap();
    let mut client = Client::new(config);
    let (one, _) = layout_of(1);
    let (two, panes) = layout_of(2);
    client.receive([shown(&one)]);
    client.receive([shown(&two), ServerMessage::Focus(panes[1])]);
    assert_eq!(client.display.focused(), Some(panes[0]));
}

#[test]
fn handler_session_actions_become_steps() {
    let scratch = Scratch::new("handler-steps");
    let config = scratch
        .load("gband.on('PaneOpened', function() gband.action.close_pane() end)")
        .unwrap();
    let mut client = Client::new(config);
    let (one, _) = layout_of(1);
    let (two, panes) = layout_of(2);
    client.receive([shown(&one)]);
    let steps = client.receive([shown(&two), ServerMessage::Focus(panes[1])]);
    assert_eq!(
        steps,
        [Step::Send(ClientMessage::Action(
            gband_core::layout::SessionAction::ClosePane(panes[1])
        ))]
    );
}

#[derive(Clone, Default)]
struct Captured(Arc<Mutex<Vec<u8>>>);

impl Write for Captured {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn event_loop_is_cut() {
    let scratch = Scratch::new("loop");
    let config = scratch
        .load(
            "count = 0
gband.on('FocusChanged', function(e)
  count = count + 1
  if e.pane == 2 then gband.action.focus_column_left() else gband.action.focus_column_right() end
end)
gband.bind('alt+l', gband.action.focus_column_right)
gband.bind('alt+x', function() pressed = true end)",
        )
        .unwrap();
    let mut client = Client::new(config);
    let (layout, _) = layout_of(2);
    client.receive([shown(&layout)]);
    let captured = Captured::default();
    let writer = captured.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || writer.clone())
        .with_ansi(false)
        .finish();
    tracing::subscriber::with_default(subscriber, || client.press("alt+l"));
    let lua = client.controls.runtime().lua();
    let count: i64 = lua.globals().get("count").unwrap();
    assert_eq!(count, 10);
    let log = String::from_utf8(captured.0.lock().unwrap().clone()).unwrap();
    assert!(log.contains("WARN") && log.contains("dropping"), "{log}");
    client.press("alt+x");
    let pressed: bool = client
        .controls
        .runtime()
        .lua()
        .globals()
        .get("pressed")
        .unwrap();
    assert!(pressed);
}

#[test]
fn key_table_change_events() {
    let (_scratch, mut client) = recording("tables", DEFAULTS);
    let (layout, _) = layout_of(2);
    client.receive([shown(&layout)]);
    client.clear();
    client.press("ctrl+space");
    client.press("h");
    let log: Vec<String> = client
        .log()
        .into_iter()
        .filter(|entry| entry.starts_with("KeyTable"))
        .collect();
    assert_eq!(
        log,
        [
            "KeyTableChanged previous=root,table=prefix",
            "KeyTableChanged previous=prefix,table=root",
        ]
    );
}

#[test]
fn current_table() {
    let scratch = Scratch::new("current");
    let config = scratch
        .load("gband.bind('prefix c', function() seen = gband.keymap.current_table() end)")
        .unwrap();
    let mut client = Client::new(config);
    client.press("ctrl+space");
    client.press("c");
    let seen: String = client
        .controls
        .runtime()
        .lua()
        .globals()
        .get("seen")
        .unwrap();
    assert_eq!(seen, "prefix");
    assert_eq!(client.controls.active_table(), "root");
}

#[test]
fn named_key_table() {
    let scratch = Scratch::new("named");
    let config = scratch
        .load(
            "gband.keymap.set('move', 'h', gband.action.focus_column_left)
gband.keymap.set('move', 'l', gband.action.focus_column_right)
gband.bind('prefix m', function() gband.keymap.enter('move') end)",
        )
        .unwrap();
    let mut client = Client::new(config);
    let (layout, panes) = layout_of(2);
    client.receive([shown(&layout)]);
    client.press("ctrl+space");
    client.press("m");
    assert_eq!(client.controls.active_table(), "move");
    client.press("l");
    assert_eq!(client.display.focused(), Some(panes[1]));
    assert_eq!(
        client.press("l"),
        [Step::Send(ClientMessage::Key {
            pane: panes[1],
            key: key("l"),
        })]
    );
}

#[test]
fn unbound_key_in_a_named_table() {
    let scratch = Scratch::new("named-unbound");
    let config = scratch
        .load(
            "gband.keymap.set('move', 'h', gband.action.focus_column_left)
gband.bind('prefix m', function() gband.keymap.enter('move') end)",
        )
        .unwrap();
    let mut client = Client::new(config);
    let (layout, _) = layout_of(1);
    client.receive([shown(&layout)]);
    client.press("ctrl+space");
    client.press("m");
    assert_eq!(client.press("x"), []);
    assert_eq!(client.controls.active_table(), "root");
}

#[test]
fn reload() {
    let scratch = Scratch::new("reload");
    let source = "runs = (runs or 0)\ngband.on('ConfigReloaded', function() runs = runs + 1 end)";
    let mut client = Client::new(scratch.load(source).unwrap());
    let reloaded = scratch.load(source);
    client.controls.reload(&mut client.display, reloaded);
    let runs: i64 = client
        .controls
        .runtime()
        .lua()
        .globals()
        .get("runs")
        .unwrap();
    assert_eq!(runs, 1);
}

#[test]
fn reload_makes_root_active() {
    let (scratch, mut client) = recording("reload-root", DEFAULTS);
    client.press("ctrl+space");
    assert_eq!(client.controls.active_table(), "prefix");
    let reloaded = scratch.load(&format!("{RECORD}{DEFAULTS}"));
    client.controls.reload(&mut client.display, reloaded);
    assert_eq!(client.controls.active_table(), "root");
    assert_eq!(
        client.log(),
        [
            "ConfigReloaded ",
            "KeyTableChanged previous=prefix,table=root"
        ]
    );
}

#[test]
fn attached_handler_runs_once() {
    let (_scratch, mut client) = recording("attached", "");
    client.controls.attached(&mut client.display, "main");
    let (layout, _) = layout_of(1);
    client.receive([shown(&layout)]);
    client.press("a");
    assert_eq!(client.log(), ["Attached session=main"]);
}

#[test]
fn plugin_error_on_the_banner() {
    let scratch = Scratch::new("plugin-banner");
    let file = scratch.plugin_file("hello", "plugin/hello.lua", "\n\nerror('boom')");
    let config = scratch.load("").unwrap();
    let client = Client::new(config);
    assert_eq!(
        client.display.banner(),
        Some(format!("hello: {}:3: boom", file.display()).as_str())
    );
}

#[test]
fn good_reload_clears_the_banner() {
    let scratch = Scratch::new("banner-cleared");
    scratch.plugin_file("hello", "plugin/hello.lua", "error('boom')");
    let mut client = Client::new(scratch.load("").unwrap());
    assert!(client.display.banner().is_some());
    fs::remove_dir_all(scratch.0.join("plugins")).unwrap();
    let reloaded = scratch.load("");
    client.controls.reload(&mut client.display, reloaded);
    assert_eq!(client.display.banner(), None);
}

#[test]
fn infinite_loop_in_a_callback() {
    let scratch = Scratch::new("callback-loop");
    scratch.plugin_file(
        "spin",
        "plugin/spin.lua",
        "gband.action.register('go', function() while true do end end)\ngband.bind('alt+s', gband.action['spin.go'])",
    );
    let config = scratch
        .load_with_budget(
            "gband.bind('alt+l', gband.action.focus_column_right)",
            100_000,
        )
        .unwrap();
    let mut client = Client::new(config);
    let (layout, panes) = layout_of(2);
    client.receive([shown(&layout)]);
    assert_eq!(client.press("alt+s"), []);
    let banner = client.display.banner().unwrap().to_owned();
    assert!(banner.starts_with("spin: "), "{banner}");
    assert!(banner.contains("instruction limit"), "{banner}");
    client.press("alt+l");
    assert_eq!(client.display.focused(), Some(panes[1]));
    client.display.set_banner(None);
    assert_eq!(client.press("alt+s"), []);
    assert_eq!(client.display.banner(), None);
}
