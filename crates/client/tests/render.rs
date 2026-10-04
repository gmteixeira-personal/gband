use std::collections::HashMap;

use gband_client::render::{FOCUSED_BORDER, Ribbon, UNFOCUSED_BORDER, render};
use gband_core::geometry::{Size, tiles};
use gband_core::layout::{Layout, PaneId, SessionAction};
use gband_core::view::{Scene, View, ViewAction};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

struct Fixture {
    layout: Layout,
    area: Size,
    view: View,
    parsers: HashMap<PaneId, vt100::Parser>,
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
            viewport_cols,
        });
        let mut fixture = Self {
            layout,
            area,
            view,
            parsers: HashMap::new(),
        };
        fixture.reset_parsers();
        (fixture, panes)
    }

    fn reset_parsers(&mut self) {
        self.parsers.clear();
        for workspace in self.layout.workspaces() {
            for tile in tiles(workspace, self.area) {
                let size = tile.terminal_size();
                self.parsers
                    .insert(tile.pane, vt100::Parser::new(size.rows, size.cols, 0));
            }
        }
    }

    fn act(&mut self, action: ViewAction, viewport_cols: u16) {
        self.view.apply(
            action,
            Scene {
                layout: &self.layout,
                area: self.area,
                viewport_cols,
            },
        );
    }

    fn write(&mut self, pane: PaneId, bytes: &[u8]) {
        self.parsers.get_mut(&pane).unwrap().process(bytes);
    }

    fn render(&self, terminal: Size) -> (Buffer, Option<Position>) {
        let mut buffer = Buffer::empty(Rect::new(0, 0, terminal.cols, terminal.rows));
        let ribbon = Ribbon {
            layout: &self.layout,
            area: self.area,
            view: &self.view,
            parsers: &self.parsers,
        };
        let cursor = render(&ribbon, &mut buffer);
        (buffer, cursor)
    }
}

fn symbol(buffer: &Buffer, x: u16, y: u16) -> &str {
    buffer[(x, y)].symbol()
}

#[test]
fn two_columns_side_by_side() {
    let (mut fixture, panes) = Fixture::new(Size::new(80, 24), 2, 80);
    fixture.write(panes[0], b"left");
    fixture.write(panes[1], b"right");
    fixture.act(ViewAction::FocusRight, 80);
    let (buffer, cursor) = fixture.render(Size::new(80, 24));

    assert_eq!(symbol(&buffer, 0, 0), "┌");
    assert_eq!(symbol(&buffer, 39, 0), "┐");
    assert_eq!(symbol(&buffer, 40, 0), "┌");
    assert_eq!(symbol(&buffer, 79, 0), "┐");
    assert_eq!(symbol(&buffer, 0, 23), "└");
    assert_eq!(symbol(&buffer, 79, 23), "┘");
    assert_eq!(symbol(&buffer, 1, 1), "l");
    assert_eq!(symbol(&buffer, 41, 1), "r");

    assert!(
        buffer[(40, 0)]
            .modifier
            .contains(FOCUSED_BORDER.add_modifier)
    );
    assert!(
        buffer[(0, 0)]
            .modifier
            .contains(UNFOCUSED_BORDER.add_modifier)
    );
    assert!(
        !buffer[(0, 0)]
            .modifier
            .contains(FOCUSED_BORDER.add_modifier)
    );
    assert_eq!(cursor, Some(Position::new(46, 1)));
}

#[test]
fn column_clipped_at_the_left_edge() {
    let (mut fixture, panes) = Fixture::new(Size::new(80, 24), 2, 80);
    for &pane in &panes {
        fixture.layout.apply(SessionAction::CycleWidth(pane));
    }
    fixture.reset_parsers();
    fixture.write(panes[0], "0123456789".repeat(5).as_bytes());
    fixture.act(ViewAction::FocusRight, 80);
    assert_eq!(fixture.view.camera(), 26);
    let (buffer, _) = fixture.render(Size::new(80, 24));

    assert_eq!(symbol(&buffer, 27, 0), "┌");
    assert_eq!(symbol(&buffer, 79, 0), "┐");
    assert_eq!(symbol(&buffer, 26, 0), "┐");
    assert_eq!(symbol(&buffer, 0, 0), "─");
    assert_eq!(symbol(&buffer, 0, 5), " ");
    assert_eq!(symbol(&buffer, 0, 1), "5");
    assert_eq!(symbol(&buffer, 1, 1), "6");
}

#[test]
fn tile_larger_than_the_terminal_is_cut() {
    let (fixture, _) = Fixture::new(Size::new(120, 40), 1, 100);
    let (buffer, _) = fixture.render(Size::new(100, 30));
    assert_eq!(symbol(&buffer, 0, 0), "┌");
    assert_eq!(symbol(&buffer, 59, 0), "┐");
    assert_eq!(symbol(&buffer, 0, 29), "│");
    assert_eq!(symbol(&buffer, 59, 29), "│");
    assert_eq!(symbol(&buffer, 60, 0), " ");
    assert_eq!(symbol(&buffer, 99, 29), " ");
}

#[test]
fn empty_workspace_is_blank_without_a_cursor() {
    let (mut fixture, _) = Fixture::new(Size::new(80, 24), 1, 80);
    fixture.act(ViewAction::WorkspaceDown, 80);
    assert_eq!(fixture.view.focused(), None);
    let (buffer, cursor) = fixture.render(Size::new(80, 24));
    assert_eq!(buffer, Buffer::empty(Rect::new(0, 0, 80, 24)));
    assert_eq!(cursor, None);
}

#[test]
fn hidden_pane_cursor_hides_the_cursor() {
    let (mut fixture, panes) = Fixture::new(Size::new(80, 24), 1, 80);
    let (_, cursor) = fixture.render(Size::new(80, 24));
    assert_eq!(cursor, Some(Position::new(1, 1)));
    fixture.write(panes[0], b"\x1b[?25l");
    let (_, cursor) = fixture.render(Size::new(80, 24));
    assert_eq!(cursor, None);
}

#[test]
fn offscreen_column_is_not_drawn() {
    let (fixture, _) = Fixture::new(Size::new(80, 24), 3, 80);
    let (buffer, _) = fixture.render(Size::new(80, 24));
    assert_eq!(symbol(&buffer, 40, 0), "┌");
    assert_eq!(symbol(&buffer, 79, 0), "┐");
}
