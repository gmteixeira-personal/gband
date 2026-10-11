use std::time::{Duration, Instant};

use gband_server::agent::{Agent, IDLE_HOLD, ScreenState, State, Tracker, identify, screen_state};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|argument| argument.to_string()).collect()
}

fn claude(title: &str, lines: &[&str]) -> ScreenState {
    screen_state(Agent::Claude, title, lines)
}

fn fixture(text: &str) -> ScreenState {
    let mut lines = text.lines();
    let title = lines.next().unwrap();
    let lines: Vec<&str> = lines.collect();
    claude(title, &lines)
}

#[test]
fn claude_code_started_from_a_shell() {
    assert_eq!(identify(&args(&["claude"])), Some(Agent::Claude));
    assert_eq!(
        identify(&args(&["/home/me/.local/bin/claude", "--resume"])),
        Some(Agent::Claude)
    );
    assert_eq!(Agent::Claude.name(), "claude");
}

#[test]
fn claude_code_run_by_node() {
    let node = args(&[
        "node",
        "/usr/lib/node_modules/@anthropic-ai/claude-code/cli.js",
    ]);
    assert_eq!(identify(&node), Some(Agent::Claude));
    let bun = args(&["/usr/bin/bun", "/opt/@anthropic-ai/claude-code/cli.js"]);
    assert_eq!(identify(&bun), Some(Agent::Claude));
    assert_eq!(identify(&args(&["node", "server.js"])), None);
    assert_eq!(identify(&args(&["node"])), None);
}

#[test]
fn other_program() {
    assert_eq!(identify(&args(&["vim"])), None);
    assert_eq!(identify(&args(&["claude-helper"])), None);
    assert_eq!(identify(&[]), None);
}

#[test]
fn spinner_in_the_title() {
    assert_eq!(claude("⠋ Fix the parser", &["❯ "]), ScreenState::Working);
    assert_eq!(claude("◐ Fix the parser", &[]), ScreenState::Working);
    assert_eq!(
        claude(
            "◓ Fix the parser",
            &["Do you want to proceed?", "Esc to cancel"]
        ),
        ScreenState::Working
    );
    assert_eq!(claude("◔ Fix the parser", &[]), ScreenState::Unknown);
    assert_eq!(claude("⠋Fix the parser", &[]), ScreenState::Unknown);
}

#[test]
fn permission_form() {
    let lines = [
        "Do you want to proceed?",
        " ❯ 1. Yes",
        "   2. No",
        " Esc to cancel",
    ];
    assert_eq!(claude("✳ Claude Code", &lines), ScreenState::Blocked);
    let select = ["Enter to select · Esc to cancel"];
    assert_eq!(claude("✳ Claude Code", &select), ScreenState::Blocked);
    assert_eq!(
        claude("✳ Claude Code", &["Do you want to proceed?", "❯ 1. Yes"]),
        ScreenState::Idle
    );
}

#[test]
fn turn_without_a_spinner_title() {
    let lines = [
        "✻ Whirlpooling… (14s · ↓ 2.8k tokens · thinking)",
        "────",
        "❯ ",
        "────",
    ];
    assert_eq!(claude("claude", &lines), ScreenState::Working);
    assert_eq!(
        claude("claude", &["✶ Thinking… (esc to interrupt)"]),
        ScreenState::Working
    );
}

#[test]
fn finished_turn_line() {
    let lines = ["✻ Crunched for 15s", "────", "❯ ", "────"];
    assert_eq!(claude("✳ Claude Code", &lines), ScreenState::Idle);
}

#[test]
fn prompt_shown() {
    assert_eq!(
        claude("✳ Claude Code", &["some output", "❯ "]),
        ScreenState::Idle
    );
    assert_eq!(claude("✳ Claude Code", &["   ❯ draft"]), ScreenState::Idle);
    assert_eq!(
        claude("✳ Claude Code", &["some output"]),
        ScreenState::Unknown
    );
}

#[test]
fn form_above_the_bottom_lines() {
    let mut lines = vec!["Do you want to proceed?", "Esc to cancel"];
    let filler: Vec<String> = (0..23).map(|line| format!("line {line}")).collect();
    lines.extend(filler.iter().map(String::as_str));
    lines.extend(["", "   ", "❯ "]);
    assert_eq!(claude("✳ Claude Code", &lines), ScreenState::Idle);
}

#[test]
fn case_of_the_text() {
    assert_eq!(
        claude("claude", &["ESC TO INTERRUPT"]),
        ScreenState::Working
    );
    assert_eq!(
        claude("claude", &["DO YOU WANT TO PROCEED?", "ESC TO CANCEL"]),
        ScreenState::Blocked
    );
}

#[test]
fn fixture_working() {
    let text = include_str!("fixtures/claude/working.txt");
    assert_eq!(fixture(text), ScreenState::Working);
    let untitled = text.replacen(text.lines().next().unwrap(), "claude", 1);
    assert_eq!(fixture(&untitled), ScreenState::Working);
}

#[test]
fn fixture_permission() {
    assert_eq!(
        fixture(include_str!("fixtures/claude/permission.txt")),
        ScreenState::Blocked
    );
}

