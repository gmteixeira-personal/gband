use std::time::{Duration, Instant};

use gband_client::animation::{
    Animations, Band, DrawnTile, Presentation, Spring, Targets, parse_animations,
};
use gband_core::geometry::Tile;
use gband_core::layout::{PaneId, WorkspaceId};

fn ms(millis: u64) -> Duration {
    Duration::from_millis(millis)
}

fn tile(pane: u32, x: u32, width: u16) -> Tile {
    Tile {
        pane: PaneId(pane),
        column: 0,
        row: 0,
        x,
        y: 0,
        width,
        height: 24,
    }
}

fn targets(workspace: u32, workspaces: &[u32], camera: i64, tiles: Vec<Tile>) -> Targets {
    Targets {
        workspace: WorkspaceId(workspace),
        workspaces: workspaces.iter().copied().map(WorkspaceId).collect(),
        camera,
        band: 24,
        focused: tiles.first().map(|tile| tile.pane),
        tiles,
    }
}

fn settled(workspace: u32, workspaces: &[u32], camera: i64, tiles: Vec<Tile>) -> Presentation {
    let mut presentation = Presentation::new(Animations::On);
    presentation.update(
        Instant::now(),
        &targets(workspace, workspaces, camera, tiles),
    );
    presentation
}

fn camera(presentation: &Presentation, now: Instant) -> i64 {
    presentation.drawn(now).bands.last().unwrap().camera
}

fn drawn_tile(presentation: &Presentation, now: Instant, pane: u32) -> Option<DrawnTile> {
    presentation.drawn(now).tiles.get(&PaneId(pane)).copied()
}

fn band(workspace: u32, top: i64) -> Band {
    Band {
        workspace: WorkspaceId(workspace),
        top,
        camera: 0,
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
fn pane_slides_after_an_open() {
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
fn closed_pane_is_dropped() {
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
fn switch_down_slides_both_workspaces() {
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
fn left_workspace_keeps_the_camera_it_was_drawn_with() {
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
    assert_eq!(bands[0].workspace, WorkspaceId(1));
    assert_eq!(bands[0].camera, left);
    assert_eq!(bands[1].camera, 0);
}

#[test]
fn two_switches_in_a_row_continue_the_slide() {
    let workspaces = [1, 2, 3];
    let mut presentation = settled(1, &workspaces, 0, vec![]);
    let start = Instant::now();
    presentation.update(start, &targets(2, &workspaces, 0, vec![]));
    let second = start + ms(50);
    let top = |presentation: &Presentation, now| presentation.drawn(now).bands.last().unwrap().top;
    let before = presentation.drawn(second).bands[0].top;
    presentation.update(second, &targets(3, &workspaces, 0, vec![]));
    let bands = presentation.drawn(second).bands;
    assert_eq!(bands[0].top, before);
    assert_eq!(bands.last().unwrap().workspace, WorkspaceId(3));
    let mut previous = top(&presentation, second);
    for millis in (50..=450).step_by(16) {
        let now = start + ms(millis);
        let drawn = top(&presentation, now);
        assert!(drawn <= previous);
        previous = drawn;
    }
    let end = second + ms(400);
    presentation.update(end, &targets(3, &workspaces, 0, vec![]));
    assert_eq!(presentation.drawn(end).bands, [band(3, 0)]);
}

#[test]
fn workspace_removed_above_the_viewed_one_does_not_move_it() {
    let mut presentation = settled(2, &[1, 2], 0, vec![tile(1, 0, 40)]);
    let now = Instant::now();
    presentation.update(now, &targets(2, &[2], 0, vec![tile(1, 0, 40)]));
    assert_eq!(presentation.drawn(now).bands, [band(2, 0)]);
    assert!(!presentation.is_animating(now));
}

#[test]
fn left_workspace_removed_mid_switch_snaps() {
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

#[test]
fn animations_variable() {
    assert_eq!(parse_animations(None), Animations::On);
    assert_eq!(parse_animations(Some("on")), Animations::On);
    assert_eq!(parse_animations(Some("off")), Animations::Off);
    assert_eq!(parse_animations(Some("fast")), Animations::On);
    assert_eq!(parse_animations(Some("")), Animations::On);
}
