use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use gband_core::geometry::{CellSize, Size};
use gband_protocol::ClientMessage;
use gband_test_support::*;

const QUIET: Duration = Duration::from_millis(500);
const SIXEL: &[u8] = b"\x1bPq#0;2;0;0;0#0~~\x1b\\";
const WRITE_SIXEL: &str = r"\033[3;5H\033Pq#0;2;0;0;0#0~~\033\\";
const ATTRIBUTES: &str = "\x1b[?62;22c";
const SIXEL_ATTRIBUTES: &str = "\x1b[?62;4;22c";

struct Program {
    server: TestServer,
    go: PathBuf,
    reply: PathBuf,
}

impl Program {
    async fn start(name: &str, output: &str, length: usize) -> Self {
        let runtime_dir = runtime_dir(name);
        let reply = runtime_dir.join("reply");
        let go = runtime_dir.join("go");
        let script = format!(
            r"stty raw -echo; while [ ! -e {go} ]; do sleep 0.05; done; printf '{output}'; head -c {length} > {path}.tmp; mv {path}.tmp {path}; exec sleep 100",
            go = go.display(),
            path = reply.display(),
        );
        let server = TestServer::start_in(runtime_dir, &["/bin/sh", "-c", &script]).await;
        Self { server, go, reply }
    }

    async fn run(&self) -> String {
        fs::write(&self.go, "").unwrap();
        wait_for_file(&self.reply).await;
        String::from_utf8_lossy(&fs::read(&self.reply).unwrap()).into_owned()
    }

    fn go(&self) {
        fs::write(&self.go, "").unwrap();
    }
}

async fn settle_sixel(client: &mut TestClient, count: usize) {
    client
        .wait_until(|client| client.sixel.len() >= count)
        .await;
    assert!(client.pump(QUIET).await);
}

async fn sixel_client(server: &TestServer, cell: Option<CellSize>) -> TestClient {
    server.attach_with_sixel(80, 24, cell).await
}

#[tokio::test(flavor = "multi_thread")]
async fn every_supporting_client_receives_the_sequence_with_its_position() {
    let program = Program::start("sixel-two", WRITE_SIXEL, 0).await;
    let mut first = sixel_client(&program.server, None).await;
    let mut second = sixel_client(&program.server, None).await;
    let window = first.first();
    program.go();
    settle_sixel(&mut first, 1).await;
    settle_sixel(&mut second, 1).await;
    assert_eq!(first.sixel, [(window, 3, 5, SIXEL.to_vec())]);
    assert_eq!(second.sixel, [(window, 3, 5, SIXEL.to_vec())]);
}

