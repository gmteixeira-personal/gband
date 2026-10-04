use gband_core::input::{Key, KeyCode};
use gband_test_support::*;

async fn converges(name: &str, program: &str) -> TestClient {
    let script = format!("read line; {program}; exec sleep 100");
    let server = TestServer::start(name, &["/bin/sh", "-c", &script]).await;
    let mut client = server.attach(80, 24).await;
    client.key(Key::plain(KeyCode::Enter)).await;
    client.wait_for_text("done").await;
    assert_converges(&server, &mut client).await;
    client
}

#[tokio::test(flavor = "multi_thread")]
async fn colours_and_attributes() {
    converges(
        "prog-sgr",
        r"printf '\033[1;31mbold red\033[0m \033[2;3;4mdim italic underline\033[0m\n\033[7minverse\033[0m \033[38;5;208mindexed\033[0m \033[48;2;1;2;3mtruecolour\033[0m\ndone'",
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn wide_characters() {
    let client = converges(
        "prog-wide",
        r"printf '漢字 x ｗｉｄｅ\n日本語のテキスト\ndone'",
    )
    .await;
    assert!(client.screen().contents().contains("漢字 x"));
}

#[tokio::test(flavor = "multi_thread")]
async fn cursor_addressing_and_erase() {
    converges(
        "prog-cup",
        r"printf '\033[2J\033[5;10Hxyz\033[3;1Habcdefgh\033[3;3H\033[K\033[5;11H\033[1K\033[8;4Hmid\033[8;5H\033[X\033[12;1Hdone'",
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn alternate_screen_with_application_cursor() {
    let client = converges(
        "prog-alt",
        r"printf 'main screen\033[?1049h\033[?1h\033[2;2Halternate\033[4;1Hdone'",
    )
    .await;
    assert!(client.screen().modes().application_cursor);
    assert!(!client.screen().contents().contains("main screen"));
}

#[tokio::test(flavor = "multi_thread")]
async fn scroll_region() {
    converges(
        "prog-region",
        r"printf '\033[2J\033[1;1Htop\033[20;1Hbottom\033[5;10r\033[10;1H'; for i in 1 2 3 4 5 6 7 8 9 10 11 12; do printf '\nline %s' $i; done; printf '\033[r\033[15;1Hdone'",
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn output_longer_than_one_screen() {
    let client = converges(
        "prog-long",
        r"for i in $(seq 1 200); do printf 'line %s\n' $i; done; printf done",
    )
    .await;
    assert!(client.screen().contents().contains("line 200"));
}
