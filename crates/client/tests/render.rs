use std::collections::HashMap;

use gband_client::render::{Ribbon, draw_frame};
use gband_core::geometry::{Size, tiles};
use gband_core::layout::{Direction, Layout, PaneId, SessionAction, Step};
use gband_core::view::{Scene, View, ViewAction};
use gband_emulator::{Emulator, Grid};
use insta::assert_snapshot;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

struct Fixture {
    layout: Layout,
    area: Size,
    view: View,
    grids: HashMap<PaneId, Grid>,
}

impl Fixture {
    fn new(area: Size, columns: usize, viewport_cols: u16) -> (Self, Vec<PaneId>) {
        let mut layout = Layout::new();
        let workspace = layout.workspaces()[0].id;
        let mut panes = Vec::new();
        for _ in 0..columns {
            let pane = layout.allocate_pane();
            layout.open(pane, workspace, panes.last().copied());
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
        self.layout.apply(action, self.area);
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

    fn render(&self, terminal: Size) -> String {
        let mut terminal = Terminal::new(TestBackend::new(terminal.cols, terminal.rows)).unwrap();
        let ribbon = Ribbon {
            layout: &self.layout,
            area: self.area,
            view: &self.view,
            grids: &self.grids,
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
