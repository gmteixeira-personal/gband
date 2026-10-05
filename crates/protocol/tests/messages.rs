use std::fmt::Debug;
use std::path::PathBuf;

use gband_core::geometry::Size;
use gband_core::input::{Key, KeyCode, Modifiers};
use gband_core::layout::{
    Direction, Layout, LayoutOptions, PaneContent, PaneHeight, PaneId, Program, Proportion,
    SessionAction, Step, Weight,
};
use gband_protocol::test::{FromProcess, Role, ToProcess};
use gband_protocol::{
    ClientMessage, Decoder, ExecutableId, Hello, HelloReply, Key as ValueKey, PROTOCOL_VERSION,
    Requirement, ServerMessage, SessionName, SessionSummary, Value, encode,
};
use serde::Serialize;
use serde::de::DeserializeOwned;

const AREA: Size = Size::new(80, 24);

fn round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(message: T) {
    let mut decoder = Decoder::new();
    decoder.feed(&encode(&message).unwrap()).unwrap();
    assert_eq!(decoder.next_message::<T>().unwrap(), Some(message));
    assert_eq!(decoder.next_message::<T>().unwrap(), None);
}

fn session(name: &str) -> SessionName {
    name.parse().unwrap()
}

#[test]
fn protocol_version_is_six() {
    assert_eq!(PROTOCOL_VERSION, 6);
}

#[test]
fn valid_session_names_parse_and_decode() {
    let longest = "x".repeat(64);
    for name in ["default", "work-2", "a_b", longest.as_str()] {
        assert_eq!(session(name).as_str(), name);
        let mut decoder = Decoder::new();
        decoder.feed(&encode(&name.to_owned()).unwrap()).unwrap();
        assert_eq!(
            decoder.next_message::<SessionName>().unwrap(),
            Some(session(name))
        );
    }
}

#[test]
fn invalid_session_names_are_refused_when_parsed_and_decoded() {
    let too_long = "x".repeat(65);
    for name in ["", too_long.as_str(), "work.2", "a/b", "a b"] {
        assert!(name.parse::<SessionName>().is_err(), "{name:?}");
        let mut decoder = Decoder::new();
        decoder.feed(&encode(&name.to_owned()).unwrap()).unwrap();
        assert!(
            decoder.next_message::<SessionName>().is_err(),
            "{name:?} decoded"
        );
    }
}

#[test]
fn invalid_session_name_in_a_request_is_refused() {
    let mut decoder = Decoder::new();
    let valid = encode(&ClientMessage::KillSession {
        session: session("abc"),
    })
    .unwrap();
    let invalid: Vec<u8> = valid
        .iter()
        .map(|&byte| if byte == b'b' { b'/' } else { byte })
        .collect();
    decoder.feed(&invalid).unwrap();
    assert!(decoder.next_message::<ClientMessage>().is_err());
}

#[test]
fn session_name_defaults_to_default() {
    assert_eq!(SessionName::default().as_str(), "default");
}

#[test]
fn requests_round_trip() {
    round_trip(ClientMessage::Attach {
        session: session("work"),
        cwd: PathBuf::from("/tmp"),
    });
    round_trip(ClientMessage::ListSessions);
    round_trip(ClientMessage::KillSession {
        session: session("work"),
    });
}

#[test]
fn session_answers_round_trip() {
    round_trip(ServerMessage::Sessions(vec![
        SessionSummary {
            name: session("default"),
            panes: 1,
            clients: 0,
        },
        SessionSummary {
            name: session("work"),
            panes: 2,
            clients: 1,
        },
    ]));
    round_trip(ServerMessage::Sessions(Vec::new()));
    round_trip(ServerMessage::Killed);
    round_trip(ServerMessage::NoSuchSession);
}

#[test]
fn hello_pair_round_trips() {
    round_trip(Hello {
        version: PROTOCOL_VERSION,
        cols: 80,
        rows: 24,
    });
    round_trip(HelloReply::Accepted { version: 1 });
    round_trip(HelloReply::Rejected { version: 7 });
}

