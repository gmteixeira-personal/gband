use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gband_client::animation::Animations;
use gband_client::{Controls, Display, Step};
use gband_core::geometry::Size;
use gband_core::layout::{Layout, LayoutOptions, PaneId};
use gband_lua::keys::parse_key;
use gband_lua::{Config, ConfigError, DEFAULTS, LoadOptions, Locations, StatusLine};
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

    fn plugin_file(&self, plugin: &str, relative: &str, source: &str) -> PathBuf {
        write(&self.0.join("plugins").join(plugin).join(relative), source)
    }

    fn load(&self, source: &str) -> Result<Config, ConfigError> {
        write(&gband_lua::user_file(&self.0.join("config")), source);
        gband_lua::load(&self.locations(), &LoadOptions::default())
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
    panes: Vec<PaneId>,
}

impl Client {
    fn new(config: Config, terminal: Size) -> Self {
        let mut display = Display::new(terminal, Animations::Off);
        let controls = Controls::new(config, &mut display);
        Self {
            display,
            controls,
            panes: Vec::new(),
        }
    }

    fn attach(&mut self, columns: usize, focused: usize) -> Vec<Step> {
        let mut steps = self.controls.attached(&mut self.display, "main");
        let mut layout = Layout::new();
        let band = layout.bands()[0].id;
        for _ in 0..columns {
            let pane = layout.allocate_pane();
            layout.open(
                pane,
                band,
                self.panes.last().copied(),
                &LayoutOptions::default(),
            );
            self.panes.push(pane);
        }
        let reported = self.display.reported_size();
        let mut messages = vec![ServerMessage::Layout {
            cols: reported.cols,
            rows: reported.rows,
            layout,
        }];
        if let Some(&pane) = self.panes.get(focused) {
            messages.push(ServerMessage::Focus(pane));
        }
        let received = self.controls.receive(&mut self.display, messages);
        steps.extend(received.steps);
        steps
    }

    fn press(&mut self, name: &str) -> Vec<Step> {
        self.controls
            .press(&mut self.display, parse_key(name).unwrap())
    }

    fn status(&self) -> String {
        let width = self.display.status_area().map_or(0, |area| area.width);
        self.display
            .status_line()
            .map_or_else(String::new, |line| text(line, width))
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

fn text(line: &StatusLine, width: u16) -> String {
    let mut cells = vec![" ".to_owned(); usize::from(width)];
    for span in &line.spans {
        let mut col = usize::from(span.col);
        for c in span.text.chars() {
            let taken = gband_lua::ui::width(&c.to_string());
            if col < cells.len() {
                cells[col] = c.to_string();
            }
            for extra in 1..taken {
                if col + extra < cells.len() {
                    cells[col + extra] = String::new();
                }
            }
            col += taken;
        }
    }
    cells.concat()
}

fn resize(size: Size) -> Step {
    Step::Send(ClientMessage::Resize {
        cols: size.cols,
        rows: size.rows,
    })
}

fn defaults(name: &str, extra: &str, terminal: Size) -> (Scratch, Client) {
    let scratch = Scratch::new(name);
    let config = scratch.load(&format!("{DEFAULTS}\n{extra}")).unwrap();
    (scratch, Client::new(config, terminal))
}

#[test]
fn status_line_at_the_bottom() {
    let (_scratch, client) = defaults("bottom", "", Size::new(80, 24));
    assert_eq!(client.display.reported_size(), Size::new(80, 23));
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 0, 80, 23));
    assert_eq!(client.display.status_area(), Some(Rect::new(0, 23, 80, 1)));
}

#[test]
fn status_line_on_top() {
    let (_scratch, client) = defaults(
        "top",
        "gband.opt.statusline_position = 'top'",
        Size::new(80, 24),
    );
    assert_eq!(client.display.reported_size(), Size::new(80, 23));
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 1, 80, 23));
    assert_eq!(client.display.status_area(), Some(Rect::new(0, 0, 80, 1)));
}

#[test]
fn status_line_off() {
    let (_scratch, client) = defaults(
        "off",
        "gband.opt.statusline_position = 'off'",
        Size::new(80, 24),
    );
    assert_eq!(client.display.reported_size(), Size::new(80, 24));
    assert_eq!(client.display.status_area(), None);
}

#[test]
fn terminal_too_short() {
    let (_scratch, client) = defaults("short", "gband.opt.statusline_height = 2", Size::new(80, 2));
    assert_eq!(client.display.reported_size(), Size::new(80, 2));
    assert_eq!(client.display.status_area(), None);
}

