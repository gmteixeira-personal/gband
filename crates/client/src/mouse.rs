use std::time::{Duration, Instant};

use gband_core::action::{Action, ClientAction};
use gband_core::geometry::{MIN_COLUMN_WIDTH, Size, WindowBox};
use gband_core::input::{
    MouseButton, MouseEvent, MouseKey, MouseKind, MouseTracking, WheelDirection,
};
use gband_core::layout::{BandId, Proportion, SessionAction, TargetPlace, WindowHeight, WindowId};
use gband_core::view::{View, ViewAction};
use gband_emulator::Emulator;
use gband_lua::plugin_windows::{PluginBox, PluginMouse, PluginMouseKind};
use gband_lua::{
    Binding, BoxCell, Event, Pointer as LuaPointer, PointerTarget, clipboard_sequence,
};
use gband_protocol::ClientMessage;
use ratatui::layout::{Position, Rect};

use crate::animation::{DrawnTile, FRAME, Hold};
use crate::bindings::{MouseCommand, Unbound, WheelCommand};
use crate::render::{Region, RegionKind, Selected};
use crate::{Controls, Display, Step};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Window(WindowId),
    DrawnPlugin {
        window: WindowId,
        plugin_window: u32,
    },
    PluginFloat(u32),
    Ribbon,
    Outside,
}

impl Target {
    pub fn window(self) -> Option<WindowId> {
        match self {
            Target::Window(window) | Target::DrawnPlugin { window, .. } => Some(window),
            _ => None,
        }
    }

