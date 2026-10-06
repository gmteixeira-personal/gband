use std::time::{Duration, Instant};

use gband_client::animation::{
    Animations, DrawnBand, DrawnTile, Hold, Presentation, Spring, Targets, parse_animations,
};
use gband_client::render::RegionKind;
use gband_client::{Controls, Display};
use gband_core::geometry::{Size, Tile, drawn_copy};
use gband_core::input::{Modifiers, MouseButton, MouseEvent, MouseKind};
use gband_core::layout::{
    BandId, Direction, Layout, LayoutOptions, Proportion, SessionAction, WindowHeight, WindowId,
};
use gband_core::view::{Scene, View, ViewAction};
use gband_lua::keys::parse_key;
use gband_protocol::ServerMessage;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

fn ms(millis: u64) -> Duration {
    Duration::from_millis(millis)
}

fn tile(window: u32, x: u32, width: u16) -> Tile {
    Tile {
        window: WindowId(window),
        column: 0,
        row: 0,
        x,
        y: 0,
        width,
        height: 24,
    }
}

fn targets(band: u32, bands: &[u32], camera: i64, tiles: Vec<Tile>) -> Targets {
    Targets {
        band: BandId(band),
        bands: bands.iter().copied().map(BandId).collect(),
        camera,
        strip: None,
        band_height: 24,
        focused: tiles.first().map(|tile| tile.window),
        tiles,
    }
}

fn settled(band: u32, bands: &[u32], camera: i64, tiles: Vec<Tile>) -> Presentation {
    let mut presentation = Presentation::new(Animations::On);
    presentation.update(Instant::now(), &targets(band, bands, camera, tiles));
    presentation
}

fn camera(presentation: &Presentation, now: Instant) -> i64 {
    presentation.drawn(now).bands.last().unwrap().camera
}

fn drawn_tile(presentation: &Presentation, now: Instant, window: u32) -> Option<DrawnTile> {
    presentation
        .drawn(now)
        .tiles
        .get(&WindowId(window))
        .copied()
}

fn band(band: u32, top: i64) -> DrawnBand {
    DrawnBand {
        band: BandId(band),
        top,
        camera: 0,
        strip: None,
    }
}

#[test]
fn spring_from_rest_reaches_its_target() {
    let start = Instant::now();
    let mut spring = Spring::at_rest(0.0, start);
    spring.retarget(40.0, start);
    assert_eq!(spring.drawn(start), 0);
    assert_eq!(spring.drawn(start + ms(50)), 17);
    assert_eq!(spring.drawn(start + ms(150)), 37);
    assert!(!spring.is_at_rest(start + ms(150)));
    assert!(spring.is_at_rest(start + ms(400)));
    assert_eq!(spring.value(start + ms(400)), 40.0);
}

#[test]
fn spring_from_rest_never_passes_its_target() {
    let start = Instant::now();
    let mut spring = Spring::at_rest(0.0, start);
    spring.retarget(40.0, start);
    let mut previous = 0;
    for millis in 0..=400 {
        let drawn = spring.drawn(start + ms(millis));
        assert!((0..=40).contains(&drawn));
        assert!(drawn >= previous);
        previous = drawn;
    }
}

#[test]
fn spring_retarget_mid_flight_keeps_its_value_and_velocity() {
    let start = Instant::now();
    let mut spring = Spring::at_rest(0.0, start);
    spring.retarget(40.0, start);
    let turn = (0..400)
        .map(|millis| start + ms(millis))
        .find(|&now| spring.drawn(now) >= 20)
        .unwrap();
    let before = spring.drawn(turn);
    let velocity = spring.velocity(turn);
    spring.retarget(0.0, turn);
    assert!((spring.drawn(turn) - before).abs() <= 1);
    assert_eq!(spring.velocity(turn), velocity);
    assert!((spring.drawn(turn + ms(1)) - before).abs() <= 1);
    assert!(spring.is_at_rest(turn + ms(400)));
    assert_eq!(spring.drawn(turn + ms(400)), 0);
}

