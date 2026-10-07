use std::collections::HashMap;
use std::time::{Duration, Instant};

use gband_core::geometry::{Size, Tile, tiles};
use gband_core::layout::{BandId, Layout, WindowId};
use gband_core::view::{Scene, View};

pub const ANIMATIONS_VARIABLE: &str = "GBAND_ANIMATIONS";
pub const FRAME: Duration = Duration::from_millis(16);
const SETTLE: Duration = Duration::from_millis(400);
const STIFFNESS: f64 = 800.0;
pub const DEFAULT_SPEED: f64 = 1.0;
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
            tracing::warn!("{ANIMATIONS_VARIABLE} value {other:?} is not on or off, ignoring it");
            Animations::On
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Pace {
    omega: f64,
    settle: Duration,
}

impl Pace {
    fn new(speed: f64) -> Self {
        Self {
            omega: STIFFNESS.sqrt() * speed,
            settle: SETTLE.div_f64(speed),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Spring {
    from: f64,
    velocity: f64,
    target: f64,
    start: Instant,
    pace: Pace,
}

impl Spring {
    pub fn at_rest(target: f64, now: Instant) -> Self {
        Self::resting(target, now, Pace::new(DEFAULT_SPEED))
    }

    fn resting(target: f64, now: Instant, pace: Pace) -> Self {
        Self {
            from: target,
            velocity: 0.0,
            target,
            start: now,
            pace,
        }
    }

    pub fn set_speed(&mut self, speed: f64, now: Instant) {
        self.repace(Pace::new(speed), now);
    }

    fn repace(&mut self, pace: Pace, now: Instant) {
        if pace == self.pace {
            return;
        }
        *self = if self.is_at_rest(now) {
            Self::resting(self.target, now, pace)
        } else {
            Self {
                from: self.value(now),
                velocity: self.velocity(now),
                target: self.target,
                start: now,
                pace,
            }
        };
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
        if now.saturating_duration_since(self.start) >= self.pace.settle {
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
            pace: self.pace,
        };
    }

    pub fn shift(&mut self, delta: f64) {
        self.from += delta;
        self.target += delta;
    }

    pub fn snap(&mut self, now: Instant) {
        *self = Self::resting(self.target, now, self.pace);
    }

    fn elapsed(&self, now: Instant) -> f64 {
        now.saturating_duration_since(self.start).as_secs_f64()
    }

    fn raw_value(&self, t: f64) -> f64 {
        let omega = self.pace.omega;
        let displacement = self.from - self.target;
        self.target
            + (displacement + (self.velocity + omega * displacement) * t) * (-omega * t).exp()
    }

    fn raw_velocity(&self, t: f64) -> f64 {
        let omega = self.pace.omega;
        let displacement = self.from - self.target;
        (self.velocity - omega * (self.velocity + omega * displacement) * t) * (-omega * t).exp()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Targets {
    pub band: BandId,
    pub bands: Vec<BandId>,
    pub camera: i64,
    pub strip: Option<u32>,
    pub band_height: u16,
    pub tiles: Vec<Tile>,
    pub focused: Option<WindowId>,
}

impl Targets {
    pub fn new(layout: &Layout, area: Size, view: &View, terminal: Size) -> Self {
        Self {
            band: view.band(),
            bands: layout.bands().iter().map(|band| band.id).collect(),
            camera: view.travel(),
            strip: view.strip(Scene {
                layout,
                area,
                viewport: terminal,
            }),
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
    fn at_rest(tile: &Tile, now: Instant, pace: Pace) -> Self {
        Self {
            x: Spring::resting(f64::from(tile.x), now, pace),
            y: Spring::resting(f64::from(tile.y), now, pace),
            width: Spring::resting(f64::from(tile.width), now, pace),
            height: Spring::resting(f64::from(tile.height), now, pace),
        }
    }

    fn retarget(&mut self, tile: &Tile, now: Instant) {
        self.x.retarget(f64::from(tile.x), now);
        self.y.retarget(f64::from(tile.y), now);
        self.width.retarget(f64::from(tile.width), now);
        self.height.retarget(f64::from(tile.height), now);
    }

    fn placed(tile: &DrawnTile, now: Instant, pace: Pace) -> Self {
        Self {
            x: Spring::resting(tile.x as f64, now, pace),
            y: Spring::resting(tile.y as f64, now, pace),
            width: Spring::resting(f64::from(tile.width), now, pace),
            height: Spring::resting(f64::from(tile.height), now, pace),
        }
    }

    fn snap(&mut self, now: Instant) {
        for spring in self.springs_mut() {
            spring.snap(now);
        }
    }

    fn repace(&mut self, pace: Pace, now: Instant) {
        for spring in self.springs_mut() {
            spring.repace(pace, now);
        }
    }

    fn springs_mut(&mut self) -> [&mut Spring; 4] {
        [&mut self.x, &mut self.y, &mut self.width, &mut self.height]
    }

    fn target(&self) -> [i64; 4] {
        self.springs().map(|spring| spring.target().round() as i64)
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
    pub strip: Option<u32>,
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
    pace: Pace,
    band: BandId,
    bands: Vec<BandId>,
    band_height: u16,
    focused: Option<WindowId>,
    camera: Spring,
    strip: Option<u32>,
    vertical: Spring,
    tiles: HashMap<WindowId, TileSprings>,
    parked: HashMap<WindowId, [i64; 4]>,
    leaving: Vec<(BandId, i64, Option<u32>)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeldBands {
    pub top: i64,
    pub peek: Option<(BandId, i64, Option<u32>)>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Hold {
    pub camera: bool,
    pub windows: Vec<WindowId>,
    pub vertical: Option<HeldBands>,
}

#[derive(Clone, Debug)]
pub struct Presentation {
    environment: Animations,
    animations: bool,
    pace: Pace,
    snap: bool,
    shown: Option<Shown>,
    hold: Hold,
}

impl Presentation {
    pub fn new(environment: Animations) -> Self {
        Self {
            environment,
            animations: true,
            pace: Pace::new(DEFAULT_SPEED),
            snap: true,
            shown: None,
            hold: Hold::default(),
        }
    }

    pub fn snap(&mut self) {
        self.snap = true;
    }

    pub fn set_animations(&mut self, animations: bool) {
        if !animations {
            self.snap = true;
        }
        self.animations = animations;
    }

    pub fn set_speed(&mut self, speed: f64, now: Instant) {
        self.pace = Pace::new(speed);
        if let Some(shown) = &mut self.shown {
            shown.repace(self.pace, now);
        }
    }

    pub fn hold(&mut self, hold: Hold) {
        self.hold = hold;
    }

    pub fn park(&mut self, window: WindowId, tile: DrawnTile, now: Instant) {
        if let Some(shown) = &mut self.shown
            && let Some(springs) = shown.tiles.get_mut(&window)
        {
            let target = springs.target();
            *springs = TileSprings::placed(&tile, now, self.pace);
            shown.parked.insert(window, target);
        }
    }

    pub fn release_bands(&mut self, now: Instant) {
        let Some(held) = self.hold.vertical.take() else {
            return;
        };
        if let Some(shown) = &mut self.shown {
            if let Some(peek) = held.peek.filter(|&(band, _, _)| band != shown.band) {
                shown.leaving.push(peek);
            }
            let top = shown.top(shown.band).unwrap_or(0.0);
            shown.vertical = Spring::resting(held.top as f64, now, self.pace);
            shown.vertical.retarget(top, now);
        }
    }

    pub fn release(&mut self, window: WindowId, tile: DrawnTile, now: Instant) {
        if let Some(shown) = &mut self.shown {
            shown
                .tiles
                .insert(window, TileSprings::placed(&tile, now, self.pace));
            shown.parked.remove(&window);
        }
    }

    pub fn update(&mut self, now: Instant, targets: &Targets) {
        let snap = std::mem::take(&mut self.snap)
            || !self.animations
            || self.environment == Animations::Off;
        match &mut self.shown {
            Some(shown) if !snap => shown.update(now, targets),
            _ => self.shown = Some(Shown::at_rest(now, targets, self.pace)),
        }
        if let Some(shown) = &mut self.shown {
            if self.hold.camera {
                shown.camera.snap(now);
            }
            for window in &self.hold.windows {
                if let Some(springs) = shown.tiles.get_mut(window) {
                    springs.snap(now);
                }
            }
            if let Some(held) = self.hold.vertical {
                shown.vertical = Spring::resting(held.top as f64, now, self.pace);
                shown.leaving.clear();
            }
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
        let drawn_band = |band: BandId, camera: i64, strip: Option<u32>| {
            let index = shown.bands.iter().position(|&id| id == band)?;
            Some(DrawnBand {
                band,
                top: index as i64 * i64::from(shown.band_height) - vertical,
                camera,
                strip,
            })
        };
        let peek = self.hold.vertical.and_then(|held| held.peek);
        let bands = shown
            .leaving
            .iter()
            .chain(&peek)
            .filter_map(|&(band, camera, strip)| drawn_band(band, camera, strip))
            .chain(drawn_band(shown.band, shown.camera.drawn(now), shown.strip))
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
    fn at_rest(now: Instant, targets: &Targets, pace: Pace) -> Self {
        Self {
            pace,
            band: targets.band,
            bands: targets.bands.clone(),
            band_height: targets.band_height,
            focused: targets.focused,
            camera: Spring::resting(targets.camera as f64, now, pace),
            strip: targets.strip,
            vertical: Spring::resting(targets.top(targets.band).unwrap_or(0.0), now, pace),
            tiles: targets
                .tiles
                .iter()
                .map(|tile| (tile.window, TileSprings::at_rest(tile, now, pace)))
                .collect(),
            parked: HashMap::new(),
            leaving: Vec::new(),
        }
    }

    fn repace(&mut self, pace: Pace, now: Instant) {
        self.pace = pace;
        self.camera.repace(pace, now);
        self.vertical.repace(pace, now);
        for springs in self.tiles.values_mut() {
            springs.repace(pace, now);
        }
    }

    fn update(&mut self, now: Instant, targets: &Targets) {
        let top = targets.top(targets.band).unwrap_or(0.0);
        let mut snap_vertical = !self
            .leaving
            .iter()
            .all(|&(band, _, _)| targets.index_of(band).is_some());
        match (targets.top(self.band), self.top(self.band)) {
            (Some(previous), Some(before)) => self.vertical.shift(previous - before),
            _ => snap_vertical = true,
        }
        if targets.band == self.band {
            if let (Some(strip), None) = (self.strip, targets.strip) {
                let period = f64::from(strip);
                let laps = ((self.camera.value(now) - targets.camera as f64) / period).round();
                self.camera.shift(-laps * period);
            }
            self.camera.retarget(targets.camera as f64, now);
            self.retarget_tiles(now, targets);
        } else {
            if !snap_vertical {
                self.leaving
                    .push((self.band, self.camera.drawn(now), self.strip));
            }
            self.leaving.retain(|&(band, _, _)| band != targets.band);
            self.camera = Spring::resting(targets.camera as f64, now, self.pace);
            self.tiles.clear();
            self.parked.clear();
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
        self.strip = targets.strip;
        self.bands.clone_from(&targets.bands);
        self.band_height = targets.band_height;
        self.focused = targets.focused;
    }

    fn top(&self, band: BandId) -> Option<f64> {
        self.bands
            .iter()
            .position(|&id| id == band)
            .map(|index| index as f64 * f64::from(self.band_height))
    }

    fn retarget_tiles(&mut self, now: Instant, targets: &Targets) {
        let pace = self.pace;
        self.tiles
            .retain(|window, _| targets.tiles.iter().any(|tile| tile.window == *window));
        self.parked
            .retain(|window, _| targets.tiles.iter().any(|tile| tile.window == *window));
        for tile in &targets.tiles {
            let target = [
                i64::from(tile.x),
                i64::from(tile.y),
                i64::from(tile.width),
                i64::from(tile.height),
            ];
            match self.parked.get(&tile.window) {
                Some(parked) if *parked == target => continue,
                Some(_) => {
                    self.parked.remove(&tile.window);
                }
                None => {}
            }
            self.tiles
                .entry(tile.window)
                .and_modify(|springs| springs.retarget(tile, now))
                .or_insert_with(|| TileSprings::at_rest(tile, now, pace));
        }
    }
}
