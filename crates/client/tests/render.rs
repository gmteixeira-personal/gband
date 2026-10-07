use std::collections::HashMap;
use std::time::Instant;

use gband_client::animation::{Animations, Drawn, DrawnBand, Presentation, Targets};
use gband_client::color::ColorSupport;
use gband_client::mouse::{Hit, Target, hit};
use gband_client::render::{
    Lifted, Overlay, Ribbon, Selected, Shown, draw_border, draw_frame, interior, regions,
};
use gband_core::geometry::{Size, boxes, tiles};
use gband_core::layout::{
    Direction, Layout, LayoutOptions, Proportion, SessionAction, Step, WindowHeight, WindowId,
};
use gband_core::view::{CenterFocusedColumn, Scene, View, ViewAction};
use gband_emulator::{Emulator, Grid};
use gband_lua::plugin_windows::{FloatingFrame, Run};
use gband_lua::{
    Bar, BarSide, Border, BorderChars, CharSet, ClientStyles, Color, Options, Palette, Sides, Slot,
    Style, WindowName, WindowNames,
};
use insta::assert_snapshot;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

struct Fixture {
    layout: Layout,
    area: Size,
    view: View,
    grids: HashMap<WindowId, Grid>,
    banner: Option<String>,
    floats: Vec<FloatingFrame>,
    float_focused: bool,
    tile_border: Border,
    floating_border: Border,
    focused_tile_chars: BorderChars,
    focused_floating_chars: BorderChars,
    styles: ClientStyles,
    palette: Palette,
    colors: ColorSupport,
    bars: Vec<(Bar, Rect)>,
    region: Option<Rect>,
    overlay: Overlay,
    titles: Option<WindowNames>,
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
            tile_border: Options::default().tile_border,
            floating_border: Options::default().floating_border,
            focused_tile_chars: Options::default().focused_tile_border_chars,
            focused_floating_chars: Options::default().focused_floating_border_chars,
            styles: ClientStyles::default(),
            palette: Palette::default(),
            colors: ColorSupport::Indexed,
            bars: Vec::new(),
            region: None,
            overlay: Overlay::default(),
            titles: None,
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

    fn hit(&self, terminal: Size, col: u16, row: u16) -> Hit {
        let drawn = self.at_rest(terminal);
        let area = Rect::new(0, 0, terminal.cols, terminal.rows);
        let ribbon = self.ribbon(area, &drawn);
        let regions = regions(&ribbon, area);
        hit(
            &regions,
            ribbon.region,
            self.view.band(),
            |_| None,
            col,
            row,
        )
    }

    fn screen(&self, terminal: Size) -> Buffer {
        let drawn = self.at_rest(terminal);
        let mut backend = Terminal::new(TestBackend::new(terminal.cols, terminal.rows)).unwrap();
        let area = backend.size().unwrap();
        let ribbon = self.ribbon(area.into(), &drawn);
        backend.draw(|frame| draw_frame(frame, &ribbon)).unwrap();
        backend.backend().buffer().clone()
    }

    fn render_drawn(&self, terminal: Size, drawn: &Drawn) -> String {
        let mut terminal = Terminal::new(TestBackend::new(terminal.cols, terminal.rows)).unwrap();
        let terminal_area = terminal.size().unwrap();
        let ribbon = self.ribbon(terminal_area.into(), drawn);
        terminal.draw(|frame| draw_frame(frame, &ribbon)).unwrap();
        let backend = terminal.backend();
        let cursor = backend.cursor_visible().then(|| backend.cursor_position());
        format!("{:?}\ncursor: {cursor:?}", backend.buffer())
    }

    fn ribbon<'a>(&'a self, terminal_area: Rect, drawn: &'a Drawn) -> Ribbon<'a> {
        Ribbon {
            layout: &self.layout,
            area: self.area,
            view: &self.view,
            grids: &self.grids,
            drawn,
            region: self.region.unwrap_or(Rect::new(
                0,
                0,
                terminal_area.width,
                terminal_area.height,
            )),
            tile_border: &self.tile_border,
            floating_border: &self.floating_border,
            focused_tile_chars: &self.focused_tile_chars,
            focused_floating_chars: &self.focused_floating_chars,
            styles: self.styles,
            palette: self.palette,
            floats: self
                .floats
                .iter()
                .enumerate()
                .map(|(index, float)| (index as u32 + 1, float))
                .collect(),
            float_focused: self.float_focused,
            colors: self.colors,
            banner: self.banner.as_deref(),
            bars: self
                .bars
                .iter()
                .map(|(bar, area)| Shown { bar, area: *area })
                .collect(),
            overlay: self.overlay,
            titles: self.titles.as_ref(),
        }
    }

