use std::time::Duration;

use gband_core::geometry::Size;
use gband_core::input::{Key, KeyCode};
use gband_core::layout::{
    BandId, Direction, Proportion, SessionAction, Step, WindowContent, WindowId,
};
use gband_protocol::{ClientMessage, ServerMessage};
use gband_test_support::*;

const QUIET: Duration = Duration::from_millis(300);

fn open_plugin(band: BandId, after: Option<WindowId>, request: u32, focus: bool) -> SessionAction {
    SessionAction::OpenWindow {
        band,
        after,
        width: None,
        floating: false,
        focus,
        content: WindowContent::Plugin { request },
    }
}

async fn opened(client: &mut TestClient, request: u32) -> Option<WindowId> {
    client
        .wait_until(|client| client.opened.iter().any(|&(seen, _)| seen == request))
        .await;
    client
        .opened
        .iter()
        .find(|&&(seen, _)| seen == request)
        .and_then(|&(_, window)| window)
}

async fn plugin_after(client: &mut TestClient, after: WindowId, request: u32) -> WindowId {
    let band = client.layout.bands()[0].id;
    client
        .act(open_plugin(band, Some(after), request, false))
        .await;
    opened(client, request)
        .await
        .expect("no plugin window opened")
}

async fn content(client: &mut TestClient, window: WindowId, output: &[u8]) {
    client
        .send(&ClientMessage::Content {
            window,
            output: output.to_vec(),
        })
        .await;
}

fn text(client: &TestClient, window: WindowId) -> String {
    client
        .window_screen(window)
        .contents()
        .trim_end()
        .to_owned()
}

