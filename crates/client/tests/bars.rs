use std::fs;
use std::time::Instant;

use gband_client::animation::Animations;
use gband_client::{Controls, Display, Step};
use gband_core::geometry::Size;
use gband_core::layout::{Layout, LayoutOptions, SessionAction, WindowId};
use gband_lua::keys::parse_key;
use gband_lua::{Config, LoadOptions, Locations};
use gband_protocol::{ClientMessage, ServerMessage};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

struct Scratch(gband_scratch::Scratch);

impl Scratch {
    fn new(name: &str) -> Self {
        Self(gband_scratch::Scratch::new(
            "client",
            &format!("bars-{name}"),
        ))
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

struct Client {
    _scratch: Scratch,
    display: Display,
    controls: Controls,
    layout: Layout,
    windows: Vec<WindowId>,
    terminal: Size,
}

const BINDINGS: &str = "gband.bind('alt+b', function() gband.bar.add({ id = 'side', side = 'left', size = 20, lines = { 'side' } }) end)
gband.bind('alt+r', function() gband.bar.remove('side') end)
gband.bind('alt+m', function() gband.bar.set_config('side', { side = 'right' }) end)
gband.bind('alt+n', function() gband.bar.set_config('side', { size = 10 }) end)
gband.bind('alt+t', function()
  gband.bar.add({ id = 'one', side = 'left', size = 5 })
  gband.bar.add({ id = 'two', side = 'right', size = 5 })
end)
";

impl Client {
    fn new(name: &str, source: &str, columns: usize, focused: usize) -> Self {
        Self::sized(name, source, Size::new(80, 24), columns, focused)
    }

    fn sized(name: &str, source: &str, terminal: Size, columns: usize, focused: usize) -> Self {
        let scratch = Scratch::new(name);
        let config = scratch.load(&format!("{BINDINGS}{source}"));
        let mut display = Display::new(terminal, Animations::Off);
        let mut controls = Controls::new(config, &mut display);
        controls.place_bars(&mut display);
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
        let mut client = Self {
            _scratch: scratch,
            display,
            controls,
            layout,
            windows,
            terminal,
        };
        client.settle();
        let focus = vec![ServerMessage::Focus(client.windows[focused])];
        client.controls.receive(&mut client.display, focus);
        client
    }

    fn settle(&mut self) {
        let area = self.display.reported_size();
        let messages = vec![ServerMessage::Layout {
            cols: area.cols,
            rows: area.rows,
            layout: self.layout.clone(),
        }];
        self.controls.receive(&mut self.display, messages);
    }

    fn full_width(&mut self, window: WindowId) {
        let area = self.display.reported_size();
        self.layout.apply(
            SessionAction::ToggleFullWidth(window),
            area,
            &LayoutOptions::default(),
        );
        self.settle();
    }

    fn press(&mut self, name: &str) -> Vec<Step> {
        self.controls
            .press(&mut self.display, parse_key(name).unwrap())
    }

    fn screen(&mut self) -> Vec<String> {
        let Size { cols, rows } = self.terminal;
        let mut terminal = Terminal::new(TestBackend::new(cols, rows)).unwrap();
        terminal
            .draw(|frame| self.display.draw(frame, Instant::now()))
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        (0..rows)
            .map(|y| (0..cols).map(|x| buffer[(x, y)].symbol()).collect())
            .collect()
    }
}

fn resizes(steps: &[Step]) -> Vec<Size> {
    steps
        .iter()
        .filter_map(|step| match step {
            Step::Send(ClientMessage::Resize { cols, rows }) => Some(Size::new(*cols, *rows)),
            _ => None,
        })
        .collect()
}

fn columns(row: &str, from: usize, to: usize) -> String {
    row.chars().skip(from).take(to - from + 1).collect()
}

fn tile_top(width: usize) -> String {
    format!("╭{}╮", "─".repeat(width - 2))
}

#[test]
fn ribbon_right_of_a_left_bar() {
    let mut client = Client::new(
        "ribbon-right",
        "gband.bar.add({ id = 'side', side = 'left', size = 20 })",
        1,
        0,
    );
    assert_eq!(client.display.reported_size(), Size::new(60, 24));
    assert_eq!(client.display.ribbon_area(), Rect::new(20, 0, 60, 24));
    let screen = client.screen();
    assert_eq!(columns(&screen[0], 20, 49), tile_top(30));
    assert_eq!(columns(&screen[0], 0, 19), " ".repeat(20));
}

#[test]
fn bar_added_by_a_key() {
    let mut client = Client::new("added", "", 1, 0);
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 0, 80, 24));
    assert_eq!(resizes(&client.press("alt+b")), vec![Size::new(60, 24)]);
    client.settle();
    assert_eq!(client.display.ribbon_area(), Rect::new(20, 0, 60, 24));
    let screen = client.screen();
    assert!(screen[0].starts_with("side"), "{}", screen[0]);
    assert_eq!(columns(&screen[0], 20, 49), tile_top(30));
    let shown: Vec<(&gband_lua::Bar, Rect)> = client.display.bars().collect();
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].1, Rect::new(0, 0, 20, 24));
}

