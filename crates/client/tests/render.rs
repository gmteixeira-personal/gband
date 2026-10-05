use std::collections::HashMap;
use std::time::Instant;

use gband_client::animation::{Animations, Band, Drawn, Presentation, Targets};
use gband_client::render::{Ribbon, draw_frame};
use gband_core::geometry::{Size, tiles};
use gband_core::layout::{Direction, Layout, LayoutOptions, PaneId, SessionAction, Step};
use gband_core::view::{CenterFocusedColumn, Scene, View, ViewAction};
use gband_emulator::{Emulator, Grid};
use insta::assert_snapshot;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

struct Fixture {
    layout: Layout,
    area: Size,
    view: View,
    grids: HashMap<PaneId, Grid>,
    banner: Option<String>,
}

impl Fixture {
    fn new(area: Size, columns: usize, viewport_cols: u16) -> (Self, Vec<PaneId>) {
        let mut layout = Layout::new();
        let workspace = layout.workspaces()[0].id;
        let mut panes = Vec::new();
        for _ in 0..columns {
            let pane = layout.allocate_pane();
            layout.open(
                pane,
                workspace,
                panes.last().copied(),
                &LayoutOptions::default(),
            );
            panes.push(pane);
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
        };
        fixture.reset_grids();
        (fixture, panes)
    }