#[test]
fn camera_move_glides_to_its_target() {
    let tiles = vec![tile(1, 0, 40), tile(2, 40, 40), tile(3, 80, 40)];
    let mut presentation = settled(1, &[1, 2], 0, tiles.clone());
    let start = Instant::now();
    presentation.update(start, &targets(1, &[1, 2], 40, tiles));
    assert_eq!(camera(&presentation, start), 0);
    assert!(presentation.is_animating(start));
    let middle = camera(&presentation, start + ms(50));
    assert!(0 < middle && middle < 40);
    assert_eq!(camera(&presentation, start + ms(400)), 40);
    assert!(!presentation.is_animating(start + ms(400)));
}

#[test]
fn window_slides_after_an_open() {
    let mut presentation = settled(1, &[1], 0, vec![tile(1, 0, 30), tile(2, 30, 30)]);
    let start = Instant::now();
    let opened = vec![tile(1, 0, 30), tile(3, 30, 45), tile(2, 75, 30)];
    presentation.update(start, &targets(1, &[1], 0, opened));
    assert_eq!(drawn_tile(&presentation, start, 2).unwrap().x, 30);
    assert_eq!(drawn_tile(&presentation, start, 3).unwrap().x, 30);
    let middle = drawn_tile(&presentation, start + ms(50), 2).unwrap().x;
    assert!(30 < middle && middle < 75);
    assert_eq!(drawn_tile(&presentation, start + ms(400), 2).unwrap().x, 75);
}

#[test]
fn closed_window_is_dropped() {
    let tiles = vec![tile(1, 0, 30), tile(2, 30, 30), tile(3, 60, 30)];
    let mut presentation = settled(1, &[1], 0, tiles);
    let start = Instant::now();
    presentation.update(
        start,
        &targets(1, &[1], 0, vec![tile(1, 0, 30), tile(3, 30, 30)]),
    );
    assert_eq!(drawn_tile(&presentation, start, 2), None);
    assert_eq!(drawn_tile(&presentation, start, 3).unwrap().x, 60);
    assert_eq!(drawn_tile(&presentation, start + ms(400), 3).unwrap().x, 30);
}

#[test]
fn width_change_morphs_the_tile() {
    let mut presentation = settled(1, &[1], 0, vec![tile(1, 0, 40)]);
    let start = Instant::now();
    presentation.update(start, &targets(1, &[1], 0, vec![tile(1, 0, 53)]));
    assert_eq!(drawn_tile(&presentation, start, 1).unwrap().width, 40);
    let middle = drawn_tile(&presentation, start + ms(50), 1).unwrap().width;
    assert!(40 < middle && middle < 53);
    assert_eq!(
        drawn_tile(&presentation, start + ms(400), 1).unwrap().width,
        53
    );
}

#[test]
fn switch_down_slides_both_bands() {
    let mut presentation = settled(1, &[1, 2], 0, vec![tile(1, 0, 40)]);
    let start = Instant::now();
    presentation.update(start, &targets(2, &[1, 2], 0, vec![tile(2, 0, 40)]));
    assert_eq!(presentation.drawn(start).bands, [band(1, 0), band(2, 24)]);
    let bands = presentation.drawn(start + ms(50)).bands;
    assert!(bands[0].top < 0 && bands[0].top > -24);
    assert_eq!(bands[1].top, bands[0].top + 24);
    let end = start + ms(400);
    presentation.update(end, &targets(2, &[1, 2], 0, vec![tile(2, 0, 40)]));
    assert_eq!(presentation.drawn(end).bands, [band(2, 0)]);
    assert!(!presentation.is_animating(end));
}

#[test]
fn left_band_keeps_the_camera_it_was_drawn_with() {
    let mut presentation = settled(1, &[1, 2], 0, vec![tile(1, 0, 40), tile(2, 40, 40)]);
    let start = Instant::now();
    presentation.update(
        start,
        &targets(1, &[1, 2], 40, vec![tile(1, 0, 40), tile(2, 40, 40)]),
    );
    let switch = start + ms(50);
    let left = camera(&presentation, switch);
    presentation.update(switch, &targets(2, &[1, 2], 0, vec![tile(3, 0, 40)]));
    let bands = presentation.drawn(switch + ms(20)).bands;
    assert_eq!(bands[0].band, BandId(1));
    assert_eq!(bands[0].camera, left);
    assert_eq!(bands[1].camera, 0);
}

