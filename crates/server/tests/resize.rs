use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use gband_core::geometry::Size;
use gband_core::layout::{Direction, PaneId, Proportion, SessionAction, Step};
use gband_protocol::{ClientMessage, ServerMessage};
use gband_test_support::*;

const BEYOND_SETTLE: Duration = Duration::from_millis(300);

struct Winches {
    dir: PathBuf,
}

impl Winches {
    fn new(name: &str) -> Self {
        Self {
            dir: runtime_dir(name),
        }
    }

    async fn server(&self) -> TestServer {
        let dir = self.dir.display();
        let script = format!(
            "trap 'echo W >> {dir}/winch-$GBAND_PANE' WINCH; echo ready; \
             while [ ! -e {dir}/exit-$GBAND_PANE ]; do read -t 1; done"
        );
        TestServer::start_in(self.dir.clone(), &["/bin/bash", "-c", &script]).await
    }

    fn count(&self, pane: PaneId) -> usize {
        fs::read_to_string(self.dir.join(format!("winch-{pane}")))
            .map_or(0, |contents| contents.lines().count())
    }

    fn exit(&self, pane: PaneId) {
        fs::write(self.dir.join(format!("exit-{pane}")), "").unwrap();
    }

    async fn settled(&self, client: &mut TestClient, pane: PaneId, size: Size) -> usize {
        client
            .wait_for_pane(pane, |screen| screen.size() == size)
            .await;
        assert!(client.pump(BEYOND_SETTLE).await);
        self.count(pane)
    }
}

async fn ready(client: &mut TestClient, pane: PaneId) {
    client.wait_for_pane_text(pane, "ready").await;
}

async fn open(client: &mut TestClient, after: PaneId) -> PaneId {
    let seen = client.focus.len();
    client
        .act(SessionAction::open(
            client.layout.bands()[0].id,
            Some(after),
            None,
        ))
        .await;
    client.wait_until(|client| client.focus.len() > seen).await;
    let opened = client.focus[seen];
    ready(client, opened).await;
    opened
}

async fn three_columns(winches: &Winches) -> (TestServer, TestClient, [PaneId; 3]) {
    let server = winches.server().await;
    let mut client = server.attach(80, 24).await;
    let a = client.first();
    ready(&mut client, a).await;
    let b = open(&mut client, a).await;
    let c = open(&mut client, b).await;
    (server, client, [a, b, c])
}

#[tokio::test(flavor = "multi_thread")]
async fn counter_sees_one_resize() {
    let winches = Winches::new("winch-counter");
    let server = winches.server().await;
    let mut client = server.attach(80, 24).await;
    let pane = client.first();
    ready(&mut client, pane).await;
    client.show_all().await;
    assert_eq!(
        winches.settled(&mut client, pane, Size::new(38, 22)).await,
        0
    );
    client.act(SessionAction::CycleWidth(pane)).await;
    assert_eq!(
        winches.settled(&mut client, pane, Size::new(51, 22)).await,
        1
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn burst_of_terminal_resizes_gives_one_sigwinch() {
    let winches = Winches::new("winch-burst");
    let server = winches.server().await;
    let mut client = server.attach(80, 24).await;
    let pane = client.first();
    ready(&mut client, pane).await;
    client.show_all().await;
    for (cols, rows) in [(90, 25), (95, 28), (100, 30)] {
        client.send(&ClientMessage::Resize { cols, rows }).await;
    }
    assert_eq!(
        winches.settled(&mut client, pane, Size::new(48, 28)).await,
        1
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn held_resize_key_gives_a_layout_per_action_and_one_sigwinch() {
    let winches = Winches::new("winch-held");
    let server = winches.server().await;
    let mut client = server.attach(80, 24).await;
    let pane = client.first();
    ready(&mut client, pane).await;
    client.show_all().await;
    assert!(client.pump(BEYOND_SETTLE).await);
    let mut width = Proportion::ONE_HALF;
    for _ in 0..4 {
        width = width.step(Step::Grow);
        client
            .act(SessionAction::StepWidth {
                pane,
                step: Step::Grow,
            })
            .await;
        loop {
            let message = client.receive().await.expect("connection closed");
            if let ServerMessage::Layout { layout, .. } = message {
                assert_eq!(layout.bands()[0].columns[0].width, width);
                break;
            }
        }
    }
    assert_eq!(width, Proportion::new(9, 10));
    assert_eq!(
        winches.settled(&mut client, pane, Size::new(70, 22)).await,
        1
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn offscreen_pane_keeps_its_size_until_shown() {
    let winches = Winches::new("winch-offscreen");
    let (_server, mut client, [a, b, c]) = three_columns(&winches).await;
    client.show(&[a, b]).await;
    client
        .send(&ClientMessage::Resize {
            cols: 100,
            rows: 30,
        })
        .await;
    assert_eq!(winches.settled(&mut client, a, Size::new(48, 28)).await, 1);
    assert_eq!(winches.settled(&mut client, b, Size::new(48, 28)).await, 1);
    assert_eq!(client.pane_screen(c).size(), Size::new(38, 22));
    assert_eq!(winches.count(c), 0);

    client.show(&[b, c]).await;
    assert_eq!(winches.settled(&mut client, c, Size::new(48, 28)).await, 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn panes_shown_by_different_clients_both_resize() {
    let winches = Winches::new("winch-two-clients");
    let (server, mut first, [a, b, c]) = three_columns(&winches).await;
    first.show(&[a]).await;
    let mut second = server.attach(80, 24).await;
    second.show(&[c]).await;
    first
        .send(&ClientMessage::Resize {
            cols: 100,
            rows: 30,
        })
        .await;
    assert_eq!(winches.settled(&mut first, a, Size::new(48, 28)).await, 1);
    assert_eq!(winches.settled(&mut first, c, Size::new(48, 28)).await, 1);
    assert_eq!(first.pane_screen(b).size(), Size::new(38, 22));
    assert_eq!(winches.count(b), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn pane_left_alone_while_detached_keeps_its_size_until_shown() {
    let winches = Winches::new("winch-detached");
    let server = winches.server().await;
    let mut client = server.attach(80, 24).await;
    let top = client.first();
    ready(&mut client, top).await;
    let bottom = open(&mut client, top).await;
    client
        .act(SessionAction::ConsumeOrExpel {
            pane: bottom,
            direction: Direction::Left,
        })
        .await;
    client.show_all().await;
    assert_eq!(
        winches.settled(&mut client, top, Size::new(38, 10)).await,
        1
    );
    client.send(&ClientMessage::Detach).await;
    assert!(client.peer.closes().await);

    winches.exit(bottom);
    let mut client = server.attach(80, 24).await;
    client.wait_until(|client| client.panes() == [top]).await;
    assert!(client.pump(BEYOND_SETTLE).await);
    assert_eq!(client.pane_screen(top).size(), Size::new(38, 10));
    assert_eq!(winches.count(top), 1);

    client.show_all().await;
    assert_eq!(
        winches.settled(&mut client, top, Size::new(38, 22)).await,
        2
    );
}
