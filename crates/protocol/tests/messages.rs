use std::fmt::Debug;

use gband_core::input::{Key, KeyCode, Modifiers};
use gband_core::layout::{Direction, Layout, PaneId, SessionAction};
use gband_protocol::{
    ClientMessage, Decoder, ExecutableId, Hello, HelloReply, PROTOCOL_VERSION, ServerMessage,
    encode,
};
use serde::Serialize;
use serde::de::DeserializeOwned;

fn round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(message: T) {
    let mut decoder = Decoder::new();
    decoder.feed(&encode(&message).unwrap()).unwrap();
    assert_eq!(decoder.next_message::<T>().unwrap(), Some(message));
    assert_eq!(decoder.next_message::<T>().unwrap(), None);
}

#[test]
fn protocol_version_is_two() {
    assert_eq!(PROTOCOL_VERSION, 2);
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
}

#[test]
fn session_actions_round_trip() {
    let layout = Layout::new();
    let workspace = layout.workspaces()[0].id;
    for action in [
        SessionAction::OpenPane {
            workspace,
            after: None,
        },
        SessionAction::OpenPane {
            workspace,
            after: Some(PaneId(4)),
        },
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
    let w1 = layout.workspaces()[0].id;
    layout.open(first, w1, None);
    layout.open(second, w1, Some(first));
    layout.apply(SessionAction::ConsumeOrExpel {
        pane: second,
        direction: Direction::Left,
    });
    layout.apply(SessionAction::CycleWidth(first));
    layout.apply(SessionAction::CycleWidth(first));
    layout.apply(SessionAction::ToggleFullWidth(first));
    let w2 = layout.workspaces()[1].id;
    layout.open(third, w2, None);
    assert_eq!(layout.workspaces().len(), 3);
    round_trip(ServerMessage::Layout {
        cols: 120,
        rows: 40,
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
