use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gband_client::animation::Animations;
use gband_client::{Controls, Display, Step};
use gband_core::geometry::Size;
use gband_core::layout::{Layout, LayoutOptions, WindowId};
use gband_lua::keys::parse_key;
use gband_lua::{Config, ConfigError, DEFAULTS, LoadOptions, Locations};
use gband_protocol::{ClientMessage, ServerMessage};
use insta::assert_snapshot;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "gband-client-statusline-{name}-{}",
            std::process::id()
        ));
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

    fn client_plugin(&self, plugin: &str, source: &str) -> PathBuf {
        let directory = self.0.join("plugins").join(plugin);
        write(
            &directory.join("plugin.lua"),
            &format!("return {{ name = '{plugin}', version = '0.1.0' }}"),
        );
        write(&directory.join("client.lua"), source)
    }

    fn load(&self, source: &str) -> Result<Config, ConfigError> {
        write(
            &gband_lua::user_file(&self.0.join("config"), gband_lua::Side::Client),
            source,
        );
        gband_lua::load(
            &self.locations(),
            gband_lua::Side::Client,
            &LoadOptions::default(),
        )
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

struct Client {
    display: Display,
    controls: Controls,
    windows: Vec<WindowId>,
}

impl Client {
    fn new(config: Config, terminal: Size) -> Self {
        let mut display = Display::new(terminal, Animations::Off);
        let controls = Controls::new(config, &mut display);
        Self {
            display,
            controls,
            windows: Vec::new(),
        }
    }

    fn attach(&mut self, columns: usize, focused: usize) -> Vec<Step> {
        let mut steps = self.controls.attached(&mut self.display, "main");
        let mut layout = Layout::new();
        let band = layout.bands()[0].id;
        for _ in 0..columns {
            let window = layout.allocate_window();
            layout.open(
                window,
                band,
                self.windows.last().copied(),
                None,
                &LayoutOptions::default(),
            );
            self.windows.push(window);
        }
        let reported = self.display.reported_size();
        let mut messages = vec![ServerMessage::Layout {
            cols: reported.cols,
            rows: reported.rows,
            layout,
        }];
        if let Some(&window) = self.windows.get(focused) {
            messages.push(ServerMessage::Focus(window));
        }
        let received = self.controls.receive(&mut self.display, messages);
        steps.extend(received.steps);
        steps
    }

    fn press(&mut self, name: &str) -> Vec<Step> {
        self.controls
            .press(&mut self.display, parse_key(name).unwrap())
    }

    fn status_area(&self) -> Option<Rect> {
        self.display
            .bars()
            .find(|(bar, _)| bar.id == "statusline")
            .map(|(_, area)| area)
    }

    fn status(&self) -> Vec<(usize, String)> {
        self.display
            .bars()
            .find(|(bar, _)| bar.id == "statusline")
            .map(|(bar, _)| {
                bar.lines
                    .iter()
                    .map(|runs| runs.iter().map(|run| run.text.as_str()).collect::<String>())
                    .enumerate()
                    .filter(|(_, text)| !text.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn row(&self, row: usize) -> String {
        self.status()
            .into_iter()
            .find(|(at, _)| *at == row)
            .map(|(_, text)| text)
            .unwrap_or_default()
    }

    fn screen(&mut self, terminal: Size) -> String {
        let mut backend = Terminal::new(TestBackend::new(terminal.cols, terminal.rows)).unwrap();
        backend
            .draw(|frame| self.display.draw(frame, Instant::now()))
            .unwrap();
        format!("{:?}", backend.backend().buffer())
    }

    fn number(&self, name: &str) -> i64 {
        self.controls.runtime().lua().globals().get(name).unwrap()
    }
}

fn resize(size: Size) -> Step {
    Step::Send(ClientMessage::Resize {
        cols: size.cols,
        rows: size.rows,
    })
}

fn resizes(steps: &[Step]) -> Vec<&Step> {
    steps
        .iter()
        .filter(|step| matches!(step, Step::Send(ClientMessage::Resize { .. })))
        .collect()
}

const SETUP: &str = "gband.plugin(\"gband.statusline\")";

fn without_status_line() -> String {
    DEFAULTS.replace(SETUP, "")
}

fn with_setup(opts: &str) -> String {
    DEFAULTS.replace(
        SETUP,
        &format!("gband.plugin(\"gband.statusline\", {opts})"),
    )
}

fn defaults(name: &str, extra: &str, terminal: Size) -> (Scratch, Client) {
    let scratch = Scratch::new(name);
    let config = scratch.load(&format!("{DEFAULTS}\n{extra}")).unwrap();
    (scratch, Client::new(config, terminal))
}

fn loaded(name: &str, source: &str, terminal: Size) -> (Scratch, Client) {
    let scratch = Scratch::new(name);
    let config = scratch.load(source).unwrap();
    (scratch, Client::new(config, terminal))
}

fn line(row: usize, text: &str) -> (usize, String) {
    (row, text.to_owned())
}

#[test]
fn default_placement() {
    let (_scratch, mut client) = defaults("default-placement", "", Size::new(80, 24));
    client.attach(1, 0);
    assert_eq!(client.display.reported_size(), Size::new(80, 24));
    assert_eq!(client.display.ribbon_area(), Rect::new(20, 0, 60, 24));
    assert_eq!(client.status_area(), Some(Rect::new(0, 0, 20, 24)));
}

#[test]
fn right_side() {
    let (_scratch, mut client) = loaded(
        "right-side",
        &with_setup("{ side = \"right\" }"),
        Size::new(80, 24),
    );
    client.attach(1, 0);
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 0, 60, 24));
    assert_eq!(client.status_area(), Some(Rect::new(60, 0, 20, 24)));
}

#[test]
fn turning_the_status_line_off() {
    let (_scratch, mut client) = loaded("off", &without_status_line(), Size::new(80, 24));
    client.attach(1, 0);
    assert_eq!(client.display.reported_size(), Size::new(80, 24));
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 0, 80, 24));
    assert_eq!(client.status_area(), None);
}

#[test]
fn terminal_too_short() {
    let (_scratch, mut client) = defaults("short", "", Size::new(20, 24));
    client.attach(1, 0);
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 0, 20, 24));
    assert_eq!(client.status_area(), None);
    let steps = client
        .controls
        .resize(&mut client.display, Size::new(30, 24));
    assert_eq!(resizes(&steps), [&resize(Size::new(30, 24))]);
    assert_eq!(client.status_area(), Some(Rect::new(0, 0, 20, 24)));
    assert_eq!(client.display.ribbon_area(), Rect::new(20, 0, 10, 24));
}