    fn reset_grids(&mut self) {
        self.grids.clear();
        for workspace in self.layout.workspaces() {
            for tile in tiles(workspace, self.area) {
                self.grids
                    .insert(tile.pane, Grid::new(tile.terminal_size()));
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

    fn write(&mut self, pane: PaneId, bytes: &[u8]) {
        self.grids.get_mut(&pane).unwrap().process(bytes);
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
        let ribbon = Ribbon {
            layout: &self.layout,
            area: self.area,
            view: &self.view,
            grids: &self.grids,
            drawn,
            banner: self.banner.as_deref(),
        };
        terminal.draw(|frame| draw_frame(frame, &ribbon)).unwrap();
        let backend = terminal.backend();
        let cursor = backend.cursor_visible().then(|| backend.cursor_position());
        format!("{:?}\ncursor: {cursor:?}", backend.buffer())
    }
}

#[test]
fn two_columns_side_by_side() {
    let (mut fixture, panes) = Fixture::new(Size::new(80, 24), 2, 80);
    fixture.write(panes[0], b"left");
    fixture.write(panes[1], b"right");
    fixture.act(ViewAction::FocusRight, 80);
    assert_snapshot!(fixture.render(Size::new(80, 24)));
}

#[test]
fn column_clipped_at_the_left_edge() {
    let (mut fixture, panes) = Fixture::new(Size::new(80, 24), 2, 80);
    for &pane in &panes {
        fixture.change(SessionAction::CycleWidth(pane), 80);
    }
    fixture.write(panes[0], "0123456789".repeat(5).as_bytes());
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
fn empty_workspace_is_blank_without_a_cursor() {
    let (mut fixture, _) = Fixture::new(Size::new(80, 24), 1, 80);
    fixture.act(ViewAction::WorkspaceDown, 80);
    assert_eq!(fixture.view.focused(), None);
    assert_snapshot!(fixture.render(Size::new(80, 24)));
}

#[test]
fn hidden_pane_cursor_hides_the_cursor() {
    let (mut fixture, panes) = Fixture::new(Size::new(40, 6), 1, 40);
    assert_snapshot!("cursor_shown", fixture.render(Size::new(40, 6)));
    fixture.write(panes[0], b"\x1b[?25l");
    assert_snapshot!("cursor_hidden", fixture.render(Size::new(40, 6)));
}

#[test]
fn offscreen_column_is_not_drawn() {
    let (fixture, _) = Fixture::new(Size::new(80, 24), 3, 80);
    assert_snapshot!(fixture.render(Size::new(80, 24)));
}

#[test]
fn coloured_pane() {
    let (mut fixture, panes) = Fixture::new(Size::new(60, 6), 1, 60);
    fixture.write(
        panes[0],
        b"\x1b[1;31mred\x1b[0m \x1b[42mgreen\x1b[0m \x1b[4;38;2;10;20;30mrgb\x1b[0m",
    );
    assert_snapshot!(fixture.render(Size::new(60, 6)));
}

#[test]
fn wide_character_cut_at_the_clip_edge() {
    let (mut fixture, panes) = Fixture::new(Size::new(80, 8), 2, 80);
    for &pane in &panes {
        fixture.change(SessionAction::CycleWidth(pane), 80);
    }
    fixture.write(panes[0], "漢字".repeat(13).as_bytes());
    fixture.act(ViewAction::FocusRight, 80);
    assert_eq!(fixture.view.camera(), 26);
    assert_snapshot!(fixture.render(Size::new(80, 8)));
}

#[test]
fn stacked_column() {
    let (mut fixture, panes) = Fixture::new(Size::new(60, 12), 2, 60);
    fixture.change(
        SessionAction::ConsumeOrExpel {
            pane: panes[1],
            direction: Direction::Left,
        },
        60,
    );
    fixture.write(panes[0], b"top");
    fixture.write(panes[1], b"bottom");
    fixture.act(ViewAction::FocusDown, 60);
    assert_eq!(fixture.view.focused(), Some(panes[1]));
    assert_snapshot!(fixture.render(Size::new(60, 12)));
}

#[test]
fn column_wider_than_the_terminal_shows_its_left_border() {
    let (mut fixture, panes) = Fixture::new(Size::new(80, 24), 1, 80);
    for _ in 0..6 {
        fixture.change(
            SessionAction::StepWidth {
                pane: panes[0],
                step: Step::Grow,
            },
            80,
        );
    }
    assert_eq!(
        tiles(&fixture.layout.workspaces()[0], fixture.area)[0].width,
        88
    );
    assert_eq!(fixture.view.camera(), 0);
    fixture.write(panes[0], "0123456789".repeat(9).as_bytes());
    assert_snapshot!(fixture.render(Size::new(80, 24)));
}

#[test]
fn grid_smaller_than_its_tile() {
    let (mut fixture, panes) = Fixture::new(Size::new(100, 30), 1, 100);
    fixture.grids.insert(panes[0], Grid::new(Size::new(38, 22)));
    fixture.write(panes[0], format!("{}\r\nsmall", "s".repeat(38)).as_bytes());
    assert_snapshot!(fixture.render(Size::new(100, 30)));
}

#[test]
fn grid_larger_than_its_tile() {
    let (mut fixture, panes) = Fixture::new(Size::new(80, 24), 1, 80);
    fixture.grids.insert(panes[0], Grid::new(Size::new(48, 28)));
    for row in 0..28 {
        let line = format!("{row:02}{}", "L".repeat(46));
        fixture.write(panes[0], format!("\x1b[{};1H{line}", row + 1).as_bytes());
    }
    fixture.write(panes[0], b"\x1b[28;48H");
    assert_snapshot!(fixture.render(Size::new(80, 24)));
}

#[test]
fn tile_morphing_wider_than_its_grid() {
    let (mut fixture, panes) = Fixture::new(Size::new(80, 12), 2, 80);
    fixture.write(panes[0], "w".repeat(38).as_bytes());
    let mut drawn = fixture.at_rest(Size::new(80, 12));
    drawn.tiles.get_mut(&panes[0]).unwrap().width = 47;
    drawn.settled = false;
    assert_snapshot!(fixture.render_drawn(Size::new(80, 12), &drawn));
}

#[test]
fn tile_morphing_narrower_than_its_grid() {
    let (mut fixture, panes) = Fixture::new(Size::new(80, 12), 2, 80);
    fixture.write(panes[0], "0123456789".repeat(4).as_bytes());
    let mut drawn = fixture.at_rest(Size::new(80, 12));
    drawn.tiles.get_mut(&panes[0]).unwrap().width = 26;
    drawn.settled = false;
    assert_snapshot!(fixture.render_drawn(Size::new(80, 12), &drawn));
}

#[test]
fn mid_scroll_frame() {
    let (mut fixture, panes) = Fixture::new(Size::new(80, 12), 3, 80);
    fixture.write(panes[1], b"second");
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
fn workspace_switch_mid_slide() {
    let (mut fixture, panes) = Fixture::new(Size::new(60, 12), 1, 60);
    let below = fixture.layout.workspaces()[1].id;
    let pane = fixture.layout.allocate_pane();
    fixture
        .layout
        .open(pane, below, None, &LayoutOptions::default());
    fixture.reset_grids();
    fixture.write(panes[0], b"upper");
    fixture.write(pane, b"lower");
    fixture.act(ViewAction::WorkspaceDown, 60);
    let above = fixture.layout.workspaces()[0].id;
    let mut drawn = fixture.at_rest(Size::new(60, 12));
    drawn.bands = vec![
        Band {
            workspace: above,
            top: -5,
            camera: 0,
        },
        Band {
            workspace: below,
            top: 7,
            camera: 0,
        },
    ];
    drawn.settled = false;
    assert_snapshot!(fixture.render_drawn(Size::new(60, 12), &drawn));
}

#[test]
fn focused_tile_is_drawn_over_overlapping_tiles() {
    let (mut fixture, panes) = Fixture::new(Size::new(80, 8), 2, 80);
    fixture.write(panes[0], b"focused");
    fixture.write(panes[1], b"other");
    assert_eq!(fixture.view.focused(), Some(panes[0]));
    let mut drawn = fixture.at_rest(Size::new(80, 8));
    drawn.tiles.get_mut(&panes[1]).unwrap().x = 20;
    drawn.settled = false;
    assert_snapshot!(fixture.render_drawn(Size::new(80, 8), &drawn));
}

#[test]
fn centred_first_column_leaves_the_strip_start_empty() {
    let (mut fixture, panes) = Fixture::new(Size::new(80, 24), 1, 80);
    fixture.view = View::with_policy(
        Scene {
            layout: &fixture.layout,
            area: fixture.area,
            viewport: Size::new(80, 24),
        },
        CenterFocusedColumn::Always,
    );
    fixture.write(panes[0], b"centred");
    assert_eq!(fixture.view.camera(), -20);
    assert_snapshot!(fixture.render(Size::new(80, 24)));
}

#[test]
fn configuration_error_banner_covers_the_bottom_row() {
    let (mut fixture, panes) = Fixture::new(Size::new(40, 6), 1, 40);
    fixture.write(panes[0], b"$ ");
    fixture.banner = Some(
        "/home/u/.config/gband/init.lua:12: unexpected symbol near 'x'\nstack traceback".to_owned(),
    );
    assert_snapshot!(fixture.render(Size::new(40, 6)));
}