    fn name(&mut self, window: WindowId, name: &str) {
        self.titles.get_or_insert_default().insert(
            window,
            WindowName {
                shown: name.to_owned(),
                manual: None,
            },
        );
    }
}

fn row_text(screen: &Buffer, row: u16) -> String {
    (0..screen.area.width)
        .map(|col| screen[(col, row)].symbol())
        .collect()
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
                by: Proportion::TENTH,
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
            strip: None,
        },
        DrawnBand {
            band: below,
            top: 7,
            camera: 0,
            strip: None,
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
        border: Some(Border::default()),
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
            floating: None,
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
fn first_column_drawn_after_the_last() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 3, 80);
    for (index, &window) in windows.iter().enumerate() {
        fixture.write(window, format!("window {index}").as_bytes());
    }
    for _ in 0..3 {
        fixture.act(ViewAction::FocusRight, 80);
    }
    assert_eq!(fixture.view.camera(), 80);
    assert_eq!(fixture.view.focused(), Some(windows[0]));
    let screen = fixture.render(Size::new(80, 24));
    let top = screen.lines().nth(4).unwrap();
    assert!(top.contains("window 2") && top.contains("window 0"));
    assert!(top.find("window 2") < top.find("window 0"));
    assert!(!screen.contains("window 1"));
    assert_snapshot!(screen);
}

#[test]
fn floating_window_stays_in_place_across_the_seam() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 4, 80);
    floating_at(&mut fixture, windows[3], 10, 6, 80);
    for (index, &window) in windows.iter().enumerate() {
        fixture.write(window, format!("window {index}").as_bytes());
    }
    fixture.act(ViewAction::FocusRight, 80);
    fixture.act(ViewAction::FocusRight, 80);
    assert_eq!(fixture.view.camera(), 40);
    let terminal = Size::new(80, 24);
    let mut presentation = Presentation::new(Animations::On);
    let targets =
        |fixture: &Fixture| Targets::new(&fixture.layout, fixture.area, &fixture.view, terminal);
    presentation.update(Instant::now(), &targets(&fixture));
    let at_forty = fixture.render(terminal);
    fixture.act(ViewAction::FocusRight, 80);
    assert_eq!(fixture.view.camera(), 80);
    let start = Instant::now();
    presentation.update(start, &targets(&fixture));
    let mut middle = presentation.drawn(start + std::time::Duration::from_millis(50));
    middle.settled = false;
    let camera = middle.bands[0].camera;
    assert!(40 < camera && camera < 80, "{camera}");
    let mid_scroll = fixture.render_drawn(terminal, &middle);
    let at_eighty = fixture.render(terminal);
    let rows = |screen: &str| -> Vec<String> {
        screen
            .lines()
            .skip(9)
            .take(12)
            .map(|line| line.chars().skip(19).take(40).collect())
            .collect()
    };
    assert_eq!(rows(&at_forty), rows(&mid_scroll));
    assert_eq!(rows(&at_forty), rows(&at_eighty));
    assert_snapshot!(at_eighty);
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

fn sides(names: &[&str]) -> Sides {
    Sides::parse(names.iter().copied()).unwrap()
}

fn bordered(width: u16, height: u16, sides: Sides, chars: CharSet) -> (Buffer, Rect) {
    let area = Rect::new(0, 0, width, height);
    let mut buffer = Buffer::filled(area, ratatui::buffer::Cell::new("x"));
    let border = Border {
        sides,
        chars: BorderChars::Named(chars),
    };
    draw_border(&mut buffer, area, &border, ratatui::style::Style::default());
    (buffer, interior(area))
}

fn rows(buffer: &Buffer) -> Vec<String> {
    let area = buffer.area;
    (area.top()..area.bottom())
        .map(|y| {
            (area.left()..area.right())
                .map(|x| buffer[(x, y)].symbol())
                .collect()
        })
        .collect()
}

