use std::path::Path;
use std::time::Duration;

use gband_test_support::*;

use gband_core::geometry::Size;
use gband_core::input::{Key, KeyCode};
use gband_protocol::{ClientMessage, ServerMessage, encode};

const QUIET: Duration = Duration::from_millis(500);

fn entry(name: &str, panes: u32, clients: u32) -> (String, u32, u32) {
    (name.to_owned(), panes, clients)
}

#[tokio::test(flavor = "multi_thread")]
async fn server_starts_with_the_configured_session() {
    let runtime_dir = runtime_dir("named");
    let config = gband_server::ServerConfig {
        session: session("work"),
        ..config(&runtime_dir, &["/bin/sh"])
    };
    let server = TestServer::start_with(runtime_dir, config).await;
    assert_eq!(server.listed().await, [entry("work", 1, 0)]);
    let mut client = server.attach_to("work", Path::new("/"), 80, 24).await;
    client.type_line("echo session=$GBAND_SESSION").await;
    client.wait_for_text("session=work\n").await;
}

#[tokio::test(flavor = "multi_thread")]
async fn default_session_is_named_default() {
    let server = TestServer::start("default-name", &["/bin/sh"]).await;
    assert_eq!(server.listed().await, [entry("default", 1, 0)]);
    let mut client = server.attach(80, 24).await;
    client.type_line("echo session=$GBAND_SESSION").await;
    client.wait_for_text("session=default\n").await;
}

