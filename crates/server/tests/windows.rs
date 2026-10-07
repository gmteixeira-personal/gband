use std::fs;
use std::time::Duration;

use gband_core::geometry::Size;
use gband_core::input::{
    Key, KeyCode, Modifiers, MouseButton, MouseEvent, MouseKind, WheelDirection,
};
use gband_core::layout::{LayoutOptions, Proportion, SessionAction, WindowContent, WindowId};
use gband_protocol::ClientMessage;
use gband_test_support::*;
use tokio::time::Instant;

async fn stty_size(client: &mut TestClient, window: WindowId, expected: &str) {
    client.type_line_to(window, "clear; stty size").await;
    client
        .wait_for_window(window, |screen| {
            screen
                .contents()
                .lines()
                .any(|line| line.trim() == expected)
        })
        .await;
}

fn window_number(screen: &Grid) -> Option<u32> {
    screen
        .contents()
        .lines()
        .find_map(|line| line.strip_prefix("window="))
        .and_then(|number| number.trim().parse().ok())
}

#[tokio::test(flavor = "multi_thread")]
async fn first_window_is_sized_from_the_initial_area() {
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
async fn opening_a_window_reaches_every_client_and_focuses_the_requester() {
    let server = TestServer::start("open", &["/bin/sh"]).await;
    let mut requester = server.attach(80, 24).await;
    let mut other = server.attach(80, 24).await;
    let first = requester.first();
    let opened = requester.open_after(first).await;
    assert_eq!(requester.windows(), vec![first, opened]);
    other
        .wait_until(|client| client.windows().len() == 2 && client.grids.len() == 2)
        .await;
    assert_eq!(other.windows(), vec![first, opened]);
    assert!(other.pump(Duration::from_millis(200)).await);
    assert!(other.focus.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn windows_are_told_apart_by_their_environment() {
    let server = TestServer::start("window-env", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let second = client.open_after(first).await;
    for window in [first, second] {
        client
            .type_line_to(window, "echo window=$GBAND_WINDOW")
            .await;
        client
            .wait_for_window(window, |screen| window_number(screen).is_some())
            .await;
    }
    assert_eq!(window_number(client.window_screen(first)), Some(first.0));
    assert_eq!(window_number(client.window_screen(second)), Some(second.0));
}

#[tokio::test(flavor = "multi_thread")]
async fn keys_reach_only_their_window() {
    let server = TestServer::start("routing", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let second = client.open_after(first).await;
    client.type_line_to(second, "echo routed-here").await;
    client.wait_for_window_text(second, "routed-here\n").await;
    client.pump(Duration::from_millis(200)).await;
    assert!(!client.window_screen(first).contents().contains("routed"));
}

#[tokio::test(flavor = "multi_thread")]
async fn keys_for_a_missing_window_are_dropped() {
    let server = TestServer::start("no-window", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    client
        .key_to(WindowId(99), Key::plain(KeyCode::Char('x')))
        .await;
    client
        .send(&ClientMessage::Paste {
            window: WindowId(99),
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
    let window = first.first();
    let band = first.layout.bands()[0].id;
    let open = SessionAction::open(band, Some(window), None);
    first.act(open.clone()).await;
    second.act(open).await;
    first.wait_until(|client| client.focus.len() == 1).await;
    second.wait_until(|client| client.focus.len() == 1).await;
    first.wait_until(|client| client.windows().len() == 3).await;
    second
        .wait_until(|client| client.windows().len() == 3)
        .await;
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
    let window = first.first();
    stty_size(&mut first, window, "38 58").await;

    let mut second = server.attach(100, 30).await;
    second.show_all().await;
    assert_eq!(second.area, Size::new(100, 30));
    second
        .wait_for(|screen| screen.size() == Size::new(48, 28))
        .await;
    stty_size(&mut second, window, "28 48").await;

    first
        .send(&ClientMessage::Resize { cols: 90, rows: 25 })
        .await;
    first
        .wait_for(|screen| screen.size() == Size::new(43, 23))
        .await;
    stty_size(&mut first, window, "23 43").await;

    second.send(&ClientMessage::Detach).await;
    assert!(second.peer.closes().await);
    tokio::time::sleep(Duration::from_millis(100)).await;
    stty_size(&mut first, window, "23 43").await;
    assert_eq!(first.area, Size::new(90, 25));
}

#[tokio::test(flavor = "multi_thread")]
async fn opening_a_window_keeps_other_sizes_and_width_changes_resize() {
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
        .wait_for_window(second, |screen| screen.size() == Size::new(43, 28))
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
async fn closing_a_shell_removes_its_window() {
    let server = TestServer::start("close", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let second = client.open_after(first).await;
    client.act(SessionAction::CloseWindow(second)).await;
    client
        .wait_until(|client| client.windows() == vec![first])
        .await;
    assert!(!client.grids.contains_key(&second));
    assert!(!server.handle.is_finished());
}

#[tokio::test(flavor = "multi_thread")]
async fn closing_a_window_that_ignores_sighup_kills_it() {
    let server = TestServer::start("close-hup", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let second = client.open_after(first).await;
    client
        .type_line_to(second, "trap '' HUP; echo trapped; sleep 100")
        .await;
    client
        .wait_for_window(second, |screen| {
            screen.contents().lines().any(|line| line == "trapped")
        })
        .await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    let started = Instant::now();
    client.act(SessionAction::CloseWindow(second)).await;
    client
        .wait_until(|client| client.windows() == vec![first])
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
        .wait_until(|client| client.windows() == vec![first])
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
async fn stacked_windows_share_the_height() {
    let server = TestServer::start("stack", &["/bin/sh"]).await;
    let mut client = server.attach(80, 25).await;
    client.show_all().await;
    let first = client.first();
    let second = client.open_after(first).await;
    client.show_all().await;
    client
        .act(SessionAction::ConsumeOrExpel {
            window: second,
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
    let width = |client: &TestClient, window: WindowId| {
        let location = client.layout.locate(window).unwrap();
        client.layout.bands()[location.band].columns[location.column].width
    };
    assert_eq!(width(&client, first), Proportion::ONE_HALF);
    assert_eq!(width(&client, second), Proportion::ONE_THIRD);
}

#[tokio::test(flavor = "multi_thread")]
async fn float_a_window_for_every_client() {
    let server = TestServer::start("float-all", &["/bin/sh"]).await;
    let mut first = server.attach(80, 24).await;
    let mut second = server.attach(80, 24).await;
    let a = first.first();
    let b = first.open_after(a).await;
    first
        .act(SessionAction::ToggleFloating {
            window: b,
            after: None,
            floating: None,
        })
        .await;
    for client in [&mut first, &mut second] {
        client
            .wait_until(|client| client.layout.floating(b).is_some())
            .await;
        let floating: Vec<_> = client.layout.bands()[0]
            .floating
            .iter()
            .map(|floating| floating.window)
            .collect();
        assert_eq!(floating, [b]);
    }
}

fn toggle(window: WindowId, floating: Option<bool>) -> SessionAction {
    SessionAction::ToggleFloating {
        window,
        after: None,
        floating,
    }
}

fn floating_list(client: &TestClient) -> Vec<WindowId> {
    client.layout.bands()[0]
        .floating
        .iter()
        .map(|floating| floating.window)
        .collect()
}

async fn widen(
    sender: &mut TestClient,
    other: &mut TestClient,
    window: WindowId,
    width: Proportion,
) {
    sender.act(SessionAction::SetWidth { window, width }).await;
    for client in [sender, other] {
        client
            .wait_until(|client| {
                client.layout.locate(window).is_some_and(|location| {
                    client.layout.bands()[location.band].columns[location.column].width == width
                })
            })
            .await;
    }
}

async fn settle(first: &mut TestClient, second: &mut TestClient, tiled: WindowId) {
    widen(first, second, tiled, Proportion::ONE_THIRD).await;
    widen(second, first, tiled, Proportion::TWO_THIRDS).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn two_clients_float_the_same_window() {
    let server = TestServer::start("float-twice", &["/bin/sh"]).await;
    let mut first = server.attach(80, 24).await;
    let mut second = server.attach(80, 24).await;
    let a = first.first();
    let b = first.open_after(a).await;
    first.act(toggle(b, Some(true))).await;
    second.act(toggle(b, Some(true))).await;
    settle(&mut first, &mut second, a).await;
    for client in [&first, &second] {
        assert_eq!(floating_list(client), [b]);
        let record = client.layout.floating(b).unwrap();
        assert_eq!(
            (record.width, record.rows, record.col, record.row),
            (Proportion::ONE_HALF, 20, 20, 2)
        );
        assert!(client.layout.locate(b).is_none());
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn two_clients_toggle_the_same_window() {
    let server = TestServer::start("toggle-twice", &["/bin/sh"]).await;
    let mut first = server.attach(80, 24).await;
    let mut second = server.attach(80, 24).await;
    let a = first.first();
    let b = first.open_after(a).await;
    first.act(toggle(b, None)).await;
    second.act(toggle(b, None)).await;
    settle(&mut first, &mut second, a).await;
    for client in [&first, &second] {
        assert!(floating_list(client).is_empty());
        assert_eq!(
            client.layout.locate(b).map(|location| location.band),
            Some(0)
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn last_request_wins() {
    let server = TestServer::start("last-wins", &["/bin/sh"]).await;
    let mut first = server.attach(80, 24).await;
    let mut second = server.attach(80, 24).await;
    let a = first.first();
    let b = first.open_after(a).await;
    first.act(toggle(b, None)).await;
    second
        .wait_until(|client| client.layout.floating(b).is_some())
        .await;
    first.act(toggle(b, Some(false))).await;
    second
        .wait_until(|client| client.layout.locate(b).is_some())
        .await;
    second.act(toggle(b, Some(true))).await;
    for client in [&mut first, &mut second] {
        client
            .wait_until(|client| floating_list(client) == [b])
            .await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn open_a_floating_window_with_focus() {
    let server = TestServer::start("open-floating", &["/bin/sh"]).await;
    let mut requester = server.attach(80, 24).await;
    let mut other = server.attach(80, 24).await;
    let band = requester.layout.bands()[0].id;
    requester
        .act(SessionAction::OpenWindow {
            band,
            after: None,
            width: None,
            floating: true,
            focus: true,
            content: WindowContent::Program(None),
        })
        .await;
    requester
        .wait_until(|client| !client.focus.is_empty())
        .await;
    let opened = requester.focus[0];
    for client in [&mut requester, &mut other] {
        client
            .wait_until(|client| {
                client.layout.bands()[0]
                    .floating
                    .last()
                    .map(|floating| floating.window)
                    == Some(opened)
            })
            .await;
    }
    requester.wait_for_prompt(opened).await;
    stty_size(&mut requester, opened, "18 38").await;
    assert!(other.pump(Duration::from_millis(200)).await);
    assert!(other.focus.is_empty());
}

const READ_INPUT: &str =
    "stty raw -echo; printf ready; dd bs=1 count=9 2>/dev/null | od -An -tx1; sleep 100";

async fn send_mouse(
    client: &mut TestClient,
    window: WindowId,
    kind: MouseKind,
    col: u16,
    row: u16,
) {
    client
        .send(&ClientMessage::Mouse {
            window,
            event: MouseEvent::new(kind, col, row, Modifiers::NONE),
        })
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn mouse_press_reaches_a_mouse_program() {
    let script = format!("printf '\\033[?1000h\\033[?1006h'; {READ_INPUT}");
    let server = TestServer::start("mouse-press", &["/bin/sh", "-c", &script]).await;
    let mut client = server.attach(80, 24).await;
    let window = client.first();
    client.wait_for_text("ready").await;
    send_mouse(
        &mut client,
        window,
        MouseKind::Press(MouseButton::Left),
        4,
        2,
    )
    .await;
    client.wait_for_text(" 1b 5b 3c 30 3b 35 3b 33 4d").await;
}

#[tokio::test(flavor = "multi_thread")]
async fn mouse_without_reporting_writes_nothing() {
    let script =
        "stty raw -echo; printf ready; dd bs=1 count=1 2>/dev/null | od -An -tx1; sleep 100";
    let server = TestServer::start("mouse-none", &["/bin/sh", "-c", script]).await;
    let mut client = server.attach(80, 24).await;
    let window = client.first();
    client.wait_for_text("ready").await;
    send_mouse(
        &mut client,
        window,
        MouseKind::Wheel(WheelDirection::Up),
        0,
        0,
    )
    .await;
    client.key_to(window, Key::plain(KeyCode::Char('x'))).await;
    client.wait_for_text(" 78").await;
}

#[tokio::test(flavor = "multi_thread")]
async fn mouse_for_a_drawn_or_missing_window_is_dropped() {
    let server = TestServer::start("mouse-drawn", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let band = client.layout.bands()[0].id;
    client
        .act(SessionAction::OpenWindow {
            band,
            after: Some(first),
            width: None,
            floating: false,
            focus: false,
            content: WindowContent::Plugin { request: 1 },
        })
        .await;
    client
        .wait_until(|client| client.windows().len() == 2)
        .await;
    let drawn = client.windows()[1];
    client
        .send(&ClientMessage::Content {
            window: drawn,
            output: b"\x1b[?1000h\x1b[?1006hdrawn".to_vec(),
        })
        .await;
    client.wait_for_window_text(drawn, "drawn").await;
    send_mouse(
        &mut client,
        drawn,
        MouseKind::Press(MouseButton::Left),
        0,
        0,
    )
    .await;
    send_mouse(
        &mut client,
        WindowId(99),
        MouseKind::Press(MouseButton::Left),
        0,
        0,
    )
    .await;
    client.type_line_to(first, "echo still-serving").await;
    client.wait_for_window_text(first, "still-serving\n").await;
    assert_eq!(client.window_screen(drawn).contents().trim_end(), "drawn");
}
