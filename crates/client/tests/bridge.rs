use std::fs;
use std::path::{Path, PathBuf};

use gband_client::animation::Animations;
use gband_client::{Controls, Display, Step};
use gband_core::geometry::Size;
use gband_core::layout::{Layout, LayoutOptions, WindowId};
use gband_lua::keys::parse_key;
use gband_lua::{Config, LoadOptions, Locations, Side};
use gband_protocol::{ClientMessage, Key, Requirement, ServerMessage, Value};

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("gband-client-bridge-{name}-{}", std::process::id()));
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

    fn plugin(&self, name: &str, version: &str) {
        write(
            &self.0.join("plugins").join(name).join("plugin.lua"),
            &format!("return {{ name = '{name}', version = '{version}' }}"),
        );
    }

    fn load(&self, source: &str) -> Config {
        write(
            &gband_lua::user_file(&self.0.join("config"), Side::Client),
            source,
        );
        gband_lua::load(&self.locations(), Side::Client, &LoadOptions::default()).unwrap()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write(path: &Path, source: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
}

fn layout_of(count: usize) -> Layout {
    let mut layout = Layout::new();
    let band = layout.bands()[0].id;
    let mut after = None;
    for _ in 0..count {
        let window = layout.allocate_window();
        layout.open(window, band, after, None, &LayoutOptions::default());
        after = Some(window);
    }
    layout
}

fn shown(layout: &Layout) -> ServerMessage {
    ServerMessage::Layout {
        cols: 80,
        rows: 24,
        layout: layout.clone(),
    }
}

fn state(window: u32, key: &str, value: Option<&str>) -> ServerMessage {
    ServerMessage::WindowState {
        window: WindowId(window),
        key: key.to_owned(),
        value: value.map(Value::string),
    }
}

fn requirements(list: &[(&str, &str)]) -> ServerMessage {
    ServerMessage::Requirements(
        list.iter()
            .map(|(plugin, requirement)| Requirement {
                plugin: (*plugin).to_owned(),
                requirement: (*requirement).to_owned(),
            })
            .collect(),
    )
}

fn event(name: &str, data: Value) -> ServerMessage {
    ServerMessage::Event {
        name: name.to_owned(),
        data,
        queued: false,
        time: 1,
    }
}

const RECORD: &str = "log = {}
local function show(value)
  if type(value) == 'table' then
    local parts = {}
    for k, v in pairs(value) do parts[#parts + 1] = tostring(k) .. ':' .. tostring(v) end
    table.sort(parts)
    return '{' .. table.concat(parts, ' ') .. '}'
  end
  return tostring(value)
end
local function record(name)
  return function(e)
    local keys = {}
    for k, v in pairs(e) do keys[#keys + 1] = k .. '=' .. show(v) end
    table.sort(keys)
    log[#log + 1] = name .. ' ' .. table.concat(keys, ',')
  end
end
for _, name in ipairs({ 'Attached', 'WindowStateChanged', 'WindowOpened' }) do
  gband.on(name, record(name))
end
";

struct Client {
    _scratch: Scratch,
    display: Display,
    controls: Controls,
}

impl Client {
    fn new(name: &str, source: &str) -> Self {
        Self::with(Scratch::new(name), source)
    }

    fn with(scratch: Scratch, source: &str) -> Self {
        let config = scratch.load(&format!("{RECORD}{source}"));
        let mut display = Display::new(Size::new(80, 24), Animations::Off);
        let mut controls = Controls::new(config, &mut display);
        controls.attach_when_ready("main");
        Self {
            _scratch: scratch,
            display,
            controls,
        }
    }

    fn receive(&mut self, messages: impl IntoIterator<Item = ServerMessage>) -> Vec<Step> {
        let received = self.controls.receive(&mut self.display, messages);
        assert_eq!(received.outcome, None);
        received.steps
    }

    fn attach(&mut self, windows: usize, extra: impl IntoIterator<Item = ServerMessage>) {
        let mut messages = vec![shown(&layout_of(windows))];
        messages.extend(extra);
        messages.push(requirements(&[]));
        self.receive(messages);
    }

    fn press(&mut self, name: &str) -> Vec<Step> {
        self.controls
            .press(&mut self.display, parse_key(name).unwrap())
    }

    fn eval<T: mlua::FromLua>(&self, source: &str) -> T {
        self.controls.runtime().lua().load(source).eval().unwrap()
    }

    fn log(&self) -> Vec<String> {
        self.eval("return log")
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

#[test]
fn state_at_attach() {
    let mut client = Client::new(
        "state-at-attach",
        "gband.on('Attached', function() seen = gband.window_state(2).agent end)",
    );
    assert!(!client.controls.is_attached());
    client.attach(3, [state(2, "agent", Some("waiting"))]);
    assert!(client.controls.is_attached());
    assert_eq!(client.eval::<String>("return seen"), "waiting");
    assert_eq!(client.log(), ["Attached session=main"]);
}

#[test]
fn attached_waits_for_the_attach_batch() {
    let mut client = Client::new("attach-batch", "");
    client.receive([shown(&layout_of(1)), state(1, "agent", Some("x"))]);
    assert!(client.log().is_empty());
    client.receive([requirements(&[])]);
    assert_eq!(client.log(), ["Attached session=main"]);
}

#[test]
fn state_change() {
    let mut client = Client::new("state-change", "");
    client.attach(3, []);
    client.clear();
    client.receive([state(1, "agent", Some("waiting"))]);
    assert_eq!(
        client.log(),
        ["WindowStateChanged key=agent,value=waiting,window=1"]
    );
}

#[test]
fn change_event() {
    let mut client = Client::new("change-event", "");
    client.attach(3, [state(2, "agent", Some("waiting"))]);
    client.clear();
    client.receive([state(2, "agent", None)]);
    assert_eq!(
        client.log(),
        ["WindowStateChanged key=agent,previous=waiting,window=2"]
    );
}

#[test]
fn read_only_copy() {
    let mut client = Client::new(
        "read-only",
        "gband.bind('alt+x', function()
  gband.window_state(2).agent = 'x'
  after = gband.window_state(2).agent
  empty = next(gband.window_state(1)) == nil
  absent = gband.window_state(9) == nil
end)",
    );
    client.attach(3, [state(2, "agent", Some("waiting"))]);
    client.press("alt+x");
    assert_eq!(client.eval::<String>("return after"), "waiting");
    assert!(client.eval::<bool>("return empty"));
    assert!(client.eval::<bool>("return absent"));
}

#[test]
fn state_dropped_with_its_window() {
    let mut client = Client::new(
        "state-dropped",
        "gband.bind('alt+x', function() absent = gband.window_state(2) == nil end)",
    );
    client.attach(2, [state(2, "agent", Some("waiting"))]);
    let mut layout = layout_of(2);
    layout.remove(WindowId(2));
    client.receive([shown(&layout)]);
    assert!(client.display.window_state(WindowId(2)).is_none());
    client.press("alt+x");
    assert!(client.eval::<bool>("return absent"));
}

#[test]
fn state_of_a_window_not_yet_in_the_layout_is_kept() {
    let mut client = Client::new("state-early", "");
    client.attach(1, []);
    client.receive([state(2, "agent", Some("waiting"))]);
    let mut layout = layout_of(1);
    let window = layout.allocate_window();
    layout.open(
        window,
        layout.bands()[0].id,
        None,
        None,
        &LayoutOptions::default(),
    );
    client.receive([shown(&layout)]);
    assert_eq!(
        client.display.window_state(WindowId(2)).unwrap()["agent"],
        Value::string("waiting")
    );
}

#[test]
fn notify_on_a_waiting_agent() {
    let mut client = Client::new(
        "notify-waiting",
        "gband.on('ServerEvent', function(e) runs = (runs or 0) + 1 title = e.data.title queued = e.queued time = e.time end, { pattern = 'agent.waiting' })",
    );
    client.attach(1, []);
    client.receive([event(
        "agent.waiting",
        Value::Table(vec![(Key::string("title"), Value::string("build"))]),
    )]);
    assert_eq!(client.eval::<i64>("return runs"), 1);
    assert_eq!(client.eval::<String>("return title"), "build");
    assert!(!client.eval::<bool>("return queued"));
    assert_eq!(client.eval::<i64>("return time"), 1);
}

#[test]
fn pattern_filters() {
    let mut client = Client::new(
        "pattern-filters",
        "gband.on('ServerEvent', function(e) ran = true end, { pattern = 'agent.waiting' })",
    );
    client.attach(1, []);
    client.receive([event("agent.done", Value::Nil)]);
    assert!(client.eval::<Option<bool>>("return ran").is_none());
}

#[test]
fn server_event_pattern() {
    let mut client = Client::new(
        "server-pattern",
        "gband.on('ServerEvent', function(e) seen = (seen or '') .. e.name end, { pattern = 'agent.done' })
gband.on('ServerEvent', function(e) all = (all or 0) + 1 end)",
    );
    client.attach(1, []);
    client.receive([
        event("agent.waiting", Value::Nil),
        event("agent.done", Value::Nil),
    ]);
    assert_eq!(client.eval::<String>("return seen"), "agent.done");
    assert_eq!(client.eval::<i64>("return all"), 2);
}

#[test]
fn code_as_data() {
    let mut client = Client::new(
        "code-as-data",
        "gband.on('ServerEvent', function(e) stored = e.data end)",
    );
    client.attach(1, []);
    client.receive([event("x", Value::string("os.exit(1)"))]);
    assert_eq!(client.eval::<String>("return stored"), "os.exit(1)");
}

fn command_step(steps: &[Step]) -> (u64, String, Value) {
    steps
        .iter()
        .find_map(|step| match step {
            Step::Send(ClientMessage::Command { call, name, args }) => {
                Some((*call, name.clone(), args.clone()))
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("no command in {steps:?}"))
}

#[test]
fn jump_to_the_next_waiting_agent() {
    let mut client = Client::new(
        "jump",
        "gband.bind('prefix a', function()
  gband.rpc('agents.next_waiting', {}, function(ok, result) answer = tostring(ok) .. ' ' .. tostring(result) end)
end)",
    );
    client.attach(4, []);
    client.press("ctrl+space");
    let steps = client.press("a");
    let (call, name, args) = command_step(&steps);
    assert_eq!(name, "agents.next_waiting");
    assert_eq!(args, Value::Table(Vec::new()));
    client.receive([
        ServerMessage::Focus(WindowId(4)),
        ServerMessage::Result {
            call,
            result: Ok(Value::Int(4)),
        },
    ]);
    assert_eq!(client.display.focused(), Some(WindowId(4)));
    assert_eq!(client.eval::<String>("return answer"), "true 4");
}

#[test]
fn unknown_command_answer() {
    let mut client = Client::new(
        "rpc-unknown",
        "gband.bind('alt+x', function()
  gband.rpc('absent', nil, function(ok, message) answer = tostring(ok) .. ' ' .. message end)
end)",
    );
    client.attach(1, []);
    let (call, _, args) = command_step(&client.press("alt+x"));
    assert_eq!(args, Value::Nil);
    client.receive([ServerMessage::Result {
        call,
        result: Err("unknown command `absent`".to_owned()),
    }]);
    assert_eq!(
        client.eval::<String>("return answer"),
        "false unknown command `absent`"
    );
    client.receive([ServerMessage::Result {
        call,
        result: Ok(Value::Nil),
    }]);
    assert_eq!(
        client.eval::<String>("return answer"),
        "false unknown command `absent`"
    );
}

#[test]
fn rpc_argument_errors() {
    for call in [
        "gband.rpc('', {})",
        "gband.rpc('x', 'args')",
        "gband.rpc('x', {}, 3)",
        "gband.rpc('x', { f = print })",
    ] {
        let mut client = Client::new(
            "rpc-errors",
            &format!("gband.bind('alt+x', function()\n{call}\nend)"),
        );
        client.attach(1, []);
        let steps = client.press("alt+x");
        assert!(
            !steps
                .iter()
                .any(|step| matches!(step, Step::Send(ClientMessage::Command { .. }))),
            "{call}"
        );
        let banner = client.display.banner().unwrap_or_default().to_owned();
        let line = RECORD.lines().count() + 2;
        assert!(
            banner.contains(&format!("init.lua:{line}:")),
            "{call}: {banner}"
        );
    }
}

#[test]
fn rpc_outside_a_callback() {
    let scratch = Scratch::new("rpc-loading");
    write(
        &gband_lua::user_file(&scratch.0.join("config"), Side::Client),
        "gband.rpc('x')",
    );
    let error = gband_lua::load(&scratch.locations(), Side::Client, &LoadOptions::default())
        .err()
        .unwrap();
    assert!(error.message.contains("gband.rpc"), "{error}");
}

#[test]
fn missing_client_side() {
    let mut client = Client::new("missing-plugin", "");
    client.receive([
        shown(&layout_of(1)),
        requirements(&[("agent-status", ">= 0.1")]),
    ]);
    let banner = client.display.banner().unwrap();
    assert!(
        banner.contains("agent-status") && banner.contains(">= 0.1"),
        "{banner}"
    );
}

#[test]
fn old_client_side() {
    let scratch = Scratch::new("old-plugin");
    scratch.plugin("agent-status", "0.1.4");
    let mut client = Client::with(scratch, "");
    client.receive([
        shown(&layout_of(1)),
        requirements(&[("agent-status", ">= 0.2, < 1")]),
    ]);
    let banner = client.display.banner().unwrap();
    assert!(
        banner.contains("agent-status")
            && banner.contains(">= 0.2, < 1")
            && banner.contains("0.1.4"),
        "{banner}"
    );
    assert_eq!(client.log(), ["Attached session=main"]);
}

#[test]
fn met() {
    let scratch = Scratch::new("met-plugin");
    scratch.plugin("agent-status", "0.1");
    let mut client = Client::with(scratch, "");
    client.receive([
        shown(&layout_of(1)),
        requirements(&[("agent-status", ">= 0.1")]),
    ]);
    assert_eq!(client.display.banner(), None);
}

#[test]
fn server_error_in_the_client() {
    let mut client = Client::new("server-error", "");
    client.attach(1, []);
    client.receive([ServerMessage::ServerError(
        "agent-status: /srv/plugins/agent-status/server.lua:5: boom".to_owned(),
    )]);
    assert_eq!(
        client.display.banner(),
        Some("server: agent-status: /srv/plugins/agent-status/server.lua:5: boom")
    );
}