#[test]
fn two_switches_in_a_row_continue_the_slide() {
    let bands = [1, 2, 3];
    let mut presentation = settled(1, &bands, 0, vec![]);
    let start = Instant::now();
    presentation.update(start, &targets(2, &bands, 0, vec![]));
    let second = start + ms(50);
    let top = |presentation: &Presentation, now| presentation.drawn(now).bands.last().unwrap().top;
    let before = presentation.drawn(second).bands[0].top;
    presentation.update(second, &targets(3, &bands, 0, vec![]));
    let shown = presentation.drawn(second).bands;
    assert_eq!(shown[0].top, before);
    assert_eq!(shown.last().unwrap().band, BandId(3));
    let mut previous = top(&presentation, second);
    for millis in (50..=450).step_by(16) {
        let now = start + ms(millis);
        let drawn = top(&presentation, now);
        assert!(drawn <= previous);
        previous = drawn;
    }
    let end = second + ms(400);
    presentation.update(end, &targets(3, &bands, 0, vec![]));
    assert_eq!(presentation.drawn(end).bands, [band(3, 0)]);
}

#[test]
fn band_removed_above_the_viewed_one_does_not_move_it() {
    let mut presentation = settled(2, &[1, 2], 0, vec![tile(1, 0, 40)]);
    let now = Instant::now();
    presentation.update(now, &targets(2, &[2], 0, vec![tile(1, 0, 40)]));
    assert_eq!(presentation.drawn(now).bands, [band(2, 0)]);
    assert!(!presentation.is_animating(now));
}

#[test]
fn left_band_removed_mid_switch_snaps() {
    let mut presentation = settled(1, &[1, 2], 0, vec![tile(1, 0, 40)]);
    let start = Instant::now();
    presentation.update(start, &targets(2, &[1, 2], 0, vec![tile(2, 0, 40)]));
    let removed = start + ms(50);
    presentation.update(removed, &targets(2, &[2], 0, vec![tile(2, 0, 40)]));
    assert_eq!(presentation.drawn(removed).bands, [band(2, 0)]);
    assert!(!presentation.is_animating(removed));
}

#[test]
fn snap_request_draws_the_targets() {
    let tiles = vec![tile(1, 0, 40), tile(2, 40, 40), tile(3, 80, 40)];
    let mut presentation = settled(1, &[1], 0, tiles.clone());
    let start = Instant::now();
    presentation.update(start, &targets(1, &[1], 40, tiles.clone()));
    presentation.snap();
    let now = start + ms(20);
    presentation.update(now, &targets(1, &[1], 40, tiles));
    assert_eq!(camera(&presentation, now), 40);
    assert!(!presentation.is_animating(now));
}

#[test]
fn disabled_presentation_always_draws_the_targets() {
    let mut presentation = Presentation::new(Animations::Off);
    let now = Instant::now();
    presentation.update(now, &targets(1, &[1, 2], 0, vec![tile(1, 0, 40)]));
    presentation.update(now, &targets(1, &[1, 2], 40, vec![tile(1, 0, 53)]));
    assert_eq!(camera(&presentation, now), 40);
    assert_eq!(drawn_tile(&presentation, now, 1).unwrap().width, 53);
    presentation.update(now, &targets(2, &[1, 2], 0, vec![]));
    assert_eq!(presentation.drawn(now).bands, [band(2, 0)]);
    assert!(!presentation.is_animating(now));
}

#[test]
fn scrolling_camera_is_not_settled() {
    let tiles = vec![tile(1, 0, 40), tile(2, 40, 40)];
    let mut presentation = settled(1, &[1], 0, tiles.clone());
    let start = Instant::now();
    presentation.update(start, &targets(1, &[1], 40, tiles));
    assert!(!presentation.is_settled(start + ms(50)));
    assert!(!presentation.drawn(start + ms(50)).settled);
    assert!(presentation.is_settled(start + ms(400)));
    assert!(presentation.drawn(start + ms(400)).settled);
}

#[test]
fn moving_unfocused_tile_keeps_the_presentation_settled() {
    let mut presentation = settled(1, &[1], 0, vec![tile(1, 0, 30), tile(2, 30, 30)]);
    let start = Instant::now();
    presentation.update(
        start,
        &targets(1, &[1], 0, vec![tile(1, 0, 30), tile(2, 60, 30)]),
    );
    assert!(presentation.is_animating(start + ms(50)));
    assert!(presentation.is_settled(start + ms(50)));
}

