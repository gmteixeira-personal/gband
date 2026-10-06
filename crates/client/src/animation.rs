use std::collections::HashMap;
use std::time::{Duration, Instant};

use gband_core::geometry::{Size, Tile, tiles};
use gband_core::layout::{BandId, Layout, WindowId};
use gband_core::view::View;

pub const ANIMATIONS_VARIABLE: &str = "GBAND_ANIMATIONS";
pub const FRAME: Duration = Duration::from_millis(16);
const SETTLE: Duration = Duration::from_millis(400);
const STIFFNESS: f64 = 800.0;
const REST_DISTANCE: f64 = 0.5;
const REST_SPEED: f64 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Animations {
    On,
    Off,
}

pub fn parse_animations(value: Option<&str>) -> Animations {
    match value {
        None | Some("on") => Animations::On,
        Some("off") => Animations::Off,
        Some(other) => {
            tracing::warn!("{ANIMATIONS_VARIABLE} value {other:?} is not on or off, using on");
            Animations::On
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Spring {
    from: f64,
    velocity: f64,
    target: f64,
    start: Instant,
}

impl Spring {
    pub fn at_rest(target: f64, now: Instant) -> Self {
        Self {
            from: target,
            velocity: 0.0,
            target,
            start: now,
        }
    }

    pub fn target(&self) -> f64 {
        self.target
    }

    pub fn value(&self, now: Instant) -> f64 {
        if self.is_at_rest(now) {
            return self.target;
        }
        self.raw_value(self.elapsed(now))
    }

    pub fn velocity(&self, now: Instant) -> f64 {
        if self.is_at_rest(now) {
            return 0.0;
        }
        self.raw_velocity(self.elapsed(now))
    }

    pub fn drawn(&self, now: Instant) -> i64 {
        self.value(now).round() as i64
    }

    pub fn is_at_rest(&self, now: Instant) -> bool {
        if now.saturating_duration_since(self.start) >= SETTLE {
            return true;
        }
        let elapsed = self.elapsed(now);
        (self.raw_value(elapsed) - self.target).abs() < REST_DISTANCE
            && self.raw_velocity(elapsed).abs() < REST_SPEED
    }

    pub fn retarget(&mut self, target: f64, now: Instant) {
        if target == self.target {
            return;
        }
        *self = Self {
            from: self.value(now),
            velocity: self.velocity(now),
            target,
            start: now,
        };
    }

    pub fn shift(&mut self, delta: f64) {
        self.from += delta;
        self.target += delta;
    }

    pub fn snap(&mut self, now: Instant) {
        *self = Self::at_rest(self.target, now);
    }

    fn elapsed(&self, now: Instant) -> f64 {
        now.saturating_duration_since(self.start).as_secs_f64()
    }

    fn raw_value(&self, t: f64) -> f64 {
        let omega = STIFFNESS.sqrt();
        let displacement = self.from - self.target;
        self.target
            + (displacement + (self.velocity + omega * displacement) * t) * (-omega * t).exp()
    }

    fn raw_velocity(&self, t: f64) -> f64 {
        let omega = STIFFNESS.sqrt();
        let displacement = self.from - self.target;
        (self.velocity - omega * (self.velocity + omega * displacement) * t) * (-omega * t).exp()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Targets {
    pub band: BandId,
    pub bands: Vec<BandId>,
    pub camera: i64,
    pub band_height: u16,
    pub tiles: Vec<Tile>,
    pub focused: Option<WindowId>,
}

impl Targets {
    pub fn new(layout: &Layout, area: Size, view: &View, terminal: Size) -> Self {
        Self {
            band: view.band(),
            bands: layout.bands().iter().map(|band| band.id).collect(),
            camera: view.camera(),
            band_height: terminal.rows,
            tiles: layout
                .band(view.band())
                .map(|band| tiles(band, area))
                .unwrap_or_default(),
            focused: view.focused(),
        }
    }

    fn index_of(&self, band: BandId) -> Option<usize> {
        self.bands.iter().position(|&id| id == band)
    }

    fn top(&self, band: BandId) -> Option<f64> {
        self.index_of(band)
            .map(|index| index as f64 * f64::from(self.band_height))
    }
}

#[derive(Clone, Copy, Debug)]
struct TileSprings {
    x: Spring,
    y: Spring,
    width: Spring,
    height: Spring,
}

impl TileSprings {
    fn at_rest(tile: &Tile, now: Instant) -> Self {
        Self {
            x: Spring::at_rest(f64::from(tile.x), now),
            y: Spring::at_rest(f64::from(tile.y), now),
            width: Spring::at_rest(f64::from(tile.width), now),
            height: Spring::at_rest(f64::from(tile.height), now),
        }
    }

    fn retarget(&mut self, tile: &Tile, now: Instant) {
        self.x.retarget(f64::from(tile.x), now);
        self.y.retarget(f64::from(tile.y), now);
        self.width.retarget(f64::from(tile.width), now);
        self.height.retarget(f64::from(tile.height), now);
    }

    fn springs(&self) -> [&Spring; 4] {
        [&self.x, &self.y, &self.width, &self.height]
    }

    fn is_at_rest(&self, now: Instant) -> bool {
        self.springs().iter().all(|spring| spring.is_at_rest(now))
    }

    fn drawn(&self, now: Instant) -> DrawnTile {
        DrawnTile {
            x: self.x.drawn(now),
            y: self.y.drawn(now),
            width: cells(self.width.drawn(now)),
            height: cells(self.height.drawn(now)),
        }
    }
}

fn cells(value: i64) -> u16 {
    value.clamp(0, i64::from(u16::MAX)) as u16
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawnBand {
    pub band: BandId,
    pub top: i64,
    pub camera: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawnTile {
    pub x: i64,
    pub y: i64,
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drawn {
    pub bands: Vec<DrawnBand>,
    pub tiles: HashMap<WindowId, DrawnTile>,
    pub settled: bool,
}

#[derive(Clone, Debug)]
struct Shown {
    band: BandId,
    bands: Vec<BandId>,
    band_height: u16,
    focused: Option<WindowId>,
    camera: Spring,
    vertical: Spring,
    tiles: HashMap<WindowId, TileSprings>,
    leaving: Vec<(BandId, i64)>,
}

#[derive(Clone, Debug)]
pub struct Presentation {
    animations: Animations,
    snap: bool,
    shown: Option<Shown>,
}

impl Presentation {
    pub fn new(animations: Animations) -> Self {
        Self {
            animations,
            snap: true,
            shown: None,
        }
    }

    pub fn snap(&mut self) {
        self.snap = true;
    }

    pub fn update(&mut self, now: Instant, targets: &Targets) {
        let snap = std::mem::take(&mut self.snap) || self.animations == Animations::Off;
        match &mut self.shown {
            Some(shown) if !snap => shown.update(now, targets),
            _ => self.shown = Some(Shown::at_rest(now, targets)),
        }
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        self.shown.as_ref().is_some_and(|shown| {
            !shown.camera.is_at_rest(now)
                || !shown.vertical.is_at_rest(now)
                || !shown.leaving.is_empty()
                || shown.tiles.values().any(|springs| !springs.is_at_rest(now))
        })
    }

    pub fn is_settled(&self, now: Instant) -> bool {
        self.shown.as_ref().is_none_or(|shown| {
            shown.camera.is_at_rest(now)
                && shown.vertical.is_at_rest(now)
                && shown
                    .focused
                    .and_then(|window| shown.tiles.get(&window))
                    .is_none_or(|springs| springs.is_at_rest(now))
        })
    }

    pub fn drawn(&self, now: Instant) -> Drawn {
        let Some(shown) = &self.shown else {
            return Drawn {
                bands: Vec::new(),
                tiles: HashMap::new(),
                settled: true,
            };
        };
        let vertical = shown.vertical.drawn(now);
        let drawn_band = |band: BandId, camera: i64| {
            let index = shown.bands.iter().position(|&id| id == band)?;
            Some(DrawnBand {
                band,
                top: index as i64 * i64::from(shown.band_height) - vertical,
                camera,
            })
        };
        let bands = shown
            .leaving
            .iter()
            .filter_map(|&(band, camera)| drawn_band(band, camera))
            .chain(drawn_band(shown.band, shown.camera.drawn(now)))
            .collect();
        Drawn {
            bands,
            tiles: shown
                .tiles
                .iter()
                .map(|(&window, springs)| (window, springs.drawn(now)))
                .collect(),
            settled: self.is_settled(now),
        }
    }
}

impl Shown {
    fn at_rest(now: Instant, targets: &Targets) -> Self {
        Self {
            band: targets.band,
            bands: targets.bands.clone(),
            band_height: targets.band_height,
            focused: targets.focused,
            camera: Spring::at_rest(targets.camera as f64, now),
            vertical: Spring::at_rest(targets.top(targets.band).unwrap_or(0.0), now),
            tiles: targets
                .tiles
                .iter()
                .map(|tile| (tile.window, TileSprings::at_rest(tile, now)))
                .collect(),
            leaving: Vec::new(),
        }
    }

    fn update(&mut self, now: Instant, targets: &Targets) {
        let top = targets.top(targets.band).unwrap_or(0.0);
        let mut snap_vertical = !self
            .leaving
            .iter()
            .all(|&(band, _)| targets.index_of(band).is_some());
        if let Some(previous) = targets.top(self.band) {
            self.vertical.shift(previous - self.vertical.target());
        } else {
            snap_vertical = true;
        }
        if targets.band == self.band {
            self.camera.retarget(targets.camera as f64, now);
            self.retarget_tiles(now, targets);
        } else {
            if !snap_vertical {
                self.leaving.push((self.band, self.camera.drawn(now)));
            }
            self.leaving.retain(|&(band, _)| band != targets.band);
            self.camera = Spring::at_rest(targets.camera as f64, now);
            self.tiles.clear();
            self.retarget_tiles(now, targets);
        }
        self.vertical.retarget(top, now);
        if snap_vertical {
            self.vertical.snap(now);
        }
        if snap_vertical || self.vertical.is_at_rest(now) {
            self.leaving.clear();
        }
        self.band = targets.band;
        self.bands.clone_from(&targets.bands);
        self.band_height = targets.band_height;
        self.focused = targets.focused;
    }

    fn retarget_tiles(&mut self, now: Instant, targets: &Targets) {
        self.tiles
            .retain(|window, _| targets.tiles.iter().any(|tile| tile.window == *window));
        for tile in &targets.tiles {
            self.tiles
                .entry(tile.window)
                .and_modify(|springs| springs.retarget(tile, now))
                .or_insert_with(|| TileSprings::at_rest(tile, now));
        }
    }
}