#[test]
fn client_messages_round_trip() {
    round_trip(ClientMessage::Key {
        pane: PaneId(1),
        key: Key::plain(KeyCode::Char('é')),
    });
    round_trip(ClientMessage::Key {
        pane: PaneId(u32::MAX),
        key: Key::new(
            KeyCode::F(12),
            Modifiers {
                shift: true,
                alt: true,
                ctrl: true,
            },
        ),
    });
    round_trip(ClientMessage::Paste {
        pane: PaneId(2),
        text: "line one\nline two".into(),
    });
    round_trip(ClientMessage::Resize {
        cols: 300,
        rows: 90,
    });
    round_trip(ClientMessage::Detach);
    round_trip(ClientMessage::Shown(vec![PaneId(1), PaneId(4)]));
    round_trip(ClientMessage::Shown(Vec::new()));
    round_trip(ClientMessage::Content {
        pane: PaneId(4),
        output: b"\x1b[1;1Hhello".to_vec(),
    });
}

#[test]
fn session_actions_round_trip() {
    let layout = Layout::new();
    let band = layout.bands()[0].id;
    for action in [
        SessionAction::open(band, None, None),
        SessionAction::open(band, Some(PaneId(4)), None),
        SessionAction::open(
            band,
            Some(PaneId(2)),
            Some(Program::Argv(vec![
                "htop".to_owned(),
                "-d".to_owned(),
                "10".to_owned(),
            ])),
        ),
        SessionAction::open(
            band,
            None,
            Some(Program::CommandLine("echo $GBAND_PANE".to_owned())),
        ),
        SessionAction::ClosePane(PaneId(5)),
        SessionAction::ConsumeOrExpel {
            pane: PaneId(3),
            direction: Direction::Left,
        },
        SessionAction::ConsumeOrExpel {
            pane: PaneId(3),
            direction: Direction::Right,
        },
        SessionAction::CycleWidth(PaneId(6)),
        SessionAction::ToggleFullWidth(PaneId(7)),
        SessionAction::StepWidth {
            pane: PaneId(8),
            step: Step::Grow,
        },
        SessionAction::StepHeight {
            pane: PaneId(2),
            step: Step::Grow,
        },
        SessionAction::StepHeight {
            pane: PaneId(2),
            step: Step::Shrink,
        },
        SessionAction::ResetHeight(PaneId(9)),
        SessionAction::OpenPane {
            band,
            after: Some(PaneId(2)),
            width: Some(Proportion::new(1, 4)),
            focus: false,
            content: PaneContent::Plugin { request: 7 },
        },
        SessionAction::SetWidth {
            pane: PaneId(1),
            width: Proportion::new(2, 5),
        },
        SessionAction::SetHeight {
            pane: PaneId(2),
            height: PaneHeight::Auto(Weight::new(3, 2)),
        },
        SessionAction::SetHeight {
            pane: PaneId(2),
            height: PaneHeight::Fixed(8),
        },
    ] {
        round_trip(ClientMessage::Action(action));
    }
}

#[test]
fn layout_round_trips() {
    let mut layout = Layout::new();
    let first = layout.allocate_pane();
    let second = layout.allocate_pane();
    let third = layout.allocate_pane();
    let w1 = layout.bands()[0].id;
    layout.open(first, w1, None, None, &LayoutOptions::default());
    layout.open(second, w1, Some(first), None, &LayoutOptions::default());
    layout.apply(
        SessionAction::ConsumeOrExpel {
            pane: second,
            direction: Direction::Left,
        },
        AREA,
        &LayoutOptions::default(),
    );
    layout.apply(
        SessionAction::CycleWidth(first),
        AREA,
        &LayoutOptions::default(),
    );
    layout.apply(
        SessionAction::CycleWidth(first),
        AREA,
        &LayoutOptions::default(),
    );
    layout.apply(
        SessionAction::ToggleFullWidth(first),
        AREA,
        &LayoutOptions::default(),
    );
    let w2 = layout.bands()[1].id;
    layout.open(third, w2, None, None, &LayoutOptions::default());
    assert_eq!(layout.bands().len(), 3);
    round_trip(ServerMessage::Layout {
        cols: 120,
        rows: 40,
        layout,
    });
}

