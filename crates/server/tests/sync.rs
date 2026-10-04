mod common;

use std::time::Duration;

use common::*;
use gband_core::input::{Key, KeyCode};
use gband_protocol::{ClientMessage, Hello, HelloReply, PROTOCOL_VERSION};

#[tokio::test(flavor = "multi_thread")]
async fn snapshot_and_updates_match_the_server() {
    let server = TestServer::start("match", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    client
        .type_line(r"printf '\033[1;31mred\033[0m \033[42mgreen\033[0m\033[?1h\033[?2004h done\n'")
        .await;
    client.wait_for_text("green done").await;
    assert_converges(&server, &mut client).await;
    assert!(client.parser.screen().application_cursor());
    assert!(client.parser.screen().bracketed_paste());
}

#[tokio::test(flavor = "multi_thread")]
async fn slow_client_converges() {
    let runtime_dir = runtime_dir("slow");
    let marker = runtime_dir.join("printed");
    let server = TestServer::start_in(runtime_dir, &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    client
        .type_line(&format!("seq 1 10000; touch {}", marker.display()))
        .await;
    wait_for_file(&marker).await;
    assert_converges(&server, &mut client).await;
    assert!(client.parser.screen().contents().contains("10000"));
}

#[tokio::test(flavor = "multi_thread")]
async fn mismatched_version_is_rejected_and_server_keeps_serving() {
    let server = TestServer::start("version", &["/bin/sh"]).await;
    let mut stream = TestClient::connect(&server.socket()).await;
    write_frame(
        &mut stream,
        &Hello {
            version: PROTOCOL_VERSION + 1,
            cols: 80,
            rows: 24,
        },
    )
    .await;
    let mut decoder = gband_protocol::Decoder::new();
    let reply: HelloReply = read_frame(&mut stream, &mut decoder).await.unwrap();
    assert_eq!(
        reply,
        HelloReply::Rejected {
            version: PROTOCOL_VERSION
        }
    );
    assert!(closes(&mut stream).await);
    server.attach(80, 24).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn non_hello_first_frame_is_closed() {
    let server = TestServer::start("nonhello", &["/bin/sh"]).await;
    let mut stream = TestClient::connect(&server.socket()).await;
    write_frame(&mut stream, &ClientMessage::Detach).await;
    assert!(closes(&mut stream).await);
    server.attach(80, 24).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn oversized_frame_is_closed() {
    use tokio::io::AsyncWriteExt;
    let server = TestServer::start("oversized", &["/bin/sh"]).await;
    let mut stream = TestClient::connect(&server.socket()).await;
    stream
        .write_all(&(17u32 * 1024 * 1024).to_be_bytes())
        .await
        .unwrap();
    assert!(closes(&mut stream).await);
    server.attach(80, 24).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn undecodable_frame_is_closed() {
    use tokio::io::AsyncWriteExt;
    let server = TestServer::start("garbage", &["/bin/sh"]).await;
    let mut stream = TestClient::connect(&server.socket()).await;
    stream.write_all(&[0, 0, 0, 1, 0xff]).await.unwrap();
    assert!(closes(&mut stream).await);
    server.attach(80, 24).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn up_is_encoded_with_application_cursor_mode() {
    let server = TestServer::start(
        "decckm",
        &[
            "/bin/sh",
            "-c",
            r"stty raw -echo; printf '\033[?1hready'; head -c 3 | od -An -tx1; exec sleep 100",
        ],
    )
    .await;
    let mut client = server.attach(80, 24).await;
    client
        .wait_for(|screen| screen.application_cursor() && screen.contents().contains("ready"))
        .await;
    client
        .send(&ClientMessage::Key(Key::plain(KeyCode::Up)))
        .await;
    client.wait_for_text("1b 4f 41").await;
}

#[tokio::test(flavor = "multi_thread")]
async fn keys_from_two_clients_reach_the_program() {
    let server = TestServer::start(
        "twokeys",
        &[
            "/bin/sh",
            "-c",
            r"stty raw -echo; printf ready; head -c 2 | od -An -c; exec sleep 100",
        ],
    )
    .await;
    let mut first = server.attach(80, 24).await;
    let mut second = server.attach(80, 24).await;
    first.wait_for_text("ready").await;
    first
        .send(&ClientMessage::Key(Key::plain(KeyCode::Char('a'))))
        .await;
    second
        .send(&ClientMessage::Key(Key::plain(KeyCode::Char('b'))))
        .await;
    first
        .wait_for(|screen| {
            let contents = screen.contents();
            contents.contains('a') && contents.lines().any(|line| line.contains(" b"))
        })
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn latest_client_sets_the_pty_size() {
    let server = TestServer::start("size", &["/bin/sh"]).await;
    let mut first = server.attach(120, 40).await;
    first.type_line("stty size").await;
    first.wait_for_text("40 120").await;

    let mut second = server.attach(100, 30).await;
    assert_eq!(second.parser.screen().size(), (30, 100));
    second.type_line("stty size").await;
    second.wait_for_text("30 100").await;

    first
        .send(&ClientMessage::Resize { cols: 90, rows: 25 })
        .await;
    first.wait_for(|screen| screen.size() == (25, 90)).await;
    first.type_line("stty size").await;
    first.wait_for_text("25 90").await;

    second.send(&ClientMessage::Detach).await;
    assert!(closes(&mut second.stream).await);
    tokio::time::sleep(Duration::from_millis(100)).await;
    first.type_line("clear; stty size").await;
    first
        .wait_for(|screen| screen.contents().starts_with("25 90"))
        .await;
    assert_eq!(first.parser.screen().size(), (25, 90));
}