#[test]
fn only_the_left_side() {
    let (buffer, inner) = bordered(10, 5, sides(&["left"]), CharSet::Plain);
    assert_eq!(
        rows(&buffer),
        [
            "│         ",
            "│xxxxxxxx ",
            "│xxxxxxxx ",
            "│xxxxxxxx ",
            "│         ",
        ]
    );
    assert_eq!(inner, Rect::new(1, 1, 8, 3));
}

#[test]
fn top_and_left_sides() {
    let (buffer, _) = bordered(10, 5, sides(&["top", "left"]), CharSet::Rounded);
    assert_eq!(
        rows(&buffer),
        [
            "╭─────────",
            "│xxxxxxxx ",
            "│xxxxxxxx ",
            "│xxxxxxxx ",
            "│         ",
        ]
    );
}

#[test]
fn no_sides() {
    let (buffer, inner) = bordered(6, 4, Sides::NONE, CharSet::Double);
    assert_eq!(rows(&buffer), ["      ", " xxxx ", " xxxx ", "      "]);
    assert_eq!(inner, Rect::new(1, 1, 4, 2));
}

#[test]
fn every_named_set() {
    for (chars, expected) in [
        (CharSet::Plain, ["┌──┐", "│xx│", "└──┘"]),
        (CharSet::Rounded, ["╭──╮", "│xx│", "╰──╯"]),
        (CharSet::Double, ["╔══╗", "║xx║", "╚══╝"]),
        (CharSet::Thick, ["┏━━┓", "┃xx┃", "┗━━┛"]),
    ] {
        let (buffer, _) = bordered(4, 3, Sides::ALL, chars);
        assert_eq!(rows(&buffer), expected, "{chars:?}");
    }
}

#[test]
fn custom_characters() {
    let area = Rect::new(0, 0, 4, 3);
    let mut buffer = Buffer::filled(area, ratatui::buffer::Cell::new("x"));
    let border = Border {
        sides: Sides::ALL,
        chars: BorderChars::custom(
            ["+", "-", "+", "|", "+", "-", "+", "|"]
                .map(str::to_owned)
                .to_vec(),
        )
        .unwrap(),
    };
    draw_border(&mut buffer, area, &border, ratatui::style::Style::default());
    assert_eq!(rows(&buffer), ["+--+", "|xx|", "+--+"]);
}

#[test]
fn tiles_without_side_borders() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 1, 80);
    fixture.tile_border.sides = sides(&["top", "bottom"]);
    fixture.write(windows[0], b"$ ");
    assert_eq!(fixture.grids[&windows[0]].size(), Size::new(38, 22));
    assert_snapshot!(fixture.render(Size::new(80, 24)));
}

#[test]
fn floating_windows_differ_from_tiles() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 3, 80);
    floating_at(&mut fixture, windows[2], 20, 6, 80);
    fixture.floating_border.chars = BorderChars::Named(CharSet::Double);
    fixture.write(windows[2], b"floating");
    assert_snapshot!(fixture.render(Size::new(80, 24)));
}

#[test]
fn floating_plugin_window_with_a_rounded_top_only() {
    let (mut fixture, _) = Fixture::new(Size::new(40, 10), 1, 40);
    let mut frame = float(2, 10, 20, 5, &["inside"]);
    frame.title = None;
    frame.border = Some(Border {
        sides: sides(&["top"]),
        chars: BorderChars::Named(CharSet::Rounded),
    });
    fixture.floats.push(frame);
    fixture.float_focused = true;
    assert_snapshot!(fixture.render(Size::new(40, 10)));
}

#[test]
fn title_over_an_undrawn_top_side() {
    let (mut fixture, _) = Fixture::new(Size::new(40, 10), 1, 40);
    let mut frame = float(2, 10, 20, 5, &["inside"]);
    frame.title = Some("list".to_owned());
    frame.border = Some(Border {
        sides: sides(&["left", "right"]),
        chars: BorderChars::default(),
    });
    fixture.floats.push(frame);
    fixture.float_focused = true;
    assert_snapshot!(fixture.render(Size::new(40, 10)));
}

fn bar(size: u16, base: Style, lines: Vec<Vec<Run>>) -> Bar {
    Bar {
        id: "side".to_owned(),
        slot: Slot {
            side: BarSide::Left,
            size,
            order: 0.0,
            seq: 1,
        },
        base,
        lines,
    }
}