const AREA: Size = Size::new(80, 24);

fn three_columns() -> (Layout, [WindowId; 3]) {
    let mut layout = Layout::new();
    let band = layout.bands()[0].id;
    let mut windows = [WindowId(0); 3];
    for index in 0..3usize {
        let window = layout.allocate_window();
        let after = index.checked_sub(1).map(|before| windows[before]);
        layout.open(window, band, after, None, &LayoutOptions::default());
        windows[index] = window;
    }
    (layout, windows)
}

fn layout_targets(layout: &Layout) -> Targets {
    let scene = Scene {
        layout,
        area: AREA,
        viewport: AREA,
    };
    Targets::new(layout, AREA, &View::new(scene), AREA)
}

fn toggle(layout: &mut Layout, window: WindowId) {
    layout.apply(
        SessionAction::ToggleFloating {
            window,
            after: None,
        },
        AREA,
        &LayoutOptions::default(),
    );
}

#[test]
fn float_a_window_between_two_columns() {
    let (mut layout, [_, b, c]) = three_columns();
    let mut presentation = Presentation::new(Animations::On);
    presentation.update(Instant::now(), &layout_targets(&layout));
    toggle(&mut layout, b);
    let start = Instant::now();
    presentation.update(start, &layout_targets(&layout));
    assert_eq!(drawn_tile(&presentation, start, b.0), None);
    assert_eq!(drawn_tile(&presentation, start, c.0).unwrap().x, 80);
    let middle = drawn_tile(&presentation, start + ms(50), c.0).unwrap().x;
    assert!(40 < middle && middle < 80);
    assert_eq!(
        drawn_tile(&presentation, start + ms(400), c.0).unwrap().x,
        40
    );
}

#[test]
fn tiled_window_appears_at_rest() {
    let (mut layout, [_, b, _]) = three_columns();
    toggle(&mut layout, b);
    let mut presentation = Presentation::new(Animations::On);
    presentation.update(Instant::now(), &layout_targets(&layout));
    toggle(&mut layout, b);
    let start = Instant::now();
    presentation.update(start, &layout_targets(&layout));
    assert_eq!(drawn_tile(&presentation, start, b.0).unwrap().x, 0);
    assert_eq!(drawn_tile(&presentation, start + ms(50), b.0).unwrap().x, 0);
}

#[test]
fn moving_a_floating_window_does_not_animate() {
    let (mut layout, [_, b, _]) = three_columns();
    toggle(&mut layout, b);
    let mut presentation = Presentation::new(Animations::On);
    presentation.update(Instant::now(), &layout_targets(&layout));
    layout.apply(
        SessionAction::MoveColumn {
            window: b,
            direction: Direction::Right,
        },
        AREA,
        &LayoutOptions::default(),
    );
    let start = Instant::now();
    presentation.update(start, &layout_targets(&layout));
    assert_eq!(drawn_tile(&presentation, start, b.0), None);
    assert!(!presentation.is_animating(start));
}

fn scene(layout: &Layout) -> Scene<'_> {
    Scene {
        layout,
        area: AREA,
        viewport: AREA,
    }
}

fn view_targets(layout: &Layout, view: &View) -> Targets {
    Targets::new(layout, AREA, view, AREA)
}

fn lefts(presentation: &Presentation, now: Instant, windows: &[WindowId]) -> Vec<i64> {
    let drawn = presentation.drawn(now);
    let band = drawn.bands.last().unwrap();
    windows
        .iter()
        .map(|window| {
            let left = drawn.tiles[window].x - band.camera;
            band.strip
                .map_or(left, |strip| drawn_copy(left, strip, AREA.cols))
        })
        .collect()
}

fn frames(presentation: &Presentation, start: Instant, windows: &[WindowId]) -> Vec<Vec<i64>> {
    (0..=25)
        .map(|frame| lefts(presentation, start + ms(16 * frame), windows))
        .collect()
}

fn moving(frames: &[Vec<i64>], step: impl Fn(i64, i64) -> bool) -> bool {
    frames.windows(2).all(|pair| step(pair[0][0], pair[1][0]))
}