#[test]
fn resize_reports_the_ribbon_area() {
    let (_scratch, mut client) = defaults("resize", "", Size::new(80, 24));
    client.attach(1, 0);
    let steps = client
        .controls
        .resize(&mut client.display, Size::new(100, 30));
    assert_eq!(steps.first(), Some(&resize(Size::new(100, 29))));
}

#[test]
fn growing_past_the_status_height_draws_it() {
    let (_scratch, mut client) =
        defaults("grow", "gband.opt.statusline_height = 2", Size::new(80, 2));
    client.attach(1, 0);
    assert_eq!(client.status(), "");
    let steps = client
        .controls
        .resize(&mut client.display, Size::new(80, 10));
    assert_eq!(steps.first(), Some(&resize(Size::new(80, 8))));
    assert!(client.status().starts_with("band 1"), "{}", client.status());
}

#[test]
fn shown_panes_follow_the_ribbon() {
    let (_scratch, mut client) = defaults("shown", "", Size::new(80, 24));
    client.attach(3, 0);
    let Some(ClientMessage::Shown(shown)) = client.display.report_shown() else {
        panic!("nothing shown");
    };
    assert_eq!(shown, client.panes[..2]);
}

#[test]
fn reload_turning_the_status_line_off() {
    let (scratch, mut client) = defaults("reload-off", "", Size::new(80, 24));
    client.attach(1, 0);
    let reloaded = scratch.load(&format!(
        "{DEFAULTS}\ngband.opt.statusline_position = 'off'"
    ));
    let steps = client.controls.reload(&mut client.display, reloaded);
    let resizes: Vec<&Step> = steps
        .iter()
        .filter(|step| matches!(step, Step::Send(ClientMessage::Resize { .. })))
        .collect();
    assert_eq!(resizes, [&resize(Size::new(80, 24))]);
    assert_eq!(client.display.status_area(), None);
}

#[test]
fn reload_moving_the_status_line() {
    let (scratch, mut client) = defaults("reload-top", "", Size::new(80, 24));
    client.attach(1, 0);
    let reloaded = scratch.load(&format!(
        "{DEFAULTS}\ngband.opt.statusline_position = 'top'"
    ));
    let steps = client.controls.reload(&mut client.display, reloaded);
    assert!(
        !steps
            .iter()
            .any(|step| matches!(step, Step::Send(ClientMessage::Resize { .. }))),
        "{steps:?}"
    );
    assert_eq!(client.display.status_area(), Some(Rect::new(0, 0, 80, 1)));
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 1, 80, 23));
}

#[test]
fn reload_turning_the_status_line_on_renders_it() {
    let (scratch, mut client) = defaults(
        "reload-on",
        "gband.opt.statusline_position = 'off'",
        Size::new(80, 24),
    );
    client.attach(2, 1);
    let reloaded = scratch.load(DEFAULTS);
    let steps = client.controls.reload(&mut client.display, reloaded);
    assert_eq!(steps.first(), Some(&resize(Size::new(80, 23))));
    assert!(client.status().ends_with(" 2/2"), "{}", client.status());
}

#[test]
fn default_segments_after_attach() {
    let (_scratch, mut client) = defaults("segments", "", Size::new(80, 24));
    client.attach(3, 1);
    let status = client.status();
    assert!(status.starts_with("band 1 "), "{status:?}");
    assert!(status.ends_with(" 2/3"), "{status:?}");
}

#[test]
fn position_follows_focus() {
    let (_scratch, mut client) = defaults("focus", "", Size::new(80, 24));
    client.attach(3, 1);
    client.press("ctrl+space");
    client.press("h");
    assert!(client.status().ends_with(" 1/3"), "{}", client.status());
}

#[test]
fn mode_segment_after_the_prefix() {
    let (_scratch, mut client) = defaults("mode", "", Size::new(80, 24));
    client.attach(3, 1);
    client.press("ctrl+space");
    assert!(
        client.status().starts_with("band 1 │ prefix "),
        "{}",
        client.status()
    );
    client.press("h");
    assert!(
        client.status().starts_with("band 1 "),
        "{}",
        client.status()
    );
    assert!(!client.status().contains("prefix"), "{}", client.status());
}

#[test]
fn width_change_renders_each_component() {
    let (_scratch, mut client) = defaults(
        "width",
        "renders = 0\ngband.ui.statusline.add({ id = 'w', render = function(ctx) renders = renders + 1 width = ctx.total_width end })",
        Size::new(80, 24),
    );
    client.attach(1, 0);
    assert_eq!(client.number("renders"), 1);
    client
        .controls
        .resize(&mut client.display, Size::new(60, 24));
    assert_eq!(client.number("renders"), 2);
    assert_eq!(client.number("width"), 60);
    client
        .controls
        .resize(&mut client.display, Size::new(60, 30));
    assert_eq!(client.number("renders"), 2);
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
    assert!(client.status().contains("tick 2"), "{}", client.status());
}

