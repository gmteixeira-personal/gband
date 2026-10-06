use std::collections::HashMap;
use std::time::Instant;

use gband_client::animation::{Animations, Drawn, DrawnBand, Presentation, Targets};
use gband_client::color::ColorSupport;
use gband_client::render::{Ribbon, draw_frame};
use gband_core::geometry::{Size, boxes, tiles};
use gband_core::layout::{
    Direction, Layout, LayoutOptions, Proportion, SessionAction, Step, WindowHeight, WindowId,
};
use gband_core::view::{CenterFocusedColumn, Scene, View, ViewAction};
use gband_emulator::{Emulator, Grid};
use gband_lua::plugin_windows::{FloatingFrame, Run};
use gband_lua::{Color, Style};
use insta::assert_snapshot;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

struct Fixture {
    layout: Layout,
    area: Size,
    view: View,
    grids: HashMap<WindowId, Grid>,
    banner: Option<String>,
    floats: Vec<FloatingFrame>,
    float_focused: bool,
}

impl Fixture {
    fn new(area: Size, columns: usize, viewport_cols: u16) -> (Self, Vec<WindowId>) {
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
        let view = View::new(Scene {
            layout: &layout,
            area,
            viewport: Size::new(viewport_cols, area.rows),
        });
        let mut fixture = Self {
            layout,
            area,
            view,
            grids: HashMap::new(),
            banner: None,
            floats: Vec::new(),
            float_focused: false,
        };
        fixture.reset_grids();
        (fixture, windows)
    }

    fn reset_grids(&mut self) {
        self.grids.clear();
        for band in self.layout.bands() {
            for tile in tiles(band, self.area) {
                self.grids
                    .insert(tile.window, Grid::new(tile.terminal_size()));
            }
            for placed in boxes(band, self.area) {
                self.grids
                    .insert(placed.window, Grid::new(placed.terminal_size()));
            }
        }
    }

    fn change(&mut self, action: SessionAction, viewport_cols: u16) {
        self.layout
            .apply(action, self.area, &LayoutOptions::default());
        let scene = Scene {
            layout: &self.layout,
            area: self.area,
            viewport: Size::new(viewport_cols, self.area.rows),
        };
        self.view.sync(scene);
        self.reset_grids();
    }

    fn act(&mut self, action: ViewAction, viewport_cols: u16) {
        let scene = Scene {
            layout: &self.layout,
            area: self.area,
            viewport: Size::new(viewport_cols, self.area.rows),
        };
        self.view.apply(action, scene);
    }

    fn write(&mut self, window: WindowId, bytes: &[u8]) {
        self.grids.get_mut(&window).unwrap().process(bytes);
    }

    fn at_rest(&self, terminal: Size) -> Drawn {
        let now = Instant::now();
        let mut presentation = Presentation::new(Animations::On);
        presentation.update(
            now,
            &Targets::new(&self.layout, self.area, &self.view, terminal),
        );
        presentation.drawn(now)
    }

    fn render(&self, terminal: Size) -> String {
        self.render_drawn(terminal, &self.at_rest(terminal))
    }

    fn render_drawn(&self, terminal: Size, drawn: &Drawn) -> String {
        let mut terminal = Terminal::new(TestBackend::new(terminal.cols, terminal.rows)).unwrap();
        let terminal_area = terminal.size().unwrap();
        let ribbon = Ribbon {
            layout: &self.layout,
            area: self.area,
            view: &self.view,
            grids: &self.grids,
            drawn,
            region: Rect::new(0, 0, terminal_area.width, terminal_area.height),
            floats: self.floats.iter().collect(),
            float_focused: self.float_focused,
            colors: ColorSupport::Indexed,
            banner: self.banner.as_deref(),
            status: None,
        };
        terminal.draw(|frame| draw_frame(frame, &ribbon)).unwrap();
        let backend = terminal.backend();
        let cursor = backend.cursor_visible().then(|| backend.cursor_position());
        format!("{:?}\ncursor: {cursor:?}", backend.buffer())
    }
}

#[test]
fn two_columns_side_by_side() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 2, 80);
    fixture.write(windows[0], b"left");
    fixture.write(windows[1], b"right");
    fixture.act(ViewAction::FocusRight, 80);
    assert_snapshot!(fixture.render(Size::new(80, 24)));
}

#[test]
fn column_clipped_at_the_left_edge() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 2, 80);
    for &window in &windows {
        fixture.change(SessionAction::CycleWidth(window), 80);
    }
    fixture.write(windows[0], "0123456789".repeat(5).as_bytes());
    fixture.act(ViewAction::FocusRight, 80);
    assert_eq!(fixture.view.camera(), 26);
    assert_snapshot!(fixture.render(Size::new(80, 24)));
}