#[test]
fn styled_bar() {
    let (mut fixture, windows) = Fixture::new(Size::new(40, 6), 1, 30);
    fixture.write(windows[0], b"$ ");
    let base = Style {
        bg: Some(Color::Index(236)),
        ..Style::default()
    };
    let title = Style { bold: true, ..base };
    fixture.bars.push((
        bar(
            10,
            base,
            vec![vec![Run {
                text: "gband".to_owned(),
                style: title,
            }]],
        ),
        Rect::new(0, 0, 10, 6),
    ));
    fixture.region = Some(Rect::new(10, 0, 30, 6));
    assert_snapshot!(fixture.render(Size::new(40, 6)));
}

#[test]
fn line_cut_at_the_bars_edge() {
    let (mut fixture, _) = Fixture::new(Size::new(20, 6), 1, 16);
    let run = |text: &str| {
        vec![Run {
            text: text.to_owned(),
            style: Style::default(),
        }]
    };
    fixture.bars.push((
        bar(4, Style::default(), vec![run("one"), run("thre"), run("x")]),
        Rect::new(0, 0, 4, 6),
    ));
    fixture.region = Some(Rect::new(4, 0, 16, 6));
    assert_snapshot!(fixture.render(Size::new(20, 6)));
}

#[test]
fn click_inside_a_tile() {
    let (fixture, windows) = Fixture::new(Size::new(80, 24), 2, 80);
    let hit = fixture.hit(Size::new(80, 24), 45, 3);
    assert_eq!(hit.target, Target::Window(windows[1]));
    assert_eq!(hit.content, Some((4, 2)));
}

#[test]
fn floating_window_over_a_tile_is_the_target() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 3, 80);
    floating_at(&mut fixture, windows[2], 40, 2, 80);
    let hit = fixture.hit(Size::new(80, 24), 45, 3);
    assert_eq!(hit.target, Target::Window(windows[2]));
    assert_eq!(hit.content, Some((4, 0)));
}

#[test]
fn border_cell_has_no_content_cell() {
    let (fixture, windows) = Fixture::new(Size::new(80, 24), 2, 80);
    let hit = fixture.hit(Size::new(80, 24), 50, 0);
    assert_eq!(hit.target, Target::Window(windows[1]));
    assert_eq!(hit.content, None);
}

#[test]
fn empty_ribbon_left_of_the_first_column() {
    let (mut fixture, _) = Fixture::new(Size::new(80, 24), 1, 80);
    fixture.view = View::with_policy(
        Scene {
            layout: &fixture.layout,
            area: fixture.area,
            viewport: Size::new(80, 24),
        },
        CenterFocusedColumn::Always,
    );
    assert_eq!(fixture.view.camera(), -20);
    assert_eq!(fixture.hit(Size::new(80, 24), 5, 3).target, Target::Ribbon);
}

#[test]
fn plugin_float_is_above_windows_and_outside_is_outside() {
    let (mut fixture, _) = Fixture::new(Size::new(80, 24), 2, 80);
    fixture.floats.push(float(6, 20, 40, 11, &[]));
    fixture.region = Some(Rect::new(10, 0, 70, 24));
    let hit = fixture.hit(Size::new(80, 24), 31, 7);
    assert_eq!(hit.target, Target::PluginFloat(1));
    assert_eq!(hit.content, Some((0, 0)));
    assert_eq!(fixture.hit(Size::new(80, 24), 3, 3).target, Target::Outside);
}

#[test]
fn selection_is_drawn_reversed() {
    let (mut fixture, windows) = Fixture::new(Size::new(40, 6), 1, 40);
    fixture.write(windows[0], b"hello world\r\nsecond line");
    fixture.overlay.selection = Some(Selected {
        window: windows[0],
        start: (6, 0),
        end: (5, 1),
    });
    assert_snapshot!(fixture.render(Size::new(40, 6)));
}

#[test]
fn lifted_tile_with_its_drop_outline() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 12), 2, 80);
    fixture.write(windows[0], b"lifted");
    fixture.write(windows[1], b"stays");
    fixture.overlay.lifted = Some(Lifted {
        window: windows[0],
        x: 50,
        y: 2,
        width: 20,
        height: 6,
        outline: Some(Rect::new(70, 0, 10, 12)),
    });
    assert_snapshot!(fixture.render(Size::new(80, 12)));
}

