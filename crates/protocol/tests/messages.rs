use std::fmt::Debug;
use std::path::PathBuf;

use gband_core::geometry::Size;
use gband_core::input::{
    Key, KeyCode, Modifiers, MouseButton, MouseEvent, MouseKind, WheelDirection,
};
use gband_core::layout::{
    Direction, FloatingWindow, Layout, LayoutOptions, Program, Proportion, SessionAction, Step,
    TargetPlace, Vertical, Weight, WindowContent, WindowHeight, WindowId,
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
fn protocol_version_is_eleven() {
    assert_eq!(PROTOCOL_VERSION, 11);
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
            windows: 1,
            clients: 0,
        },
        SessionSummary {
            name: session("work"),
            windows: 2,
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
        window: WindowId(1),
        key: Key::plain(KeyCode::Char('é')),
    });
    round_trip(ClientMessage::Key {
        window: WindowId(u32::MAX),
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
        window: WindowId(2),
        text: "line one\nline two".into(),
    });
    round_trip(ClientMessage::Resize {
        cols: 300,
        rows: 90,
    });
    round_trip(ClientMessage::Detach);
    round_trip(ClientMessage::Shown(vec![WindowId(1), WindowId(4)]));
    round_trip(ClientMessage::Shown(Vec::new()));
    round_trip(ClientMessage::Content {
        window: WindowId(4),
        output: b"\x1b[1;1Hhello".to_vec(),
    });
    round_trip(ClientMessage::Mouse {
        window: WindowId(2),
        event: MouseEvent::new(MouseKind::Press(MouseButton::Left), 4, 2, Modifiers::CTRL),
    });
    round_trip(ClientMessage::Mouse {
        window: WindowId(2),
        event: MouseEvent::new(
            MouseKind::Wheel(WheelDirection::Down),
            0,
            0,
            Modifiers::NONE,
        ),
    });
    round_trip(ClientMessage::Mouse {
        window: WindowId(2),
        event: MouseEvent::new(MouseKind::Motion(None), 300, 90, Modifiers::SHIFT),
    });
}

#[test]
fn rename_round_trips() {
    round_trip(ClientMessage::Rename {
        window: WindowId(2),
        name: Some("logs".to_owned()),
    });
    round_trip(ClientMessage::Rename {
        window: WindowId(3),
        name: None,
    });
}

#[test]
fn window_name_round_trips() {
    round_trip(ServerMessage::WindowName {
        window: WindowId(4),
        automatic: "vim".to_owned(),
        manual: Some("notes".to_owned()),
    });
    round_trip(ServerMessage::WindowName {
        window: WindowId(4),
        automatic: "bash".to_owned(),
        manual: None,
    });
}

#[test]
fn session_actions_round_trip() {
    let layout = Layout::new();
    let band = layout.bands()[0].id;
    for action in [
        SessionAction::open(band, None, None),
        SessionAction::open(band, Some(WindowId(4)), None),
        SessionAction::open(
            band,
            Some(WindowId(2)),
            Some(Program::Argv(vec![
                "htop".to_owned(),
                "-d".to_owned(),
                "10".to_owned(),
            ])),
        ),
        SessionAction::open(
            band,
            None,
            Some(Program::CommandLine("echo $GBAND_WINDOW".to_owned())),
        ),
        SessionAction::CloseWindow(WindowId(5)),
        SessionAction::ConsumeOrExpel {
            window: WindowId(3),
            direction: Direction::Left,
        },
        SessionAction::ConsumeOrExpel {
            window: WindowId(3),
            direction: Direction::Right,
        },
        SessionAction::CycleWidth(WindowId(6)),
        SessionAction::ToggleFullWidth(WindowId(7)),
        SessionAction::StepWidth {
            window: WindowId(8),
            step: Step::Grow,
            by: Proportion::TENTH,
        },
        SessionAction::StepWidth {
            window: WindowId(8),
            step: Step::Shrink,
            by: Proportion::new(1, 4),
        },
        SessionAction::StepHeight {
            window: WindowId(2),
            step: Step::Grow,
            by: Proportion::TENTH,
        },
        SessionAction::StepHeight {
            window: WindowId(2),
            step: Step::Shrink,
            by: Proportion::TENTH,
        },
        SessionAction::ResetHeight(WindowId(9)),
        SessionAction::OpenWindow {
            band,
            after: Some(WindowId(2)),
            width: Some(Proportion::new(1, 4)),
            floating: false,
            focus: false,
            content: WindowContent::Plugin { request: 7 },
        },
        SessionAction::OpenWindow {
            band,
            after: None,
            width: Some(Proportion::ONE_THIRD),
            floating: true,
            focus: true,
            content: WindowContent::Program(None),
        },
        SessionAction::ToggleFloating {
            window: WindowId(3),
            after: Some(WindowId(1)),
            floating: None,
        },
        SessionAction::ToggleFloating {
            window: WindowId(3),
            after: None,
            floating: None,
        },
        SessionAction::ToggleFloating {
            window: WindowId(3),
            after: None,
            floating: Some(true),
        },
        SessionAction::ToggleFloating {
            window: WindowId(4),
            after: Some(WindowId(1)),
            floating: Some(false),
        },
        SessionAction::MoveColumn {
            window: WindowId(2),
            direction: Direction::Right,
        },
        SessionAction::MoveWindow {
            window: WindowId(2),
            direction: Vertical::Up,
        },
        SessionAction::SetPosition {
            window: WindowId(4),
            col: 12,
            row: 3,
        },
        SessionAction::SetWidth {
            window: WindowId(1),
            width: Proportion::new(2, 5),
        },
        SessionAction::SetHeight {
            window: WindowId(2),
            height: WindowHeight::Auto(Weight::new(3, 2)),
        },
        SessionAction::SetHeight {
            window: WindowId(2),
            height: WindowHeight::Fixed(8),
        },
        SessionAction::MoveToPlace {
            window: WindowId(3),
            reference: WindowId(5),
            place: TargetPlace::ColumnRight,
        },
        SessionAction::MoveToPlace {
            window: WindowId(3),
            reference: WindowId(5),
            place: TargetPlace::Above,
        },
    ] {
        round_trip(ClientMessage::Action(action));
    }
}