#[test]
fn heights_in_the_layout_round_trip() {
    let mut layout = Layout::new();
    let band = layout.bands()[0].id;
    let panes: Vec<PaneId> = (0..3).map(|_| layout.allocate_pane()).collect();
    layout.open(panes[0], band, None, None, &LayoutOptions::default());
    for pair in panes.windows(2) {
        layout.open(
            pair[1],
            band,
            Some(pair[0]),
            None,
            &LayoutOptions::default(),
        );
        layout.apply(
            SessionAction::ConsumeOrExpel {
                pane: pair[1],
                direction: Direction::Left,
            },
            AREA,
            &LayoutOptions::default(),
        );
    }
    let grow = |pane| SessionAction::StepHeight {
        pane,
        step: Step::Grow,
    };
    layout.apply(grow(panes[1]), AREA, &LayoutOptions::default());
    layout.apply(grow(panes[0]), AREA, &LayoutOptions::default());
    layout.remove(panes[2]);
    layout.apply(grow(panes[0]), Size::new(80, 50), &LayoutOptions::default());
    assert_eq!(
        layout.bands()[0].columns[0].heights,
        [PaneHeight::Fixed(14), PaneHeight::Auto(Weight::new(10, 7))]
    );
    round_trip(ServerMessage::Layout {
        cols: 80,
        rows: 24,
        layout,
    });
}

#[test]
fn server_messages_round_trip() {
    round_trip(ServerMessage::Info {
        pid: 4_000_000,
        executable: ExecutableId {
            device: u64::MAX,
            inode: 42,
        },
    });
    round_trip(ServerMessage::Snapshot {
        pane: PaneId(1),
        cols: 80,
        rows: 24,
        contents: b"\x1b[H\x1b[2Jprompt$ ".to_vec(),
    });
    round_trip(ServerMessage::Update {
        pane: PaneId(1),
        contents: Vec::new(),
    });
    round_trip(ServerMessage::Update {
        pane: PaneId(9),
        contents: vec![0xff; 70_000],
    });
    round_trip(ServerMessage::Focus(PaneId(3)));
    round_trip(ServerMessage::Exited);
    round_trip(ServerMessage::Opened {
        request: 7,
        pane: Some(PaneId(9)),
    });
    round_trip(ServerMessage::Opened {
        request: 8,
        pane: None,
    });
}

#[test]
fn hello_encoding_is_pinned() {
    let hello = Hello {
        version: 1,
        cols: 80,
        rows: 24,
    };
    assert_eq!(encode(&hello).unwrap(), [0, 0, 0, 3, 0x01, 0x50, 0x18]);
}

#[test]
fn hello_reply_encoding_is_pinned() {
    assert_eq!(
        encode(&HelloReply::Accepted { version: 1 }).unwrap(),
        [0, 0, 0, 2, 0x00, 0x01]
    );
    assert_eq!(
        encode(&HelloReply::Rejected { version: 1 }).unwrap(),
        [0, 0, 0, 2, 0x01, 0x01]
    );
}

fn text(value: &str) -> Value {
    Value::string(value)
}