#[test]
fn resize_reports_the_terminal_size() {
    let (_scratch, mut client) = defaults("resize", "", Size::new(80, 24));
    client.attach(1, 0);
    let steps = client
        .controls
        .resize(&mut client.display, Size::new(100, 30));
    assert_eq!(steps.first(), Some(&resize(Size::new(100, 30))));
    assert_eq!(client.display.ribbon_area(), Rect::new(20, 0, 80, 30));
}

#[test]
fn shown_windows_follow_the_ribbon() {
    let (_scratch, mut client) = defaults("shown", "", Size::new(80, 24));
    client.attach(3, 0);
    let Some(ClientMessage::Shown(shown)) = client.display.report_shown() else {
        panic!("nothing shown");
    };
    assert_eq!(shown, client.windows[..2]);
}

#[test]
fn reload_turning_the_status_line_off() {
    let (scratch, mut client) = defaults("reload-off", "", Size::new(80, 24));
    client.attach(1, 0);
    let steps = client
        .controls
        .reload(&mut client.display, scratch.load(&without_status_line()));
    assert!(resizes(&steps).is_empty(), "{steps:?}");
    assert_eq!(client.status_area(), None);
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 0, 80, 24));
}

#[test]
fn reload_moving_the_status_line() {
    let (scratch, mut client) = defaults("reload-right", "", Size::new(80, 24));
    client.attach(1, 0);
    let steps = client.controls.reload(
        &mut client.display,
        scratch.load(&with_setup("{ side = \"right\" }")),
    );
    assert!(resizes(&steps).is_empty(), "{steps:?}");
    assert_eq!(client.status_area(), Some(Rect::new(60, 0, 20, 24)));
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 0, 60, 24));
}

