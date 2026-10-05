use gband_core::event::LayoutEvent;
use gband_core::layout::{PaneId, SessionAction};
use gband_protocol::{ClientMessage, socket_path};
use gband_server::{CAPACITY, Published, SessionEvent};
use gband_test_support::*;
use tokio::sync::broadcast::{self, Receiver, error::RecvError};
use tokio::time::timeout;

async fn start(name: &str) -> (TestServer, Receiver<Published>) {
    let runtime_dir = runtime_dir(name);
    let (sender, receiver) = broadcast::channel(CAPACITY);
    let socket = socket_path(&runtime_dir);
    let server = gband_server::run_with_events(config(&runtime_dir, &["/bin/sh"]), sender);
    let server = TestServer::start_running(runtime_dir, socket, server).await;
    (server, receiver)
}

async fn next(events: &mut Receiver<Published>) -> SessionEvent {
    let published = timeout(TIMEOUT, events.recv())
        .await
        .expect("no session event arrived")
        .expect("the bus failed");
    assert_eq!(published.session.as_str(), "default");
    published.event
}

async fn until(
    events: &mut Receiver<Published>,
    wanted: impl Fn(&SessionEvent) -> bool,
) -> Vec<SessionEvent> {
    let mut seen = Vec::new();
    loop {
        let event = next(events).await;
        let done = wanted(&event);
        seen.push(event);
        if done {
            return seen;
        }
    }
}

fn opened(event: &SessionEvent) -> Option<PaneId> {
    match event {
        SessionEvent::Layout(LayoutEvent::PaneOpened { pane, .. }) => Some(*pane),
        _ => None,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn subscriber_sees_a_pane_opened_by_a_client() {
    let (server, mut events) = start("ev-open").await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    until(&mut events, |event| {
        matches!(event, SessionEvent::ClientAttached { .. })
    })
    .await;

    let second = client.open_after(first).await;
    assert_eq!(opened(&next(&mut events).await), Some(second));
}

#[tokio::test(flavor = "multi_thread")]
async fn pane_exit_comes_before_its_close() {
    let (server, mut events) = start("ev-exit").await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let second = client.open_after(first).await;
    until(&mut events, |event| opened(event) == Some(second)).await;

    client.type_line_to(second, "exit 3").await;
    let seen = until(&mut events, |event| {
        matches!(
            event,
            SessionEvent::Layout(LayoutEvent::PaneClosed { pane, .. }) if *pane == second
        )
    })
    .await;
    let exited = seen
        .iter()
        .position(|event| matches!(event, SessionEvent::PaneExited { pane, .. } if *pane == second))
        .expect("no exit before the close");
    assert_eq!(exited, seen.len() - 2, "{seen:?}");
    let SessionEvent::PaneExited { code, signal, .. } = &seen[exited] else {
        unreachable!();
    };
    assert_eq!((*code, *signal), (Some(3), None));
}

#[tokio::test(flavor = "multi_thread")]
async fn attach_then_detach_name_the_same_client() {
    let (server, mut events) = start("ev-attach").await;
    let mut client = server.attach(80, 24).await;
    let seen = until(&mut events, |event| {
        matches!(event, SessionEvent::ClientAttached { .. })
    })
    .await;
    let Some(SessionEvent::ClientAttached { client: attached }) = seen.last().cloned() else {
        unreachable!();
    };

    client.send(&ClientMessage::Detach).await;
    assert_eq!(
        next(&mut events).await,
        SessionEvent::ClientDetached { client: attached }
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn lagging_subscriber_is_told_and_blocks_nothing() {
    let (server, mut events) = start("ev-lag").await;
    let mut client = server.attach(80, 24).await;
    let pane = client.first();
    for _ in 0..2000 {
        client.act(SessionAction::CycleWidth(pane)).await;
    }
    client.act(SessionAction::ToggleFullWidth(pane)).await;
    client
        .wait_until(|client| client.layout.bands()[0].columns[0].full_width)
        .await;

    match events.recv().await {
        Err(RecvError::Lagged(missed)) => assert!(missed > 0),
        other => panic!("expected a lag, got {other:?}"),
    }
    assert!(matches!(
        next(&mut events).await,
        SessionEvent::Layout(LayoutEvent::ColumnWidthChanged { .. })
    ));
}
