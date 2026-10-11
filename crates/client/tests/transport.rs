use std::path::PathBuf;

use gband_client::{ClientConfig, UnixTransport, connect, relay};
use gband_core::geometry::Size;
use gband_protocol::{
    ClientMessage, Hello, HelloReply, PROTOCOL_VERSION, ServerMessage, SessionName,
};
use gband_test_support::{IDENTITY, Peer, RelayTransport, TestClient, TestServer};
use tokio::task::JoinHandle;

const PIPE_LEN: usize = 64 * 1024;

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
    let connection = connect(&config, &relay, Size::new(80, 24)).await.unwrap();
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

fn relay_to(server: &TestServer) -> (Peer, JoinHandle<anyhow::Result<()>>) {
    let transport = UnixTransport {
        socket: server.socket(),
        executable_path: PathBuf::from("/nonexistent/gband"),
        session: SessionName::default(),
        start_server: true,
        log_dir: server.runtime_dir.to_path_buf(),
    };
    let (input, relay_input) = tokio::io::duplex(PIPE_LEN);
    let (relay_output, output) = tokio::io::duplex(PIPE_LEN);
    let relayed = tokio::spawn(async move {
        relay(&transport, Size::new(80, 24), relay_input, relay_output).await
    });
    (Peer::new(Box::new(output), Box::new(input)), relayed)
}

async fn hello(peer: &mut Peer) -> HelloReply {
    peer.send(&Hello {
        version: PROTOCOL_VERSION,
        cols: 80,
        rows: 24,
    })
    .await;
    peer.recv().await.unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn relay_to_a_running_server() {
    let server = TestServer::start("relay-running", &["/bin/sh"]).await;
    let (mut peer, relayed) = relay_to(&server);
    assert_eq!(
        hello(&mut peer).await,
        HelloReply::Accepted {
            version: PROTOCOL_VERSION
        }
    );
    drop(peer);
    relayed.await.unwrap().unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn relay_ends_with_the_session() {
    let server = TestServer::start("relay-ends", &["/bin/sh"]).await;
    let (mut peer, relayed) = relay_to(&server);
    hello(&mut peer).await;
    let info: ServerMessage = peer.recv().await.unwrap();
    let mut client =
        TestClient::attach_over(peer, info, SessionName::default(), &server.runtime_dir).await;
    client.wait_for_prompt(client.first()).await;
    client.type_line("exit").await;
    client.drain_to_end().await;
    assert!(client.exited);
    drop(client);
    relayed.await.unwrap().unwrap();
}
