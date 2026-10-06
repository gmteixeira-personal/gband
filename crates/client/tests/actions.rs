use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use gband_client::animation::Animations;
use gband_client::{Controls, Display, Step, dispatch};
use gband_core::action::{Action, SessionCommand};
use gband_core::geometry::Size;
use gband_core::input::Key;
use gband_core::layout::{
    BandId, Direction, Layout, LayoutOptions, Program, Proportion, SessionAction, Vertical,
    WindowContent, WindowHeight, WindowId,
};
use gband_core::view::ViewAction;
use gband_lua::keys::parse_key;
use gband_lua::{Config, ConfigError, DEFAULTS, LoadOptions, Locations};
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
async fn close_window_action_closes_the_focused_second_window() {
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
        Action::Session(SessionCommand::OpenWindow),
    )
    .await;
    receive_until(&mut client, &mut display, |client, display| {
        client.windows().len() == 2 && display.focused() != Some(first)
    })
    .await;
    let second = display.focused().unwrap();
    assert_eq!(client.windows(), [first, second]);

    run(
        &mut client,
        &mut display,
        Action::Session(SessionCommand::CloseWindow),
    )
    .await;
    receive_until(&mut client, &mut display, |client, _| {
        client.windows() == [first]
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
async fn shown_windows_follow_the_view_and_are_not_repeated() {
    let server = TestServer::start("client-shown", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let a = client.first();
    let b = client.open_after(a).await;
    let c = client.open_after(b).await;
    let d = client.open_after(c).await;
    client
        .act(SessionAction::ConsumeOrExpel {
            window: d,
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
        let path = gband_lua::user_file(&self.0, gband_lua::Side::Client);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, source).unwrap();
        let locations = Locations {
            config: self.0.clone(),
            plugins: None,
        };
        gband_lua::load(&locations, gband_lua::Side::Client, &LoadOptions::default())
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn three_columns() -> (Display, Vec<WindowId>) {
    let mut layout = Layout::new();
    let band = layout.bands()[0].id;
    let mut windows = Vec::new();
    for _ in 0..3 {
        let window = layout.allocate_window();
        layout.open(
            window,
            band,
            windows.last().copied(),
            None,
            &LayoutOptions::default(),
        );
        windows.push(window);
    }
    let mut display = Display::new(Size::new(80, 24), Animations::Off);
    display.apply(ServerMessage::Layout {
        cols: 80,
        rows: 24,
        layout,
    });
    (display, windows)
}

#[test]
fn new_actions_resolve_against_the_view() {
    let mut layout = Layout::new();
    let band = layout.bands()[0].id;
    let windows: Vec<WindowId> = (0..2).map(|_| layout.allocate_window()).collect();
    layout.open(windows[0], band, None, None, &LayoutOptions::default());
    layout.open(
        windows[1],
        band,
        Some(windows[0]),
        None,
        &LayoutOptions::default(),
    );
    layout.apply(
        SessionAction::ToggleFloating {
            window: windows[1],
            after: None,
        },
        Size::new(80, 24),
        &LayoutOptions::default(),
    );
    let mut display = Display::new(Size::new(80, 24), Animations::Off);
    display.apply(ServerMessage::Layout {
        cols: 80,
        rows: 24,
        layout,
    });
    assert_eq!(display.focused(), Some(windows[0]));
    let sent = |display: &mut Display, command: SessionCommand| {
        dispatch(display, Action::Session(command))
    };
    assert_eq!(
        sent(&mut display, SessionCommand::MoveColumn(Direction::Right)),
        Step::Send(ClientMessage::Action(SessionAction::MoveColumn {
            window: windows[0],
            direction: Direction::Right,
        }))
    );
    assert_eq!(
        dispatch(&mut display, Action::View(ViewAction::SwitchLayer)),
        Step::Nothing
    );
    assert_eq!(display.focused(), Some(windows[1]));
    assert_eq!(display.view_state("root").column, None);
    assert_eq!(
        sent(&mut display, SessionCommand::MoveWindow(Vertical::Down)),
        Step::Send(ClientMessage::Action(SessionAction::MoveWindow {
            window: windows[1],
            direction: Vertical::Down,
        }))
    );
    assert_eq!(
        sent(&mut display, SessionCommand::ToggleFloating),
        Step::Send(ClientMessage::Action(SessionAction::ToggleFloating {
            window: windows[1],
            after: Some(windows[0]),
        }))
    );
    assert_eq!(
        sent(&mut display, SessionCommand::OpenWindow),
        Step::Send(ClientMessage::Action(SessionAction::open(
            band,
            Some(windows[0]),
            None
        )))
    );
    assert_eq!(
        sent(&mut display, SessionCommand::CloseWindow),
        Step::Send(ClientMessage::Action(SessionAction::CloseWindow(
            windows[1]
        )))
    );
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
    let (mut display, windows) = three_columns();
    let mut controls = Controls::new(config, &mut display);
    assert_eq!(display.focused(), Some(windows[0]));
    assert_eq!(
        controls.press(&mut display, key("alt+w")),
        [Step::Nothing, Step::Nothing]
    );
    assert_eq!(display.focused(), Some(windows[2]));
}

#[test]
fn focus_goes_round_the_band_by_default() {
    let config = Scratch::new("loop-bands-on").load("").unwrap();
    let (mut display, windows) = three_columns();
    let _controls = Controls::new(config, &mut display);
    for _ in 0..3 {
        dispatch(&mut display, Action::View(ViewAction::FocusRight));
    }
    assert_eq!(display.focused(), Some(windows[0]));
}

#[test]
fn loop_bands_off_stops_focus_at_the_last_column() {
    let config = Scratch::new("loop-bands-off")
        .load("gband.set { loop_bands = false }")
        .unwrap();
    let (mut display, windows) = three_columns();
    let _controls = Controls::new(config, &mut display);
    for _ in 0..3 {
        dispatch(&mut display, Action::View(ViewAction::FocusRight));
    }
    assert_eq!(display.focused(), Some(windows[2]));
}

#[test]
fn center_column_binding_moves_the_camera_locally() {
    let scratch = Scratch::new("center");
    let config = scratch
        .load("gband.bind('alt+c', gband.action.center_column)")
        .unwrap();
    let (mut display, windows) = three_columns();
    let mut controls = Controls::new(config, &mut display);
    let camera = |display: &mut Display| display.present(Instant::now()).unwrap().bands[0].camera;
    for _ in 0..2 {
        dispatch(&mut display, Action::View(ViewAction::FocusRight));
    }
    assert_eq!(camera(&mut display), 40);
    assert_eq!(controls.press(&mut display, key("alt+c")), [Step::Nothing]);
    assert_eq!(camera(&mut display), 60);
    assert_eq!(display.focused(), Some(windows[2]));
}

#[test]
fn center_column_on_a_floating_window_sends_its_centred_position() {
    let mut layout = Layout::new();
    let band = layout.bands()[0].id;
    let tiled = layout.allocate_window();
    let floating = layout.allocate_window();
    layout.open(tiled, band, None, None, &LayoutOptions::default());
    layout.open(floating, band, Some(tiled), None, &LayoutOptions::default());
    for action in [
        SessionAction::ToggleFloating {
            window: floating,
            after: None,
        },
        SessionAction::SetPosition {
            window: floating,
            col: 0,
            row: 0,
        },
    ] {
        layout.apply(action, Size::new(80, 24), &LayoutOptions::default());
    }
    let mut display = Display::new(Size::new(80, 24), Animations::Off);
    display.apply(ServerMessage::Layout {
        cols: 80,
        rows: 24,
        layout,
    });
    dispatch(
        &mut display,
        Action::View(ViewAction::FocusWindow(floating)),
    );
    let camera = display.present(Instant::now()).unwrap().bands[0].camera;
    assert_eq!(
        dispatch(&mut display, Action::View(ViewAction::CenterColumn)),
        Step::Send(ClientMessage::Action(SessionAction::SetPosition {
            window: floating,
            col: 20,
            row: 2,
        }))
    );
    assert_eq!(
        display.present(Instant::now()).unwrap().bands[0].camera,
        camera
    );
    assert_eq!(display.focused(), Some(floating));
}

#[test]
fn spawn_a_command_line_sends_open_window_with_the_program() {
    let scratch = Scratch::new("spawn");
    let config = scratch
        .load("gband.bind('alt+n', function() gband.spawn({ cmd = 'fish' }) end)")
        .unwrap();
    let (mut display, windows) = three_columns();
    let mut controls = Controls::new(config, &mut display);
    assert_eq!(
        controls.press(&mut display, key("alt+n")),
        [Step::Send(ClientMessage::Action(SessionAction::open(
            BandId(1),
            Some(windows[0]),
            Some(Program::CommandLine("fish".to_owned()))
        )))]
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
    let (mut display, windows) = three_columns();
    let mut controls = Controls::new(config, &mut display);
    for _ in 0..2 {
        dispatch(&mut display, Action::View(ViewAction::FocusRight));
    }
    assert_eq!(display.focused(), Some(windows[2]));
    controls.press(&mut display, key("alt+e"));
    assert_eq!(display.focused(), Some(windows[1]));
    let banner = display.banner().unwrap();
    let expected = format!(
        "{}:9: broken",
        gband_lua::user_file(&scratch.0, gband_lua::Side::Client).display()
    );
    assert_eq!(banner, expected);
}

#[test]
fn send_prefix_follows_the_prefix_option() {
    let scratch = Scratch::new("prefix");
    let config = scratch
        .load(&format!("{DEFAULTS}\ngband.set {{ prefix = 'ctrl+b' }}"))
        .unwrap();
    let (mut display, windows) = three_columns();
    let mut controls = Controls::new(config, &mut display);
    assert_eq!(controls.press(&mut display, key("ctrl+b")), []);
    assert_eq!(
        controls.press(&mut display, key("ctrl+b")),
        [Step::Send(ClientMessage::Key {
            window: windows[0],
            key: key("ctrl+b"),
        })]
    );
    assert_eq!(controls.active_table(), "root");
    assert_eq!(
        controls.press(&mut display, key("ctrl+space")),
        [Step::Send(ClientMessage::Key {
            window: windows[0],
            key: key("ctrl+space"),
        })]
    );
}

#[test]
fn navigation_mode_opens_a_window_with_n_and_leaves_with_enter() {
    let scratch = Scratch::new("navigation");
    let (mut display, windows) = three_columns();
    let mut controls = Controls::new(scratch.load(DEFAULTS).unwrap(), &mut display);
    controls.press(&mut display, key("ctrl+space"));
    assert_eq!(controls.press(&mut display, key("enter")), []);
    assert_eq!(controls.active_table(), "root");
    controls.press(&mut display, key("ctrl+space"));
    let steps = controls.press(&mut display, key("n"));
    assert!(
        matches!(
            steps.as_slice(),
            [Step::Send(ClientMessage::Action(
                SessionAction::OpenWindow { .. }
            ))]
        ),
        "{steps:?}"
    );
    assert_eq!(controls.active_table(), "root");
    assert_eq!(
        controls.press(&mut display, key("x")),
        [Step::Send(ClientMessage::Key {
            window: windows[0],
            key: key("x"),
        })]
    );
}

#[test]
fn reload_replaces_the_bindings_and_ends_a_prefix_sequence() {
    let scratch = Scratch::new("reload");
    let (mut display, windows) = three_columns();
    let mut controls = Controls::new(scratch.load(DEFAULTS).unwrap(), &mut display);
    assert_eq!(controls.press(&mut display, key("ctrl+space")), []);
    controls.reload(
        &mut display,
        scratch.load("gband.bind('alt+l', gband.action.focus_column_right)"),
    );
    assert_eq!(
        controls.press(&mut display, key("q")),
        [Step::Send(ClientMessage::Key {
            window: windows[0],
            key: key("q"),
        })]
    );
    controls.press(&mut display, key("alt+l"));
    assert_eq!(display.focused(), Some(windows[1]));
}

#[test]
fn failed_reload_keeps_the_running_configuration_until_a_good_one() {
    let scratch = Scratch::new("broken");
    let (mut display, windows) = three_columns();
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
    assert_eq!(display.focused(), Some(windows[1]));
    assert_eq!(
        controls.press(&mut display, key("alt+j")),
        [Step::Send(ClientMessage::Key {
            window: windows[1],
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
        Some(ClientMessage::Shown(vec![
            WindowId(1),
            WindowId(2),
            WindowId(3)
        ]))
    );
}

fn bound(name: &str, source: &str) -> (Scratch, Display, Controls, Vec<WindowId>) {
    let scratch = Scratch::new(name);
    let config = scratch
        .load(&format!("gband.bind('alt+x', function()\n{source}\nend)"))
        .unwrap();
    let (mut display, windows) = three_columns();
    let controls = Controls::new(config, &mut display);
    (scratch, display, controls, windows)
}

#[test]
fn targeted_session_actions_send_their_target() {
    let (_scratch, mut display, mut controls, windows) = bound(
        "targeted",
        "gband.action.close_window({ window = 3 })\ngband.window.set_width(2, 0.4)\ngband.window.set_height(1, { rows = 8 })",
    );
    assert_eq!(
        controls.press(&mut display, key("alt+x")),
        [
            Step::Send(ClientMessage::Action(SessionAction::CloseWindow(
                windows[2]
            ))),
            Step::Send(ClientMessage::Action(SessionAction::SetWidth {
                window: windows[1],
                width: Proportion::new(2, 5),
            })),
            Step::Send(ClientMessage::Action(SessionAction::SetHeight {
                window: windows[0],
                height: WindowHeight::Fixed(8),
            })),
        ]
    );
    assert_eq!(display.focused(), Some(windows[0]));
}

#[test]
fn input_to_a_named_window_is_sent_as_keys_and_pastes() {
    let (_scratch, mut display, mut controls, windows) = bound(
        "input",
        "gband.window.send_text(2, 'a\\n')\ngband.window.paste(3, 'b c')",
    );
    assert_eq!(
        controls.press(&mut display, key("alt+x")),
        [
            Step::Send(ClientMessage::Key {
                window: windows[1],
                key: key("a"),
            }),
            Step::Send(ClientMessage::Key {
                window: windows[1],
                key: key("enter"),
            }),
            Step::Send(ClientMessage::Paste {
                window: windows[2],
                text: "b c".to_owned(),
            }),
        ]
    );
    assert_eq!(controls.active_table(), "root");
}

#[test]
fn focus_and_view_by_number_change_the_view() {
    let (_scratch, mut display, mut controls, windows) = bound(
        "focus-view",
        "gband.window.focus(3)\ngband.band.view(2)\ngband.band.view(1)",
    );
    assert_eq!(
        controls.press(&mut display, key("alt+x")),
        [Step::Nothing, Step::Nothing, Step::Nothing]
    );
    assert_eq!(display.focused(), Some(windows[2]));
}

#[test]
fn tiled_plugin_window_sends_open_window_with_plugin_content() {
    let (_scratch, mut display, mut controls, windows) = bound(
        "tiled-plugin-window",
        "win = gband.win.open({ kind = 'tiled', after = 2, focus = false })",
    );
    let steps = controls.press(&mut display, key("alt+x"));
    let plugin_window: u32 = controls.runtime().lua().globals().get("win").unwrap();
    assert_eq!(
        steps,
        [Step::Send(ClientMessage::Action(
            SessionAction::OpenWindow {
                band: BandId(1),
                after: Some(windows[1]),
                width: None,
                floating: false,
                focus: false,
                content: WindowContent::Plugin {
                    request: plugin_window
                },
            }
        ))]
    );
}
