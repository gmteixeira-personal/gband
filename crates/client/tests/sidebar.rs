use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use gband_client::animation::Animations;
use gband_client::{Controls, Display, Step};
use gband_core::geometry::Size;
use gband_core::input::{Modifiers, MouseButton, MouseEvent, MouseKind, WheelDirection};
use gband_core::layout::{Layout, LayoutOptions, WindowId};
use gband_lua::keys::parse_key;
use gband_lua::{Bar, Color, Config, ConfigError, DEFAULTS, LoadOptions, Locations, Style};
use gband_protocol::{ClientMessage, ServerMessage};
use insta::assert_snapshot;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

struct Scratch(gband_scratch::Scratch);

impl Scratch {
    fn new(name: &str) -> Self {
        Self(gband_scratch::Scratch::new(
            "client",
            &format!("sidebar-{name}"),
        ))
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
        self.load_styled(source, "modal")
    }

    fn load_styled(&self, source: &str, style: &str) -> Result<Config, ConfigError> {
        write(
            &self.0.join("config").join("user").join("keystyle.lua"),
            &format!("return \"{style}\"\n"),
        );
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

fn write(path: &Path, source: &str) -> PathBuf {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
    path.to_path_buf()
}

struct Client {
    display: Display,
    controls: Controls,
    layout: Layout,
    windows: Vec<WindowId>,
}

impl Client {
    fn new(config: Config, terminal: Size) -> Self {
        let mut display = Display::new(terminal, Animations::Off);
        let mut controls = Controls::new(config, &mut display);
        controls.place_bars(&mut display);
        Self {
            display,
            controls,
            layout: Layout::new(),
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
        self.layout = layout;
        steps.extend(self.send_layout());
        if let Some(&window) = self.windows.get(focused) {
            steps.extend(self.focus(window));
        }
        steps
    }

    fn attach_bands(&mut self, bands: usize) {
        self.controls.attached(&mut self.display, "main");
        let mut layout = Layout::new();
        for index in 0..bands {
            let band = layout.bands()[index].id;
            let window = layout.allocate_window();
            layout.open(window, band, None, None, &LayoutOptions::default());
            self.windows.push(window);
        }
        self.layout = layout;
        self.send_layout();
        self.focus(self.windows[0]);
    }

    fn open_in_band(&mut self, index: usize) {
        let band = self.layout.bands()[index].id;
        let window = self.layout.allocate_window();
        self.layout
            .open(window, band, None, None, &LayoutOptions::default());
        self.windows.push(window);
        self.send_layout();
    }

    fn close(&mut self, window: WindowId) {
        self.layout.remove(window);
        self.windows.retain(|&kept| kept != window);
        self.send_layout();
    }

    fn send_layout(&mut self) -> Vec<Step> {
        let reported = self.display.reported_size();
        let message = ServerMessage::Layout {
            cols: reported.cols,
            rows: reported.rows,
            layout: self.layout.clone(),
        };
        self.controls
            .receive(&mut self.display, vec![message])
            .steps
    }

    fn focus(&mut self, window: WindowId) -> Vec<Step> {
        self.controls
            .receive(&mut self.display, vec![ServerMessage::Focus(window)])
            .steps
    }

    fn press(&mut self, name: &str) -> Vec<Step> {
        self.controls
            .press(&mut self.display, parse_key(name).unwrap())
    }

    fn sidebar(&self) -> Option<(&Bar, Rect)> {
        self.display.bars().find(|(bar, _)| bar.id == "sidebar")
    }

    fn sidebar_area(&self) -> Option<Rect> {
        self.sidebar().map(|(_, area)| area)
    }

    fn rows(&self) -> Vec<(usize, String)> {
        self.sidebar()
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
        self.rows()
            .into_iter()
            .find(|(at, _)| *at == row)
            .map(|(_, text)| text)
            .unwrap_or_default()
    }

    fn click(&mut self, button: MouseButton, col: u16, row: u16) {
        for kind in [MouseKind::Press(button), MouseKind::Release(button)] {
            let event = MouseEvent::new(kind, col, row, Modifiers::NONE);
            self.controls
                .mouse(&mut self.display, event, Instant::now());
        }
    }

    fn wheel(&mut self, direction: WheelDirection, col: u16, row: u16, modifiers: Modifiers) {
        let event = MouseEvent::new(MouseKind::Wheel(direction), col, row, modifiers);
        self.controls
            .mouse(&mut self.display, event, Instant::now());
    }

    fn viewed(&self) -> u32 {
        self.display.view_state("root").band.index
    }

    fn style(&self, row: usize) -> Style {
        let (bar, _) = self.sidebar().expect("the sidebar is shown");
        bar.lines[row][0].style
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

const SETUP: &str = "gband.plugin(\"gband.sidebar\")";

fn without_sidebar() -> String {
    DEFAULTS.replace(SETUP, "")
}

fn defaults(name: &str, extra: &str, terminal: Size) -> (Scratch, Client) {
    let scratch = Scratch::new(name);
    let config = scratch.load(&format!("{DEFAULTS}\n{extra}")).unwrap();
    (scratch, Client::new(config, terminal))
}

fn line(row: usize, text: &str) -> (usize, String) {
    (row, text.to_owned())
}

#[test]
fn rows_of_a_new_session() {
    let (_scratch, mut client) = defaults("new-session", "", Size::new(80, 24));
    assert_eq!(client.display.reported_size(), Size::new(79, 24));
    let steps = client.attach(1, 0);
    assert!(
        !steps
            .iter()
            .any(|step| matches!(step, Step::Send(ClientMessage::Resize { .. }))),
        "{steps:?}"
    );
    assert_eq!(client.sidebar_area(), Some(Rect::new(0, 0, 1, 24)));
    assert_eq!(client.display.ribbon_area(), Rect::new(1, 0, 79, 24));
    assert_eq!(client.rows(), [line(0, "I"), line(2, "1"), line(3, "2")]);
}

#[test]
fn short_terminal() {
    let (_scratch, mut client) = defaults("short", "", Size::new(80, 3));
    client.attach(1, 0);
    assert_eq!(client.rows(), [line(0, "I")]);
}

#[test]
fn terminal_background() {
    let (_scratch, mut client) = defaults("background", "", Size::new(80, 24));
    client.attach(1, 0);
    let (bar, _) = client.sidebar().expect("the sidebar is shown");
    assert_eq!(bar.base, Style::default());
    for runs in &bar.lines {
        for run in runs {
            assert_eq!(run.style.bg, None, "{run:?}");
        }
    }
}

#[test]
fn navigation_mode() {
    let (_scratch, mut client) = defaults("navigation", "", Size::new(80, 24));
    client.attach(1, 0);
    client.press("ctrl+space");
    assert_eq!(client.row(0), "N");
}

#[test]
fn back_to_interactive_mode() {
    let (_scratch, mut client) = defaults("back", "", Size::new(80, 24));
    client.attach(1, 0);
    client.press("ctrl+space");
    client.press("escape");
    assert_eq!(client.row(0), "I");
}

#[test]
fn prefix_without_a_mode() {
    let scratch = Scratch::new("prefix-direct");
    let config = scratch.load_styled(DEFAULTS, "direct").unwrap();
    let mut client = Client::new(config, Size::new(80, 24));
    client.attach(1, 0);
    client.press("ctrl+space");
    assert_eq!(client.row(0), "P");
    client.press("h");
    assert_eq!(client.row(0), "I");
}

#[test]
fn user_mode() {
    let (_scratch, mut client) = defaults(
        "user-mode",
        "gband.keymap.mode('resize', { label = 'resize' })\ngband.keymap.set('resize', 'h', function() end)\ngband.bind('alt+r', function() gband.keymap.enter('resize') end)",
        Size::new(80, 24),
    );
    client.attach(1, 0);
    client.press("alt+r");
    assert_eq!(client.row(0), "R");
}

#[test]
fn window_opened_in_the_empty_band() {
    let (_scratch, mut client) = defaults("opened", "", Size::new(80, 24));
    client.attach(1, 0);
    client.open_in_band(1);
    assert_eq!(
        client.rows(),
        [line(0, "I"), line(2, "1"), line(3, "2"), line(4, "3")]
    );
}

#[test]
fn band_removed() {
    let (_scratch, mut client) = defaults("removed", "", Size::new(80, 24));
    client.attach_bands(2);
    assert_eq!(client.row(4), "3");
    let second = client.windows[1];
    client.close(second);
    assert_eq!(client.rows(), [line(0, "I"), line(2, "1"), line(3, "2")]);
}

#[test]
fn tenth_band() {
    let (_scratch, mut client) = defaults("tenth", "", Size::new(80, 24));
    client.attach_bands(10);
    let labels: Vec<String> = (2..=12).map(|row| client.row(row)).collect();
    assert_eq!(
        labels,
        ["1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b"]
    );
}

#[test]
fn more_bands_than_rows() {
    let (_scratch, mut client) = defaults("more-bands", "", Size::new(80, 8));
    client.attach_bands(5);
    assert_eq!(
        client.rows(),
        [
            line(0, "I"),
            line(2, "1"),
            line(3, "2"),
            line(4, "3"),
            line(5, "4"),
            line(6, "5")
        ]
    );
}

#[test]
fn viewed_band() {
    let (_scratch, mut client) = defaults("viewed", "", Size::new(80, 24));
    client.attach_bands(2);
    assert!(client.style(2).bold);
    assert_eq!(client.style(2).fg, Some(Color::Rgb(0xc0, 0xca, 0xf5)));
    for row in [3, 4] {
        assert!(!client.style(row).bold, "{row}");
        assert_eq!(client.style(row).fg, Some(Color::Rgb(0x56, 0x5f, 0x89)));
    }
}

#[test]
fn view_moves_down() {
    let (_scratch, mut client) = defaults("view-down", "", Size::new(80, 24));
    client.attach_bands(2);
    let below = client.windows[1];
    client.focus(below);
    assert!(client.style(3).bold);
    assert!(!client.style(2).bold);
    assert!(!client.style(4).bold);
}

#[test]
fn error_reported() {
    let (scratch, mut client) = defaults("error-reported", "", Size::new(80, 24));
    client.attach(1, 0);
    client
        .controls
        .reload(&mut client.display, scratch.load("local = 1"));
    assert_eq!(client.row(23), "!");
    assert_eq!(client.style(23).fg, Some(Color::Rgb(0xf7, 0x76, 0x8e)));
    assert!(client.display.banner().is_some());
    let screen = client.screen(Size::new(80, 24));
    assert!(!screen.contains("init.lua"), "{screen}");
}

#[test]
fn error_cleared() {
    let scratch = Scratch::new("error-cleared");
    scratch.client_plugin("hello", "error('boom')");
    let config = scratch.load(&format!("{DEFAULTS}\n{CLEAR}")).unwrap();
    let mut client = Client::new(config, Size::new(80, 24));
    client.attach(1, 0);
    assert_eq!(client.row(23), "!");
    client.press("alt+x");
    assert_eq!(client.row(23), "");
}

#[test]
fn sidebar_hidden() {
    let (_scratch, mut client) = defaults("hidden", "", Size::new(1, 6));
    client.attach(1, 0);
    client.display.report_error("boom".to_owned());
    client.controls.refresh(&mut client.display);
    assert_eq!(client.sidebar_area(), None);
    let screen = client.screen(Size::new(1, 6));
    assert!(screen.contains("\"b\""), "{screen}");
}

#[test]
fn theme_the_viewed_band() {
    let (_scratch, mut client) = defaults(
        "theme",
        "gband.hl.set('SidebarBandActive', { fg = 2 })",
        Size::new(80, 24),
    );
    client.attach(1, 0);
    assert_eq!(client.style(2).fg, Some(Color::Index(2)));
}

#[test]
fn turning_the_sidebar_off() {
    let (scratch, mut client) = defaults("off", "", Size::new(80, 24));
    client.attach(1, 0);
    let steps = client
        .controls
        .reload(&mut client.display, scratch.load(&without_sidebar()));
    let resizes: Vec<&Step> = steps
        .iter()
        .filter(|step| matches!(step, Step::Send(ClientMessage::Resize { .. })))
        .collect();
    assert_eq!(
        resizes,
        [&Step::Send(ClientMessage::Resize { cols: 80, rows: 24 })]
    );
    client.send_layout();
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 0, 80, 24));
    assert_eq!(client.sidebar_area(), None);
    let screen = client.screen(Size::new(80, 24));
    let top = format!("\"┌{}┐{}\"", "─".repeat(38), " ".repeat(40));
    assert!(screen.contains(&top), "{screen}");
}

#[test]
fn failed_reload_keeps_the_bars() {
    let (scratch, mut client) = defaults("reload-failed", "", Size::new(80, 24));
    client.attach(1, 0);
    client
        .controls
        .reload(&mut client.display, scratch.load("local = 1"));
    assert_eq!(client.sidebar_area(), Some(Rect::new(0, 0, 1, 24)));
    assert_eq!(client.row(23), "!");
}

#[test]
fn plugin_error_at_start() {
    let scratch = Scratch::new("error-start");
    let file = scratch.client_plugin("hello", "\n\nerror('boom')");
    let config = scratch.load(DEFAULTS).unwrap();
    let mut client = Client::new(config, Size::new(200, 24));
    client.attach(1, 0);
    let expected = format!("hello: {}:3: boom", file.display());
    assert_eq!(client.row(23), "!");
    assert_eq!(client.row(2), "1");
    assert_eq!(client.display.errors(), std::slice::from_ref(&expected));
    let listed: Vec<String> = client
        .controls
        .runtime()
        .lua()
        .load("return gband.errors()")
        .eval()
        .unwrap();
    assert_eq!(listed, std::slice::from_ref(&expected));
    assert_eq!(client.display.reported_size(), Size::new(199, 24));
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
    let scratch = Scratch::new("error-good-load");
    scratch.client_plugin("hello", "error('boom')");
    let config = scratch.load(DEFAULTS).unwrap();
    let mut client = Client::new(config, Size::new(200, 24));
    client.attach(1, 0);
    assert_eq!(client.row(23), "!");
    fs::remove_dir_all(scratch.0.join("plugins")).unwrap();
    let reloaded = scratch.load(DEFAULTS);
    client.controls.reload(&mut client.display, reloaded);
    assert_eq!(client.row(23), "");
    assert!(client.display.errors().is_empty());
}

const CLEAR: &str =
    "gband.bind('alt+x', function() gband.clear_errors() seen = #gband.errors() end)";

#[test]
fn cleared_by_hand() {
    let scratch = Scratch::new("cleared-by-hand");
    scratch.client_plugin("alpha", "error('first')");
    scratch.client_plugin("beta", "error('second')");
    let config = scratch.load(&format!("{DEFAULTS}\n{CLEAR}")).unwrap();
    let mut client = Client::new(config, Size::new(200, 24));
    client.attach(1, 0);
    assert_eq!(client.display.errors().len(), 2);
    assert_eq!(client.row(23), "!");
    client.press("alt+x");
    assert_eq!(client.number("seen"), 0);
    assert!(client.display.errors().is_empty());
    assert_eq!(client.display.banner(), None);
    assert_eq!(client.row(23), "");
}

#[test]
fn banner_cleared_by_hand() {
    let scratch = Scratch::new("banner-cleared");
    scratch.client_plugin("hello", "error('boom')");
    let config = scratch
        .load(&format!("{}\n{CLEAR}", without_sidebar()))
        .unwrap();
    let mut client = Client::new(config, Size::new(200, 6));
    client.attach(1, 0);
    assert!(client.screen(Size::new(200, 6)).contains("boom"));
    client.press("alt+x");
    assert_eq!(client.display.banner(), None);
    let screen = client.screen(Size::new(200, 6));
    assert!(!screen.contains("boom"), "{screen}");
}

#[test]
fn error_after_a_clear() {
    let scratch = Scratch::new("error-after-clear");
    scratch.client_plugin("hello", "error('first')");
    let config = scratch
        .load(&format!(
            "{DEFAULTS}\n{CLEAR}\ngband.bind('alt+f', function() error('boom') end)"
        ))
        .unwrap();
    let mut client = Client::new(config, Size::new(200, 24));
    client.attach(1, 0);
    client.press("alt+x");
    assert!(client.display.errors().is_empty());
    assert_eq!(client.row(23), "");
    client.press("alt+f");
    let errors = client.display.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("boom"), "{errors:?}");
    assert_eq!(client.row(23), "!");
}

#[test]
fn error_in_the_clearing_callback() {
    let scratch = Scratch::new("error-in-clearing");
    scratch.client_plugin("hello", "error('first')");
    let source = format!(
        "{DEFAULTS}\ngband.bind('alt+x', function()\n  gband.clear_errors()\n  error('late')\nend)"
    );
    let line = source
        .lines()
        .position(|text| text.contains("'late'"))
        .unwrap()
        + 1;
    let config = scratch.load(&source).unwrap();
    let mut client = Client::new(config, Size::new(200, 24));
    client.attach(1, 0);
    client.press("alt+x");
    let errors = client.display.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].contains(&format!("init.lua:{line}: late")),
        "{errors:?}"
    );
    assert_eq!(client.row(23), "!");
}