fn sidebar(rows: u16) -> (Bar, Rect) {
    let label = |text: &str| {
        vec![Run {
            text: text.to_owned(),
            style: Style::default(),
        }]
    };
    (
        bar(1, Style::default(), vec![label("I"), label(""), label("1")]),
        Rect::new(0, 0, 1, rows),
    )
}

#[test]
fn tile_taller_than_the_ribbon_area() {
    let (mut fixture, _) = Fixture::new(Size::new(120, 40), 1, 99);
    fixture.bars.push(sidebar(30));
    fixture.region = Some(Rect::new(1, 0, 99, 30));
    let screen = fixture.render(Size::new(100, 30));
    assert!(screen.contains("\"I╭"), "{screen}");
    assert_snapshot!(screen);
}

#[test]
fn ribbon_beside_the_sidebar() {
    let (mut fixture, _) = Fixture::new(Size::new(80, 24), 2, 79);
    fixture.bars.push(sidebar(24));
    fixture.region = Some(Rect::new(1, 0, 79, 24));
    assert_snapshot!(fixture.render(Size::new(80, 24)));
}

fn corners(buffer: &Buffer, left: u16, top: u16, right: u16, bottom: u16) -> [&str; 4] {
    [
        buffer[(left, top)].symbol(),
        buffer[(right, top)].symbol(),
        buffer[(left, bottom)].symbol(),
        buffer[(right, bottom)].symbol(),
    ]
}

const ROUNDED: [&str; 4] = ["╭", "╮", "╰", "╯"];

#[test]
fn rounded_by_default() {
    let (fixture, _) = Fixture::new(Size::new(80, 24), 2, 80);
    let screen = fixture.screen(Size::new(80, 24));
    assert_eq!(corners(&screen, 0, 0, 39, 23), ROUNDED);
    assert_eq!(corners(&screen, 40, 0, 79, 23), ROUNDED);
}

#[test]
fn focused_tile_with_its_own_characters() {
    let (mut fixture, _) = Fixture::new(Size::new(80, 24), 2, 80);
    fixture.act(ViewAction::FocusRight, 80);
    fixture.focused_tile_chars = BorderChars::Named(CharSet::Thick);
    let screen = fixture.screen(Size::new(80, 24));
    assert_eq!(corners(&screen, 40, 0, 79, 23), ["┏", "┓", "┗", "┛"]);
    assert_eq!(screen[(50, 0)].symbol(), "━");
    assert_eq!(screen[(40, 5)].symbol(), "┃");
    assert_eq!(corners(&screen, 0, 0, 39, 23), ROUNDED);
    fixture.act(ViewAction::FocusLeft, 80);
    let screen = fixture.screen(Size::new(80, 24));
    assert_eq!(corners(&screen, 0, 0, 39, 23), ["┏", "┓", "┗", "┛"]);
    assert_eq!(corners(&screen, 40, 0, 79, 23), ROUNDED);
}

#[test]
fn focused_floating_window_with_its_own_characters() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 3, 80);
    floating_at(&mut fixture, windows[1], 0, 0, 80);
    floating_at(&mut fixture, windows[2], 40, 8, 80);
    fixture.act(ViewAction::FocusWindow(windows[2]), 80);
    fixture.focused_floating_chars = BorderChars::Named(CharSet::Double);
    let screen = fixture.screen(Size::new(80, 24));
    assert_eq!(corners(&screen, 40, 8, 79, 19), ["╔", "╗", "╚", "╝"]);
    assert_eq!(corners(&screen, 0, 0, 39, 11), ROUNDED);
}

#[test]
fn focused_characters_keep_the_sides() {
    let (mut fixture, _) = Fixture::new(Size::new(80, 24), 1, 80);
    fixture.tile_border.sides = sides(&["left"]);
    fixture.focused_tile_chars = BorderChars::Named(CharSet::Thick);
    let screen = fixture.screen(Size::new(80, 24));
    for row in 0..24 {
        assert_eq!(screen[(0, row)].symbol(), "┃", "row {row}");
        assert_eq!(screen[(39, row)].symbol(), " ", "row {row}");
    }
    for col in 1..39 {
        assert_eq!(screen[(col, 0)].symbol(), " ", "col {col}");
        assert_eq!(screen[(col, 23)].symbol(), " ", "col {col}");
    }
}

