use std::fs;
use std::path::PathBuf;

use gband_client::animation::Animations;
use gband_client::{Controls, Display, Step, dispatch};
use gband_core::action::{Action, SessionCommand};
use gband_core::geometry::Size;
use gband_core::input::Key;
use gband_core::layout::{
    BandId, Direction, Layout, LayoutOptions, PaneId, Program, SessionAction,
};
use gband_core::view::ViewAction;
use gband_lua::keys::parse_key;
use gband_lua::{Config, ConfigError, DEFAULTS};
use gband_protocol::{ClientMessage, ServerMessage};
use gband_test_support::{TIMEOUT, TestClient, TestServer};
use tokio::time::timeout;

async fn run(client: &mut TestClient, display: &mut Display, action: Action) {
    let Step::Send(message) = dispatch(display, action) else {
        panic!("{action:?} sent nothing");
    };
    client.send(&message).await;
}

async fn receive_until(
    client: &mut TestClient,
    display: &mut Display,
    done: impl Fn(&TestClient, &Display) -> bool,
) {
    timeout(TIMEOUT, async {
        while !done(client, display) {
            let message = client.receive().await.expect("the server closed");
            display.apply(message);
        }
    })
    .await
    .expect("timed out waiting for the server");
}

#[tokio::test(flavor = "multi_thread")]
async fn close_pane_action_closes_the_focused_second_pane() {
    let server = TestServer::start("client-actions", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    let mut display = Display::new(Size::new(80, 24), Animations::On);
    display.apply(ServerMessage::Layout {
        cols: client.area.cols,
        rows: client.area.rows,
        layout: client.layout.clone(),
    });
    assert_eq!(display.focused(), Some(first));

    run(
        &mut client,
        &mut display,
        Action::Session(SessionCommand::OpenPane),
    )
    .await;
    receive_until(&mut client, &mut display, |client, display| {
        client.panes().len() == 2 && display.focused() != Some(first)
    })
    .await;
    let second = display.focused().unwrap();
    assert_eq!(client.panes(), [first, second]);

    run(
        &mut client,
        &mut display,
        Action::Session(SessionCommand::ClosePane),
    )
    .await;
    receive_until(&mut client, &mut display, |client, _| {
        client.panes() == [first]
    })
    .await;
    assert_eq!(display.focused(), Some(first));
}

async fn report(
    client: &mut TestClient,
    display: &mut Display,
    action: Option<ViewAction>,
) -> Option<ClientMessage> {
    if let Some(action) = action {
        assert!(matches!(
            dispatch(display, Action::View(action)),
            Step::Nothing
        ));
    }
    let message = display.report_shown();
    if let Some(message) = &message {
        client.send(message).await;
    }
    message
}

#[tokio::test(flavor = "multi_thread")]
async fn shown_panes_follow_the_view_and_are_not_repeated() {
    let server = TestServer::start("client-shown", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let a = client.first();
    let b = client.open_after(a).await;
    let c = client.open_after(b).await;
    let d = client.open_after(c).await;
    client
        .act(SessionAction::ConsumeOrExpel {
            pane: d,
            direction: Direction::Left,
        })
        .await;
    client
        .wait_until(|client| client.layout.bands()[0].columns.len() == 3)
        .await;
    let mut display = Display::new(Size::new(80, 24), Animations::On);
    assert_eq!(display.report_shown(), None);
    display.apply(ServerMessage::Layout {
        cols: client.area.cols,
        rows: client.area.rows,
        layout: client.layout.clone(),
    });

    assert_eq!(
        report(&mut client, &mut display, None).await,
        Some(ClientMessage::Shown(vec![a, b]))
    );
    assert_eq!(report(&mut client, &mut display, None).await, None);
    let right = Some(ViewAction::FocusRight);
    assert_eq!(report(&mut client, &mut display, right).await, None);
    assert_eq!(
        report(&mut client, &mut display, right).await,
        Some(ClientMessage::Shown(vec![b, c, d]))
    );
    assert_eq!(display.focused(), Some(c));
    let down = Some(ViewAction::FocusDown);
    assert_eq!(report(&mut client, &mut display, down).await, None);
    assert_eq!(display.focused(), Some(d));
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "gband-client-actions-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn load(&self, source: &str) -> Result<Config, ConfigError> {
        let path = gband_lua::user_file(&self.0);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, source).unwrap();
        gband_lua::load(&self.0)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn three_columns() -> (Display, Vec<PaneId>) {
    let mut layout = Layout::new();
    let band = layout.bands()[0].id;
    let mut panes = Vec::new();
    for _ in 0..3 {
        let pane = layout.allocate_pane();
        layout.open(pane, band, panes.last().copied(), &LayoutOptions::default());
        panes.push(pane);
    }
    let mut display = Display::new(Size::new(80, 24), Animations::Off);
    display.apply(ServerMessage::Layout {
        cols: 80,
        rows: 24,
        layout,
    });
    (display, panes)
}

fn key(name: &str) -> Key {
    parse_key(name).unwrap()
}

#[test]
fn action_called_from_a_function() {
    let scratch = Scratch::new("function");
    let config = scratch
        .load(
            "gband.bind('alt+w', function()\n  gband.action.focus_column_right()\n  gband.action.focus_column_right()\nend)",
        )
        .unwrap();
    let (mut display, panes) = three_columns();
    let mut controls = Controls::new(config, &mut display);
    assert_eq!(display.focused(), Some(panes[0]));
    assert_eq!(
        controls.press(&mut display, key("alt+w")),
        [Step::Nothing, Step::Nothing]
    );
    assert_eq!(display.focused(), Some(panes[2]));
}

#[test]
fn spawn_a_command_line_sends_open_pane_with_the_program() {
    let scratch = Scratch::new("spawn");
    let config = scratch
        .load("gband.bind('alt+n', function() gband.spawn({ cmd = 'fish' }) end)")
        .unwrap();
    let (mut display, panes) = three_columns();
    let mut controls = Controls::new(config, &mut display);
    assert_eq!(
        controls.press(&mut display, key("alt+n")),
        [Step::Send(ClientMessage::Action(SessionAction::OpenPane {
            band: BandId(1),
            after: Some(panes[0]),
            program: Some(Program::CommandLine("fish".to_owned())),
        }))]
    );
}

#[test]
fn error_in_a_binding_function_keeps_the_dispatched_actions() {
    let scratch = Scratch::new("error");
    let config = scratch
        .load(
            "gband.bind('alt+e', function()\n  gband.action.focus_column_left()\n\n\n\n\n\n\n  error('broken')\nend)",
        )
        .unwrap();
    let (mut display, panes) = three_columns();
    let mut controls = Controls::new(config, &mut display);
    for _ in 0..2 {
        dispatch(&mut display, Action::View(ViewAction::FocusRight));
    }
    assert_eq!(display.focused(), Some(panes[2]));
    controls.press(&mut display, key("alt+e"));
    assert_eq!(display.focused(), Some(panes[1]));
    let banner = display.banner().unwrap();
    let expected = format!("{}:9: broken", gband_lua::user_file(&scratch.0).display());
    assert_eq!(banner, expected);
}

#[test]
fn send_prefix_follows_the_prefix_option() {
    let scratch = Scratch::new("prefix");
    let config = scratch
        .load(&format!("{DEFAULTS}\ngband.set {{ prefix = 'ctrl+b' }}"))
        .unwrap();
    let (mut display, panes) = three_columns();
    let mut controls = Controls::new(config, &mut display);
    assert_eq!(controls.press(&mut display, key("ctrl+b")), []);
    assert_eq!(
        controls.press(&mut display, key("ctrl+b")),
        [Step::Send(ClientMessage::Key {
            pane: panes[0],
            key: key("ctrl+b"),
        })]
    );
    assert_eq!(
        controls.press(&mut display, key("ctrl+space")),
        [Step::Send(ClientMessage::Key {
            pane: panes[0],
            key: key("ctrl+space"),
        })]
    );
}

#[test]
fn reload_replaces_the_bindings_and_ends_a_prefix_sequence() {
    let scratch = Scratch::new("reload");
    let (mut display, panes) = three_columns();
    let mut controls = Controls::new(scratch.load(DEFAULTS).unwrap(), &mut display);
    assert_eq!(controls.press(&mut display, key("ctrl+space")), []);
    controls.reload(
        &mut display,
        scratch.load("gband.bind('alt+l', gband.action.focus_column_right)"),
    );
    assert_eq!(
        controls.press(&mut display, key("q")),
        [Step::Send(ClientMessage::Key {
            pane: panes[0],
            key: key("q"),
        })]
    );
    controls.press(&mut display, key("alt+l"));
    assert_eq!(display.focused(), Some(panes[1]));
}

#[test]
fn failed_reload_keeps_the_running_configuration_until_a_good_one() {
    let scratch = Scratch::new("broken");
    let (mut display, panes) = three_columns();
    let mut controls = Controls::new(
        scratch
            .load("gband.bind('alt+l', gband.action.focus_column_right)")
            .unwrap(),
        &mut display,
    );
    controls.reload(
        &mut display,
        scratch.load("gband.bind('alt+j', gband.action.focus_column_left)\nlocal = 1"),
    );
    let banner = display.banner().unwrap().to_owned();
    assert!(banner.contains("init.lua:2:"), "{banner}");
    controls.press(&mut display, key("alt+l"));
    assert_eq!(display.focused(), Some(panes[1]));
    assert_eq!(
        controls.press(&mut display, key("alt+j")),
        [Step::Send(ClientMessage::Key {
            pane: panes[1],
            key: key("alt+j"),
        })]
    );
    controls.reload(&mut display, scratch.load(""));
    assert_eq!(display.banner(), None);
}

#[test]
fn camera_policy_follows_the_configuration() {
    let scratch = Scratch::new("camera");
    let (mut display, _) = three_columns();
    let mut controls = Controls::new(
        scratch
            .load(&format!(
                "{DEFAULTS}\ngband.set {{ center_focused_column = 'always' }}"
            ))
            .unwrap(),
        &mut display,
    );
    controls.press(&mut display, key("ctrl+space"));
    controls.press(&mut display, key("l"));
    let shown = display.report_shown();
    assert_eq!(
        shown,
        Some(ClientMessage::Shown(vec![PaneId(1), PaneId(2), PaneId(3)]))
    );
}