#[test]
fn two_bars_from_one_callback() {
    let mut client = Client::new("two", "", 1, 0);
    assert_eq!(resizes(&client.press("alt+t")), vec![Size::new(70, 24)]);
}

#[test]
fn camera_follows_the_narrower_ribbon() {
    let mut client = Client::new("camera", "", 2, 1);
    let screen = client.screen();
    assert_eq!(columns(&screen[0], 40, 40), "╭");
    assert_eq!(resizes(&client.press("alt+b")), vec![Size::new(60, 24)]);
    client.settle();
    let screen = client.screen();
    assert_eq!(columns(&screen[0], 50, 79), tile_top(30));
    assert!(screen.iter().all(|row| columns(row, 79, 79) != " "));
    assert_eq!(client.display.focused(), Some(client.windows[1]));
}

#[test]
fn full_width_beside_a_bar() {
    let mut client = Client::new(
        "full-width",
        "gband.bar.add({ id = 'side', side = 'left', size = 20 })",
        1,
        0,
    );
    let window = client.windows[0];
    client.full_width(window);
    assert_eq!(client.display.reported_size(), Size::new(60, 24));
    let screen = client.screen();
    assert_eq!(columns(&screen[0], 20, 79), tile_top(60));
}

#[test]
fn full_width_follows_the_bars_size() {
    let mut client = Client::new(
        "full-width-size",
        "gband.bar.add({ id = 'side', side = 'left', size = 20 })",
        1,
        0,
    );
    let window = client.windows[0];
    client.full_width(window);
    assert_eq!(resizes(&client.press("alt+n")), vec![Size::new(70, 24)]);
    client.settle();
    let screen = client.screen();
    assert_eq!(columns(&screen[0], 10, 79), tile_top(70));
}

#[test]
fn removed_bar_returns_the_columns() {
    let mut client = Client::new("removed", "", 1, 0);
    client.press("alt+b");
    assert_eq!(resizes(&client.press("alt+r")), vec![Size::new(80, 24)]);
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 0, 80, 24));
    assert_eq!(client.display.bars().count(), 0);
}

#[test]
fn moving_a_bar() {
    let mut client = Client::new("moving", "", 1, 0);
    client.press("alt+b");
    assert_eq!(resizes(&client.press("alt+m")), Vec::new());
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 0, 60, 24));
    let screen = client.screen();
    assert_eq!(columns(&screen[0], 60, 63), "side");
}

#[test]
fn moving_the_sidebar() {
    let mut client = Client::new(
        "moving-sidebar",
        "gband.plugin('gband.sidebar')\ngband.bind('alt+s', function() gband.bar.set_config('sidebar', { side = 'right' }) end)",
        1,
        0,
    );
    assert_eq!(client.display.ribbon_area(), Rect::new(1, 0, 79, 24));
    assert_eq!(resizes(&client.press("alt+s")), Vec::new());
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 0, 79, 24));
}

#[test]
fn bar_too_wide_to_show() {
    let client = Client::sized(
        "too-wide",
        "gband.bar.add({ id = 'side', side = 'left', size = 10 })",
        Size::new(10, 24),
        1,
        0,
    );
    assert_eq!(client.display.bars().count(), 0);
    assert_eq!(client.display.reported_size(), Size::new(10, 24));
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
        Some(&Step::Send(ClientMessage::Resize { cols: 80, rows: 30 }))
    );
    assert_eq!(resizes(&steps).len(), 1);
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 0, 80, 30));
    let steps = client
        .controls
        .resize(&mut client.display, Size::new(20, 30));
    assert_eq!(resizes(&steps), vec![Size::new(20, 30)]);
    assert_eq!(client.display.ribbon_area(), Rect::new(0, 0, 20, 30));
    assert_eq!(client.display.bars().count(), 0);
}