#[test]
fn reload_keeps_an_unchanged_status_line() {
    let (scratch, mut client) = defaults("reload-same", "", Size::new(80, 24));
    client.attach(2, 1);
    let steps = client
        .controls
        .reload(&mut client.display, scratch.load(DEFAULTS));
    assert!(resizes(&steps).is_empty(), "{steps:?}");
    assert_eq!(client.display.ribbon_area(), Rect::new(20, 0, 60, 24));
    assert_eq!(client.row(23), "2/2");
}

#[test]
fn reload_turning_the_status_line_on_renders_it() {
    let (scratch, mut client) = loaded("reload-on", &without_status_line(), Size::new(80, 24));
    client.attach(2, 1);
    let steps = client
        .controls
        .reload(&mut client.display, scratch.load(DEFAULTS));
    assert!(resizes(&steps).is_empty(), "{steps:?}");
    assert_eq!(client.row(23), "2/2");
}

#[test]
fn failed_reload_keeps_the_bars() {
    let (scratch, mut client) = defaults("reload-failed", "", Size::new(80, 24));
    client.attach(1, 0);
    client
        .controls
        .reload(&mut client.display, scratch.load("local = 1"));
    assert_eq!(client.status_area(), Some(Rect::new(0, 0, 20, 24)));
    assert_eq!(client.row(0), "error");
}

#[test]
fn default_segments_after_attach() {
    let (_scratch, mut client) = defaults("segments", "", Size::new(80, 24));
    client.attach(3, 1);
    assert_eq!(
        client.status(),
        [
            line(0, "band 1"),
            line(1, "C-space navigation"),
            line(23, "2/3")
        ]
    );
}

#[test]
fn position_follows_focus() {
    let (_scratch, mut client) = defaults("focus", "", Size::new(80, 24));
    client.attach(3, 1);
    client.press("ctrl+space");
    client.press("h");
    assert_eq!(client.row(23), "1/3");
}

#[test]
fn mode_segment_after_the_prefix() {
    let (_scratch, mut client) = defaults("mode", "", Size::new(80, 24));
    client.attach(3, 1);
    client.press("ctrl+space");
    assert_eq!(client.row(1), "navigation");
    client.press("h");
    assert_eq!(client.row(1), "navigation");
    client.press("escape");
    assert_eq!(client.row(1), "C-space navigation");
}

#[test]
fn hints_after_the_prefix() {
    let (_scratch, mut client) = defaults("hints", "", Size::new(80, 24));
    client.attach(3, 1);
    client.press("ctrl+space");
    assert_eq!(client.row(2), "h left  l right");
    assert_eq!(client.row(3), "j down  k up");
    let rows = client.status();
    let (last, text) = &rows[rows.len() - 2];
    assert!(text.ends_with(" …"), "{rows:?}");
    assert_eq!(*last, 21);
    assert_eq!(rows.last().unwrap(), &line(23, "2/3"));
}

#[test]
fn back_to_root() {
    let (_scratch, mut client) = defaults("back-to-root", "", Size::new(80, 24));
    client.attach(3, 1);
    client.press("ctrl+space");
    client.press("h");
    assert_eq!(client.row(2), "h left  l right");
    client.press("escape");
    assert_eq!(
        client.status(),
        [
            line(0, "band 1"),
            line(1, "C-space navigation"),
            line(23, "1/3")
        ]
    );
}

