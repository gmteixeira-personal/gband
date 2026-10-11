mod common;

use std::collections::BTreeMap;
use std::sync::Arc;

use common::*;
use gband_core::action::Action;
use gband_core::geometry::Size;
use gband_core::layout::{BandId, Layout, LayoutOptions, WindowId};
use gband_core::view::ViewAction;
use gband_lua::{BandState, Config, Dispatch, Outcome, ViewState, WindowStates};
use gband_protocol::{Key, Value};

const AREA: Size = Size::new(80, 24);

fn opened(layout: &mut Layout, band: usize, after: Option<WindowId>) -> WindowId {
    let window = layout.allocate_window();
    let id = layout.bands()[band].id;
    layout.open(window, id, after, None, &LayoutOptions::default());
    window
}

fn agent(state: &str) -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "agent_state".to_owned(),
        Value::Table(vec![
            (Key::string("agent"), Value::string("claude")),
            (Key::string("state"), Value::string(state)),
        ]),
    )])
}

fn view(layout: Layout, agents: &[WindowId], focused: Option<WindowId>) -> ViewState {
    let states: WindowStates = agents
        .iter()
        .map(|&window| (window, agent("working")))
        .collect();
    let count = layout.bands().len() as u32;
    ViewState {
        table: "root".to_owned(),
        band: BandState {
            number: 1,
            index: 1,
            count,
        },
        window: focused.map(|window| window.0),
        width: 80,
        layout: Arc::new(layout),
        area: AREA,
        ribbon: Size::new(80, 23),
        states: Arc::new(states),
        ..ViewState::default()
    }
}

struct Agents {
    config: Config,
    _scratch: Scratch,
}

impl Agents {
    fn new(name: &str, state: ViewState) -> Self {
        let scratch = Scratch::new(name);
        scratch.write(JOB);
        let config = scratch.loaded();
        clean(&config.runtime.set_state(state));
        Self {
            config,
            _scratch: scratch,
        }
    }

    fn windows(&self) -> Vec<u32> {
        eval(
            &self.config,
            "local out = {} for _, e in ipairs(gband.agents.list()) do out[#out + 1] = e.window end return out",
        )
    }

    fn run(&self, code: &str) -> Outcome {
        run_job(&self.config, code)
    }
}

fn focus(window: WindowId) -> Dispatch {
    Dispatch::Action(Action::View(ViewAction::FocusWindow(window)))
}

fn three_agents() -> (Layout, [WindowId; 3]) {
    let mut layout = Layout::new();
    let one = opened(&mut layout, 0, None);
    let two = opened(&mut layout, 1, None);
    let three = opened(&mut layout, 0, None);
    let band = &layout.bands()[0];
    assert_eq!(band.columns[0].windows, [three]);
    (layout, [three, one, two])
}

#[test]
fn agents_in_layout_order() {
    let (layout, order) = three_agents();
    let agents = Agents::new("agents-order", view(layout, &order, None));
    assert_eq!(agents.windows(), order.map(|window| window.0));
    let entry: Vec<String> = eval(
        &agents.config,
        "local e = gband.agents.list()[3] return { tostring(e.window), tostring(e.band), e.agent, e.state }",
    );
    assert_eq!(
        entry,
        [
            order[2].0.to_string(),
            "2".to_owned(),
            "claude".to_owned(),
            "working".to_owned()
        ]
    );
}

#[test]
fn floating_agents_after_the_tiled_ones() {
    let mut layout = Layout::new();
    let tiled = opened(&mut layout, 0, None);
    let floating = layout.allocate_window();
    layout.open_floating(floating, BandId(1), None, AREA, &LayoutOptions::default());
    let later = opened(&mut layout, 0, Some(tiled));
    let agents = Agents::new(
        "agents-floating",
        view(layout, &[floating, tiled, later], None),
    );
    assert_eq!(agents.windows(), [tiled.0, later.0, floating.0]);
}

#[test]
fn plain_windows_left_out() {
    let mut layout = Layout::new();
    let shell = opened(&mut layout, 0, None);
    let claude = opened(&mut layout, 0, Some(shell));
    let agents = Agents::new("agents-plain", view(layout, &[claude], None));
    assert_eq!(agents.windows(), [claude.0]);
}

#[test]
fn focus_by_position() {
    let (layout, order) = three_agents();
    let agents = Agents::new("agents-focus", view(layout, &order, Some(order[0])));
    let outcome = agents.run("focused = gband.agents.focus(3)");
    clean(&outcome);
    assert_eq!(outcome.dispatched, [focus(order[2])]);
    assert!(global::<bool>(&agents.config, "focused"));
}

#[test]
fn no_such_position() {
    let (layout, order) = three_agents();
    let agents = Agents::new("agents-missing", view(layout, &order, Some(order[0])));
    let outcome = agents.run("focused = gband.agents.focus(5)");
    clean(&outcome);
    assert!(outcome.dispatched.is_empty());
    assert!(!global::<bool>(&agents.config, "focused"));
}

#[test]
fn step_wraps() {
    let (layout, order) = three_agents();
    let agents = Agents::new("agents-wrap", view(layout.clone(), &order, Some(order[2])));
    let outcome = agents.run("stepped = gband.agents.step(1)");
    clean(&outcome);
    assert_eq!(outcome.dispatched, [focus(order[0])]);
    assert_eq!(global::<u32>(&agents.config, "stepped"), order[0].0);
    let agents = Agents::new("agents-wrap-back", view(layout, &order, Some(order[0])));
    let outcome = agents.run("stepped = gband.agents.step(-1)");
    assert_eq!(outcome.dispatched, [focus(order[2])]);
}

#[test]
fn step_from_a_plain_window() {
    let mut layout = Layout::new();
    let shell = opened(&mut layout, 0, None);
    let first = opened(&mut layout, 0, Some(shell));
    let last = opened(&mut layout, 0, Some(first));
    let state = view(layout, &[first, last], Some(shell));
    let agents = Agents::new("agents-plain-step", state.clone());
    assert_eq!(
        agents.run("gband.agents.step(-1)").dispatched,
        [focus(last)]
    );
    let agents = Agents::new("agents-plain-step-down", state);
    assert_eq!(
        agents.run("gband.agents.step(1)").dispatched,
        [focus(first)]
    );
}

#[test]
fn step_with_no_agent() {
    let mut layout = Layout::new();
    let shell = opened(&mut layout, 0, None);
    let agents = Agents::new("agents-none", view(layout, &[], Some(shell)));
    let outcome = agents.run("stepped = gband.agents.step(1) == nil");
    clean(&outcome);
    assert!(outcome.dispatched.is_empty());
    assert!(global::<bool>(&agents.config, "stepped"));
}

#[test]
fn bad_step() {
    let (layout, order) = three_agents();
    for (name, call) in [
        ("agents-bad-step", "gband.agents.step(2)"),
        ("agents-bad-focus", "gband.agents.focus(1.5)"),
        ("agents-zero-focus", "gband.agents.focus(0)"),
    ] {
        let agents = Agents::new(name, view(layout.clone(), &order, Some(order[0])));
        let outcome = agents.run(&format!("local a = 1\nlocal b = 2\n{call}"));
        assert!(outcome.dispatched.is_empty(), "{call}");
        let [error] = outcome.errors.as_slice() else {
            panic!("{call}: {:?}", outcome.errors);
        };
        assert_eq!(
            error.location.as_ref().map(|(_, line)| *line),
            Some(4),
            "{call}"
        );
    }
}