#[test]
fn plugin_error_on_the_banner() {
    let scratch = Scratch::new("error-banner");
    let file = scratch.client_plugin("hello", "\n\nerror('boom')");
    let config = scratch.load(&without_sidebar()).unwrap();
    let mut client = Client::new(config, Size::new(200, 6));
    client.attach(1, 0);
    let expected = format!("hello: {}:3: boom", file.display());
    assert_eq!(client.display.banner(), Some(expected.as_str()));
    let screen = client.screen(Size::new(200, 6));
    assert!(screen.contains(&expected), "{screen}");
}

#[test]
fn default_sidebar_snapshot() {
    let (_scratch, mut client) = defaults("snapshot-default", "", Size::new(40, 8));
    client.attach(2, 1);
    assert_snapshot!(client.screen(Size::new(40, 8)));
}

#[test]
fn error_marker_snapshot() {
    let (_scratch, mut client) = defaults("snapshot-error", "", Size::new(40, 6));
    client.attach(1, 0);
    client
        .display
        .report_error("user/init.lua:3: boom\nmore".to_owned());
    client.controls.refresh(&mut client.display);
    assert_eq!(
        client.rows(),
        [line(0, "I"), line(2, "1"), line(3, "2"), line(5, "!")]
    );
    assert_snapshot!(client.screen(Size::new(40, 6)));
}