#[test]
fn value_round_trip() {
    let value = Value::Table(vec![
        (ValueKey::string("n"), Value::Int(3)),
        (ValueKey::string("f"), Value::Float(0.5)),
        (ValueKey::string("b"), Value::Bytes(vec![0, 255])),
        (
            ValueKey::string("list"),
            Value::Table(vec![
                (ValueKey::Int(1), text("x")),
                (ValueKey::Int(2), text("y")),
            ]),
        ),
        (ValueKey::Int(1), Value::Bool(false)),
        (ValueKey::string("1"), Value::Nil),
    ]);
    let mut decoder = Decoder::new();
    decoder.feed(&encode(&value).unwrap()).unwrap();
    let decoded = decoder.next_message::<Value>().unwrap().unwrap();
    assert_eq!(decoded, value);
    let Value::Table(entries) = decoded else {
        panic!("a table");
    };
    assert!(matches!(entries[0].1, Value::Int(3)));
    assert!(matches!(entries[1].1, Value::Float(_)));
}

fn nested(depth: usize) -> Value {
    (0..depth).fold(Value::Nil, |inner, _| {
        Value::Table(vec![(ValueKey::Int(1), inner)])
    })
}

#[test]
fn values_nested_more_than_32_deep_are_refused() {
    let mut decoder = Decoder::new();
    decoder.feed(&encode(&nested(32)).unwrap()).unwrap();
    assert_eq!(decoder.next_message::<Value>().unwrap(), Some(nested(32)));
    let mut decoder = Decoder::new();
    decoder.feed(&encode(&nested(33)).unwrap()).unwrap();
    assert!(decoder.next_message::<Value>().is_err());
}

#[test]
fn command_round_trip() {
    round_trip(ClientMessage::Command {
        call: 7,
        name: "agents.next_waiting".to_owned(),
        args: Value::Table(vec![(ValueKey::string("state"), text("agent"))]),
    });
}

#[test]
fn bridge_messages_round_trip() {
    round_trip(ServerMessage::Result {
        call: 7,
        result: Err("unknown command absent".to_owned()),
    });
    round_trip(ServerMessage::Result {
        call: 8,
        result: Ok(Value::Int(4)),
    });
    round_trip(ServerMessage::Event {
        name: "agent.waiting".to_owned(),
        data: Value::Table(vec![(ValueKey::string("pane"), Value::Int(1))]),
        queued: true,
        time: 1_700_000_000_000,
    });
    round_trip(ServerMessage::PaneState {
        pane: PaneId(2),
        key: "agent".to_owned(),
        value: Some(text("waiting")),
    });
    round_trip(ServerMessage::PaneState {
        pane: PaneId(2),
        key: "agent".to_owned(),
        value: None,
    });
    round_trip(ServerMessage::Requirements(vec![Requirement {
        plugin: "agent-status".to_owned(),
        requirement: ">= 0.1".to_owned(),
    }]));
    round_trip(ServerMessage::ServerError("server: boom".to_owned()));
}

#[test]
fn test_channel_messages_round_trip() {
    round_trip(ToProcess::Start {
        time: Some(1_735_732_800),
    });
    round_trip(ToProcess::Start { time: None });
    round_trip(ToProcess::Eval {
        id: 1,
        source: "return gband.side, select('#', ...)".to_owned(),
        args: vec![Value::Int(1), Value::Int(2)],
    });
    round_trip(ToProcess::Settle {
        round: 1,
        markers: Some(3),
    });
    round_trip(ToProcess::Settle {
        round: 2,
        markers: None,
    });
    round_trip(ToProcess::Reload { id: 4 });
    round_trip(ToProcess::SetTime { time: Some(0) });
    round_trip(FromProcess::Hello { role: Role::Client });
    round_trip(FromProcess::Hello { role: Role::Server });
    round_trip(FromProcess::Answer {
        id: 1,
        result: Ok(vec![text("client"), Value::Int(2), Value::Nil]),
    });
    round_trip(FromProcess::Answer {
        id: 2,
        result: Err("boom".to_owned()),
    });
    round_trip(FromProcess::Settled { round: 1, sent: 0 });
    round_trip(FromProcess::Reloaded { id: 4, error: None });
    round_trip(FromProcess::Reloaded {
        id: 5,
        error: Some("init.lua:1: bad".to_owned()),
    });
}
