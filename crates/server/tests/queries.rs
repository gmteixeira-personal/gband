mod common;

use std::fs;

use common::*;

async fn program_reads(name: &str, output: &str, expected: &str) {
    let runtime_dir = runtime_dir(name);
    let reply = runtime_dir.join("reply");
    let script = format!(
        "stty raw -echo; printf '{output}'; head -c {length} > {path}.tmp; mv {path}.tmp {path}; exec sleep 100",
        length = expected.len(),
        path = reply.display(),
    );
    let _server = TestServer::start_in(runtime_dir, &["/bin/sh", "-c", &script]).await;
    wait_for_file(&reply).await;
    assert_eq!(
        String::from_utf8_lossy(&fs::read(&reply).unwrap()),
        expected
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn device_attributes_are_answered_without_a_client() {
    program_reads("da1", r"\033[0c", "\x1b[?62;22c").await;
}

#[tokio::test(flavor = "multi_thread")]
async fn operating_status_is_answered() {
    program_reads("dsr5", r"\033[5n", "\x1b[0n").await;
}

#[tokio::test(flavor = "multi_thread")]
async fn cursor_position_is_answered_from_the_grid() {
    program_reads("dsr6", r"\033[5;10H\033[6n", "\x1b[5;10R").await;
}

#[tokio::test(flavor = "multi_thread")]
async fn version_names_gband() {
    let expected = format!("\x1bP>|gband {}\x1b\\", env!("CARGO_PKG_VERSION"));
    program_reads("xtversion", r"\033[>q", &expected).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn set_mode_reports_set() {
    program_reads("mode-set", r"\033[?2004h\033[?2004$p", "\x1b[?2004;1$y").await;
}

#[tokio::test(flavor = "multi_thread")]
async fn reset_mode_reports_reset() {
    program_reads("mode-rst", r"\033[?1$p", "\x1b[?1;2$y").await;
}

#[tokio::test(flavor = "multi_thread")]
async fn unknown_mode_reports_unrecognised() {
    program_reads("mode-unk", r"\033[?7727$p", "\x1b[?7727;0$y").await;
}

#[tokio::test(flavor = "multi_thread")]
async fn kitty_keyboard_query_stays_unanswered() {
    program_reads("kitty", r"\033[?u\033[c", "\x1b[?62;22c").await;
}