#[tokio::test(flavor = "multi_thread")]
async fn plugin_window_opens_blank_then_reports_and_focuses() {
    let server = TestServer::start("plugin-open", &["/bin/sh"]).await;
    let mut owner = server.attach(80, 24).await;
    let mut other = server.attach(80, 24).await;
    let first = owner.first();
    let band = owner.layout.bands()[0].id;
    owner.act(open_plugin(band, Some(first), 7, true)).await;
    let mut order = Vec::new();
    while owner.focus.is_empty() {
        let message = owner.receive().await.expect("connection closed");
        match message {
            ServerMessage::Layout { .. } => order.push("layout"),
            ServerMessage::Snapshot { .. } => order.push("snapshot"),
            ServerMessage::Opened { .. } => order.push("opened"),
            ServerMessage::Focus(_) => order.push("focus"),
            _ => {}
        }
    }
    let window = owner.opened[0].1.expect("no window opened");
    assert_eq!(owner.opened, [(7, Some(window))]);
    assert_eq!(owner.focus, [window]);
    assert_eq!(order, ["layout", "snapshot", "opened", "focus"]);
    assert_eq!(owner.windows(), [first, window]);
    assert_eq!(text(&owner, window), "");
    other
        .wait_until(|client| client.grids.contains_key(&window))
        .await;
    assert!(other.pump(QUIET).await);
    assert!(other.opened.is_empty());
    assert!(other.focus.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn ignored_request_is_reported_with_no_window() {
    let server = TestServer::start("plugin-ignored", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let before = client.layout.clone();
    client.act(open_plugin(BandId(99), None, 8, true)).await;
    assert_eq!(opened(&mut client, 8).await, None);
    assert_eq!(client.layout, before);
}

#[tokio::test(flavor = "multi_thread")]
async fn open_without_focus_tells_no_client_to_focus() {
    let server = TestServer::start("plugin-nofocus", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let window = plugin_after(&mut client, first, 1).await;
    assert!(client.pump(QUIET).await);
    assert!(client.focus.is_empty());
    assert_eq!(client.windows(), [first, window]);
}

#[tokio::test(flavor = "multi_thread")]
async fn open_with_a_width() {
    let server = TestServer::start("plugin-width", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let band = client.layout.bands()[0].id;
    client
        .act(SessionAction::OpenWindow {
            band,
            after: Some(first),
            width: Some(Proportion::new(1, 4)),
            floating: false,
            focus: false,
            content: WindowContent::Plugin { request: 3 },
        })
        .await;
    opened(&mut client, 3).await.unwrap();
    assert_eq!(
        client.layout.bands()[0].columns[1].width,
        Proportion::new(1, 4)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn content_reaches_every_client() {
    let server = TestServer::start("plugin-content", &["/bin/sh"]).await;
    let mut owner = server.attach(80, 24).await;
    let mut other = server.attach(80, 24).await;
    let first = owner.first();
    let window = plugin_after(&mut owner, first, 1).await;
    content(&mut owner, window, b"\x1b[1;1Hhello").await;
    for client in [&mut owner, &mut other] {
        client
            .wait_for_window(window, |screen| screen.contents().trim_end() == "hello")
            .await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn content_replaces_the_screen() {
    let server = TestServer::start("plugin-replace", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let window = plugin_after(&mut client, first, 1).await;
    content(&mut client, window, b"one").await;
    client.wait_for_window_text(window, "one").await;
    content(&mut client, window, b"two").await;
    client
        .wait_for_window(window, |screen| screen.contents().trim_end() == "two")
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn content_from_another_client_is_ignored() {
    let server = TestServer::start("plugin-foreign", &["/bin/sh"]).await;
    let mut owner = server.attach(80, 24).await;
    let mut other = server.attach(80, 24).await;
    let first = owner.first();
    let window = plugin_after(&mut owner, first, 1).await;
    other
        .wait_until(|client| client.grids.contains_key(&window))
        .await;
    content(&mut other, window, b"intruder").await;
    content(&mut other, first, b"intruder").await;
    assert!(owner.pump(QUIET).await);
    assert_eq!(text(&owner, window), "");
    assert!(!owner.window_screen(first).contents().contains("intruder"));
}

#[tokio::test(flavor = "multi_thread")]
async fn key_to_a_plugin_window_is_dropped() {
    let server = TestServer::start("plugin-key", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let window = plugin_after(&mut client, first, 1).await;
    client.key_to(window, Key::plain(KeyCode::Char('x'))).await;
    client
        .send(&ClientMessage::Paste {
            window,
            text: "pasted".into(),
        })
        .await;
    client.type_line_to(first, "echo still-serving").await;
    client.wait_for_window_text(first, "still-serving\n").await;
    assert_eq!(text(&client, window), "");
}

#[tokio::test(flavor = "multi_thread")]
async fn closing_a_plugin_window_is_immediate() {
    let server = TestServer::start("plugin-close", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let window = plugin_after(&mut client, first, 1).await;
    client.act(SessionAction::CloseWindow(window)).await;
    tokio::time::timeout(
        Duration::from_millis(500),
        client.wait_until(|client| client.windows() == [first]),
    )
    .await
    .expect("the plugin window did not leave at once");
}

#[tokio::test(flavor = "multi_thread")]
async fn plugin_windows_leave_with_their_owner() {
    let server = TestServer::start("plugin-owner", &["/bin/sh"]).await;
    let mut owner = server.attach(80, 24).await;
    let mut other = server.attach(80, 24).await;
    let first = owner.first();
    let window = plugin_after(&mut owner, first, 1).await;
    other
        .wait_until(|client| client.windows().contains(&window))
        .await;
    owner.send(&ClientMessage::Detach).await;
    drop(owner);
    other.wait_until(|client| client.windows() == [first]).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn session_ends_when_only_plugin_windows_remain() {
    let server = TestServer::start("plugin-last", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let window = plugin_after(&mut client, first, 1).await;
    client.type_line_to(first, "exit").await;
    client.drain_to_end().await;
    assert!(client.exited);
    assert!(!client.windows().contains(&window));
    server.finished().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn plugin_windows_count_in_the_session_list() {
    let server = TestServer::start("plugin-count", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    plugin_after(&mut client, first, 1).await;
    assert_eq!(server.listed().await, [("default".to_owned(), 2, 1)]);
}

#[tokio::test(flavor = "multi_thread")]
async fn set_a_column_width() {
    let server = TestServer::start("set-width", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    client.act(SessionAction::ToggleFullWidth(first)).await;
    client
        .act(SessionAction::SetWidth {
            window: first,
            width: Proportion::new(2, 5),
        })
        .await;
    client
        .wait_until(|client| {
            let column = &client.layout.bands()[0].columns[0];
            column.width == Proportion::new(2, 5) && !column.full_width
        })
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn plugin_window_is_resized_like_a_pty() {
    let server = TestServer::start("plugin-resize", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let window = plugin_after(&mut client, first, 1).await;
    content(&mut client, window, b"kept").await;
    client.show_all().await;
    client
        .wait_for_window(window, |screen| screen.size() == Size::new(38, 22))
        .await;
    client
        .act(SessionAction::StepWidth {
            window,
            step: Step::Grow,
            by: Proportion::TENTH,
        })
        .await;
    client
        .wait_for_window(window, |screen| {
            screen.size() == Size::new(46, 22) && screen.contents().contains("kept")
        })
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn stacked_shell_takes_the_column_when_the_owner_detaches() {
    let server = TestServer::start("plugin-stacked", &["/bin/sh"]).await;
    let mut owner = server.attach(80, 24).await;
    let mut other = server.attach(80, 24).await;
    let first = owner.first();
    other.type_line_to(first, "echo before-detach").await;
    other.wait_for_window_text(first, "before-detach\n").await;
    let window = plugin_after(&mut owner, first, 1).await;
    owner
        .act(SessionAction::ConsumeOrExpel {
            window,
            direction: Direction::Left,
        })
        .await;
    other
        .wait_until(|client| client.layout.bands()[0].columns[0].windows == [first, window])
        .await;
    other.show_all().await;
    other
        .wait_for_window(first, |screen| screen.size() == Size::new(38, 10))
        .await;
    owner.send(&ClientMessage::Detach).await;
    drop(owner);
    other.wait_until(|client| client.windows() == [first]).await;
    other.show_all().await;
    other
        .wait_for_window(first, |screen| {
            screen.size() == Size::new(38, 22) && screen.contents().contains("before-detach")
        })
        .await;
    other.type_line_to(first, "echo still-here").await;
    other.wait_for_window_text(first, "still-here\n").await;
}