#[test]
fn drop_outline_uses_the_focused_characters() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 12), 2, 80);
    fixture.focused_tile_chars = BorderChars::Named(CharSet::Double);
    fixture.colors = ColorSupport::TrueColor;
    fixture.overlay.lifted = Some(Lifted {
        window: windows[0],
        x: 50,
        y: 2,
        width: 20,
        height: 6,
        outline: Some(Rect::new(0, 0, 10, 12)),
    });
    let screen = fixture.screen(Size::new(80, 12));
    assert_eq!(corners(&screen, 0, 0, 9, 11), ["╔", "╗", "╚", "╝"]);
    let cell = &screen[(0, 0)];
    assert_eq!(cell.fg, ratatui::style::Color::Rgb(0xb1, 0xb9, 0xf9));
    assert!(cell.modifier.contains(ratatui::style::Modifier::BOLD));
}

#[test]
fn built_in_border_look() {
    let (mut fixture, _) = Fixture::new(Size::new(80, 24), 2, 80);
    fixture.act(ViewAction::FocusRight, 80);
    fixture.colors = ColorSupport::TrueColor;
    let screen = fixture.screen(Size::new(80, 24));
    let focused = &screen[(40, 0)];
    assert_eq!(focused.fg, ratatui::style::Color::Rgb(0xb1, 0xb9, 0xf9));
    assert!(focused.modifier.contains(ratatui::style::Modifier::BOLD));
    let other = &screen[(0, 0)];
    assert_eq!(other.fg, ratatui::style::Color::Reset);
    assert!(other.modifier.contains(ratatui::style::Modifier::DIM));
}

#[test]
fn colored_focused_border() {
    let (mut fixture, _) = Fixture::new(Size::new(80, 24), 2, 80);
    fixture.act(ViewAction::FocusRight, 80);
    fixture.styles.border_focused = Style {
        fg: Some(Color::Index(2)),
        bold: true,
        ..Style::default()
    };
    let screen = fixture.screen(Size::new(80, 24));
    assert_eq!(screen[(40, 0)].fg, ratatui::style::Color::Indexed(2));
    assert!(
        screen[(40, 0)]
            .modifier
            .contains(ratatui::style::Modifier::BOLD)
    );
    assert!(
        screen[(0, 0)]
            .modifier
            .contains(ratatui::style::Modifier::DIM)
    );
}

#[test]
fn banner_group() {
    let (mut fixture, _) = Fixture::new(Size::new(40, 6), 1, 40);
    fixture.colors = ColorSupport::TrueColor;
    fixture.styles.banner = Style {
        fg: Some(Color::Rgb(0xff, 0xff, 0xff)),
        bg: Some(Color::Rgb(0xaa, 0x00, 0x00)),
        ..Style::default()
    };
    fixture.banner = Some("boom".to_owned());
    let screen = fixture.screen(Size::new(40, 6));
    let cell = &screen[(0, 5)];
    assert_eq!(cell.symbol(), "b");
    assert_eq!(cell.fg, ratatui::style::Color::Rgb(0xff, 0xff, 0xff));
    assert_eq!(cell.bg, ratatui::style::Color::Rgb(0xaa, 0x00, 0x00));
    assert!(!cell.modifier.contains(ratatui::style::Modifier::REVERSED));
}

#[test]
fn palette_covers_tiles_bars_and_the_empty_ribbon() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 1, 79);
    fixture.colors = ColorSupport::TrueColor;
    fixture.palette.bg = Some((0x28, 0x28, 0x28));
    fixture.palette.colors[1] = Some((0xcc, 0x24, 0x1d));
    fixture.bars.push(sidebar(24));
    fixture.region = Some(Rect::new(1, 0, 79, 24));
    fixture.write(windows[0], b"\x1b[31mx\x1b[0m");
    let screen = fixture.screen(Size::new(80, 24));
    let background = ratatui::style::Color::Rgb(0x28, 0x28, 0x28);
    let x = &screen[(2, 1)];
    assert_eq!(x.symbol(), "x");
    assert_eq!(x.fg, ratatui::style::Color::Rgb(0xcc, 0x24, 0x1d));
    assert_eq!(x.bg, background);
    assert_eq!(screen[(60, 10)].bg, background);
    assert_eq!(screen[(0, 10)].bg, background);
}