#[test]
fn click_a_band_label() {
    let (_scratch, mut client) = defaults("click-label", "", Size::new(80, 24));
    client.attach_bands(2);
    assert_eq!(client.viewed(), 1);
    client.click(MouseButton::Left, 0, 4);
    assert_eq!(client.viewed(), 3);
    assert!(client.style(4).bold);
    assert!(!client.style(2).bold);
}

#[test]
fn click_on_the_right_sidebar() {
    let scratch = Scratch::new("click-right");
    let source = DEFAULTS.replace(
        SETUP,
        "gband.plugin(\"gband.sidebar\", { side = \"right\" })",
    );
    let config = scratch.load(&source).unwrap();
    let mut client = Client::new(config, Size::new(80, 24));
    client.attach_bands(1);
    client.click(MouseButton::Left, 79, 3);
    assert_eq!(client.viewed(), 2);
}

#[test]
fn click_beside_the_labels() {
    let (_scratch, mut client) = defaults("click-beside", "", Size::new(80, 24));
    client.attach_bands(2);
    for row in [0, 1, 5, 23] {
        client.click(MouseButton::Left, 0, row);
        assert_eq!(client.viewed(), 1, "{row}");
    }
}

#[test]
fn right_click_on_a_label() {
    let (_scratch, mut client) = defaults("click-right-button", "", Size::new(80, 24));
    client.attach_bands(1);
    client.click(MouseButton::Right, 0, 3);
    assert_eq!(client.viewed(), 1);
}