#[tokio::test(flavor = "multi_thread")]
async fn attach_creates_a_missing_session_in_the_client_directory() {
    let server = TestServer::start("create", &["/bin/sh"]).await;
    let mut work = server.attach_to("work", Path::new("/tmp"), 80, 24).await;
    work.type_line("echo cwd=$(pwd) session=$GBAND_SESSION")
        .await;
    work.wait_for_text("cwd=/tmp session=work\n").await;
    assert_eq!(
        server.listed().await,
        [entry("default", 1, 0), entry("work", 1, 1)]
    );
    let mut default = server.attach(80, 24).await;
    default.type_line("echo cwd=$(pwd)").await;
    let expected = format!("cwd={}\n", server.runtime_dir.display());
    default.wait_for_text(&expected).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn key_before_a_request_closes_the_connection() {
    let server = TestServer::start("key-first", &["/bin/sh"]).await;
    let (mut peer, _) = TestClient::accepted(&server.socket(), 80, 24).await;
    peer.send(&ClientMessage::Key {
        pane: gband_core::layout::PaneId(1),
        key: Key::plain(KeyCode::Enter),
    })
    .await;
    assert!(peer.closes().await);
    assert_eq!(server.listed().await, [entry("default", 1, 0)]);
}

#[tokio::test(flavor = "multi_thread")]
async fn invalid_session_name_closes_the_connection() {
    let server = TestServer::start("bad-name", &["/bin/sh"]).await;
    let (mut peer, _) = TestClient::accepted(&server.socket(), 80, 24).await;
    let valid = encode(&ClientMessage::KillSession {
        session: session("abc"),
    })
    .unwrap();
    let invalid: Vec<u8> = valid
        .iter()
        .map(|&byte| if byte == b'b' { b'/' } else { byte })
        .collect();
    peer.send_bytes(&invalid).await;
    assert!(peer.closes().await);
    assert_eq!(server.listed().await, [entry("default", 1, 0)]);
}

#[tokio::test(flavor = "multi_thread")]
async fn simultaneous_attaches_to_a_missing_session_share_it() {
    let server = TestServer::start("race", &["/bin/sh"]).await;
    let (mut first, mut second) = tokio::join!(
        server.attach_to("work", Path::new("/tmp"), 80, 24),
        server.attach_to("work", Path::new("/tmp"), 80, 24),
    );
    assert_eq!(
        server.listed().await,
        [entry("default", 1, 0), entry("work", 1, 2)]
    );
    first.type_line("echo shared-$((6 * 7))").await;
    second.wait_for_text("shared-42\n").await;
    first.wait_for_text("shared-42\n").await;
}

#[tokio::test(flavor = "multi_thread")]
async fn sessions_are_listed_in_name_order_with_their_counts() {
    let server = TestServer::start("list", &["/bin/sh"]).await;
    let mut work = server.attach_to("work", Path::new("/tmp"), 80, 24).await;
    let first = work.first();
    work.wait_for_prompt(first).await;
    work.open_after(first).await;
    assert_eq!(
        server.listed().await,
        [entry("default", 1, 0), entry("work", 2, 1)]
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn killing_a_session_ends_only_that_session() {
    let server = TestServer::start("kill-one", &["/bin/sh"]).await;
    let mut default = server.attach(80, 24).await;
    let mut work = server.attach_to("work", Path::new("/tmp"), 80, 24).await;
    assert_eq!(server.kill("work").await, ServerMessage::Killed);
    work.drain_to_end().await;
    assert!(work.exited);
    assert_eq!(server.listed().await, [entry("default", 1, 1)]);
    assert!(default.pump(QUIET).await);
    assert!(!default.exited);
    default.type_line("echo still-here").await;
    default.wait_for_text("still-here\n").await;
    assert!(!server.handle.is_finished());
}

#[tokio::test(flavor = "multi_thread")]
async fn killing_the_last_session_ends_the_server() {
    let server = TestServer::start("kill-last", &["/bin/sh"]).await;
    assert_eq!(server.kill("default").await, ServerMessage::Killed);
    let socket = server.socket();
    server.finished().await.unwrap();
    assert!(!socket.exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn killing_an_unknown_session_answers_no_such_session() {
    let server = TestServer::start("kill-nope", &["/bin/sh"]).await;
    assert_eq!(server.kill("nope").await, ServerMessage::NoSuchSession);
    assert_eq!(server.listed().await, [entry("default", 1, 0)]);
}

#[tokio::test(flavor = "multi_thread")]
async fn opening_a_pane_sends_no_layout_to_another_session() {
    let server = TestServer::start("layout-apart", &["/bin/sh"]).await;
    let mut play = server.attach_to("play", Path::new("/tmp"), 80, 24).await;
    let mut work = server.attach_to("work", Path::new("/tmp"), 80, 24).await;
    let before = play.layout.clone();
    let first = work.first();
    work.wait_for_prompt(first).await;
    let opened = work.open_after(first).await;
    assert!(play.pump(QUIET).await);
    assert_eq!(play.layout, before);
    assert_eq!(play.panes().len(), 1);
    assert!(work.layout.contains(opened));
}

#[tokio::test(flavor = "multi_thread")]
async fn keys_stay_in_their_session() {
    let server = TestServer::start("keys-apart", &["/bin/sh"]).await;
    let mut play = server.attach_to("play", Path::new("/tmp"), 80, 24).await;
    let mut work = server.attach_to("work", Path::new("/tmp"), 80, 24).await;
    assert_eq!(play.first(), work.first());
    work.type_line("echo from-$((6 * 7))-work").await;
    work.wait_for_text("from-42-work\n").await;
    assert!(play.pump(QUIET).await);
    assert!(!play.screen().contents().contains("from-"));
}

#[tokio::test(flavor = "multi_thread")]
async fn attaching_to_another_session_keeps_the_screen_area() {
    let server = TestServer::start("area-apart", &["/bin/sh"]).await;
    let mut default = server.attach(120, 40).await;
    let mut work = server.attach_to("work", Path::new("/tmp"), 100, 30).await;
    assert_eq!(work.area, Size::new(100, 30));
    assert!(default.pump(QUIET).await);
    assert_eq!(default.area, Size::new(120, 40));
    let observer = server.attach(0, 0).await;
    assert_eq!(observer.area, Size::new(120, 40));
    work.send(&ClientMessage::Resize { cols: 90, rows: 25 })
        .await;
    work.wait_until(|client| client.area == Size::new(90, 25))
        .await;
    assert!(default.pump(QUIET).await);
    assert_eq!(default.area, Size::new(120, 40));
}

#[tokio::test(flavor = "multi_thread")]
async fn session_ending_tells_only_its_clients() {
    let server = TestServer::start("end-one", &["/bin/sh"]).await;
    let mut default = server.attach(80, 24).await;
    let mut work = server.attach_to("work", Path::new("/tmp"), 80, 24).await;
    work.type_line("exit").await;
    work.drain_to_end().await;
    assert!(work.exited);
    assert!(default.pump(QUIET).await);
    assert!(!default.exited);
    assert_eq!(server.listed().await, [entry("default", 1, 1)]);
    assert!(!server.handle.is_finished());
}

#[tokio::test(flavor = "multi_thread")]
async fn attach_after_a_session_ended_creates_it_again() {
    let server = TestServer::start("recreate", &["/bin/sh"]).await;
    let mut work = server.attach_to("work", Path::new("/tmp"), 80, 24).await;
    work.type_line("exit").await;
    work.drain_to_end().await;
    let mut again = server.attach_to("work", Path::new("/"), 80, 24).await;
    again.type_line("echo cwd=$(pwd)").await;
    again.wait_for_text("cwd=/\n").await;
    assert_eq!(
        server.listed().await,
        [entry("default", 1, 0), entry("work", 1, 1)]
    );
}
