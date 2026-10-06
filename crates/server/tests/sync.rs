use gband_core::input::{Key, KeyCode};
use gband_protocol::{ClientMessage, Hello, HelloReply, PROTOCOL_VERSION};
use gband_test_support::*;

#[tokio::test(flavor = "multi_thread")]
async fn snapshot_and_updates_match_the_server() {
    let server = TestServer::start("match", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    client
        .type_line(r"printf '\033[1;31mred\033[0m \033[42mgreen\033[0m\033[?1h\033[?2004h done\n'")
        .await;
    client.wait_for_text("green done").await;
    assert_converges(&server, &mut client).await;
    assert!(client.screen().modes().application_cursor);
    assert!(client.screen().modes().bracketed_paste);
}

#[tokio::test(flavor = "multi_thread")]
async fn slow_client_converges() {
    let runtime_dir = runtime_dir("slow");
    let first_marker = runtime_dir.join("first");
    let second_marker = runtime_dir.join("second");
    let server = TestServer::start_in(runtime_dir, &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let second = client.open_after(first).await;
    client
        .type_line_to(
            first,
            &format!("seq 1 10000; touch {}", first_marker.display()),
        )
        .await;
    client
        .type_line_to(
            second,
            &format!("seq 5 10005; touch {}", second_marker.display()),
        )
        .await;
    wait_for_file(&first_marker).await;
    wait_for_file(&second_marker).await;
    assert_converges(&server, &mut client).await;
    assert!(client.window_screen(first).contents().contains("10000"));
    assert!(client.window_screen(second).contents().contains("10005"));
}

#[tokio::test(flavor = "multi_thread")]
async fn mismatched_version_is_rejected_and_server_keeps_serving() {
    let server = TestServer::start("version", &["/bin/sh"]).await;
    let mut peer = Peer::connect(&server.socket()).await;
    peer.send(&Hello {
        version: PROTOCOL_VERSION + 1,
        cols: 80,
        rows: 24,
    })
    .await;
    let reply: HelloReply = peer.recv().await.unwrap();
    assert_eq!(
        reply,
        HelloReply::Rejected {
            version: PROTOCOL_VERSION
        }
    );
    assert!(peer.closes().await);
    server.attach(80, 24).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn version_seven_client_is_rejected_by_version_eight() {
    let server = TestServer::start("version-seven", &["/bin/sh"]).await;
    let mut peer = Peer::connect(&server.socket()).await;
    peer.send(&Hello {
        version: 7,
        cols: 80,
        rows: 24,
    })
    .await;
    let reply: HelloReply = peer.recv().await.unwrap();
    assert_eq!(reply, HelloReply::Rejected { version: 8 });
    assert!(peer.closes().await);
}

#[tokio::test(flavor = "multi_thread")]
async fn non_hello_first_frame_is_closed() {
    let server = TestServer::start("nonhello", &["/bin/sh"]).await;
    let mut peer = Peer::connect(&server.socket()).await;
    peer.send(&ClientMessage::Detach).await;
    if let Some(reply) = peer.recv::<HelloReply>().await {
        assert_eq!(
            reply,
            HelloReply::Rejected {
                version: PROTOCOL_VERSION
            }
        );
    }
    assert!(peer.closes().await);
    server.attach(80, 24).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn later_hello_with_other_fields_is_rejected() {
    let server = TestServer::start("later-hello", &["/bin/sh"]).await;
    let mut peer = Peer::connect(&server.socket()).await;
    peer.send(&(PROTOCOL_VERSION + 1, [0xffu8, 0xff, 0xff]))
        .await;
    let reply: HelloReply = peer.recv().await.unwrap();
    assert_eq!(
        reply,
        HelloReply::Rejected {
            version: PROTOCOL_VERSION
        }
    );
    assert!(peer.closes().await);
    server.attach(80, 24).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn matching_version_with_a_truncated_size_is_closed_unanswered() {
    let server = TestServer::start("truncated", &["/bin/sh"]).await;
    let mut peer = Peer::connect(&server.socket()).await;
    peer.send(&(PROTOCOL_VERSION, 80u16)).await;
    assert_eq!(peer.recv::<HelloReply>().await, None);
    server.attach(80, 24).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn empty_first_frame_is_closed_unanswered() {
    let server = TestServer::start("empty", &["/bin/sh"]).await;
    let mut peer = Peer::connect(&server.socket()).await;
    peer.send_bytes(&[0, 0, 0, 0]).await;
    assert_eq!(peer.recv::<HelloReply>().await, None);
    server.attach(80, 24).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn oversized_frame_is_closed() {
    let server = TestServer::start("oversized", &["/bin/sh"]).await;
    let mut peer = Peer::connect(&server.socket()).await;
    peer.send_bytes(&(17u32 * 1024 * 1024).to_be_bytes()).await;
    assert!(peer.closes().await);
    server.attach(80, 24).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn undecodable_frame_is_closed() {
    let server = TestServer::start("garbage", &["/bin/sh"]).await;
    let mut peer = Peer::connect(&server.socket()).await;
    peer.send_bytes(&[0, 0, 0, 1, 0xff]).await;
    assert!(peer.closes().await);
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
        .wait_for(|screen| screen.modes().application_cursor && screen.contents().contains("ready"))
        .await;
    client.key(Key::plain(KeyCode::Up)).await;
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
    first.key(Key::plain(KeyCode::Char('a'))).await;
    second.key(Key::plain(KeyCode::Char('b'))).await;
    first
        .wait_for(|screen| {
            let contents = screen.contents();
            contents.contains('a') && contents.lines().any(|line| line.contains(" b"))
        })
        .await;
}
