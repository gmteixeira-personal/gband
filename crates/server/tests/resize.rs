use std::cell::Cell;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use gband_core::geometry::Size;
use gband_core::layout::{Direction, Proportion, SessionAction, Step, WindowHeight, WindowId};
use gband_protocol::{ClientMessage, ServerMessage};
use gband_test_support::*;

const BEYOND_SETTLE: Duration = Duration::from_millis(300);

struct Winches {
    dir: PathBuf,
    scratch: Cell<Option<Scratch>>,
}

impl Winches {
    fn new(name: &str) -> Self {
        let scratch = runtime_dir(name);
        Self {
            dir: scratch.to_path_buf(),
            scratch: Cell::new(Some(scratch)),
        }
    }

    async fn server(&self) -> TestServer {
        let dir = self.dir.display();
        let script = format!(
            "trap 'echo W >> {dir}/winch-$GBAND_WINDOW' WINCH; echo ready; \
             while [ ! -e {dir}/exit-$GBAND_WINDOW ]; do read -t 1; done"
        );
        let scratch = self.scratch.take().expect("one server per test");
        TestServer::start_in(scratch, &["/bin/bash", "-c", &script]).await
    }

    fn count(&self, window: WindowId) -> usize {
        fs::read_to_string(self.dir.join(format!("winch-{window}")))
            .map_or(0, |contents| contents.lines().count())
    }

    fn exit(&self, window: WindowId) {
        fs::write(self.dir.join(format!("exit-{window}")), "").unwrap();
    }

    async fn settled(&self, client: &mut TestClient, window: WindowId, size: Size) -> usize {
        client
            .wait_for_window(window, |screen| screen.size() == size)
            .await;
        assert!(client.pump(BEYOND_SETTLE).await);
        self.count(window)
    }
}

async fn ready(client: &mut TestClient, window: WindowId) {
    client.wait_for_window_text(window, "ready").await;
}

async fn open(client: &mut TestClient, after: WindowId) -> WindowId {
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

async fn three_columns(winches: &Winches) -> (TestServer, TestClient, [WindowId; 3]) {
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
    let window = client.first();
    ready(&mut client, window).await;
    client.show_all().await;
    assert_eq!(
        winches
            .settled(&mut client, window, Size::new(38, 22))
            .await,
        0
    );
    client.act(SessionAction::CycleWidth(window)).await;
    assert_eq!(
        winches
            .settled(&mut client, window, Size::new(51, 22))
            .await,
        1
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn burst_of_terminal_resizes_gives_one_sigwinch() {
    let winches = Winches::new("winch-burst");
    let server = winches.server().await;
    let mut client = server.attach(80, 24).await;
    let window = client.first();
    ready(&mut client, window).await;
    client.show_all().await;
    for (cols, rows) in [(90, 25), (95, 28), (100, 30)] {
        client.send(&ClientMessage::Resize { cols, rows }).await;
    }
    assert_eq!(
        winches
            .settled(&mut client, window, Size::new(48, 28))
            .await,
        1
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn held_resize_key_gives_a_layout_per_action_and_one_sigwinch() {
    let winches = Winches::new("winch-held");
    let server = winches.server().await;
    let mut client = server.attach(80, 24).await;
    let window = client.first();
    ready(&mut client, window).await;
    client.show_all().await;
    assert!(client.pump(BEYOND_SETTLE).await);
    let mut width = Proportion::ONE_HALF;
    for _ in 0..4 {
        width = width.step(Step::Grow, Proportion::TENTH);
        client
            .act(SessionAction::StepWidth {
                window,
                step: Step::Grow,
                by: Proportion::TENTH,
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
        winches
            .settled(&mut client, window, Size::new(70, 22))
            .await,
        1
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn offscreen_window_keeps_its_size_until_shown() {
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
    assert_eq!(client.window_screen(c).size(), Size::new(38, 22));
    assert_eq!(winches.count(c), 0);

    client.show(&[b, c]).await;
    assert_eq!(winches.settled(&mut client, c, Size::new(48, 28)).await, 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn windows_shown_by_different_clients_both_resize() {
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
    assert_eq!(first.window_screen(b).size(), Size::new(38, 22));
    assert_eq!(winches.count(b), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn window_left_alone_while_detached_keeps_its_size_until_shown() {
    let winches = Winches::new("winch-detached");
    let server = winches.server().await;
    let mut client = server.attach(80, 24).await;
    let top = client.first();
    ready(&mut client, top).await;
    let bottom = open(&mut client, top).await;
    client
        .act(SessionAction::ConsumeOrExpel {
            window: bottom,
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
    client.wait_until(|client| client.windows() == [top]).await;
    assert!(client.pump(BEYOND_SETTLE).await);
    assert_eq!(client.window_screen(top).size(), Size::new(38, 10));
    assert_eq!(winches.count(top), 1);

    client.show_all().await;
    assert_eq!(
        winches.settled(&mut client, top, Size::new(38, 22)).await,
        2
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn floating_window_takes_its_box_size() {
    let winches = Winches::new("winch-floating");
    let server = winches.server().await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    ready(&mut client, first).await;
    let window = open(&mut client, first).await;
    for action in [
        SessionAction::ToggleFloating {
            window,
            after: None,
        },
        SessionAction::SetWidth {
            window,
            width: Proportion::ONE_THIRD,
        },
        SessionAction::SetHeight {
            window,
            height: WindowHeight::Fixed(12),
        },
    ] {
        client.act(action).await;
    }
    client
        .wait_until(|client| {
            client
                .layout
                .floating(window)
                .is_some_and(|floating| floating.rows == 12)
        })
        .await;
    client.show_all().await;
    winches
        .settled(&mut client, window, Size::new(24, 10))
        .await;
    client
        .act(SessionAction::StepHeight {
            window,
            step: Step::Grow,
            by: Proportion::TENTH,
        })
        .await;
    winches
        .settled(&mut client, window, Size::new(24, 12))
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn step_named_by_a_client_reaches_every_client() {
    let server = TestServer::start("step-from-client", &["/bin/sh"]).await;
    let mut first = server.attach(80, 24).await;
    let mut second = server.attach(80, 24).await;
    let window = first.first();
    let width = |client: &TestClient| client.layout.bands()[0].columns[0].width;
    assert_eq!(width(&first), Proportion::ONE_HALF);
    first
        .act(SessionAction::StepWidth {
            window,
            step: Step::Grow,
            by: Proportion::new(1, 4),
        })
        .await;
    first
        .wait_until(|client| width(client) == Proportion::new(3, 4))
        .await;
    second
        .wait_until(|client| width(client) == Proportion::new(3, 4))
        .await;
}