#[test]
fn fixture_finished() {
    assert_eq!(
        fixture(include_str!("fixtures/claude/finished.txt")),
        ScreenState::Idle
    );
}

#[test]
fn fixture_start() {
    assert_eq!(
        fixture(include_str!("fixtures/claude/start.txt")),
        ScreenState::Idle
    );
}

fn ms(milliseconds: u64) -> Duration {
    Duration::from_millis(milliseconds)
}

fn working_tracker(start: Instant) -> Tracker {
    let mut tracker = Tracker::new(Agent::Claude);
    assert_eq!(
        tracker.observe(ScreenState::Working, start, false),
        Some(State::Working)
    );
    tracker
}

#[test]
fn state_of_a_new_agent_window() {
    let mut tracker = Tracker::new(Agent::Claude);
    assert_eq!(tracker.state(), None);
    assert_eq!(
        tracker.observe(ScreenState::Unknown, Instant::now(), false),
        Some(State::Unknown)
    );
    assert_eq!(
        tracker.observe(ScreenState::Unknown, Instant::now(), false),
        None
    );
    assert_eq!(tracker.state(), Some(State::Unknown));
}

#[test]
fn agent_finishes_in_the_background() {
    let start = Instant::now();
    let mut tracker = working_tracker(start);
    assert_eq!(tracker.observe(ScreenState::Idle, start, false), None);
    assert_eq!(
        tracker.observe(ScreenState::Idle, start + IDLE_HOLD, false),
        Some(State::Done)
    );
}

#[test]
fn focus_clears_done() {
    let start = Instant::now();
    let mut tracker = working_tracker(start);
    tracker.observe(ScreenState::Idle, start, false);
    tracker.observe(ScreenState::Idle, start + IDLE_HOLD, false);
    assert_eq!(tracker.seen(), Some(State::Idle));
    assert_eq!(tracker.seen(), None);
    assert_eq!(
        tracker.observe(ScreenState::Idle, start + ms(900), false),
        None
    );
    assert_eq!(tracker.state(), Some(State::Idle));
}

#[test]
fn done_stays_until_seen() {
    let start = Instant::now();
    let mut tracker = working_tracker(start);
    tracker.observe(ScreenState::Idle, start, false);
    tracker.observe(ScreenState::Idle, start + IDLE_HOLD, false);
    assert_eq!(
        tracker.observe(ScreenState::Idle, start + ms(2000), false),
        None
    );
    assert_eq!(tracker.state(), Some(State::Done));
    assert_eq!(
        tracker.observe(ScreenState::Working, start + ms(2100), false),
        Some(State::Working)
    );
}

#[test]
fn agent_finishes_in_the_focused_window() {
    let start = Instant::now();
    let mut tracker = working_tracker(start);
    tracker.observe(ScreenState::Idle, start, true);
    assert_eq!(
        tracker.observe(ScreenState::Idle, start + IDLE_HOLD, true),
        Some(State::Idle)
    );
}

#[test]
fn spinner_pauses() {
    let start = Instant::now();
    let mut tracker = working_tracker(start);
    assert_eq!(
        tracker.observe(ScreenState::Idle, start + ms(100), false),
        None
    );
    assert!(tracker.holding());
    assert_eq!(
        tracker.observe(ScreenState::Idle, start + ms(300), false),
        None
    );
    assert_eq!(
        tracker.observe(ScreenState::Working, start + ms(300), false),
        None
    );
    assert!(!tracker.holding());
    assert_eq!(
        tracker.observe(ScreenState::Idle, start + ms(700), false),
        None
    );
    assert_eq!(tracker.state(), Some(State::Working));
}

#[test]
fn turn_ends() {
    let start = Instant::now();
    let mut tracker = working_tracker(start);
    let mut published = None;
    for step in 0..=10 {
        let now = start + ms(100 * step);
        if let Some(state) = tracker.observe(ScreenState::Idle, now, false) {
            published = Some((state, now - start));
        }
        if published.is_some() {
            break;
        }
    }
    let (state, after) = published.unwrap();
    assert_eq!(state, State::Done);
    assert!(
        after >= IDLE_HOLD && after < Duration::from_secs(1),
        "{after:?}"
    );
}

#[test]
fn form_appears() {
    let start = Instant::now();
    let mut tracker = working_tracker(start);
    assert_eq!(
        tracker.observe(ScreenState::Blocked, start + ms(100), false),
        Some(State::Blocked)
    );
    tracker.observe(ScreenState::Idle, start + ms(200), false);
    assert_eq!(
        tracker.observe(ScreenState::Idle, start + ms(700), false),
        Some(State::Done)
    );
}

#[test]
fn idle_reached_from_unknown() {
    let start = Instant::now();
    let mut tracker = Tracker::new(Agent::Claude);
    tracker.observe(ScreenState::Unknown, start, false);
    assert_eq!(
        tracker.observe(ScreenState::Idle, start + ms(10), false),
        Some(State::Idle)
    );
}

#[test]
fn state_names() {
    let names: Vec<_> = [
        State::Blocked,
        State::Working,
        State::Done,
        State::Idle,
        State::Unknown,
    ]
    .iter()
    .map(|state| state.name())
    .collect();
    assert_eq!(names, ["blocked", "working", "done", "idle", "unknown"]);
}