#[test]
fn tile_larger_than_the_terminal_is_cut() {
    let (fixture, _) = Fixture::new(Size::new(120, 40), 1, 100);
    assert_snapshot!(fixture.render(Size::new(100, 30)));
}

#[test]
fn empty_band_is_blank_without_a_cursor() {
    let (mut fixture, _) = Fixture::new(Size::new(80, 24), 1, 80);
    fixture.act(ViewAction::BandDown, 80);
    assert_eq!(fixture.view.focused(), None);
    assert_snapshot!(fixture.render(Size::new(80, 24)));
}

#[test]
fn hidden_window_cursor_hides_the_cursor() {
    let (mut fixture, windows) = Fixture::new(Size::new(40, 6), 1, 40);
    assert_snapshot!("cursor_shown", fixture.render(Size::new(40, 6)));
    fixture.write(windows[0], b"\x1b[?25l");
    assert_snapshot!("cursor_hidden", fixture.render(Size::new(40, 6)));
}

#[test]
fn offscreen_column_is_not_drawn() {
    let (fixture, _) = Fixture::new(Size::new(80, 24), 3, 80);
    assert_snapshot!(fixture.render(Size::new(80, 24)));
}

#[test]
fn coloured_window() {
    let (mut fixture, windows) = Fixture::new(Size::new(60, 6), 1, 60);
    fixture.write(
        windows[0],
        b"\x1b[1;31mred\x1b[0m \x1b[42mgreen\x1b[0m \x1b[4;38;2;10;20;30mrgb\x1b[0m",
    );
    assert_snapshot!(fixture.render(Size::new(60, 6)));
}

#[test]
fn wide_character_cut_at_the_clip_edge() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 8), 2, 80);
    for &window in &windows {
        fixture.change(SessionAction::CycleWidth(window), 80);
    }
    fixture.write(windows[0], "漢字".repeat(13).as_bytes());
    fixture.act(ViewAction::FocusRight, 80);
    assert_eq!(fixture.view.camera(), 26);
    assert_snapshot!(fixture.render(Size::new(80, 8)));
}

#[test]
fn stacked_column() {
    let (mut fixture, windows) = Fixture::new(Size::new(60, 12), 2, 60);
    fixture.change(
        SessionAction::ConsumeOrExpel {
            window: windows[1],
            direction: Direction::Left,
        },
        60,
    );
    fixture.write(windows[0], b"top");
    fixture.write(windows[1], b"bottom");
    fixture.act(ViewAction::FocusDown, 60);
    assert_eq!(fixture.view.focused(), Some(windows[1]));
    assert_snapshot!(fixture.render(Size::new(60, 12)));
}

#[test]
fn column_wider_than_the_terminal_shows_its_left_border() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 1, 80);
    for _ in 0..6 {
        fixture.change(
            SessionAction::StepWidth {
                window: windows[0],
                step: Step::Grow,
            },
            80,
        );
    }
    assert_eq!(tiles(&fixture.layout.bands()[0], fixture.area)[0].width, 88);
    assert_eq!(fixture.view.camera(), 0);
    fixture.write(windows[0], "0123456789".repeat(9).as_bytes());
    assert_snapshot!(fixture.render(Size::new(80, 24)));
}

#[test]
fn grid_smaller_than_its_tile() {
    let (mut fixture, windows) = Fixture::new(Size::new(100, 30), 1, 100);
    fixture
        .grids
        .insert(windows[0], Grid::new(Size::new(38, 22)));
    fixture.write(
        windows[0],
        format!("{}\r\nsmall", "s".repeat(38)).as_bytes(),
    );
    assert_snapshot!(fixture.render(Size::new(100, 30)));
}

#[test]
fn grid_larger_than_its_tile() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 1, 80);
    fixture
        .grids
        .insert(windows[0], Grid::new(Size::new(48, 28)));
    for row in 0..28 {
        let line = format!("{row:02}{}", "L".repeat(46));
        fixture.write(windows[0], format!("\x1b[{};1H{line}", row + 1).as_bytes());
    }
    fixture.write(windows[0], b"\x1b[28;48H");
    assert_snapshot!(fixture.render(Size::new(80, 24)));
}

#[test]
fn tile_morphing_wider_than_its_grid() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 12), 2, 80);
    fixture.write(windows[0], "w".repeat(38).as_bytes());
    let mut drawn = fixture.at_rest(Size::new(80, 12));
    drawn.tiles.get_mut(&windows[0]).unwrap().width = 47;
    drawn.settled = false;
    assert_snapshot!(fixture.render_drawn(Size::new(80, 12), &drawn));
}