#[test]
fn click_in_navigation_mode() {
    let (_scratch, mut client) = defaults("click-navigation", "", Size::new(80, 24));
    client.attach_bands(2);
    client.press("ctrl+space");
    client.click(MouseButton::Left, 0, 3);
    assert_eq!(client.viewed(), 2);
    assert_eq!(client.row(0), "N");
}

#[test]
fn wheel_down_over_the_labels() {
    let (_scratch, mut client) = defaults("wheel-down", "", Size::new(80, 24));
    client.attach_bands(2);
    client.wheel(WheelDirection::Down, 0, 2, Modifiers::NONE);
    assert_eq!(client.viewed(), 2);
    assert!(client.style(3).bold);
    assert!(!client.style(2).bold);
}

#[test]
fn wheel_steps_move_one_band_each() {
    let (_scratch, mut client) = defaults("wheel-steps", "", Size::new(80, 24));
    client.attach_bands(2);
    client.wheel(WheelDirection::Down, 0, 2, Modifiers::NONE);
    client.wheel(WheelDirection::Down, 0, 2, Modifiers::NONE);
    assert_eq!(client.viewed(), 3);
}

#[test]
fn wheel_up() {
    let (_scratch, mut client) = defaults("wheel-up", "", Size::new(80, 24));
    client.attach_bands(2);
    client.click(MouseButton::Left, 0, 4);
    assert_eq!(client.viewed(), 3);
    client.wheel(WheelDirection::Up, 0, 2, Modifiers::NONE);
    assert_eq!(client.viewed(), 2);
}

