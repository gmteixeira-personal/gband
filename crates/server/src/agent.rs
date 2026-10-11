use std::time::{Duration, Instant};

use gband_protocol::{Key, Value};

pub const SCREEN_LINES: usize = 20;
pub const IDLE_HOLD: Duration = Duration::from_millis(500);

const FORM_LINES: usize = SCREEN_LINES;
const STATUS_LINES: usize = 12;
const TURN_GLYPHS: [char; 6] = ['·', '✢', '✳', '✶', '✻', '✽'];
const FORM_PROMPTS: [&str; 3] = [
    "do you want to proceed?",
    "enter to confirm",
    "enter to select",
];
const CLAUDE_PACKAGE: &str = "@anthropic-ai/claude-code/";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Agent {
    Claude,
}

impl Agent {
    pub fn name(self) -> &'static str {
        match self {
            Agent::Claude => "claude",
        }
    }
}

pub fn identify(args: &[String]) -> Option<Agent> {
    let base = |index: usize| {
        args.get(index)
            .map(|argument| argument.rsplit('/').next().unwrap_or(argument))
    };
    match base(0)? {
        "claude" => Some(Agent::Claude),
        "node" | "bun"
            if args
                .get(1)
                .is_some_and(|script| script.contains(CLAUDE_PACKAGE)) =>
        {
            Some(Agent::Claude)
        }
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenState {
    Blocked,
    Working,
    Idle,
    Unknown,
}

pub fn screen_state<S: AsRef<str>>(agent: Agent, title: &str, lines: &[S]) -> ScreenState {
    let lines: Vec<String> = lines
        .iter()
        .map(|line| line.as_ref().to_ascii_lowercase())
        .filter(|line| !line.trim_matches(' ').is_empty())
        .collect();
    match agent {
        Agent::Claude => claude_rules(title, &lines),
    }
}

fn bottom(lines: &[String], count: usize) -> &[String] {
    &lines[lines.len().saturating_sub(count)..]
}

fn any_line(lines: &[String], test: impl Fn(&str) -> bool) -> bool {
    lines.iter().any(|line| test(line))
}

fn claude_rules(title: &str, lines: &[String]) -> ScreenState {
    let form = bottom(lines, FORM_LINES);
    let status = bottom(lines, STATUS_LINES);
    if spinner_title(title) {
        ScreenState::Working
    } else if any_line(form, |line| line.contains("esc to cancel"))
        && any_line(form, |line| {
            FORM_PROMPTS.iter().any(|text| line.contains(text))
        })
    {
        ScreenState::Blocked
    } else if any_line(status, |line| {
        line.contains("esc to interrupt") || turn_line(line)
    }) {
        ScreenState::Working
    } else if any_line(status, |line| line.trim_start_matches(' ').starts_with('❯')) {
        ScreenState::Idle
    } else {
        ScreenState::Unknown
    }
}

fn spinner_title(title: &str) -> bool {
    let mut chars = title.chars();
    chars
        .next()
        .is_some_and(|first| matches!(first, '\u{2800}'..='\u{28FF}' | '\u{25D0}'..='\u{25D3}'))
        && chars.next() == Some(' ')
}

fn turn_line(line: &str) -> bool {
    let mut chars = line.trim_start_matches(' ').chars();
    chars
        .next()
        .is_some_and(|first| TURN_GLYPHS.contains(&first))
        && chars.next() == Some(' ')
        && line.contains('…')
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Blocked,
    Working,
    Done,
    Idle,
    Unknown,
}

impl State {
    pub fn name(self) -> &'static str {
        match self {
            State::Blocked => "blocked",
            State::Working => "working",
            State::Done => "done",
            State::Idle => "idle",
            State::Unknown => "unknown",
        }
    }
}

#[derive(Debug)]
pub struct Tracker {
    agent: Agent,
    published: Option<State>,
    idle_since: Option<Instant>,
}

impl Tracker {
    pub fn new(agent: Agent) -> Self {
        Self {
            agent,
            published: None,
            idle_since: None,
        }
    }

    pub fn agent(&self) -> Agent {
        self.agent
    }

    pub fn state(&self) -> Option<State> {
        self.published
    }

    pub fn holding(&self) -> bool {
        self.idle_since.is_some()
    }

    pub fn observe(&mut self, screen: ScreenState, now: Instant, focused: bool) -> Option<State> {
        let state = match screen {
            ScreenState::Idle => {
                let busy = matches!(
                    self.published,
                    Some(State::Working | State::Blocked | State::Done)
                );
                if matches!(self.published, Some(State::Working | State::Blocked)) {
                    let since = *self.idle_since.get_or_insert(now);
                    if now.duration_since(since) < IDLE_HOLD {
                        return None;
                    }
                }
                if busy && !focused {
                    State::Done
                } else {
                    State::Idle
                }
            }
            ScreenState::Working => State::Working,
            ScreenState::Blocked => State::Blocked,
            ScreenState::Unknown => State::Unknown,
        };
        self.publish(state)
    }

    pub fn seen(&mut self) -> Option<State> {
        if self.published == Some(State::Done) {
            self.publish(State::Idle)
        } else {
            None
        }
    }

    fn publish(&mut self, state: State) -> Option<State> {
        self.idle_since = None;
        (self.published != Some(state)).then(|| {
            self.published = Some(state);
            state
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentChange {
    Set(Agent, State),
    Removed,
}

impl AgentChange {
    pub fn value(self) -> Option<Value> {
        match self {
            AgentChange::Set(agent, state) => Some(Value::Table(vec![
                (Key::string("agent"), Value::string(agent.name())),
                (Key::string("state"), Value::string(state.name())),
            ])),
            AgentChange::Removed => None,
        }
    }
}