#[test]
fn height_change_renders_each_component() {
    let (_scratch, mut client) = defaults(
        "height",
        "renders = 0\ngband.ui.statusline.add({ id = 'w', render = function(ctx) renders = renders + 1 height = ctx.total_height end })",
        Size::new(80, 24),
    );
    client.attach(1, 0);
    assert_eq!(client.number("renders"), 1);
    client
        .controls
        .resize(&mut client.display, Size::new(60, 24));
    assert_eq!(client.number("renders"), 1);
    client
        .controls
        .resize(&mut client.display, Size::new(60, 30));
    assert_eq!(client.number("renders"), 2);
    assert_eq!(client.number("height"), 30);
}

#[test]
fn frames_do_not_render() {
    let (_scratch, mut client) = defaults(
        "frames",
        "renders = 0\ngband.ui.statusline.add({ id = 'f', render = function() renders = renders + 1 return 'f' end })",
        Size::new(80, 24),
    );
    client.attach(2, 0);
    for _ in 0..20 {
        client.screen(Size::new(80, 24));
    }
    assert_eq!(client.number("renders"), 1);
}

#[test]
fn timers_drive_interval_components() {
    let (_scratch, mut client) = defaults(
        "timers",
        "ticks = 0\ngband.ui.statusline.add({ id = 't', redraw_interval = 100, render = function() ticks = ticks + 1 return 'tick ' .. ticks end })",
        Size::new(80, 24),
    );
    client.attach(1, 0);
    let deadline = client.controls.next_timer().unwrap();
    client
        .controls
        .fire_timers(&mut client.display, deadline + Duration::from_millis(1));
    assert_eq!(client.number("ticks"), 2);
    assert!(
        client.status().iter().any(|(_, text)| text == "tick 2"),
        "{:?}",
        client.status()
    );
}

#[test]
fn plugin_error_at_start() {
    let scratch = Scratch::new("error-start");
    let file = scratch.client_plugin("hello", "\n\nerror('boom')");
    let config = scratch.load(DEFAULTS).unwrap();
    let mut client = Client::new(config, Size::new(200, 24));
    client.attach(1, 0);
    let expected = format!("hello: {}:3: boom", file.display());
    assert_eq!(client.row(0), "error");
    assert_eq!(client.row(1), "band 1");
    assert_eq!(client.display.errors(), std::slice::from_ref(&expected));
    let listed: Vec<String> = client
        .controls
        .runtime()
        .lua()
        .load("return gband.errors()")
        .eval()
        .unwrap();
    assert_eq!(listed, std::slice::from_ref(&expected));
    assert_eq!(client.display.reported_size(), Size::new(200, 24));
    let screen = client.screen(Size::new(200, 24));
    assert!(!screen.contains("boom"), "{screen}");
}

#[test]
fn errors_kept_in_order() {
    let scratch = Scratch::new("error-order");
    scratch.client_plugin("alpha", "error('first')");
    scratch.client_plugin("beta", "error('second')");
    let config = scratch.load(DEFAULTS).unwrap();
    let mut client = Client::new(config, Size::new(80, 24));
    client.attach(1, 0);
    let errors = client.display.errors();
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(errors[0].starts_with("alpha: "), "{errors:?}");
    assert!(errors[1].starts_with("beta: "), "{errors:?}");
}

#[test]
fn cleared_by_a_good_load() {
    let scratch = Scratch::new("error-cleared");
    scratch.client_plugin("hello", "error('boom')");
    let config = scratch.load(DEFAULTS).unwrap();
    let mut client = Client::new(config, Size::new(200, 24));
    client.attach(1, 0);
    assert_eq!(client.row(0), "error");
    fs::remove_dir_all(scratch.0.join("plugins")).unwrap();
    let reloaded = scratch.load(DEFAULTS);
    client.controls.reload(&mut client.display, reloaded);
    assert_eq!(client.row(0), "band 1");
    assert!(client.display.errors().is_empty());
}

