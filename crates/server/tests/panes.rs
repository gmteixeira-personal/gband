use std::fs;
use std::time::Duration;

use gband_core::geometry::Size;
use gband_core::input::{Key, KeyCode};
use gband_core::layout::{LayoutOptions, PaneId, Proportion, SessionAction};
use gband_protocol::ClientMessage;
use gband_test_support::*;
use tokio::time::Instant;

async fn stty_size(client: &mut TestClient, pane: PaneId, expected: &str) {
    client.type_line_to(pane, "clear; stty size").await;
    client
        .wait_for_pane(pane, |screen| {
            screen
                .contents()
                .lines()
                .any(|line| line.trim() == expected)
        })
        .await;
}

fn pane_number(screen: &Grid) -> Option<u32> {
    screen
        .contents()
        .lines()
        .find_map(|line| line.strip_prefix("pane="))
        .and_then(|number| number.trim().parse().ok())
}

#[tokio::test(flavor = "multi_thread")]
async fn first_pane_is_sized_from_the_initial_area() {
    let runtime_dir = runtime_dir("init-size");
    let record = runtime_dir.join("size");
    let script = format!("stty size > {}; exec sleep 100", record.display());
    let _server = TestServer::start_in(runtime_dir, &["/bin/sh", "-c", &script]).await;
    wait_for_file(&record).await;
    let deadline = Instant::now() + TIMEOUT;
    loop {
        let size = fs::read_to_string(&record).unwrap();
        if size == "22 38\n" {
            break;
        }
        assert!(Instant::now() < deadline, "size record holds {size:?}");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn opening_a_pane_reaches_every_client_and_focuses_the_requester() {
    let server = TestServer::start("open", &["/bin/sh"]).await;
    let mut requester = server.attach(80, 24).await;
    let mut other = server.attach(80, 24).await;
    let first = requester.first();
    let opened = requester.open_after(first).await;
    assert_eq!(requester.panes(), vec![first, opened]);
    other
        .wait_until(|client| client.panes().len() == 2 && client.grids.len() == 2)
        .await;
    assert_eq!(other.panes(), vec![first, opened]);
    assert!(other.pump(Duration::from_millis(200)).await);
    assert!(other.focus.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn panes_are_told_apart_by_their_environment() {
    let server = TestServer::start("pane-env", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let second = client.open_after(first).await;
    for pane in [first, second] {
        client.type_line_to(pane, "echo pane=$GBAND_PANE").await;
        client
            .wait_for_pane(pane, |screen| pane_number(screen).is_some())
            .await;
    }
    assert_eq!(pane_number(client.pane_screen(first)), Some(first.0));
    assert_eq!(pane_number(client.pane_screen(second)), Some(second.0));
}

#[tokio::test(flavor = "multi_thread")]
async fn keys_reach_only_their_pane() {
    let server = TestServer::start("routing", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let second = client.open_after(first).await;
    client.type_line_to(second, "echo routed-here").await;
    client.wait_for_pane_text(second, "routed-here\n").await;
    client.pump(Duration::from_millis(200)).await;
    assert!(!client.pane_screen(first).contents().contains("routed"));
}

#[tokio::test(flavor = "multi_thread")]
async fn keys_for_a_missing_pane_are_dropped() {
    let server = TestServer::start("no-pane", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    client
        .key_to(PaneId(99), Key::plain(KeyCode::Char('x')))
        .await;
    client
        .send(&ClientMessage::Paste {
            pane: PaneId(99),
            text: "ignored".into(),
        })
        .await;
    client.type_line("echo still-serving").await;
    client.wait_for_text("still-serving\n").await;
}

#[tokio::test(flavor = "multi_thread")]
async fn concurrent_opens_give_every_client_the_same_layout() {
    let server = TestServer::start("conc-open", &["/bin/sh"]).await;
    let mut first = server.attach(80, 24).await;
    let mut second = server.attach(80, 24).await;
    let pane = first.first();
    let band = first.layout.bands()[0].id;
    let open = SessionAction::open(band, Some(pane), None);
    first.act(open.clone()).await;
    second.act(open).await;
    first.wait_until(|client| client.focus.len() == 1).await;
    second.wait_until(|client| client.focus.len() == 1).await;
    first.wait_until(|client| client.panes().len() == 3).await;
    second.wait_until(|client| client.panes().len() == 3).await;
    assert_eq!(first.layout, second.layout);
    assert_ne!(first.focus, second.focus);
}

#[tokio::test(flavor = "multi_thread")]
async fn latest_client_sets_the_area() {
    let server = TestServer::start("area", &["/bin/sh"]).await;
    let mut first = server.attach(120, 40).await;
    first.show_all().await;
    first
        .wait_for(|screen| screen.size() == Size::new(58, 38))
        .await;
    let pane = first.first();
    stty_size(&mut first, pane, "38 58").await;

    let mut second = server.attach(100, 30).await;
    second.show_all().await;
    assert_eq!(second.area, Size::new(100, 30));
    second
        .wait_for(|screen| screen.size() == Size::new(48, 28))
        .await;
    stty_size(&mut second, pane, "28 48").await;

    first
        .send(&ClientMessage::Resize { cols: 90, rows: 25 })
        .await;
    first
        .wait_for(|screen| screen.size() == Size::new(43, 23))
        .await;
    stty_size(&mut first, pane, "23 43").await;

    second.send(&ClientMessage::Detach).await;
    assert!(second.peer.closes().await);
    tokio::time::sleep(Duration::from_millis(100)).await;
    stty_size(&mut first, pane, "23 43").await;
    assert_eq!(first.area, Size::new(90, 25));
}

#[tokio::test(flavor = "multi_thread")]
async fn opening_a_pane_keeps_other_sizes_and_width_changes_resize() {
    let runtime_dir = runtime_dir("keep-size");
    let record = runtime_dir.join("winched");
    let server = TestServer::start_in(runtime_dir, &["/bin/sh"]).await;
    let mut client = server.attach(90, 30).await;
    client.show_all().await;
    client
        .wait_for(|screen| screen.size() == Size::new(43, 28))
        .await;
    let first = client.first();
    client.wait_for_prompt(first).await;
    client
        .type_line(&format!(
            "trap 'echo WINCHED >> {}' WINCH",
            record.display()
        ))
        .await;
    let second = client.open_after(first).await;
    client.show_all().await;
    client
        .wait_for_pane(second, |screen| screen.size() == Size::new(43, 28))
        .await;
    stty_size(&mut client, first, "28 43").await;
    assert!(!record.exists());

    client.act(SessionAction::CycleWidth(first)).await;
    client
        .wait_for(|screen| screen.size() == Size::new(58, 28))
        .await;
    stty_size(&mut client, first, "28 58").await;
    wait_for_file(&record).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn closing_a_shell_removes_its_pane() {
    let server = TestServer::start("close", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let second = client.open_after(first).await;
    client.act(SessionAction::ClosePane(second)).await;
    client
        .wait_until(|client| client.panes() == vec![first])
        .await;
    assert!(!client.grids.contains_key(&second));
    assert!(!server.handle.is_finished());
}

#[tokio::test(flavor = "multi_thread")]
async fn closing_a_pane_that_ignores_sighup_kills_it() {
    let server = TestServer::start("close-hup", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let second = client.open_after(first).await;
    client
        .type_line_to(second, "trap '' HUP; echo trapped; sleep 100")
        .await;
    client
        .wait_for_pane(second, |screen| {
            screen.contents().lines().any(|line| line == "trapped")
        })
        .await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    let started = Instant::now();
    client.act(SessionAction::ClosePane(second)).await;
    client
        .wait_until(|client| client.panes() == vec![first])
        .await;
    assert!(started.elapsed() < Duration::from_secs(3));
}

#[tokio::test(flavor = "multi_thread")]
async fn one_of_two_shells_exiting_leaves_the_session_running() {
    let server = TestServer::start("one-exits", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let second = client.open_after(first).await;
    client.type_line_to(second, "exit").await;
    client
        .wait_until(|client| client.panes() == vec![first])
        .await;
    client.type_line("echo survivor").await;
    client.wait_for_text("survivor\n").await;
    assert!(!client.exited);

    client.type_line("exit").await;
    client.drain_to_end().await;
    assert!(client.exited);
    let socket = server.socket();
    server.finished().await.unwrap();
    assert!(!socket.exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn stacked_panes_share_the_height() {
    let server = TestServer::start("stack", &["/bin/sh"]).await;
    let mut client = server.attach(80, 25).await;
    client.show_all().await;
    let first = client.first();
    let second = client.open_after(first).await;
    client.show_all().await;
    client
        .act(SessionAction::ConsumeOrExpel {
            pane: second,
            direction: gband_core::layout::Direction::Left,
        })
        .await;
    client
        .wait_for(|screen| screen.size() == Size::new(38, 11))
        .await;
    stty_size(&mut client, first, "11 38").await;
    stty_size(&mut client, second, "10 38").await;
}

#[tokio::test(flavor = "multi_thread")]
async fn new_default_width_applies_to_columns_opened_after_it() {
    let runtime_dir = runtime_dir("reload-width");
    let (options, receiver) = tokio::sync::watch::channel(LayoutOptions::default());
    let config = gband_server::ServerConfig {
        options: receiver,
        ..config(&runtime_dir, &["/bin/sh"])
    };
    let server = TestServer::start_with(runtime_dir, config).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    options.send_replace(LayoutOptions {
        default_width: Proportion::ONE_THIRD,
        ..LayoutOptions::default()
    });
    let second = client.open_after(first).await;
    let width = |client: &TestClient, pane: PaneId| {
        let location = client.layout.locate(pane).unwrap();
        client.layout.bands()[location.band].columns[location.column].width
    };
    assert_eq!(width(&client, first), Proportion::ONE_HALF);
    assert_eq!(width(&client, second), Proportion::ONE_THIRD);
}