#[test]
fn tile_morphing_narrower_than_its_grid() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 12), 2, 80);
    fixture.write(windows[0], "0123456789".repeat(4).as_bytes());
    let mut drawn = fixture.at_rest(Size::new(80, 12));
    drawn.tiles.get_mut(&windows[0]).unwrap().width = 26;
    drawn.settled = false;
    assert_snapshot!(fixture.render_drawn(Size::new(80, 12), &drawn));
}

#[test]
fn mid_scroll_frame() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 12), 3, 80);
    fixture.write(windows[1], b"second");
    fixture.act(ViewAction::FocusRight, 80);
    fixture.act(ViewAction::FocusRight, 80);
    assert_eq!(fixture.view.camera(), 40);
    let mut drawn = fixture.at_rest(Size::new(80, 12));
    drawn.bands[0].camera = 20;
    drawn.settled = false;
    let screen = fixture.render_drawn(Size::new(80, 12), &drawn);
    assert!(screen.ends_with("cursor: None"));
    assert_snapshot!(screen);
}

#[test]
fn band_switch_mid_slide() {
    let (mut fixture, windows) = Fixture::new(Size::new(60, 12), 1, 60);
    let below = fixture.layout.bands()[1].id;
    let window = fixture.layout.allocate_window();
    fixture
        .layout
        .open(window, below, None, None, &LayoutOptions::default());
    fixture.reset_grids();
    fixture.write(windows[0], b"upper");
    fixture.write(window, b"lower");
    fixture.act(ViewAction::BandDown, 60);
    let above = fixture.layout.bands()[0].id;
    let mut drawn = fixture.at_rest(Size::new(60, 12));
    drawn.bands = vec![
        DrawnBand {
            band: above,
            top: -5,
            camera: 0,
        },
        DrawnBand {
            band: below,
            top: 7,
            camera: 0,
        },
    ];
    drawn.settled = false;
    assert_snapshot!(fixture.render_drawn(Size::new(60, 12), &drawn));
}

#[test]
fn focused_tile_is_drawn_over_overlapping_tiles() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 8), 2, 80);
    fixture.write(windows[0], b"focused");
    fixture.write(windows[1], b"other");
    assert_eq!(fixture.view.focused(), Some(windows[0]));
    let mut drawn = fixture.at_rest(Size::new(80, 8));
    drawn.tiles.get_mut(&windows[1]).unwrap().x = 20;
    drawn.settled = false;
    assert_snapshot!(fixture.render_drawn(Size::new(80, 8), &drawn));
}

#[test]
fn centred_first_column_leaves_the_strip_start_empty() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 1, 80);
    fixture.view = View::with_policy(
        Scene {
            layout: &fixture.layout,
            area: fixture.area,
            viewport: Size::new(80, 24),
        },
        CenterFocusedColumn::Always,
    );
    fixture.write(windows[0], b"centred");
    assert_eq!(fixture.view.camera(), -20);
    assert_snapshot!(fixture.render(Size::new(80, 24)));
}

#[test]
fn configuration_error_banner_covers_the_bottom_row() {
    let (mut fixture, windows) = Fixture::new(Size::new(40, 6), 1, 40);
    fixture.write(windows[0], b"$ ");
    fixture.banner = Some(
        "/home/u/.config/gband/init.lua:12: unexpected symbol near 'x'\nstack traceback".to_owned(),
    );
    assert_snapshot!(fixture.render(Size::new(40, 6)));
}

fn float(row: u16, col: u16, width: u16, height: u16, lines: &[&str]) -> FloatingFrame {
    let base = Style::default();
    let inner = usize::from(width.saturating_sub(2));
    FloatingFrame {
        row,
        col,
        width,
        height,
        border: true,
        title: Some("Keys".to_owned()),
        base,
        border_style: Style {
            fg: Some(Color::Index(8)),
            ..base
        },
        title_style: Style {
            fg: Some(Color::Index(8)),
            bold: true,
            ..base
        },
        lines: (0..height.saturating_sub(2))
            .map(|row| {
                let text = lines.get(usize::from(row)).copied().unwrap_or("");
                vec![Run {
                    text: format!("{text:<inner$}"),
                    style: base,
                }]
            })
            .collect(),
        z: 1,
        focused: true,
    }
}

#[test]
fn float_over_two_tiles() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 2, 80);
    fixture.write(windows[0], b"left side of the screen");
    fixture.write(windows[1], b"right side of the screen");
    fixture.floats.push(float(
        6,
        20,
        40,
        11,
        &["C-h  focus left", "C-l  focus right"],
    ));
    fixture.float_focused = true;
    assert_snapshot!(fixture.render(Size::new(80, 24)));
}