#[test]
fn title_on_a_tile() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 1, 80);
    fixture.name(windows[0], "vim");
    let screen = fixture.screen(Size::new(80, 24));
    assert_eq!(
        &row_text(&screen, 0)[..],
        format!("╭vim{}╮{}", "─".repeat(35), " ".repeat(40))
    );
}

#[test]
fn long_title_cut() {
    let (mut fixture, windows) = Fixture::new(Size::new(20, 6), 1, 20);
    fixture.name(windows[0], "cargo test --workspace");
    let screen = fixture.screen(Size::new(20, 6));
    assert_eq!(
        row_text(&screen, 0),
        format!("╭cargo te╮{}", " ".repeat(10))
    );
}

#[test]
fn wide_title_cut_leaves_out_whole_characters() {
    let (mut fixture, windows) = Fixture::new(Size::new(20, 6), 1, 20);
    fixture.name(windows[0], "ab漢字漢字");
    let screen = fixture.screen(Size::new(20, 6));
    let symbols: Vec<&str> = [1, 2, 3, 5, 7, 9]
        .into_iter()
        .map(|col| screen[(col, 0)].symbol())
        .collect();
    assert_eq!(symbols, ["a", "b", "漢", "字", "漢", "╮"]);
}

#[test]
fn focused_title_styled_as_the_focused_border() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 2, 80);
    fixture.name(windows[0], "one");
    fixture.name(windows[1], "two");
    fixture.styles.border_focused = Style {
        fg: Some(Color::Index(2)),
        bold: true,
        ..Style::default()
    };
    let screen = fixture.screen(Size::new(80, 24));
    let focused = fixture.view.focused().unwrap();
    let (focused_col, other_col) = if focused == windows[0] {
        (1, 41)
    } else {
        (41, 1)
    };
    assert_eq!(
        screen[(focused_col, 0)].fg,
        ratatui::style::Color::Indexed(2)
    );
    assert!(
        screen[(focused_col, 0)]
            .modifier
            .contains(ratatui::style::Modifier::BOLD)
    );
    assert_eq!(screen[(other_col, 0)].fg, ratatui::style::Color::Reset);
    assert!(
        screen[(other_col, 0)]
            .modifier
            .contains(ratatui::style::Modifier::DIM)
    );
    assert_eq!(
        screen[(focused_col, 0)].style(),
        screen[(focused_col - 1, 0)].style()
    );
    assert_eq!(
        screen[(other_col, 0)].style(),
        screen[(other_col - 1, 0)].style()
    );
}

#[test]
fn no_title_without_a_top_side() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 1, 80);
    fixture.name(windows[0], "vim");
    fixture.tile_border.sides = Sides {
        top: false,
        ..Sides::ALL
    };
    let screen = fixture.screen(Size::new(80, 24));
    assert!(!row_text(&screen, 0).contains("vim"));
}

#[test]
fn no_title_while_titles_are_off() {
    let (fixture, _) = Fixture::new(Size::new(80, 24), 1, 80);
    let screen = fixture.screen(Size::new(80, 24));
    assert_eq!(
        row_text(&screen, 0),
        format!("╭{}╮{}", "─".repeat(38), " ".repeat(40))
    );
}

#[test]
fn title_of_a_clipped_column() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 2, 80);
    for &window in &windows {
        fixture.change(SessionAction::CycleWidth(window), 80);
    }
    let title = "0123456789".repeat(5);
    fixture.name(windows[0], &title);
    fixture.act(ViewAction::FocusRight, 80);
    assert_eq!(fixture.view.camera(), 26);
    let screen = fixture.screen(Size::new(80, 24));
    assert!(row_text(&screen, 0).starts_with(&title[25..]));
}

#[test]
fn title_on_a_floating_window() {
    let (mut fixture, windows) = Fixture::new(Size::new(80, 24), 1, 80);
    fixture.change(
        SessionAction::ToggleFloating {
            window: windows[0],
            after: None,
            floating: None,
        },
        80,
    );
    fixture.name(windows[0], "float");
    let screen = fixture.screen(Size::new(80, 24));
    let row = (0..24)
        .find(|&row| row_text(&screen, row).contains('╭'))
        .unwrap();
    let text = row_text(&screen, row);
    assert!(text.contains("╭float─"), "{text}");
}
