use gband_client::{Display, Step, dispatch};
use gband_core::action::{Action, SessionCommand};
use gband_core::geometry::Size;
use gband_core::layout::{Direction, SessionAction};
use gband_core::view::ViewAction;
use gband_protocol::{ClientMessage, ServerMessage};
use gband_test_support::{TIMEOUT, TestClient, TestServer};
use tokio::time::timeout;

async fn run(client: &mut TestClient, display: &mut Display, action: Action) {
    let Step::Send(message) = dispatch(display, action) else {
        panic!("{action:?} sent nothing");
    };
    client.send(&message).await;
}

async fn receive_until(
    client: &mut TestClient,
    display: &mut Display,
    done: impl Fn(&TestClient, &Display) -> bool,
) {
    timeout(TIMEOUT, async {
        while !done(client, display) {
            let message = client.receive().await.expect("the server closed");
            display.apply(message);
        }
    })
    .await
    .expect("timed out waiting for the server");
}

#[tokio::test(flavor = "multi_thread")]
async fn close_pane_action_closes_the_focused_second_pane() {
    let server = TestServer::start("client-actions", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let mut display = Display::new(Size::new(80, 24));
    display.apply(ServerMessage::Layout {
        cols: client.area.cols,
        rows: client.area.rows,
        layout: client.layout.clone(),
    });
    assert_eq!(display.focused(), Some(first));

    run(
        &mut client,
        &mut display,
        Action::Session(SessionCommand::OpenPane),
    )
    .await;
    receive_until(&mut client, &mut display, |client, display| {
        client.panes().len() == 2 && display.focused() != Some(first)
    })
    .await;
    let second = display.focused().unwrap();
    assert_eq!(client.panes(), [first, second]);

    run(
        &mut client,
        &mut display,
        Action::Session(SessionCommand::ClosePane),
    )
    .await;
    receive_until(&mut client, &mut display, |client, _| {
        client.panes() == [first]
    })
    .await;
    assert_eq!(display.focused(), Some(first));
}

async fn report(
    client: &mut TestClient,
    display: &mut Display,
    action: Option<ViewAction>,
) -> Option<ClientMessage> {
    if let Some(action) = action {
        assert!(matches!(
            dispatch(display, Action::View(action)),
            Step::Nothing
        ));
    }
    let message = display.report_shown();
    if let Some(message) = &message {
        client.send(message).await;
    }
    message
}

#[tokio::test(flavor = "multi_thread")]
async fn shown_panes_follow_the_view_and_are_not_repeated() {
    let server = TestServer::start("client-shown", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let a = client.first();
    let b = client.open_after(a).await;
    let c = client.open_after(b).await;
    let d = client.open_after(c).await;
    client
        .act(SessionAction::ConsumeOrExpel {
            pane: d,
            direction: Direction::Left,
        })
        .await;
    client
        .wait_until(|client| client.layout.workspaces()[0].columns.len() == 3)
        .await;
    let mut display = Display::new(Size::new(80, 24));
    assert_eq!(display.report_shown(), None);
    display.apply(ServerMessage::Layout {
        cols: client.area.cols,
        rows: client.area.rows,
        layout: client.layout.clone(),
    });

    assert_eq!(
        report(&mut client, &mut display, None).await,
        Some(ClientMessage::Shown(vec![a, b]))
    );
    assert_eq!(report(&mut client, &mut display, None).await, None);
    let right = Some(ViewAction::FocusRight);
    assert_eq!(report(&mut client, &mut display, right).await, None);
    assert_eq!(
        report(&mut client, &mut display, right).await,
        Some(ClientMessage::Shown(vec![b, c, d]))
    );
    assert_eq!(display.focused(), Some(c));
    let down = Some(ViewAction::FocusDown);
    assert_eq!(report(&mut client, &mut display, down).await, None);
    assert_eq!(display.focused(), Some(d));
}