#[test]
fn wheel_on_any_row() {
    for row in [0, 1, 23] {
        let (_scratch, mut client) = defaults(&format!("wheel-row-{row}"), "", Size::new(80, 24));
        client.attach_bands(1);
        client.wheel(WheelDirection::Down, 0, row, Modifiers::NONE);
        assert_eq!(client.viewed(), 2, "{row}");
    }
}

#[test]
fn wheel_past_the_last_band() {
    let (_scratch, mut client) = defaults("wheel-last", "", Size::new(80, 24));
    client.attach_bands(1);
    client.click(MouseButton::Left, 0, 3);
    assert_eq!(client.viewed(), 2);
    client.wheel(WheelDirection::Down, 0, 2, Modifiers::NONE);
    assert_eq!(client.viewed(), 2);
}

#[test]
fn wheel_on_the_right_sidebar() {
    let scratch = Scratch::new("wheel-right");
    let source = DEFAULTS.replace(
        SETUP,
        "gband.plugin(\"gband.sidebar\", { side = \"right\" })",
    );
    let config = scratch.load(&source).unwrap();
    let mut client = Client::new(config, Size::new(80, 24));
    client.attach_bands(1);
    client.wheel(WheelDirection::Down, 79, 2, Modifiers::NONE);
    assert_eq!(client.viewed(), 2);
}

#[test]
fn wheel_with_alt_held() {
    let (_scratch, mut client) = defaults("wheel-alt", "", Size::new(80, 24));
    client.attach_bands(2);
    client.wheel(WheelDirection::Down, 0, 2, Modifiers::ALT);
    assert_eq!(client.viewed(), 2);
}

#[test]
fn wheel_in_navigation_mode() {
    let (_scratch, mut client) = defaults("wheel-navigation", "", Size::new(80, 24));
    client.attach_bands(1);
    client.press("ctrl+space");
    client.wheel(WheelDirection::Down, 0, 2, Modifiers::NONE);
    assert_eq!(client.viewed(), 2);
    assert_eq!(client.row(0), "N");
}