#[test]
fn layout_round_trips() {
    let mut layout = Layout::new();
    let first = layout.allocate_window();
    let second = layout.allocate_window();
    let third = layout.allocate_window();
    let w1 = layout.bands()[0].id;
    layout.open(first, w1, None, None, &LayoutOptions::default());
    layout.open(second, w1, Some(first), None, &LayoutOptions::default());
    layout.apply(
        SessionAction::ConsumeOrExpel {
            window: second,
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
fn floating_windows_in_the_layout_round_trip() {
    let mut layout = Layout::new();
    let options = LayoutOptions::default();
    let windows: Vec<WindowId> = (0..3).map(|_| layout.allocate_window()).collect();
    let band = layout.bands()[0].id;
    layout.open(windows[0], band, None, None, &options);
    layout.open(windows[1], band, Some(windows[0]), None, &options);
    layout.open_floating(windows[2], band, None, AREA, &options);
    for action in [
        SessionAction::ToggleFloating {
            window: windows[1],
            after: None,
            floating: None,
        },
        SessionAction::SetWidth {
            window: windows[1],
            width: Proportion::ONE_THIRD,
        },
        SessionAction::SetHeight {
            window: windows[1],
            height: WindowHeight::Fixed(10),
        },
        SessionAction::SetPosition {
            window: windows[1],
            col: 5,
            row: 3,
        },
        SessionAction::ToggleFloating {
            window: windows[2],
            after: Some(windows[0]),
            floating: None,
        },
        SessionAction::ToggleFloating {
            window: windows[2],
            after: None,
            floating: None,
        },
    ] {
        layout.apply(action, AREA, &options);
    }
    let floating: Vec<_> = layout.bands()[0]
        .floating
        .iter()
        .map(|floating| floating.window)
        .collect();
    assert_eq!(floating, [windows[1], windows[2]]);
    assert_eq!(
        *layout.floating(windows[1]).unwrap(),
        FloatingWindow {
            window: windows[1],
            col: 5,
            row: 3,
            width: Proportion::ONE_THIRD,
            full_width: false,
            rows: 10,
        }
    );
    round_trip(ServerMessage::Layout {
        cols: 80,
        rows: 24,
        layout,
    });
}

#[test]
fn heights_in_the_layout_round_trip() {
    let mut layout = Layout::new();
    let band = layout.bands()[0].id;
    let windows: Vec<WindowId> = (0..3).map(|_| layout.allocate_window()).collect();
    layout.open(windows[0], band, None, None, &LayoutOptions::default());
    for pair in windows.windows(2) {
        layout.open(
            pair[1],
            band,
            Some(pair[0]),
            None,
            &LayoutOptions::default(),
        );
        layout.apply(
            SessionAction::ConsumeOrExpel {
                window: pair[1],
                direction: Direction::Left,
            },
            AREA,
            &LayoutOptions::default(),
        );
    }
    let grow = |window| SessionAction::StepHeight {
        window,
        step: Step::Grow,
        by: Proportion::TENTH,
    };
    layout.apply(grow(windows[1]), AREA, &LayoutOptions::default());
    layout.apply(grow(windows[0]), AREA, &LayoutOptions::default());
    layout.remove(windows[2]);
    layout.apply(
        grow(windows[0]),
        Size::new(80, 50),
        &LayoutOptions::default(),
    );
    assert_eq!(
        layout.bands()[0].columns[0].heights,
        [
            WindowHeight::Fixed(14),
            WindowHeight::Auto(Weight::new(10, 7))
        ]
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
        window: WindowId(1),
        cols: 80,
        rows: 24,
        contents: b"\x1b[H\x1b[2Jprompt$ ".to_vec(),
    });
    round_trip(ServerMessage::Update {
        window: WindowId(1),
        contents: Vec::new(),
    });
    round_trip(ServerMessage::Update {
        window: WindowId(9),
        contents: vec![0xff; 70_000],
    });
    round_trip(ServerMessage::Focus(WindowId(3)));
    round_trip(ServerMessage::Exited);
    round_trip(ServerMessage::Opened {
        request: 7,
        window: Some(WindowId(9)),
    });
    round_trip(ServerMessage::Opened {
        request: 8,
        window: None,
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
        data: Value::Table(vec![(ValueKey::string("window"), Value::Int(1))]),
        queued: true,
        time: 1_700_000_000_000,
    });
    round_trip(ServerMessage::WindowState {
        window: WindowId(2),
        key: "agent".to_owned(),
        value: Some(text("waiting")),
    });
    round_trip(ServerMessage::WindowState {
        window: WindowId(2),
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
        input: Some(3),
    });
    round_trip(ToProcess::Settle {
        round: 2,
        input: None,
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
