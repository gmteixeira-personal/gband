use gband_client::{ClientConfig, connect};
use gband_protocol::{ClientMessage, ServerMessage, SessionName};
use gband_test_support::{IDENTITY, Peer, RelayTransport, TestClient, TestServer};

#[tokio::test(flavor = "multi_thread")]
async fn attach_through_a_relay() {
    let server = TestServer::start("client-relay", &["/bin/sh"]).await;
    let relay = RelayTransport {
        socket: server.socket(),
    };
    let config = ClientConfig {
        session: SessionName::default(),
        identity: IDENTITY,
        replace_mismatched: false,
        kill_command: "gband kill-server".to_owned(),
    };
    let connection = connect(&config, &relay).await.unwrap();
    assert!(!connection.stale_server);
    let info = ServerMessage::Info {
        pid: connection.pid,
        executable: connection.executable,
    };
    let mut client = TestClient::attach_over(
        Peer::from(connection),
        info,
        SessionName::default(),
        &server.runtime_dir,
    )
    .await;
    client.wait_for_prompt(client.first()).await;
    client.type_line(r"printf 'relayed\n'").await;
    client.wait_for_line("relayed").await;
    client.send(&ClientMessage::Detach).await;
    assert!(client.peer.closes().await);

    assert!(!server.handle.is_finished());
    assert_eq!(server.listed().await, [("default".to_owned(), 1, 0)]);
}
