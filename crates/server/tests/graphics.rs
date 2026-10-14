use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use gband_test_support::*;

const QUIET: Duration = Duration::from_millis(500);
const WRITE_IMAGE: &str = r"printf '\033_Ga=T;AAAA\033\\'";
const IMAGE: &[u8] = b"\x1b_Ga=T;AAAA\x1b\\";
const ATTRIBUTES: &str = "\x1b[?62;22c";

async fn settle_graphics(client: &mut TestClient, count: usize) {
    client
        .wait_until(|client| client.graphics.len() >= count)
        .await;
    assert!(client.pump(QUIET).await);
}

#[tokio::test(flavor = "multi_thread")]
async fn every_supporting_client_receives_the_sequence() {
    let server = TestServer::start("gfx-two", &["/bin/sh"]).await;
    let mut first = server.attach(80, 24).await;
    let mut second = server.attach(80, 24).await;
    let window = first.first();
    first.type_line(WRITE_IMAGE).await;
    settle_graphics(&mut first, 1).await;
    settle_graphics(&mut second, 1).await;
    assert_eq!(first.graphics, [(window, IMAGE.to_vec())]);
    assert_eq!(second.graphics, [(window, IMAGE.to_vec())]);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_client_without_support_receives_nothing() {
    let server = TestServer::start("gfx-unsupported", &["/bin/sh"]).await;
    let mut supporting = server.attach(80, 24).await;
    let mut plain = server.attach_without_graphics(80, 24).await;
    supporting.type_line(WRITE_IMAGE).await;
    settle_graphics(&mut supporting, 1).await;
    assert!(plain.pump(QUIET).await);
    assert_eq!(plain.graphics, []);
}

#[tokio::test(flavor = "multi_thread")]
async fn order_is_kept() {
    let server = TestServer::start("gfx-order", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let window = client.first();
    client
        .type_line(r"printf '\033_Ga=T;QQ==\033\\'; printf '\033_Ga=T;Qg==\033\\'")
        .await;
    settle_graphics(&mut client, 2).await;
    assert_eq!(
        client.graphics,
        [
            (window, b"\x1b_Ga=T;QQ==\x1b\\".to_vec()),
            (window, b"\x1b_Ga=T;Qg==\x1b\\".to_vec()),
        ]
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_sequence_before_attaching_never_reaches_the_client() {
    let script = format!("{WRITE_IMAGE}; printf done; exec sleep 100");
    let server = TestServer::start("gfx-before", &["/bin/sh", "-c", &script]).await;
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
    assert_eq!(client.graphics, []);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_sequence_reaches_only_its_own_session() {
    let server = TestServer::start("gfx-sessions", &["/bin/sh"]).await;
    let mut a = server.attach_to("a", Path::new("/"), 80, 24).await;
    let mut b = server.attach_to("b", Path::new("/"), 80, 24).await;
    a.type_line(WRITE_IMAGE).await;
    settle_graphics(&mut a, 1).await;
    assert!(b.pump(QUIET).await);
    assert_eq!(a.graphics.len(), 1);
    assert_eq!(b.graphics, []);
}

async fn query_reply(name: &str, query: &str, length: usize, supporting: bool) -> String {
    let runtime_dir = runtime_dir(name);
    let reply = runtime_dir.join("reply");
    let go = runtime_dir.join("go");
    let script = format!(
        r"stty raw -echo; while [ ! -e {go} ]; do sleep 0.05; done; printf '{query}\033[c'; head -c {length} > {path}.tmp; mv {path}.tmp {path}; exec sleep 100",
        go = go.display(),
        path = reply.display(),
    );
    let server = TestServer::start_in(runtime_dir, &["/bin/sh", "-c", &script]).await;
    let mut client = if supporting {
        server.attach(80, 24).await
    } else {
        server.attach_without_graphics(80, 24).await
    };
    fs::write(&go, "").unwrap();
    wait_for_file(&reply).await;
    assert!(client.pump(QUIET).await);
    assert_eq!(client.graphics, []);
    String::from_utf8_lossy(&fs::read(&reply).unwrap()).into_owned()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_query_is_answered_when_a_client_supports_graphics() {
    let reply = query_reply("gfx-query", r"\033_Gi=31,s=1,v=1,a=q;AAAA\033\\", 21, true).await;
    assert_eq!(reply, format!("\x1b_Gi=31;OK\x1b\\{ATTRIBUTES}"));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_query_is_silent_without_a_supporting_client() {
    let reply = query_reply("gfx-query-none", r"\033_Gi=31,a=q;AAAA\033\\", 9, false).await;
    assert_eq!(reply, ATTRIBUTES);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_query_without_an_id_answers_zero() {
    let reply = query_reply("gfx-query-zero", r"\033_Ga=q,s=1,v=1;AAAA\033\\", 20, true).await;
    assert_eq!(reply, format!("\x1b_Gi=0;OK\x1b\\{ATTRIBUTES}"));
}
