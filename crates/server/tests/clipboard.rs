use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use gband_test_support::*;

const QUIET: Duration = Duration::from_millis(500);
const WRITE_HI: &str = r"printf '\033]52;c;aGk=\007'";

async fn settle_clipboards(client: &mut TestClient) {
    client
        .wait_until(|client| !client.clipboards.is_empty())
        .await;
    assert!(client.pump(QUIET).await);
}

#[tokio::test(flavor = "multi_thread")]
async fn every_attached_client_receives_the_write_once() {
    let server = TestServer::start("clip-two", &["/bin/sh"]).await;
    let mut first = server.attach(80, 24).await;
    let mut second = server.attach(80, 24).await;
    let window = first.first();
    first.type_line(WRITE_HI).await;
    settle_clipboards(&mut first).await;
    settle_clipboards(&mut second).await;
    assert_eq!(first.clipboards, [(window, "hi".to_owned())]);
    assert_eq!(second.clipboards, [(window, "hi".to_owned())]);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_write_reaches_only_its_own_session() {
    let server = TestServer::start("clip-sessions", &["/bin/sh"]).await;
    let mut a = server.attach_to("a", Path::new("/"), 80, 24).await;
    let mut b = server.attach_to("b", Path::new("/"), 80, 24).await;
    a.type_line(WRITE_HI).await;
    settle_clipboards(&mut a).await;
    assert!(b.pump(QUIET).await);
    assert_eq!(a.clipboards.len(), 1);
    assert_eq!(b.clipboards, []);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_write_before_attaching_never_reaches_the_client() {
    let script = format!("{WRITE_HI}; printf done; exec sleep 100");
    let server = TestServer::start("clip-before", &["/bin/sh", "-c", &script]).await;
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
    let mut client = server.attach(80, 24).await;
    assert!(client.pump(QUIET).await);
    assert_eq!(client.clipboards, []);
}

#[tokio::test(flavor = "multi_thread")]
async fn only_the_latest_write_is_sent() {
    let server = TestServer::start("clip-latest", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let window = client.first();
    client
        .type_line(r"printf '\033]52;c;YQ==\007\033]52;c;Yg==\007'")
        .await;
    settle_clipboards(&mut client).await;
    assert_eq!(client.clipboards, [(window, "b".to_owned())]);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_query_is_never_answered() {
    let runtime_dir = runtime_dir("clip-query");
    let reply = runtime_dir.join("reply");
    let go = runtime_dir.join("go");
    let script = format!(
        r"stty raw -echo; while [ ! -e {go} ]; do sleep 0.05; done; printf '\033]52;c;?\007\033[c'; head -c 9 > {path}.tmp; mv {path}.tmp {path}; exec sleep 100",
        go = go.display(),
        path = reply.display(),
    );
    let server = TestServer::start_in(runtime_dir, &["/bin/sh", "-c", &script]).await;
    let mut client = server.attach(80, 24).await;
    fs::write(&go, "").unwrap();
    wait_for_file(&reply).await;
    assert_eq!(
        String::from_utf8_lossy(&fs::read(&reply).unwrap()),
        "\x1b[?62;22c"
    );
    assert!(client.pump(QUIET).await);
    assert_eq!(client.clipboards, []);
}