#[test]
fn plugin_error_at_start() {
    let scratch = Scratch::new("error-start");
    let file = scratch.plugin_file("hello", "plugin/hello.lua", "\n\nerror('boom')");
    let config = scratch.load(DEFAULTS).unwrap();
    let mut client = Client::new(config, Size::new(400, 24));
    client.attach(1, 0);
    let expected = format!("hello: {}:3: boom", file.display());
    assert!(
        client.status().starts_with(&format!("{expected} │ band 1")),
        "{}",
        client.status()
    );
    assert_eq!(client.display.reported_size(), Size::new(400, 23));
}

#[test]
fn cleared_by_a_good_load() {
    let scratch = Scratch::new("error-cleared");
    scratch.plugin_file("hello", "plugin/hello.lua", "error('boom')");
    let config = scratch.load(DEFAULTS).unwrap();
    let mut client = Client::new(config, Size::new(200, 24));
    client.attach(1, 0);
    assert!(
        client.status().starts_with("hello: "),
        "{}",
        client.status()
    );
    fs::remove_dir_all(scratch.0.join("plugins")).unwrap();
    let reloaded = scratch.load(DEFAULTS);
    client.controls.reload(&mut client.display, reloaded);
    assert!(client.status().starts_with("band 1"), "{}", client.status());
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
    assert!(
        client.status().contains("bad binding"),
        "{}",
        client.status()
    );
}

#[test]
fn failing_component_reaches_the_status_line() {
    let (_scratch, mut client) = defaults(
        "error-render",
        "gband.ui.statusline.add({ id = 'bad', render = function() error('render boom') end })",
        Size::new(200, 24),
    );
    client.attach(1, 0);
    assert!(
        client.status().contains("render boom"),
        "{}",
        client.status()
    );
    assert!(client.status().contains("band 1"), "{}", client.status());
}

#[test]
fn plugin_error_on_the_banner() {
    let scratch = Scratch::new("error-banner");
    let file = scratch.plugin_file("hello", "plugin/hello.lua", "\n\nerror('boom')");
    let config = scratch
        .load(&format!(
            "{DEFAULTS}\ngband.opt.statusline_position = 'off'"
        ))
        .unwrap();
    let mut client = Client::new(config, Size::new(200, 6));
    client.attach(1, 0);
    let expected = format!("hello: {}:3: boom", file.display());
    assert_eq!(client.display.banner(), Some(expected.as_str()));
    let screen = client.screen(Size::new(200, 6));
    assert!(screen.contains(&expected), "{screen}");
}

#[test]
fn default_status_line() {
    let (_scratch, mut client) = defaults("snapshot-default", "", Size::new(40, 8));
    client.attach(2, 1);
    assert_snapshot!(client.screen(Size::new(40, 8)));
}

#[test]
fn top_placement() {
    let (_scratch, mut client) = defaults(
        "snapshot-top",
        "gband.opt.statusline_position = 'top'",
        Size::new(40, 8),
    );
    client.attach(2, 1);
    assert_snapshot!(client.screen(Size::new(40, 8)));
}

#[test]
fn narrow_width_drops_and_cuts() {
    let scratch = Scratch::new("snapshot-narrow");
    let config = scratch
        .load(
            "gband.ui.statusline.add({ id = 'low', priority = 1, render = function() return 'dropped' end })\ngband.ui.statusline.add({ id = 'high', priority = 2, align = 'right', render = function() return 'a segment too long to fit' end })",
        )
        .unwrap();
    let mut client = Client::new(config, Size::new(16, 5));
    client.attach(1, 0);
    assert_eq!(client.status(), "a segment too l…");
    assert_snapshot!(client.screen(Size::new(16, 5)));
}

#[test]
fn error_item() {
    let (_scratch, mut client) = defaults("snapshot-error", "", Size::new(40, 6));
    client.attach(1, 0);
    client
        .display
        .set_banner(Some("user/init.lua:3: boom\nmore".to_owned()));
    client.controls.refresh(&mut client.display);
    let status = client.status();
    assert!(
        status.starts_with("user/init.lua:3: boom │ band 1 "),
        "{status:?}"
    );
    assert!(status.ends_with(" 1/1"), "{status:?}");
    assert_snapshot!(client.screen(Size::new(40, 6)));
}