fn follows_on(frames: &[Vec<i64>]) -> bool {
    frames.iter().all(|frame| {
        let next = frame[1];
        next >= i64::from(AREA.cols) || next + 40 <= 0 || next == frame[0] + 40
    })
}

#[test]
fn scroll_right_across_the_seam() {
    let (layout, [a, b, _]) = three_columns();
    let mut view = View::new(scene(&layout));
    for _ in 0..3 {
        view.apply(ViewAction::FocusRight, scene(&layout));
    }
    assert_eq!((view.focused(), view.camera()), (Some(a), 80));
    let mut presentation = Presentation::new(Animations::On);
    presentation.update(Instant::now(), &view_targets(&layout, &view));
    view.apply(ViewAction::FocusRight, scene(&layout));
    assert_eq!((view.camera(), view.travel()), (0, 120));
    let start = Instant::now();
    presentation.update(start, &view_targets(&layout, &view));
    let frames = frames(&presentation, start, &[a, b]);
    assert_eq!(frames[0][0], 40);
    assert_eq!(frames.last().unwrap(), &[0, 40]);
    assert!(frames.iter().any(|frame| 0 < frame[0] && frame[0] < 40));
    assert!(moving(&frames, |before, after| after <= before));
    assert!(follows_on(&frames));
}

#[test]
fn scroll_left_across_the_seam() {
    let (layout, [a, _, c]) = three_columns();
    let mut view = View::new(scene(&layout));
    let mut presentation = Presentation::new(Animations::On);
    presentation.update(Instant::now(), &view_targets(&layout, &view));
    view.apply(ViewAction::FocusLeft, scene(&layout));
    assert_eq!(
        (view.focused(), view.camera(), view.travel()),
        (Some(c), 80, -40)
    );
    let start = Instant::now();
    presentation.update(start, &view_targets(&layout, &view));
    let frames = frames(&presentation, start, &[c, a]);
    assert_eq!(frames[0][1], 0);
    assert_eq!(frames.last().unwrap(), &[0, 40]);
    assert!(frames.iter().any(|frame| -40 < frame[0] && frame[0] < 0));
    assert!(moving(&frames, |before, after| after >= before));
    assert!(follows_on(&frames));
}

#[test]
fn band_that_stops_looping_scrolls_on_without_a_jump() {
    let (mut layout, [a, _, c]) = three_columns();
    let mut view = View::new(scene(&layout));
    for _ in 0..3 {
        view.apply(ViewAction::FocusRight, scene(&layout));
    }
    let mut presentation = Presentation::new(Animations::On);
    presentation.update(Instant::now(), &view_targets(&layout, &view));
    assert_eq!(lefts(&presentation, Instant::now(), &[c, a]), [0, 40]);
    layout.remove(c);
    view.sync(scene(&layout));
    assert_eq!(view.strip(scene(&layout)), None);
    assert_eq!((view.camera(), view.travel()), (0, 0));
    let start = Instant::now();
    presentation.update(start, &view_targets(&layout, &view));
    let frames = frames(&presentation, start, &[a]);
    assert_eq!(frames[0], [40]);
    assert_eq!(frames.last().unwrap(), &[0]);
    assert!(moving(&frames, |before, after| after <= before));
}

#[test]
fn animations_variable() {
    assert_eq!(parse_animations(None), Animations::On);
    assert_eq!(parse_animations(Some("on")), Animations::On);
    assert_eq!(parse_animations(Some("off")), Animations::Off);
    assert_eq!(parse_animations(Some("fast")), Animations::On);
    assert_eq!(parse_animations(Some("")), Animations::On);
}

#[test]
fn parked_tile_stays_until_its_target_changes_then_animates() {
    let now = Instant::now();
    let old = vec![tile(1, 0, 40), tile(2, 40, 40)];
    let mut presentation = settled(1, &[1], 0, old.clone());
    let released = DrawnTile {
        x: 55,
        y: 2,
        width: 40,
        height: 24,
    };
    presentation.park(WindowId(1), released, now);
    presentation.update(now + ms(16), &targets(1, &[1], 0, old));
    assert_eq!(drawn_tile(&presentation, now + ms(16), 1), Some(released));
    let moved = vec![tile(2, 0, 40), tile(1, 40, 40)];
    presentation.update(now + ms(32), &targets(1, &[1], 0, moved));
    let midway = drawn_tile(&presentation, now + ms(80), 1).unwrap();
    assert!(midway.x < 55 && midway.x > 40, "{midway:?}");
    assert!(midway.y < 2, "{midway:?}");
    assert_eq!(
        drawn_tile(&presentation, now + ms(1000), 1).map(|drawn| drawn.x),
        Some(40)
    );
}

