use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use gband_client::animation::Animations;
use gband_client::{Controls, Display, Step};
use gband_core::geometry::Size;
use gband_core::layout::{Layout, LayoutOptions, WindowId};
use gband_lua::keys::parse_key;
use gband_lua::{Config, LoadOptions, Locations};
use gband_protocol::{ClientMessage, ServerMessage};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("gband-client-bars-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn load(&self, source: &str) -> Config {
        let path = gband_lua::user_file(&self.0.join("config"), gband_lua::Side::Client);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, source).unwrap();
        let locations = Locations {
            config: self.0.join("config"),
            plugins: None,
        };
        let config =
            gband_lua::load(&locations, gband_lua::Side::Client, &LoadOptions::default()).unwrap();
        assert!(config.errors.is_empty(), "{:?}", config.errors);
        config
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Client {
    _scratch: Scratch,
    display: Display,
    controls: Controls,
    windows: Vec<WindowId>,
}

const BINDINGS: &str = "gband.bind('alt+b', function() gband.bar.add({ id = 'side', side = 'left', size = 20, lines = { 'side' } }) end)
gband.bind('alt+r', function() gband.bar.remove('side') end)
gband.bind('alt+m', function() gband.bar.set_config('side', { side = 'right' }) end)
";

impl Client {
    fn new(name: &str, source: &str, columns: usize, focused: usize) -> Self {
        let scratch = Scratch::new(name);
        let config = scratch.load(&format!("{BINDINGS}{source}"));
        let mut display = Display::new(Size::new(80, 24), Animations::Off);
        let mut controls = Controls::new(config, &mut display);
        controls.attached(&mut display, "main");
        let mut layout = Layout::new();
        let band = layout.bands()[0].id;
        let mut windows = Vec::new();
        for _ in 0..columns {
            let window = layout.allocate_window();
            layout.open(
                window,
                band,
                windows.last().copied(),
                None,
                &LayoutOptions::default(),
            );
            windows.push(window);
        }
        let messages = vec![
            ServerMessage::Layout {
                cols: 80,
                rows: 24,
                layout,
            },
            ServerMessage::Focus(windows[focused]),
        ];
        controls.receive(&mut display, messages);
        Self {
            _scratch: scratch,
            display,
            controls,
            windows,
        }
    }

    fn press(&mut self, name: &str) -> Vec<Step> {
        self.controls
            .press(&mut self.display, parse_key(name).unwrap())
    }

    fn screen(&mut self) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal
            .draw(|frame| self.display.draw(frame, Instant::now()))
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        (0..24)
            .map(|y| (0..80).map(|x| buffer[(x, y)].symbol()).collect())
            .collect()
    }
}

fn no_resize(steps: &[Step]) {
    assert!(
        !steps
            .iter()
            .any(|step| matches!(step, Step::Send(ClientMessage::Resize { .. }))),
        "{steps:?}"
    );
}

fn columns(row: &str, from: usize, to: usize) -> String {
    row.chars().skip(from).take(to - from + 1).collect()
}

#[test]
fn ribbon_right_of_a_left_bar() {
    let mut client = Client::new(
        "ribbon-right",
        "gband.bar.add({ id = 'side', side = 'left', size = 20 })",
        1,
        0,
    );
    assert_eq!(client.display.reported_size(), Size::new(80, 24));
    assert_eq!(client.display.ribbon_area(), Rect::new(20, 0, 60, 24));
    let screen = client.screen();
    assert_eq!(columns(&screen[0], 20, 59), format!("┌{}┐", "─".repeat(38)));
    assert_eq!(columns(&screen[0], 0, 19), " ".repeat(20));
}

#[test]
fn bar_added_by_a_key() {
    let mut client = Client::new("added", "", 1, 0);
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 0, 80, 24));
    no_resize(&client.press("alt+b"));
    assert_eq!(client.display.ribbon_area(), Rect::new(20, 0, 60, 24));
    assert_eq!(client.display.reported_size(), Size::new(80, 24));
    let screen = client.screen();
    assert!(screen[0].starts_with("side"), "{}", screen[0]);
    assert_eq!(columns(&screen[0], 20, 20), "┌");
    assert_eq!(columns(&screen[0], 59, 59), "┐");
    let shown: Vec<(&gband_lua::Bar, Rect)> = client.display.bars().collect();
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].1, Rect::new(0, 0, 20, 24));
}

#[test]
fn camera_follows_the_narrower_ribbon() {
    let mut client = Client::new("camera", "", 2, 1);
    let screen = client.screen();
    assert_eq!(columns(&screen[0], 40, 40), "┌");
    no_resize(&client.press("alt+b"));
    let screen = client.screen();
    assert_eq!(columns(&screen[0], 40, 40), "┌");
    assert_eq!(columns(&screen[0], 79, 79), "┐");
    assert_eq!(client.display.focused(), Some(client.windows[1]));
}

#[test]
fn removed_bar_returns_the_columns() {
    let mut client = Client::new("removed", "", 1, 0);
    client.press("alt+b");
    no_resize(&client.press("alt+r"));
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 0, 80, 24));
    assert_eq!(client.display.bars().count(), 0);
}

#[test]
fn moving_a_bar() {
    let mut client = Client::new("moving", "", 1, 0);
    client.press("alt+b");
    no_resize(&client.press("alt+m"));
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 0, 60, 24));
    let screen = client.screen();
    assert_eq!(columns(&screen[0], 60, 63), "side");
}

#[test]
fn moving_the_status_line() {
    let mut client = Client::new(
        "moving-status",
        "gband.plugin('gband.statusline')\ngband.bind('alt+s', function() gband.bar.set_config('statusline', { side = 'right' }) end)",
        1,
        0,
    );
    assert_eq!(client.display.ribbon_area(), Rect::new(20, 0, 60, 24));
    no_resize(&client.press("alt+s"));
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 0, 60, 24));
}

#[test]
fn terminal_resize_places_the_bars_again() {
    let mut client = Client::new(
        "resize",
        "gband.bar.add({ id = 'side', side = 'right', size = 20 })",
        1,
        0,
    );
    let steps = client
        .controls
        .resize(&mut client.display, Size::new(100, 30));
    assert_eq!(
        steps.first(),
        Some(&Step::Send(ClientMessage::Resize {
            cols: 100,
            rows: 30
        }))
    );
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 0, 80, 30));
    client
        .controls
        .resize(&mut client.display, Size::new(20, 30));
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 0, 20, 30));
    assert_eq!(client.display.bars().count(), 0);
}