#[test]
fn cursor_returns_when_no_float_is_focused() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 2, 80);
    fixture.write(windows[0], b"$ ");
    fixture.floats.push(float(6, 20, 40, 11, &[]));
    assert!(
        fixture
            .render(Size::new(80, 24))
            .ends_with("cursor: Some(Position { x: 3, y: 1 })")
    );
    fixture.float_focused = true;
    assert!(fixture.render(Size::new(80, 24)).ends_with("cursor: None"));
}

#[test]
fn error_banner_over_a_float() {
    let (mut fixture, windows) = Fixture::new(Size::new(40, 8), 1, 40);
    fixture.write(windows[0], b"$ ");
    fixture.floats.push(float(4, 5, 30, 4, &["covered"]));
    fixture.banner = Some("init.lua:3: broken".to_owned());
    assert_snapshot!(fixture.render(Size::new(40, 8)));
}

#[test]
fn centered_float() {
    let (fixture, _) = Fixture::new(Size::new(80, 23), 1, 80);
    let mut fixture = fixture;
    fixture.floats.push(float(6, 20, 40, 11, &["a", "b", "c"]));
    assert_snapshot!(fixture.render(Size::new(80, 23)));
}

#[test]
fn later_float_is_drawn_on_top() {
    let (mut fixture, _) = Fixture::new(Size::new(40, 10), 1, 40);
    let mut lower = float(1, 1, 20, 6, &["lower"]);
    lower.z = 1;
    let mut upper = float(3, 10, 20, 6, &["upper"]);
    upper.z = 2;
    fixture.floats = vec![lower, upper];
    assert_snapshot!(fixture.render(Size::new(40, 10)));
}

fn floating_at(fixture: &mut Fixture, window: WindowId, col: u16, row: u16, viewport_cols: u16) {
    let height = WindowHeight::Fixed(12);
    for action in [
        SessionAction::ToggleFloating {
            window,
            after: None,
        },
        SessionAction::SetWidth {
            window,
            width: Proportion::ONE_HALF,
        },
        SessionAction::SetHeight { window, height },
        SessionAction::SetPosition { window, col, row },
    ] {
        fixture.change(action, viewport_cols);
    }
}

#[test]
fn floating_window_over_two_tiles() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 3, 80);
    floating_at(&mut fixture, windows[2], 20, 6, 80);
    fixture.write(windows[0], b"left side of the screen");
    fixture.write(windows[1], b"right side of the screen");
    fixture.write(windows[2], b"floating");
    assert_eq!(fixture.view.focused(), Some(windows[0]));
    assert_snapshot!(fixture.render(Size::new(80, 24)));
}

#[test]
fn floating_window_ignores_the_camera() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 4, 80);
    floating_at(&mut fixture, windows[3], 20, 6, 80);
    for (index, &window) in windows.iter().enumerate() {
        fixture.write(window, format!("window {index}").as_bytes());
    }
    let at_zero = fixture.render(Size::new(80, 24));
    fixture.act(ViewAction::FocusRight, 80);
    fixture.act(ViewAction::FocusRight, 80);
    assert_eq!(fixture.view.camera(), 40);
    let at_forty = fixture.render(Size::new(80, 24));
    let rows = |screen: &str| -> Vec<String> {
        screen
            .lines()
            .skip(7)
            .take(12)
            .map(|line| line.chars().skip(21).take(40).collect())
            .collect()
    };
    assert_eq!(rows(&at_zero), rows(&at_forty));
    assert_snapshot!(at_forty);
}

#[test]
fn float_over_a_floating_window() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 2, 80);
    floating_at(&mut fixture, windows[1], 20, 6, 80);
    fixture.write(windows[1], b"floating window under the float");
    fixture.floats.push(float(8, 30, 30, 6, &["on top"]));
    fixture.float_focused = true;
    assert_snapshot!(fixture.render(Size::new(80, 24)));
}

#[test]
fn focused_floating_window_holds_the_cursor() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 2, 80);
    floating_at(&mut fixture, windows[1], 20, 6, 80);
    fixture.act(ViewAction::SwitchLayer, 80);
    fixture.write(windows[1], b"$ ");
    assert!(
        fixture
            .render(Size::new(80, 24))
            .ends_with("cursor: Some(Position { x: 23, y: 7 })")
    );
}

#[test]
fn cursor_under_a_floating_window() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 2, 80);
    floating_at(&mut fixture, windows[1], 0, 0, 80);
    fixture.write(windows[0], b"$ ");
    assert_eq!(fixture.view.focused(), Some(windows[0]));
    assert!(fixture.render(Size::new(80, 24)).ends_with("cursor: None"));
    fixture.change(
        SessionAction::SetPosition {
            window: windows[1],
            col: 40,
            row: 0,
        },
        80,
    );
    fixture.write(windows[0], b"$ ");
    assert!(
        fixture
            .render(Size::new(80, 24))
            .ends_with("cursor: Some(Position { x: 3, y: 1 })")
    );
}