#[test]
fn released_tile_glides_back_to_its_slot() {
    let now = Instant::now();
    let slots = vec![tile(1, 0, 40), tile(2, 40, 40)];
    let mut presentation = settled(1, &[1], 0, slots.clone());
    let released = DrawnTile {
        x: 30,
        y: 0,
        width: 40,
        height: 24,
    };
    presentation.release(WindowId(1), released, now);
    presentation.update(now + ms(16), &targets(1, &[1], 0, slots));
    let midway = drawn_tile(&presentation, now + ms(32), 1).unwrap();
    assert!(midway.x > 0 && midway.x < 30, "{midway:?}");
}

#[test]
fn held_camera_and_windows_draw_at_once() {
    let now = Instant::now();
    let mut presentation = settled(1, &[1], 0, vec![tile(1, 0, 40)]);
    presentation.hold(Hold {
        camera: true,
        windows: vec![WindowId(1)],
    });
    presentation.update(now + ms(16), &targets(1, &[1], 25, vec![tile(1, 0, 48)]));
    assert_eq!(camera(&presentation, now + ms(16)), 25);
    assert_eq!(
        drawn_tile(&presentation, now + ms(16), 1).map(|drawn| drawn.width),
        Some(48)
    );
    presentation.hold(Hold::default());
    presentation.update(now + ms(32), &targets(1, &[1], 0, vec![tile(1, 0, 40)]));
    assert_ne!(camera(&presentation, now + ms(48)), 0);
}

fn draw(display: &mut Display) {
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal
        .draw(|frame| display.draw(frame, Instant::now()))
        .unwrap();
}

#[test]
fn floating_box_follows_the_pointer_at_once() {
    let area = Size::new(80, 24);
    let mut layout = Layout::new();
    let band = layout.bands()[0].id;
    let tiled = layout.allocate_window();
    let floating = layout.allocate_window();
    for window in [tiled, floating] {
        layout.open(
            window,
            band,
            Some(tiled).filter(|_| window != tiled),
            None,
            &LayoutOptions::default(),
        );
    }
    for action in [
        SessionAction::ToggleFloating {
            window: floating,
            after: None,
        },
        SessionAction::SetWidth {
            window: floating,
            width: Proportion::new(1, 4),
        },
        SessionAction::SetHeight {
            window: floating,
            height: WindowHeight::Fixed(8),
        },
        SessionAction::SetPosition {
            window: floating,
            col: 10,
            row: 4,
        },
    ] {
        layout.apply(action, area, &LayoutOptions::default());
    }
    let mut display = Display::new(area, Animations::On);
    display.apply(ServerMessage::Layout {
        cols: area.cols,
        rows: area.rows,
        layout,
    });
    let mut controls = Controls::new(gband_lua::defaults(gband_lua::Side::Client), &mut display);
    controls.refresh(&mut display);
    controls.press(&mut display, parse_key("ctrl+space").unwrap());
    draw(&mut display);
    let boxed = |display: &Display| {
        display
            .regions()
            .iter()
            .find(|region| region.kind == RegionKind::Floating)
            .map(|region| (region.x, region.y))
            .unwrap()
    };
    let (x, y) = boxed(&display);
    let (col, row) = (x as u16 + 2, y as u16 + 2);
    let left = MouseButton::Left;
    let at = |kind, col| MouseEvent::new(kind, col, row, Modifiers::NONE);
    controls.mouse(
        &mut display,
        at(MouseKind::Press(left), col),
        Instant::now(),
    );
    controls.mouse(
        &mut display,
        at(MouseKind::Motion(Some(left)), col + 5),
        Instant::now(),
    );
    draw(&mut display);
    assert_eq!(boxed(&display), (x + 5, y));
}