    pub fn plugin_window(self) -> Option<u32> {
        match self {
            Target::DrawnPlugin { plugin_window, .. } | Target::PluginFloat(plugin_window) => {
                Some(plugin_window)
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    pub target: Target,
    pub region: Option<Region>,
    pub content: Option<(u16, u16)>,
}

impl Hit {
    fn boxed(&self, col: u16, row: u16) -> Option<BoxCell> {
        let region = self.region.filter(|_| {
            matches!(
                self.target,
                Target::Window(_) | Target::DrawnPlugin { .. } | Target::PluginFloat(_)
            )
        })?;
        Some(BoxCell {
            col: (i64::from(col) - region.x) as u16,
            row: (i64::from(row) - region.y) as u16,
            width: region.width,
            height: region.height,
        })
    }
}

pub fn hit(
    regions: &[Region],
    ribbon: Rect,
    band: BandId,
    plugin_window_of: impl Fn(WindowId) -> Option<u32>,
    col: u16,
    row: u16,
) -> Hit {
    let found = regions
        .iter()
        .rev()
        .find(|region| region.band.is_none_or(|drawn| drawn == band) && region.contains(col, row));
    let Some(region) = found else {
        let target = if ribbon.contains(Position::new(col, row)) {
            Target::Ribbon
        } else {
            Target::Outside
        };
        return Hit {
            target,
            region: None,
            content: None,
        };
    };
    let target = match (region.kind, region.window) {
        (RegionKind::PluginFloat(plugin_window), _) => Target::PluginFloat(plugin_window),
        (_, Some(window)) => match plugin_window_of(window) {
            Some(plugin_window) => Target::DrawnPlugin {
                window,
                plugin_window,
            },
            None => Target::Window(window),
        },
        (_, None) => Target::Ribbon,
    };
    Hit {
        target,
        region: Some(*region),
        content: region.content_cell(col, row),
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Edges {
    pub left: bool,
    pub right: bool,
    pub top: bool,
    pub bottom: bool,
}

pub fn pick_edges(x: u16, y: u16, width: u16, height: u16) -> Edges {
    let mut edges = Edges::default();
    if x < width / 3 {
        edges.left = true;
    } else if x >= width - width / 3 {
        edges.right = true;
    }
    if y < height / 3 {
        edges.top = true;
    } else if y >= height - height / 3 {
        edges.bottom = true;
    }
    if edges == Edges::default() {
        let right = width.saturating_sub(1).saturating_sub(x);
        let bottom = height.saturating_sub(1).saturating_sub(y);
        let nearest = right.min(bottom).min(x).min(y);
        if right == nearest {
            edges.right = true;
        } else if bottom == nearest {
            edges.bottom = true;
        } else if x == nearest {
            edges.left = true;
        } else {
            edges.top = true;
        }
    }
    edges
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DropColumn {
    pub column: usize,
    pub left: i64,
    pub width: u16,
    pub tiles: Vec<(WindowId, i64, u16)>,
}

pub fn drop_columns(regions: &[Region], band: BandId) -> Vec<DropColumn> {
    let mut columns: Vec<DropColumn> = Vec::new();
    for region in regions.iter().filter(|region| region.band == Some(band)) {
        let (RegionKind::Tile { column, .. }, Some(window)) = (region.kind, region.window) else {
            continue;
        };
        let tile = (window, region.y, region.height);
        match columns.iter_mut().find(|drop| drop.column == column) {
            Some(drop) => drop.tiles.push(tile),
            None => columns.push(DropColumn {
                column,
                left: region.x,
                width: region.width,
                tiles: vec![tile],
            }),
        }
    }
    for drop in &mut columns {
        drop.tiles.sort_by_key(|&(_, top, _)| top);
    }
    columns.sort_by_key(|drop| drop.left);
    columns
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DropPlace {
    pub column: usize,
    pub place: TargetPlace,
    pub window: Option<WindowId>,
    pub outline: Rect,
}

pub fn drop_place(
    columns: &[DropColumn],
    lifted: WindowId,
    ribbon: Rect,
    col: i64,
    row: i64,
) -> Option<DropPlace> {
    let first = columns.first()?;
    let last = columns.last()?;
    let height = ribbon.height;
    let rect = |left: i64, width: u16, top: i64, rows: u16| {
        let x = left.max(i64::from(ribbon.x));
        let right = (left + i64::from(width)).min(i64::from(ribbon.right()));
        let y = top.max(i64::from(ribbon.y));
        let bottom = (top + i64::from(rows)).min(i64::from(ribbon.bottom()));
        Rect::new(
            x as u16,
            y as u16,
            (right - x).max(0) as u16,
            (bottom - y).max(0) as u16,
        )
    };
    let quarter_of = |drop: &DropColumn| drop.width / 4;
    let beside = |drop: &DropColumn, place: TargetPlace| {
        let quarter = quarter_of(drop).max(1);
        let left = match place {
            TargetPlace::ColumnLeft => drop.left,
            _ => drop.left + i64::from(drop.width - quarter),
        };
        DropPlace {
            column: drop.column,
            place,
            window: None,
            outline: rect(left, quarter, i64::from(ribbon.y), height),
        }
    };
    let Some(drop) = columns
        .iter()
        .find(|drop| (drop.left..drop.left + i64::from(drop.width)).contains(&col))
    else {
        return Some(if col < first.left {
            beside(first, TargetPlace::ColumnLeft)
        } else {
            beside(last, TargetPlace::ColumnRight)
        });
    };
    let offset = col - drop.left;
    let quarter = i64::from(quarter_of(drop));
    if offset < quarter {
        return Some(beside(drop, TargetPlace::ColumnLeft));
    }
    if offset >= i64::from(drop.width) - quarter {
        return Some(beside(drop, TargetPlace::ColumnRight));
    }
    let others: Vec<&(WindowId, i64, u16)> = drop
        .tiles
        .iter()
        .filter(|(window, _, _)| *window != lifted)
        .collect();
    let &&(last_window, last_top, last_height) = others.last()?;
    let (window, top, rows, above) = others
        .iter()
        .find(|(_, top, rows)| row < top + i64::from(*rows))
        .map_or(
            (last_window, last_top, last_height, false),
            |&&(window, top, rows)| {
                let above = row < top || row - top < i64::from(rows / 2);
                (window, top, rows, above)
            },
        );
    let half = rows / 2;
    let (place, outline) = if above {
        (TargetPlace::Above, rect(drop.left, drop.width, top, half))
    } else {
        (
            TargetPlace::Below,
            rect(drop.left, drop.width, top + i64::from(half), rows - half),
        )
    };
    Some(DropPlace {
        column: drop.column,
        place,
        window: Some(window),
        outline,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
    pub window: WindowId,
    pub anchor: (u16, u16),
    pub head: (u16, u16),
}

impl Selection {
    pub fn selected(&self) -> Selected {
        let key = |(col, row): (u16, u16)| (row, col);
        let (start, end) = if key(self.anchor) <= key(self.head) {
            (self.anchor, self.head)
        } else {
            (self.head, self.anchor)
        };
        Selected {
            window: self.window,
            start,
            end,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resized {
    Tile {
        window: WindowId,
        width: u16,
        height: u16,
        travel: i64,
    },
    Floating {
        window: WindowId,
        origin: WindowBox,
    },
    Plugin {
        plugin_window: u32,
        origin: PluginBox,
        border: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Motion {
    MoveFloating {
        window: WindowId,
        origin: WindowBox,
    },
    MovePlugin {
        plugin_window: u32,
        origin: PluginBox,
    },
    Lift {
        window: WindowId,
        region: Region,
        drawn: Option<DrawnTile>,
        columns: Vec<DropColumn>,
        lifted: bool,
    },
    Resize {
        target: Resized,
        edges: Edges,
    },
    Slide {
        travel: i64,
        axis: Option<Axis>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical { band: BandId },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gesture {
    pub button: MouseButton,
    pub press: (u16, u16),
    pub motion: Motion,
}

impl Gesture {
    pub fn window(&self) -> Option<WindowId> {
        match &self.motion {
            Motion::MoveFloating { window, .. } | Motion::Lift { window, .. } => Some(*window),
            Motion::Resize {
                target: Resized::Tile { window, .. } | Resized::Floating { window, .. },
                ..
            } => Some(*window),
            _ => None,
        }
    }

    pub fn moved(&self, (col, row): (u16, u16)) -> (i64, i64) {
        (
            i64::from(col) - i64::from(self.press.0),
            i64::from(row) - i64::from(self.press.1),
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Held {
    #[default]
    Free,
    Ignored,
    Forward {
        window: WindowId,
        buttons: u8,
    },
    Select,
    Plugin(u32),
    Gesture(Gesture),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Geometry {
    pub position: Option<(u16, u16)>,
    pub width: Option<u16>,
    pub height: Option<u16>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sends {
    pub window: WindowId,
    pub pending: Geometry,
    pub sent: Geometry,
}

#[derive(Debug, Default)]
pub struct Pointer {
    pub held: Held,
    pub selection: Option<Selection>,
    pub floating: Option<WindowBox>,
    pub last_cell: Option<(u16, u16)>,
    pub copy_buffer: String,
    pub sends: Option<Sends>,
    pub last_flush: Option<Instant>,
    pub wheel_binding: Option<(MouseKey, Instant)>,
}

impl Pointer {
    pub fn gesture(&self) -> Option<&Gesture> {
        match &self.held {
            Held::Gesture(gesture) => Some(gesture),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Pressing {
    pub hit: Hit,
    pub button: MouseButton,
    pub cell: (u16, u16),
}

const WHEEL_COOLDOWN: Duration = Duration::from_millis(150);

fn reports_mouse(display: &Display, window: WindowId) -> bool {
    display
        .grids
        .get(&window)
        .is_some_and(|grid| grid.modes().mouse_tracking != MouseTracking::None)
}

fn payload(hit: &Hit, event: MouseEvent, table: &str) -> LuaPointer {
    LuaPointer {
        col: event.col,
        row: event.row,
        modifiers: event.modifiers,
        target: match hit.target {
            Target::Window(_) => PointerTarget::Window,
            Target::DrawnPlugin { .. } | Target::PluginFloat(_) => PointerTarget::PluginWindow,
            Target::Ribbon => PointerTarget::Ribbon,
            Target::Outside => PointerTarget::Outside,
        },
        window: hit.target.window(),
        plugin_window: hit.target.plugin_window(),
        content: hit.content,
        boxed: hit.boxed(event.col, event.row),
        table: table.to_owned(),
    }
}

fn lua_event(hit: &Hit, event: MouseEvent, table: &str) -> Option<Event> {
    let pointer = payload(hit, event, table);
    Some(match event.kind {
        MouseKind::Press(button) => Event::MousePressed { button, pointer },
        MouseKind::Release(button) => Event::MouseReleased { button, pointer },
        MouseKind::Motion(Some(button)) => Event::MouseDragged { button, pointer },
        MouseKind::Motion(None) => return None,
        MouseKind::Wheel(direction) => Event::MouseScrolled { direction, pointer },
    })
}

fn shifted(origin: u16, by: i64, limit: u16) -> u16 {
    (i64::from(origin) + by).clamp(0, i64::from(limit)) as u16
}

fn resized(size: u16, by: i64, minimum: u16, maximum: u16) -> u16 {
    (i64::from(size) + by).clamp(i64::from(minimum), i64::from(maximum.max(minimum))) as u16
}

#[derive(Clone, Copy)]
struct Cells {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

fn resize_cells(
    origin: Cells,
    edges: Edges,
    (dx, dy): (i64, i64),
    minimum: u16,
    limit: Size,
) -> Cells {
    let width = if edges.right {
        resized(
            origin.width,
            dx,
            minimum,
            limit.cols.saturating_sub(origin.x),
        )
    } else if edges.left {
        resized(origin.width, -dx, minimum, origin.x + origin.width)
    } else {
        origin.width
    };
    let height = if edges.bottom {
        resized(
            origin.height,
            dy,
            minimum,
            limit.rows.saturating_sub(origin.y),
        )
    } else if edges.top {
        resized(origin.height, -dy, minimum, origin.y + origin.height)
    } else {
        origin.height
    };
    Cells {
        x: if edges.left {
            (origin.x + origin.width).saturating_sub(width)
        } else {
            origin.x
        },
        y: if edges.top {
            (origin.y + origin.height).saturating_sub(height)
        } else {
            origin.y
        },
        width,
        height,
    }
}

impl Controls {
    pub fn mouse(&mut self, display: &mut Display, event: MouseEvent, now: Instant) -> Vec<Step> {
        let cell = (event.col, event.row);
        if matches!(event.kind, MouseKind::Motion(_)) && display.pointer.last_cell == Some(cell) {
            return Vec::new();
        }
        display.pointer.last_cell = Some(cell);
        let table = self.leader.active().to_owned();
        let hit = display.hit(event.col, event.row, now);
        let mut steps = self.react(display, Vec::new(), |controls, display, steps| {
            match event.kind {
                MouseKind::Wheel(direction) => {
                    controls.wheel(display, &hit, event, direction, now, steps)
                }
                MouseKind::Press(button) => {
                    controls.mouse_press(display, &hit, event, button, &table, steps)
                }
                MouseKind::Motion(button) => {
                    controls.mouse_motion(display, &hit, event, button, steps)
                }
                MouseKind::Release(button) => {
                    controls.mouse_release(display, &hit, event, button, now, steps)
                }
            }
            if let Some(event) = lua_event(&hit, event, &table) {
                controls.pending.push(event);
            }
        });
        steps.extend(flush_sends(display, now));
        steps
    }

    pub fn flush(&mut self, display: &mut Display, now: Instant) -> Vec<Step> {
        flush_sends(display, now)
    }

    pub fn next_flush(&self, display: &Display) -> Option<Instant> {
        let sends = display.pointer.sends.as_ref()?;
        (sends.pending != Geometry::default()).then(|| {
            display
                .pointer
                .last_flush
                .map_or_else(Instant::now, |last| last + FRAME)
        })
    }

    fn plugin_mouse(
        &mut self,
        display: &mut Display,
        plugin_window: u32,
        kind: PluginMouseKind,
        hit: &Hit,
        event: MouseEvent,
        steps: &mut Vec<Step>,
    ) {
        let on_it = hit.target.plugin_window() == Some(plugin_window);
        let mouse = PluginMouse {
            kind,
            content: hit.content.filter(|_| on_it),
            boxed: hit.boxed(event.col, event.row).filter(|_| on_it),
            modifiers: event.modifiers,
        };
        let outcome = self.runtime.plugin_window_mouse(plugin_window, &mouse);
        self.apply(display, outcome, steps);
    }

    fn wheel(
        &mut self,
        display: &mut Display,
        hit: &Hit,
        event: MouseEvent,
        direction: WheelDirection,
        now: Instant,
        steps: &mut Vec<Step>,
    ) {
        if display.pointer.gesture().is_none() {
            let key = MouseKey::new(direction, event.modifiers);
            let table = self.leader.active().to_owned();
            if display
                .pointer
                .wheel_binding
                .is_some_and(|(last, at)| last == key && now < at + WHEEL_COOLDOWN)
            {
                return;
            }
            if let WheelCommand::Run { binding, ends } = self.leader.handle_wheel(&self.keymap, key)
            {
                let declined = match binding {
                    Binding::Action(action) => {
                        self.take_wheel_step(display, key, now, ends);
                        self.run_action(display, action, steps);
                        false
                    }
                    Binding::Callback(callback) => {
                        let pointer = payload(hit, event, &table);
                        let outcome = self.runtime.call_scrolled(callback, direction, &pointer);
                        let declined = outcome.declined;
                        if !declined {
                            self.take_wheel_step(display, key, now, ends);
                        }
                        self.apply(display, outcome, steps);
                        declined
                    }
                };
                if !declined {
                    return;
                }
            }
        }
        match hit.target {
            Target::Window(window) => {
                let cell = hit.content.or_else(|| {
                    hit.region.map(|region| {
                        region.nearest_content(i64::from(event.col), i64::from(event.row))
                    })
                });
                if let Some((col, row)) = cell {
                    steps.push(Step::Send(ClientMessage::Mouse {
                        window,
                        event: event.at(col, row),
                    }));
                }
            }
            Target::DrawnPlugin { plugin_window, .. } | Target::PluginFloat(plugin_window) => self
                .plugin_mouse(
                    display,
                    plugin_window,
                    PluginMouseKind::Scroll(direction),
                    hit,
                    event,
                    steps,
                ),
            Target::Ribbon | Target::Outside => {}
        }
    }

    fn take_wheel_step(&mut self, display: &mut Display, key: MouseKey, now: Instant, ends: bool) {
        display.pointer.wheel_binding = Some((key, now));
        if ends {
            self.leader.reset();
        }
    }

    fn focus_clicked(&mut self, display: &mut Display, window: WindowId, steps: &mut Vec<Step>) {
        let outcome = self.runtime.unfocus_plugin_windows();
        self.apply(display, outcome, steps);
        self.run_action(
            display,
            Action::View(ViewAction::FocusWindow(window)),
            steps,
        );
    }

    fn forward(display: &Display, window: WindowId, event: MouseEvent, steps: &mut Vec<Step>) {
        let Some(region) = display.region_of(window) else {
            return;
        };
        let (col, row) = region.nearest_content(i64::from(event.col), i64::from(event.row));
        steps.push(Step::Send(ClientMessage::Mouse {
            window,
            event: event.at(col, row),
        }));
    }

    fn mouse_press(
        &mut self,
        display: &mut Display,
        hit: &Hit,
        event: MouseEvent,
        button: MouseButton,
        table: &str,
        steps: &mut Vec<Step>,
    ) {
        display.pointer.selection = None;
        match &mut display.pointer.held {
            Held::Gesture(_) => return,
            Held::Forward { window, buttons } => {
                let window = *window;
                *buttons += 1;
                Self::forward(display, window, event, steps);
                return;
            }
            _ => {}
        }
        let key = MouseKey::new(button, event.modifiers);
        match self.leader.handle_mouse(&self.keymap, key) {
            MouseCommand::Run { binding, unbound } => {
                display.pointer.held = Held::Ignored;
                self.pressing = Some(Pressing {
                    hit: *hit,
                    button,
                    cell: (event.col, event.row),
                });
                match binding {
                    Binding::Action(action) => self.run_action(display, action, steps),
                    Binding::Callback(callback) => {
                        let pointer = payload(hit, event, table);
                        let outcome = self.runtime.call_pressed(callback, button, &pointer);
                        if outcome.declined {
                            self.pressing = None;
                            self.apply(display, outcome, steps);
                            match unbound {
                                Unbound::Default => {
                                    self.press_default(display, hit, event, button, steps);
                                }
                                Unbound::Discard => display.pointer.held = Held::Ignored,
                            }
                            return;
                        }
                        self.apply(display, outcome, steps);
                    }
                }
                self.pressing = None;
            }
            MouseCommand::Discard => display.pointer.held = Held::Ignored,
            MouseCommand::Default => self.press_default(display, hit, event, button, steps),
        }
    }

    fn press_default(
        &mut self,
        display: &mut Display,
        hit: &Hit,
        event: MouseEvent,
        button: MouseButton,
        steps: &mut Vec<Step>,
    ) {
        display.pointer.held = Held::Free;
        match hit.target {
            Target::Window(window) => {
                self.focus_clicked(display, window, steps);
                let Some((col, row)) = hit.content else {
                    return;
                };
                if reports_mouse(display, window) && !(event.modifiers.ctrl && event.modifiers.alt)
                {
                    steps.push(Step::Send(ClientMessage::Mouse {
                        window,
                        event: event.at(col, row),
                    }));
                    display.pointer.held = Held::Forward { window, buttons: 1 };
                    return;
                }
                match button {
                    MouseButton::Left => {
                        display.pointer.selection = Some(Selection {
                            window,
                            anchor: (col, row),
                            head: (col, row),
                        });
                        display.pointer.held = Held::Select;
                    }
                    MouseButton::Right if !display.pointer.copy_buffer.is_empty() => {
                        steps.push(Step::Send(ClientMessage::Paste {
                            window,
                            text: display.pointer.copy_buffer.clone(),
                        }));
                    }
                    MouseButton::Right | MouseButton::Middle => {}
                }
            }
            Target::DrawnPlugin { plugin_window, .. } | Target::PluginFloat(plugin_window) => {
                self.plugin_mouse(
                    display,
                    plugin_window,
                    PluginMouseKind::Press(button),
                    hit,
                    event,
                    steps,
                );
                display.pointer.held = Held::Plugin(plugin_window);
            }
            Target::Ribbon | Target::Outside => {}
        }
    }

    fn mouse_motion(
        &mut self,
        display: &mut Display,
        hit: &Hit,
        event: MouseEvent,
        button: Option<MouseButton>,
        steps: &mut Vec<Step>,
    ) {
        match display.pointer.held.clone() {
            Held::Gesture(_) => self.gesture_motion(display, (event.col, event.row), steps),
            Held::Forward { window, .. } => {
                if button.is_some() {
                    Self::forward(display, window, event, steps);
                }
            }
            Held::Select => {
                let Some(selection) = display.pointer.selection else {
                    return;
                };
                if button != Some(MouseButton::Left) {
                    return;
                }
                if let Some(region) = display.region_of(selection.window) {
                    let head = region.nearest_content(i64::from(event.col), i64::from(event.row));
                    display.pointer.selection = Some(Selection { head, ..selection });
                }
            }
            Held::Plugin(plugin_window) => {
                if let Some(button) = button {
                    self.plugin_mouse(
                        display,
                        plugin_window,
                        PluginMouseKind::Drag(button),
                        hit,
                        event,
                        steps,
                    );
                }
            }
            Held::Free | Held::Ignored => {
                if button.is_some() {
                    return;
                }
                let Some(window) = display.focused() else {
                    return;
                };
                let inside = display
                    .region_of(window)
                    .and_then(|region| region.content_cell(event.col, event.row));
                if let Some((col, row)) = inside
                    && reports_mouse(display, window)
                {
                    steps.push(Step::Send(ClientMessage::Mouse {
                        window,
                        event: event.at(col, row),
                    }));
                }
            }
        }
    }

    fn mouse_release(
        &mut self,
        display: &mut Display,
        hit: &Hit,
        event: MouseEvent,
        button: MouseButton,
        now: Instant,
        steps: &mut Vec<Step>,
    ) {
        match std::mem::take(&mut display.pointer.held) {
            Held::Gesture(gesture) if gesture.button == button => {
                self.end_gesture(display, gesture, (event.col, event.row), now, steps);
            }
            Held::Gesture(gesture) => display.pointer.held = Held::Gesture(gesture),
            Held::Forward { window, buttons } => {
                Self::forward(display, window, event, steps);
                if buttons > 1 {
                    display.pointer.held = Held::Forward {
                        window,
                        buttons: buttons - 1,
                    };
                }
            }
            Held::Select => Self::copy(display, steps),
            Held::Plugin(plugin_window) => self.plugin_mouse(
                display,
                plugin_window,
                PluginMouseKind::Release(button),
                hit,
                event,
                steps,
            ),
            Held::Free | Held::Ignored => {}
        }
    }

    fn copy(display: &mut Display, steps: &mut Vec<Step>) {
        let Some(selection) = display.pointer.selection else {
            return;
        };
        if selection.anchor == selection.head {
            display.pointer.selection = None;
            return;
        }
        let selected = selection.selected();
        let Some(grid) = display.grids.get(&selection.window) else {
            return;
        };
        let text = grid.text_between(selected.start, selected.end);
        if let Some(sequence) = clipboard_sequence(text.as_bytes()) {
            steps.push(Step::Write(sequence));
        }
        display.pointer.copy_buffer = text;
    }

    pub(crate) fn start_gesture(
        &mut self,
        display: &mut Display,
        pressing: Pressing,
        kind: ClientAction,
        steps: &mut Vec<Step>,
    ) {
        let Pressing { hit, button, cell } = pressing;
        let travel = display.view.as_ref().map_or(0, View::travel);
        let region = hit.region;
        let edges = region.map_or_else(Edges::default, |region| {
            pick_edges(
                (i64::from(cell.0) - region.x).clamp(0, i64::from(u16::MAX)) as u16,
                (i64::from(cell.1) - region.y).clamp(0, i64::from(u16::MAX)) as u16,
                region.width,
                region.height,
            )
        });
        let ribbon = display.ribbon;
        let plugin_box = |region: Region| PluginBox {
            col: (region.x - i64::from(ribbon.x)).max(0) as u16,
            row: (region.y - i64::from(ribbon.y)).max(0) as u16,
            width: region.width,
            height: region.height,
        };
        let motion = match (kind, hit.target, region) {
            (ClientAction::DragBand, _, _) | (ClientAction::DragWindow, Target::Ribbon, _) => {
                Some(Motion::Slide { travel, axis: None })
            }
            (
                ClientAction::DragWindow | ClientAction::DragResize,
                Target::PluginFloat(plugin_window),
                Some(region),
            ) => {
                let outcome = self.runtime.raise_plugin_window(plugin_window);
                self.apply(display, outcome, steps);
                let origin = plugin_box(region);
                Some(if kind == ClientAction::DragWindow {
                    Motion::MovePlugin {
                        plugin_window,
                        origin,
                    }
                } else {
                    Motion::Resize {
                        target: Resized::Plugin {
                            plugin_window,
                            origin,
                            border: region.border,
                        },
                        edges,
                    }
                })
            }
            (ClientAction::DragWindow | ClientAction::DragResize, target, Some(region)) => {
                match target.window() {
                    Some(window) => {
                        self.focus_clicked(display, window, steps);
                        display.window_motion(kind, window, region, edges, travel)
                    }
                    None => None,
                }
            }
            _ => None,
        };
        let Some(motion) = motion else {
            return;
        };
        let gesture = Gesture {
            button,
            press: cell,
            motion,
        };
        display.pointer.sends = display.initial_sends(&gesture);
        display.pointer.held = Held::Gesture(gesture);
    }

    fn gesture_motion(&mut self, display: &mut Display, cell: (u16, u16), steps: &mut Vec<Step>) {
        let Held::Gesture(mut gesture) = std::mem::take(&mut display.pointer.held) else {
            return;
        };
        let (dx, dy) = gesture.moved(cell);
        let area = display.area;
        let press = gesture.press;
        match &mut gesture.motion {
            Motion::MoveFloating { origin, .. } => {
                let x = shifted(origin.x, dx, area.cols.saturating_sub(origin.width));
                let y = shifted(origin.y, dy, area.rows.saturating_sub(origin.height));
                display.pointer.floating = Some(WindowBox { x, y, ..*origin });
                display.pend(|pending| pending.position = Some((x, y)));
            }
            Motion::MovePlugin {
                plugin_window,
                origin,
            } => {
                let placed = PluginBox {
                    col: shifted(origin.col, dx, u16::MAX),
                    row: shifted(origin.row, dy, u16::MAX),
                    ..*origin
                };
                let outcome = self.runtime.set_plugin_window_box(*plugin_window, placed);
                self.apply(display, outcome, steps);
            }
            Motion::Lift { lifted, .. } => {
                if cell != press {
                    *lifted = true;
                }
            }
            Motion::Resize { target, edges } => {
                let edges = *edges;
                match *target {
                    Resized::Tile {
                        window,
                        width,
                        height,
                        travel,
                    } => {
                        let widened = if edges.right {
                            resized(width, dx, MIN_COLUMN_WIDTH, u16::MAX)
                        } else if edges.left {
                            resized(width, -dx, MIN_COLUMN_WIDTH, u16::MAX)
                        } else {
                            width
                        };
                        let heightened = if edges.bottom {
                            resized(height, dy, 1, u16::MAX)
                        } else if edges.top {
                            resized(height, -dy, 1, u16::MAX)
                        } else {
                            height
                        };
                        display.pend(|pending| {
                            if edges.left || edges.right {
                                pending.width = Some(widened);
                            }
                            if edges.top || edges.bottom {
                                pending.height = Some(heightened);
                            }
                        });
                        if edges.left {
                            let moved = travel + i64::from(widened) - i64::from(width);
                            display.with_view(|view, scene| view.slide(moved, scene));
                        }
                        display.presentation.hold(Hold {
                            camera: edges.left,
                            windows: vec![window],
                            ..Hold::default()
                        });
                    }
                    Resized::Floating { origin, .. } => {
                        let placed = resize_cells(
                            Cells {
                                x: origin.x,
                                y: origin.y,
                                width: origin.width,
                                height: origin.height,
                            },
                            edges,
                            (dx, dy),
                            MIN_COLUMN_WIDTH,
                            area,
                        );
                        display.pointer.floating = Some(WindowBox {
                            x: placed.x,
                            y: placed.y,
                            width: placed.width,
                            height: placed.height,
                            ..origin
                        });
                        display.pend(|pending| {
                            if edges.left || edges.right {
                                pending.width = Some(placed.width);
                            }
                            if edges.top || edges.bottom {
                                pending.height = Some(placed.height);
                            }
                            if edges.left || edges.top {
                                pending.position = Some((placed.x, placed.y));
                            }
                        });
                    }
                    Resized::Plugin {
                        plugin_window,
                        origin,
                        border,
                    } => {
                        let limit = Size::new(
                            display.ribbon.width.max(origin.col + origin.width),
                            display.ribbon.height.max(origin.row + origin.height),
                        );
                        let placed = resize_cells(
                            Cells {
                                x: origin.col,
                                y: origin.row,
                                width: origin.width,
                                height: origin.height,
                            },
                            edges,
                            (dx, dy),
                            if border { 3 } else { 1 },
                            limit,
                        );
                        let placed = PluginBox {
                            col: placed.x,
                            row: placed.y,
                            width: placed.width,
                            height: placed.height,
                        };
                        let outcome = self.runtime.set_plugin_window_box(plugin_window, placed);
                        self.apply(display, outcome, steps);
                    }
                }
            }
            Motion::Slide { travel, axis } => {
                if axis.is_none() && (dx.abs() >= 2 || dy.abs() >= 1) {
                    *axis = if dx.abs() >= 2 * dy.abs() {
                        Some(Axis::Horizontal)
                    } else {
                        display
                            .view
                            .as_ref()
                            .map(|view| Axis::Vertical { band: view.band() })
                    };
                }
                match *axis {
                    None => {}
                    Some(Axis::Horizontal) => {
                        let moved = *travel - dx;
                        display.with_view(|view, scene| view.slide(moved, scene));
                        display.presentation.hold(Hold {
                            camera: true,
                            ..Hold::default()
                        });
                    }
                    Some(Axis::Vertical { band }) => display.hold_bands(band, dy),
                }
            }
        }
        display.pointer.held = Held::Gesture(gesture);
    }

    fn end_gesture(
        &mut self,
        display: &mut Display,
        gesture: Gesture,
        cell: (u16, u16),
        now: Instant,
        steps: &mut Vec<Step>,
    ) {
        let (dx, dy) = gesture.moved(cell);
        match gesture.motion {
            Motion::Lift {
                window,
                drawn,
                columns,
                lifted: true,
                ..
            } => {
                let place = drop_place(
                    &columns,
                    window,
                    display.ribbon,
                    i64::from(cell.0),
                    i64::from(cell.1),
                );
                let action = place.and_then(|place| display.drop_action(window, place));
                let tile = drawn.map(|drawn| DrawnTile {
                    x: drawn.x + dx,
                    y: drawn.y + dy,
                    ..drawn
                });
                match (action, tile) {
                    (Some(action), tile) => {
                        if let Some(tile) = tile {
                            display.presentation.park(window, tile, now);
                        }
                        steps.push(Step::Send(ClientMessage::Action(action)));
                    }
                    (None, Some(tile)) => display.presentation.release(window, tile, now),
                    (None, None) => {}
                }
            }
            Motion::Resize {
                target: Resized::Tile { .. },
                edges,
            } if edges.left => {
                display.with_view(|view, scene| {
                    view.unpin();
                    view.sync(scene);
                });
            }
            Motion::Slide {
                axis: Some(Axis::Vertical { band }),
                ..
            } => display.release_bands(band, dy, now),
            Motion::Slide { travel, .. } => {
                display.with_view(|view, scene| view.release_slide(scene, travel));
            }
            _ => {}
        }
        display.presentation.hold(Hold::default());
    }
}

fn flush_sends(display: &mut Display, now: Instant) -> Vec<Step> {
    let area = display.area;
    let idle = display.pointer.gesture().is_none();
    let last = display.pointer.last_flush;
    let Some(sends) = &mut display.pointer.sends else {
        return Vec::new();
    };
    if sends.pending == Geometry::default() {
        if idle {
            display.pointer.sends = None;
        }
        return Vec::new();
    }
    if last.is_some_and(|last| now < last + FRAME) {
        return Vec::new();
    }
    let window = sends.window;
    let pending = std::mem::take(&mut sends.pending);
    let mut actions = Vec::new();
    if let Some(width) = pending
        .width
        .filter(|&width| Some(width) != sends.sent.width)
    {
        sends.sent.width = Some(width);
        actions.push(SessionAction::SetWidth {
            window,
            width: Proportion::new(u32::from(width), u32::from(area.cols.max(1))).lowest(),
        });
    }
    if let Some(height) = pending
        .height
        .filter(|&height| Some(height) != sends.sent.height)
    {
        sends.sent.height = Some(height);
        actions.push(SessionAction::SetHeight {
            window,
            height: WindowHeight::Fixed(height),
        });
    }
    if let Some((col, row)) = pending
        .position
        .filter(|&position| Some(position) != sends.sent.position)
    {
        sends.sent.position = Some((col, row));
        actions.push(SessionAction::SetPosition { window, col, row });
    }
    if idle {
        display.pointer.sends = None;
    }
    if !actions.is_empty() {
        display.pointer.last_flush = Some(now);
    }
    actions
        .into_iter()
        .map(|action| Step::Send(ClientMessage::Action(action)))
        .collect()
}
