use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::action::SessionCommand;
use crate::geometry::{Size, Span, column_spans, tiles};
use crate::layout::{Band, BandId, Layout, Location, PaneId, SessionAction};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewAction {
    FocusLeft,
    FocusRight,
    FocusDown,
    FocusUp,
    BandDown,
    BandUp,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CenterFocusedColumn {
    #[default]
    Never,
    Always,
    OnOverflow,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct BandView {
    focus: Option<PaneId>,
    camera: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scene<'a> {
    pub layout: &'a Layout,
    pub area: Size,
    pub viewport: Size,
}

#[derive(Clone, Debug)]
pub struct View {
    band: BandId,
    bands: HashMap<BandId, BandView>,
    position: Location,
    recency: HashMap<PaneId, u64>,
    tick: u64,
    policy: CenterFocusedColumn,
}

impl View {
    pub fn new(scene: Scene<'_>) -> Self {
        Self::with_policy(scene, CenterFocusedColumn::default())
    }

    pub fn with_policy(scene: Scene<'_>, policy: CenterFocusedColumn) -> Self {
        let first = &scene.layout.bands()[0];
        let mut view = Self {
            band: first.id,
            bands: HashMap::new(),
            position: Location::default(),
            recency: HashMap::new(),
            tick: 0,
            policy,
        };
        if let Some(pane) = first.first_pane() {
            view.set_focus(pane);
        }
        view.sync(scene);
        view
    }

    pub fn set_center_focused_column(&mut self, policy: CenterFocusedColumn) {
        self.policy = policy;
    }

    pub fn band(&self) -> BandId {
        self.band
    }

    pub fn focused(&self) -> Option<PaneId> {
        self.bands.get(&self.band).and_then(|state| state.focus)
    }

    pub fn camera(&self) -> i64 {
        self.bands.get(&self.band).map_or(0, |state| state.camera)
    }

    pub fn resolve(&self, command: SessionCommand) -> Option<SessionAction> {
        let focused = self.focused();
        Some(match command {
            SessionCommand::OpenPane => SessionAction::OpenPane {
                band: self.band,
                after: focused,
                program: None,
            },
            SessionCommand::ClosePane => SessionAction::ClosePane(focused?),
            SessionCommand::ConsumeOrExpel(direction) => SessionAction::ConsumeOrExpel {
                pane: focused?,
                direction,
            },
            SessionCommand::CycleWidth => SessionAction::CycleWidth(focused?),
            SessionCommand::ToggleFullWidth => SessionAction::ToggleFullWidth(focused?),
            SessionCommand::StepWidth(step) => SessionAction::StepWidth {
                pane: focused?,
                step,
            },
            SessionCommand::StepHeight(step) => SessionAction::StepHeight {
                pane: focused?,
                step,
            },
            SessionCommand::ResetHeight => SessionAction::ResetHeight(focused?),
        })
    }

    pub fn shown(&self, scene: Scene<'_>) -> Vec<PaneId> {
        let Some(band) = scene.layout.band(self.band) else {
            return Vec::new();
        };
        let left = self.camera();
        let right = left + i64::from(scene.viewport.cols);
        tiles(band, scene.area)
            .into_iter()
            .filter(|tile| {
                tile.width > 0
                    && tile.height > 0
                    && i64::from(tile.x) < right
                    && i64::from(tile.span().end()) > left
                    && tile.y < scene.viewport.rows
            })
            .map(|tile| tile.pane)
            .collect()
    }

    pub fn apply(&mut self, action: ViewAction, scene: Scene<'_>) {
        let previous = self.focused();
        let Some(index) = scene.layout.band_index(self.band) else {
            self.sync(scene);
            return;
        };
        let bands = scene.layout.bands();
        let band = &bands[index];
        match action {
            ViewAction::FocusLeft | ViewAction::FocusRight => {
                if let Some(pane) = self.neighbour_column(band, action) {
                    self.set_focus(pane);
                }
            }
            ViewAction::FocusDown | ViewAction::FocusUp => {
                if let Some(pane) = self.neighbour_row(band, action) {
                    self.set_focus(pane);
                }
            }
            ViewAction::BandDown | ViewAction::BandUp => {
                let target = match action {
                    ViewAction::BandDown => index + 1,
                    _ => match index.checked_sub(1) {
                        Some(target) => target,
                        None => return,
                    },
                };
                let Some(target) = bands.get(target) else {
                    return;
                };
                self.enter(target);
            }
        }
        self.settle(scene, previous);
    }

    pub fn focus_pane(&mut self, pane: PaneId, scene: Scene<'_>) {
        let Some(location) = scene.layout.locate(pane) else {
            return;
        };
        let previous = self.focused();
        self.band = scene.layout.bands()[location.band].id;
        self.set_focus(pane);
        self.settle(scene, previous);
    }

    pub fn sync(&mut self, scene: Scene<'_>) {
        self.settle(scene, None);
    }

    fn settle(&mut self, scene: Scene<'_>, previous: Option<PaneId>) {
        let layout = scene.layout;
        self.bands.retain(|&id, _| layout.band_index(id).is_some());
        self.recency.retain(|&pane, _| layout.contains(pane));

        let index = match layout.band_index(self.band) {
            Some(index) => index,
            None => {
                let index = self.position.band.min(layout.bands().len() - 1);
                self.enter(&layout.bands()[index]);
                index
            }
        };
        let band = &layout.bands()[index];
        let focus = self.bands.entry(band.id).or_default().focus;
        match focus {
            Some(pane) if band.locate(pane).is_some() => {}
            Some(_) => match self.clamped(band) {
                Some(pane) => self.set_focus(pane),
                None => self.clear_focus(),
            },
            None => match band.first_pane() {
                Some(pane) => self.set_focus(pane),
                None => self.clear_focus(),
            },
        }
        self.position = match self.focused().and_then(|pane| layout.locate(pane)) {
            Some(location) => location,
            None => Location {
                band: index,
                column: 0,
                row: 0,
            },
        };
        self.follow(band, scene, previous);
    }

    fn enter(&mut self, band: &Band) {
        self.band = band.id;
        let state = self.bands.entry(band.id).or_default();
        let remembered = state.focus.filter(|&pane| band.locate(pane).is_some());
        state.focus = remembered.or_else(|| band.first_pane());
        if let Some(pane) = state.focus {
            self.touch(pane);
        }
    }

    fn clamped(&self, band: &Band) -> Option<PaneId> {
        let column = band
            .columns
            .get(self.position.column)
            .or(band.columns.last())?;
        let row = self.position.row.min(column.panes.len() - 1);
        Some(column.panes[row])
    }

    fn neighbour_column(&self, band: &Band, action: ViewAction) -> Option<PaneId> {
        let (column, _) = band.locate(self.focused()?)?;
        let target = match action {
            ViewAction::FocusLeft => column.checked_sub(1)?,
            _ => column + 1,
        };
        let panes = &band.columns.get(target)?.panes;
        let recent = panes
            .iter()
            .filter_map(|pane| self.recency.get(pane).map(|&tick| (tick, *pane)))
            .max()
            .map(|(_, pane)| pane);
        Some(recent.unwrap_or(panes[0]))
    }

    fn neighbour_row(&self, band: &Band, action: ViewAction) -> Option<PaneId> {
        let (column, row) = band.locate(self.focused()?)?;
        let target = match action {
            ViewAction::FocusUp => row.checked_sub(1)?,
            _ => row + 1,
        };
        band.columns[column].panes.get(target).copied()
    }

    fn set_focus(&mut self, pane: PaneId) {
        self.bands.entry(self.band).or_default().focus = Some(pane);
        self.touch(pane);
    }

    fn clear_focus(&mut self) {
        self.bands.entry(self.band).or_default().focus = None;
    }

    fn touch(&mut self, pane: PaneId) {
        self.tick += 1;
        self.recency.insert(pane, self.tick);
    }

    fn follow(&mut self, band: &Band, scene: Scene<'_>, previous: Option<PaneId>) {
        let Some((column, _)) = self.focused().and_then(|pane| band.locate(pane)) else {
            return;
        };
        let spans = column_spans(band, scene.area);
        let span = spans[column];
        let viewport = i64::from(scene.viewport.cols);
        let state = self.bands.entry(band.id).or_default();
        let centre = match self.policy {
            CenterFocusedColumn::Never => false,
            CenterFocusedColumn::Always => true,
            CenterFocusedColumn::OnOverflow => previous
                .and_then(|pane| band.locate(pane))
                .filter(|&(from, _)| from != column)
                .is_some_and(|(from, _)| {
                    let beside = if from < column {
                        column - 1
                    } else {
                        column + 1
                    };
                    let (left, right) = (spans[column.min(beside)], spans[column.max(beside)]);
                    i64::from(right.end()) - i64::from(left.x) > viewport
                }),
        };
        state.camera = if centre {
            centred(span, viewport)
        } else {
            revealed(span, viewport, state.camera)
        };
    }
}

fn centred(span: Span, viewport: i64) -> i64 {
    let width = i64::from(span.width);
    let x = i64::from(span.x);
    if width >= viewport {
        x
    } else {
        x - (viewport - width) / 2
    }
}

fn revealed(span: Span, viewport: i64, camera: i64) -> i64 {
    let (x, end, width) = (
        i64::from(span.x),
        i64::from(span.end()),
        i64::from(span.width),
    );
    if x >= camera && end <= camera + viewport {
        camera
    } else if width >= viewport || x < camera {
        x
    } else {
        end - viewport
    }
}