#[tokio::test(flavor = "multi_thread")]
async fn order_is_kept() {
    let program = Program::start("sixel-order", r"\033PqA\033\\\033PqB\033\\", 0).await;
    let mut client = sixel_client(&program.server, None).await;
    let window = client.first();
    program.go();
    settle_sixel(&mut client, 2).await;
    assert_eq!(
        client.sixel,
        [
            (window, 1, 1, b"\x1bPqA\x1b\\".to_vec()),
            (window, 1, 1, b"\x1bPqB\x1b\\".to_vec()),
        ]
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_client_without_support_receives_nothing() {
    let program = Program::start("sixel-unsupported", WRITE_SIXEL, 0).await;
    let mut supporting = sixel_client(&program.server, None).await;
    let mut plain = program.server.attach_without_graphics(80, 24).await;
    program.go();
    settle_sixel(&mut supporting, 1).await;
    assert!(plain.pump(QUIET).await);
    assert_eq!(plain.sixel, []);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_sequence_before_attaching_never_reaches_the_client() {
    let script = format!("printf '{WRITE_SIXEL}'; printf done; exec sleep 100");
    let server = TestServer::start("sixel-before", &["/bin/sh", "-c", &script]).await;
    let deadline = Instant::now() + TIMEOUT;
    while !server
        .screens()
        .await
        .values()
        .any(|screen| screen.contents().contains("done"))
    {
        assert!(Instant::now() < deadline, "the program never wrote");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let mut client = sixel_client(&server, None).await;
    assert!(client.pump(QUIET).await);
    assert_eq!(client.sixel, []);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_sequence_reaches_only_its_own_session() {
    let server = TestServer::start("sixel-sessions", &["/bin/sh"]).await;
    let offer = Offer {
        sixel: true,
        ..Offer::NONE
    };
    let (peer, info) = TestClient::accepted(&server.socket(), 80, 24).await;
    let mut a = TestClient::attach_with(peer, info, session("a"), Path::new("/"), offer).await;
    let (peer, info) = TestClient::accepted(&server.socket(), 80, 24).await;
    let mut b = TestClient::attach_with(peer, info, session("b"), Path::new("/"), offer).await;
    a.type_line(r"printf '\033PqA\033\\'").await;
    settle_sixel(&mut a, 1).await;
    assert!(b.pump(QUIET).await);
    assert_eq!(a.sixel.len(), 1);
    assert_eq!(b.sixel, []);
}

#[tokio::test(flavor = "multi_thread")]
async fn device_attributes_offer_sixel_to_a_supporting_client() {
    let program = Program::start("sixel-da", r"\033[c", SIXEL_ATTRIBUTES.len()).await;
    let _client = sixel_client(&program.server, None).await;
    assert_eq!(program.run().await, SIXEL_ATTRIBUTES);
}

#[tokio::test(flavor = "multi_thread")]
async fn device_attributes_without_a_supporting_client() {
    let program = Program::start("sixel-da-none", r"\033[c", ATTRIBUTES.len()).await;
    let _client = program.server.attach_without_graphics(80, 24).await;
    assert_eq!(program.run().await, ATTRIBUTES);
}

#[tokio::test(flavor = "multi_thread")]
async fn cell_size_and_text_area_are_answered() {
    let expected = "\x1b[6;20;10t\x1b[4;440;380t";
    let program = Program::start("sixel-cell", r"\033[16t\033[14t", expected.len()).await;
    let client = sixel_client(&program.server, Some(CellSize::new(10, 20))).await;
    assert_eq!(client.grids[&client.first()].size(), Size::new(38, 22));
    assert_eq!(program.run().await, expected);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_latest_sixel_client_sets_the_cell_size() {
    let program = Program::start("sixel-latest", r"\033[16t", 9).await;
    let _first = sixel_client(&program.server, Some(CellSize::new(10, 20))).await;
    let _second = sixel_client(&program.server, Some(CellSize::new(8, 16))).await;
    assert_eq!(program.run().await, "\x1b[6;16;8t");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_cell_size_falls_back_when_the_latest_detaches() {
    let program = Program::start("sixel-fallback", r"\033[16t", 10).await;
    let _first = sixel_client(&program.server, Some(CellSize::new(10, 20))).await;
    let mut second = sixel_client(&program.server, Some(CellSize::new(8, 16))).await;
    second.send(&ClientMessage::Detach).await;
    let deadline = Instant::now() + TIMEOUT;
    while program.server.listed().await != [("default".to_owned(), 1, 1)] {
        assert!(
            Instant::now() < deadline,
            "the second client never detached"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(program.run().await, "\x1b[6;20;10t");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_resize_updates_the_cell_size() {
    let program = Program::start("sixel-resize", r"\033[16t", 9).await;
    let mut client = sixel_client(&program.server, Some(CellSize::new(10, 20))).await;
    client
        .send(&ClientMessage::Resize {
            cols: 80,
            rows: 24,
            cell: Some(CellSize::new(9, 19)),
        })
        .await;
    assert!(client.pump(QUIET).await);
    assert_eq!(program.run().await, "\x1b[6;19;9t");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_cell_size_is_unanswered_without_a_sixel_client() {
    let program = Program::start("sixel-cell-none", r"\033[16t\033[c", ATTRIBUTES.len()).await;
    let _client = program.server.attach_without_graphics(80, 24).await;
    assert_eq!(program.run().await, ATTRIBUTES);
}