#[test]
fn error_from_a_binding_reaches_the_status_line() {
    let (_scratch, mut client) = defaults(
        "error-binding",
        "gband.bind('alt+e', function() error('bad binding') end)",
        Size::new(200, 24),
    );
    client.attach(1, 0);
    client.press("alt+e");
    assert_eq!(client.row(0), "error");
    let last = client.display.errors().last().unwrap().clone();
    assert!(last.contains("bad binding"), "{last}");
}

#[test]
fn failing_component_reaches_the_status_line() {
    let (_scratch, mut client) = defaults(
        "error-render",
        "gband.ui.statusline.add({ id = 'bad', render = function() error('render boom') end })",
        Size::new(200, 24),
    );
    client.attach(1, 0);
    assert_eq!(client.row(0), "error");
    assert_eq!(client.row(1), "band 1");
    let last = client.display.errors().last().unwrap().clone();
    assert!(last.contains("render boom"), "{last}");
}

#[test]
fn plugin_error_on_the_banner() {
    let scratch = Scratch::new("error-banner");
    let file = scratch.client_plugin("hello", "\n\nerror('boom')");
    let config = scratch.load(&without_status_line()).unwrap();
    let mut client = Client::new(config, Size::new(200, 6));
    client.attach(1, 0);
    let expected = format!("hello: {}:3: boom", file.display());
    assert_eq!(client.display.banner(), Some(expected.as_str()));
    let screen = client.screen(Size::new(200, 6));
    assert!(screen.contains(&expected), "{screen}");
}

#[test]
fn banner_without_a_status_line() {
    let (_scratch, mut client) = defaults("banner-narrow", "", Size::new(20, 6));
    client.attach(1, 0);
    client
        .display
        .report_error("user/init.lua:3: boom".to_owned());
    client.controls.refresh(&mut client.display);
    assert_eq!(client.status_area(), None);
    let screen = client.screen(Size::new(20, 6));
    assert!(screen.contains("user/init.lua:3: bo"), "{screen}");
}

#[test]
fn default_status_line() {
    let (_scratch, mut client) = defaults("snapshot-default", "", Size::new(40, 8));
    client.attach(2, 1);
    assert_snapshot!(client.screen(Size::new(40, 8)));
}

#[test]
fn prefix_hints_cut() {
    let (_scratch, mut client) = defaults("snapshot-prefix-hints", "", Size::new(80, 8));
    client.attach(2, 1);
    client.press("ctrl+space");
    assert_snapshot!(client.screen(Size::new(80, 8)));
}

#[test]
fn right_placement() {
    let (_scratch, mut client) = loaded(
        "snapshot-right",
        &with_setup("{ side = \"right\" }"),
        Size::new(40, 8),
    );
    client.attach(2, 1);
    assert_snapshot!(client.screen(Size::new(40, 8)));
}

#[test]
fn short_terminal_drops_and_cuts() {
    let scratch = Scratch::new("snapshot-short");
    let config = scratch
        .load(
            "gband.plugin('gband.statusline', { min_width = 1, max_width = 10 })\ngband.ui.statusline.add({ id = 'low', priority = 1, render = function() return 'dropped' end })\ngband.ui.statusline.add({ id = 'high', priority = 2, align = 'bottom', render = function() return 'a segment too long to fit' end })",
        )
        .unwrap();
    let mut client = Client::new(config, Size::new(30, 2));
    client.attach(1, 0);
    assert_eq!(client.status(), [line(1, "a segment…")]);
    assert_snapshot!(client.screen(Size::new(30, 2)));
}

#[test]
fn error_item() {
    let (_scratch, mut client) = defaults("snapshot-error", "", Size::new(40, 6));
    client.attach(1, 0);
    client
        .display
        .report_error("user/init.lua:3: boom\nmore".to_owned());
    client.controls.refresh(&mut client.display);
    assert_eq!(
        client.status(),
        [
            line(0, "error"),
            line(1, "band 1"),
            line(2, "C-space navigation"),
            line(5, "1/1")
        ]
    );
    assert_snapshot!(client.screen(Size::new(40, 6)));
}
