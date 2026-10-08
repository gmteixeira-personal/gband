use std::fs;
use std::time::{Duration, Instant};

use gband_client::animation::Animations;
use gband_client::mouse::{Edges, Target, pick_edges};
use gband_client::{Controls, Display, Step, dispatch};
use gband_core::action::{Action, SessionCommand};
use gband_core::geometry::Size;
use gband_core::geometry::tiles;
use gband_core::input::{Key, Modifiers, MouseButton, MouseEvent, MouseKind, WheelDirection};
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

struct Scratch(gband_scratch::Scratch);

impl Scratch {
    fn new(name: &str) -> Self {
        Self(gband_scratch::Scratch::new(
            "client",
            &format!("actions-{name}"),
        ))
    }

    fn load(&self, source: &str) -> Result<Config, ConfigError> {
        let path = gband_lua::user_file(&self.0, gband_lua::Side::Client);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, source).unwrap();
        let locations = Locations {
            config: self.0.to_path_buf(),
            plugins: None,
        };
        gband_lua::load(&locations, gband_lua::Side::Client, &LoadOptions::default())
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
            floating: None,
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
            floating: None,
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
            floating: None,
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
    controls.place_bars(&mut display);
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
    assert_eq!(controls.active_table(), "prefix");
    assert_eq!(controls.press(&mut display, key("enter")), []);
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
    controls.place_bars(&mut display);
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
fn step_from_the_clients_option() {
    let scratch = Scratch::new("step-option");
    let config = scratch
        .load(
            "gband.opt.width_step = 1/20\ngband.keymap.set('root', 'alt+g', gband.action.grow_column_width)\ngband.bind('alt+h', function() gband.action.shrink_window_height({ step = 1/4 }) end)\ngband.bind('alt+w', function() gband.action.grow_column_width() end)",
        )
        .unwrap();
    let (mut display, windows) = three_columns();
    let mut controls = Controls::new(config, &mut display);
    dispatch(&mut display, Action::View(ViewAction::FocusRight));
    dispatch(&mut display, Action::View(ViewAction::FocusRight));
    assert_eq!(display.focused(), Some(windows[2]));
    let grow = || {
        Step::Send(ClientMessage::Action(SessionAction::StepWidth {
            window: windows[2],
            step: gband_core::layout::Step::Grow,
            by: Proportion::new(1, 20),
        }))
    };
    assert_eq!(controls.press(&mut display, key("alt+g")), [grow()]);
    assert_eq!(controls.press(&mut display, key("alt+w")), [grow()]);
    assert_eq!(
        controls.press(&mut display, key("alt+h")),
        [Step::Send(ClientMessage::Action(
            SessionAction::StepHeight {
                window: windows[2],
                step: gband_core::layout::Step::Shrink,
                by: Proportion::new(1, 4),
            }
        ))]
    );
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

const AREA: Size = Size::new(80, 24);

fn columns_of(count: usize, width: Option<Proportion>) -> (Layout, Vec<WindowId>) {
    let mut layout = Layout::new();
    let band = layout.bands()[0].id;
    let mut windows = Vec::new();
    for _ in 0..count {
        let window = layout.allocate_window();
        layout.open(
            window,
            band,
            windows.last().copied(),
            width,
            &LayoutOptions::default(),
        );
        windows.push(window);
    }
    (layout, windows)
}

fn float(layout: &mut Layout, window: WindowId, width: Proportion, rows: u16, col: u16, row: u16) {
    for action in [
        SessionAction::ToggleFloating {
            window,
            after: None,
            floating: None,
        },
        SessionAction::SetWidth { window, width },
        SessionAction::SetHeight {
            window,
            height: WindowHeight::Fixed(rows),
        },
        SessionAction::SetPosition { window, col, row },
    ] {
        layout.apply(action, AREA, &LayoutOptions::default());
    }
}

struct Mouse {
    _scratch: Scratch,
    display: Display,
    controls: Controls,
    layout: Layout,
    now: Instant,
}

impl Mouse {
    fn new(name: &str, source: &str, layout: Layout) -> Self {
        Self::with(name, source, layout, Animations::Off)
    }

    fn with(name: &str, source: &str, layout: Layout, animations: Animations) -> Self {
        let scratch = Scratch::new(name);
        let config = scratch.load(source).unwrap();
        let mut display = Display::new(AREA, animations);
        display.apply(ServerMessage::Layout {
            cols: AREA.cols,
            rows: AREA.rows,
            layout: layout.clone(),
        });
        let controls = Controls::new(config, &mut display);
        Self {
            _scratch: scratch,
            display,
            controls,
            layout,
            now: Instant::now(),
        }
    }

    fn event(&mut self, kind: MouseKind, col: u16, row: u16, modifiers: Modifiers) -> Vec<Step> {
        self.now += Duration::from_millis(20);
        let event = MouseEvent::new(kind, col, row, modifiers);
        self.controls.mouse(&mut self.display, event, self.now)
    }

    fn drag(&mut self, button: MouseButton, from: (u16, u16), to: (u16, u16)) -> Vec<Step> {
        self.drag_with(button, from, to, Modifiers::NONE)
    }

    fn drag_with(
        &mut self,
        button: MouseButton,
        from: (u16, u16),
        to: (u16, u16),
        modifiers: Modifiers,
    ) -> Vec<Step> {
        let mut steps = self.event(MouseKind::Press(button), from.0, from.1, modifiers);
        steps.extend(self.event(MouseKind::Motion(Some(button)), to.0, to.1, modifiers));
        steps.extend(self.event(MouseKind::Release(button), to.0, to.1, modifiers));
        self.now += Duration::from_millis(20);
        steps.extend(self.controls.flush(&mut self.display, self.now));
        steps
    }

    fn key(&mut self, name: &str) -> Vec<Step> {
        self.controls.press(&mut self.display, key(name))
    }

    fn apply(&mut self, steps: &[Step]) {
        for action in sent(steps) {
            self.layout.apply(action, AREA, &LayoutOptions::default());
        }
        self.display.apply(ServerMessage::Layout {
            cols: AREA.cols,
            rows: AREA.rows,
            layout: self.layout.clone(),
        });
    }

    fn global<T: mlua::FromLua>(&self, name: &str) -> T {
        self.controls.runtime().lua().globals().get(name).unwrap()
    }
}

fn sent(steps: &[Step]) -> Vec<SessionAction> {
    steps
        .iter()
        .filter_map(|step| match step {
            Step::Send(ClientMessage::Action(action)) => Some(action.clone()),
            _ => None,
        })
        .collect()
}

fn messages(steps: &[Step]) -> Vec<&ClientMessage> {
    steps
        .iter()
        .filter_map(|step| match step {
            Step::Send(message) => Some(message),
            _ => None,
        })
        .collect()
}

const NAVIGATION: &str = "gband.keymap.mode('prefix')\ngband.keymap.set('prefix', 'escape', function() gband.keymap.enter('root') end)\n";

#[test]
fn mouse_binding_in_root_replaces_the_default() {
    let (layout, windows) = columns_of(2, None);
    let mut mouse = Mouse::new(
        "mouse-root",
        "gband.bind('leftmouse', function(e) got = e.button .. ' ' .. e.window end)",
        layout,
    );
    assert_eq!(mouse.display.focused(), Some(windows[0]));
    let mut steps = mouse.event(MouseKind::Press(MouseButton::Left), 45, 3, Modifiers::NONE);
    steps.extend(mouse.event(
        MouseKind::Release(MouseButton::Left),
        45,
        3,
        Modifiers::NONE,
    ));
    assert_eq!(
        mouse.global::<String>("got"),
        format!("left {}", windows[1].0)
    );
    assert_eq!(mouse.display.focused(), Some(windows[0]));
    assert!(messages(&steps).is_empty(), "{steps:?}");
}

impl Mouse {
    fn click(&mut self, button: MouseButton, cell: (u16, u16)) -> Vec<Step> {
        let mut steps = self.event(MouseKind::Press(button), cell.0, cell.1, Modifiers::NONE);
        steps.extend(self.event(MouseKind::Release(button), cell.0, cell.1, Modifiers::NONE));
        steps
    }

    fn copy_ls(&mut self, window: WindowId) {
        self.display.apply(ServerMessage::Snapshot {
            window,
            cols: 38,
            rows: 22,
            contents: b"ls".to_vec(),
        });
        self.drag(MouseButton::Left, (1, 1), (2, 1));
        assert_eq!(self.display.copy_buffer(), "ls");
    }
}

fn pasted(steps: &[Step]) -> Vec<(WindowId, &str)> {
    steps
        .iter()
        .filter_map(|step| match step {
            Step::Send(ClientMessage::Paste { window, text }) => Some((*window, text.as_str())),
            _ => None,
        })
        .collect()
}

#[test]
fn declined_click_reaches_the_program() {
    let (layout, windows) = columns_of(2, None);
    let mut mouse = Mouse::new(
        "declined-click",
        "gband.bind('leftmouse', function() calls = (calls or 0) + 1 return false end)",
        layout,
    );
    mouse.report_mouse(windows[1]);
    let steps = mouse.click(MouseButton::Left, (45, 3));
    assert_eq!(mouse.global::<i64>("calls"), 1);
    assert_eq!(mouse.display.focused(), Some(windows[1]));
    assert_eq!(
        mouse_messages(&steps),
        [
            (
                windows[1],
                MouseEvent::new(MouseKind::Press(MouseButton::Left), 4, 2, Modifiers::NONE)
            ),
            (
                windows[1],
                MouseEvent::new(MouseKind::Release(MouseButton::Left), 4, 2, Modifiers::NONE)
            ),
        ]
    );
}

#[test]
fn border_taken_content_declined() {
    let (layout, windows) = columns_of(2, None);
    let mut mouse = Mouse::new(
        "declined-content",
        "gband.bind('rightmouse', function(e)
  if e.content_col ~= nil then return false end
  calls = (calls or 0) + 1
end)",
        layout,
    );
    mouse.copy_ls(windows[0]);
    let steps = mouse.click(MouseButton::Right, (50, 0));
    assert_eq!(mouse.global::<i64>("calls"), 1);
    assert_eq!(mouse.display.focused(), Some(windows[0]));
    assert!(messages(&steps).is_empty(), "{steps:?}");
    let steps = mouse.click(MouseButton::Right, (45, 3));
    assert_eq!(mouse.global::<i64>("calls"), 1);
    assert_eq!(mouse.display.focused(), Some(windows[1]));
    assert_eq!(pasted(&steps), [(windows[1], "ls")]);
}

#[test]
fn declined_right_click_pastes() {
    let (layout, windows) = columns_of(2, None);
    let mut mouse = Mouse::new(
        "declined-paste",
        "gband.bind('rightmouse', function() return false end)",
        layout,
    );
    mouse.copy_ls(windows[0]);
    let steps = mouse.click(MouseButton::Right, (45, 3));
    assert_eq!(mouse.display.focused(), Some(windows[1]));
    assert_eq!(pasted(&steps), [(windows[1], "ls")]);
}

#[test]
fn actions_of_a_declining_function_stand() {
    let (layout, windows) = columns_of(2, None);
    let mut mouse = Mouse::new(
        "declined-actions",
        &format!(
            "{NAVIGATION}gband.bind('leftmouse', function() gband.keymap.enter('prefix') return false end)"
        ),
        layout,
    );
    mouse.event(MouseKind::Press(MouseButton::Left), 45, 3, Modifiers::NONE);
    mouse.event(motion(MouseButton::Left), 47, 3, Modifiers::NONE);
    assert_eq!(mouse.display.focused(), Some(windows[1]));
    let selected = mouse.display.selection().unwrap();
    assert_eq!(
        (selected.window, selected.start, selected.end),
        (windows[1], (4, 2), (6, 2))
    );
    assert_eq!(mouse.controls.active_table(), "prefix");
}

#[test]
fn declined_in_a_mode() {
    let (layout, windows) = columns_of(2, None);
    let mut mouse = Mouse::new(
        "declined-mode",
        &format!(
            "{NAVIGATION}gband.keymap.set('prefix', 'leftmouse', function() return false end)"
        ),
        layout,
    );
    mouse.report_mouse(windows[1]);
    mouse.key("ctrl+space");
    let steps = mouse.click(MouseButton::Left, (45, 3));
    assert_eq!(mouse.display.focused(), Some(windows[0]));
    assert!(messages(&steps).is_empty(), "{steps:?}");
    assert_eq!(mouse.controls.active_table(), "prefix");
}

#[test]
fn declined_press_ends_the_sequence() {
    let (layout, windows) = columns_of(2, None);
    let mut mouse = Mouse::new(
        "declined-sequence",
        "gband.keymap.set('prefix', 'leftmouse', function() return false end)",
        layout,
    );
    mouse.key("ctrl+space");
    assert_eq!(mouse.controls.active_table(), "prefix");
    let steps = mouse.click(MouseButton::Left, (45, 3));
    assert!(messages(&steps).is_empty(), "{steps:?}");
    assert_eq!(mouse.display.focused(), Some(windows[0]));
    assert_eq!(
        mouse.key("h"),
        [Step::Send(ClientMessage::Key {
            window: windows[0],
            key: key("h"),
        })]
    );
}

#[test]
fn declining_skips_an_earlier_binding() {
    let (layout, windows) = columns_of(2, None);
    let mut mouse = Mouse::new(
        "declined-earlier",
        "gband.opt.mouse_mod = 'alt'
gband.keymap.set('root', 'alt+leftmouse', gband.action.drag_window)
gband.keymap.set('root', 'mod+leftmouse', function() return false end)",
        layout,
    );
    let steps = mouse.drag_with(MouseButton::Left, (45, 3), (50, 3), Modifiers::ALT);
    assert_eq!(mouse.display.focused(), Some(windows[1]));
    assert!(sent(&steps).is_empty(), "{steps:?}");
    let selected = mouse.display.selection().unwrap();
    assert_eq!(
        (selected.window, selected.start, selected.end),
        (windows[1], (4, 2), (9, 2))
    );
}

#[test]
fn error_takes_the_press() {
    let (layout, windows) = columns_of(2, None);
    let mut mouse = Mouse::new(
        "declined-error",
        "gband.bind('leftmouse', function() error('broken') end)",
        layout,
    );
    mouse.report_mouse(windows[1]);
    let steps = mouse.click(MouseButton::Left, (45, 3));
    assert!(mouse.display.banner().unwrap().ends_with("broken"));
    assert_eq!(mouse.display.focused(), Some(windows[0]));
    assert!(messages(&steps).is_empty(), "{steps:?}");
}

#[test]
fn declining_function_starts_no_gesture() {
    let (mut layout, windows) = columns_of(2, None);
    float(&mut layout, windows[1], Proportion::ONE_HALF, 12, 10, 4);
    let mut mouse = Mouse::new(
        "declined-gesture",
        &format!(
            "{NAVIGATION}gband.keymap.set('prefix', 'leftmouse', function() gband.action.drag_window() return false end)"
        ),
        layout,
    );
    mouse.key("ctrl+space");
    let steps = mouse.drag(MouseButton::Left, (20, 6), (25, 6));
    assert!(messages(&steps).is_empty(), "{steps:?}");
    assert_eq!(mouse.controls.active_table(), "prefix");
}

#[test]
fn declined_press_in_root_reaches_the_program() {
    let (mut layout, windows) = columns_of(2, None);
    float(&mut layout, windows[1], Proportion::ONE_HALF, 12, 10, 4);
    let mut mouse = Mouse::new(
        "declined-root-drag",
        "gband.bind('leftmouse', function() gband.action.drag_window() return false end)",
        layout,
    );
    mouse.display.apply(ServerMessage::Snapshot {
        window: windows[1],
        cols: 38,
        rows: 10,
        contents: b"\x1b[?1002h\x1b[?1006h".to_vec(),
    });
    let steps = mouse.drag(MouseButton::Left, (13, 6), (17, 6));
    assert!(sent(&steps).is_empty(), "{steps:?}");
    let left = MouseButton::Left;
    assert_eq!(
        mouse_messages(&steps),
        [
            (
                windows[1],
                MouseEvent::new(MouseKind::Press(left), 2, 1, Modifiers::NONE)
            ),
            (
                windows[1],
                MouseEvent::new(motion(left), 6, 1, Modifiers::NONE)
            ),
            (
                windows[1],
                MouseEvent::new(MouseKind::Release(left), 6, 1, Modifiers::NONE)
            ),
        ]
    );
}

#[test]
fn key_binding_returns_false() {
    let (layout, _) = columns_of(2, None);
    let mut mouse = Mouse::new(
        "key-returns-false",
        "gband.bind('alt+x', function() calls = (calls or 0) + 1 return false end)",
        layout,
    );
    let steps = mouse.key("alt+x");
    assert_eq!(mouse.global::<i64>("calls"), 1);
    assert!(messages(&steps).is_empty(), "{steps:?}");
}

#[test]
fn mouse_binding_declines() {
    let (layout, windows) = columns_of(2, None);
    let mut mouse = Mouse::new(
        "mouse-declines",
        "gband.bind('leftmouse', function() calls = (calls or 0) + 1 return false end)",
        layout,
    );
    mouse.click(MouseButton::Left, (45, 3));
    assert_eq!(mouse.global::<i64>("calls"), 1);
    assert_eq!(mouse.display.focused(), Some(windows[1]));
}

#[test]
fn registered_action_declines() {
    let (layout, windows) = columns_of(2, None);
    let mut mouse = Mouse::new(
        "action-declines",
        "gband.action.register('pass', function(e) got = e.button return false end)
gband.keymap.set('root', 'leftmouse', gband.action.pass)",
        layout,
    );
    mouse.click(MouseButton::Left, (45, 3));
    assert_eq!(mouse.global::<String>("got"), "left");
    assert_eq!(mouse.display.focused(), Some(windows[1]));
}

#[test]
fn return_value_through_keymap_run() {
    let (layout, windows) = columns_of(2, None);
    let mut mouse = Mouse::new(
        "declines-through-run",
        "gband.bind('leftmouse', function(...) args = select('#', ...) return false end)
gband.bind('alt+r', function() ran = gband.keymap.run('root', 'leftmouse') end)",
        layout,
    );
    let steps = mouse.key("alt+r");
    assert_eq!(mouse.global::<i64>("args"), 0);
    assert!(mouse.global::<bool>("ran"));
    assert_eq!(mouse.display.focused(), Some(windows[0]));
    assert!(messages(&steps).is_empty(), "{steps:?}");
}

#[test]
fn unbound_mouse_name_in_a_mode_is_discarded() {
    let (layout, windows) = columns_of(2, None);
    let mut mouse = Mouse::new(
        "mouse-mode",
        &format!("{NAVIGATION}gband.keymap.set('prefix', 'leftmouse', gband.action.drag_window)"),
        layout,
    );
    mouse.key("ctrl+space");
    assert_eq!(mouse.controls.active_table(), "prefix");
    let steps = mouse.drag_with(MouseButton::Left, (45, 3), (50, 5), Modifiers::CTRL);
    assert!(messages(&steps).is_empty(), "{steps:?}");
    assert_eq!(mouse.display.focused(), Some(windows[0]));
    assert_eq!(mouse.controls.active_table(), "prefix");
}

#[test]
fn modifier_with_a_mouse_name() {
    let (layout, _) = columns_of(2, None);
    let mut mouse = Mouse::new(
        "mouse-modifier",
        &format!(
            "{NAVIGATION}gband.keymap.set('prefix', 'alt+rightmouse', function() ran = true end)"
        ),
        layout,
    );
    mouse.key("ctrl+space");
    mouse.event(MouseKind::Press(MouseButton::Right), 45, 3, Modifiers::ALT);
    assert!(mouse.global::<bool>("ran"));
    assert_eq!(mouse.controls.active_table(), "prefix");
}

#[test]
fn drag_action_from_a_key_does_nothing() {
    let (mut layout, windows) = columns_of(2, None);
    float(&mut layout, windows[1], Proportion::ONE_HALF, 12, 10, 4);
    let mut mouse = Mouse::new(
        "mouse-key-drag",
        &format!("{NAVIGATION}gband.keymap.set('prefix', 'm', gband.action.drag_window)"),
        layout,
    );
    mouse.key("ctrl+space");
    let steps = mouse.key("m");
    assert!(messages(&steps).is_empty(), "{steps:?}");
    let steps = mouse.event(MouseKind::Motion(None), 30, 8, Modifiers::NONE);
    assert!(messages(&steps).is_empty(), "{steps:?}");
}

#[test]
fn drag_action_from_a_binding_function() {
    let (mut layout, windows) = columns_of(2, None);
    float(&mut layout, windows[1], Proportion::ONE_HALF, 12, 10, 4);
    let mut mouse = Mouse::new(
        "mouse-function-drag",
        &format!(
            "{NAVIGATION}gband.keymap.set('prefix', 'leftmouse', function() gband.action.drag_window() end)"
        ),
        layout,
    );
    mouse.key("ctrl+space");
    let steps = mouse.drag(MouseButton::Left, (20, 6), (25, 6));
    assert_eq!(
        sent(&steps),
        [SessionAction::SetPosition {
            window: windows[1],
            col: 15,
            row: 4
        }]
    );
}

const DRAG: &str = "gband.keymap.mode('prefix')
gband.keymap.set('prefix', 'leftmouse', gband.action.drag_window)
gband.keymap.set('prefix', 'rightmouse', gband.action.drag_resize_window)
gband.keymap.set('prefix', 'middlemouse', gband.action.drag_band)
";

fn dragging(name: &str, layout: Layout) -> Mouse {
    let mut mouse = Mouse::new(name, DRAG, layout);
    mouse.key("ctrl+space");
    mouse
}

#[test]
fn drag_a_floating_window() {
    let (mut layout, windows) = columns_of(2, None);
    float(&mut layout, windows[1], Proportion::ONE_HALF, 12, 10, 4);
    let mut mouse = dragging("mouse-move-float", layout);
    let steps = mouse.drag(MouseButton::Left, (20, 6), (35, 8));
    assert_eq!(
        sent(&steps),
        [SessionAction::SetPosition {
            window: windows[1],
            col: 25,
            row: 6
        }]
    );
    assert_eq!(mouse.display.focused(), Some(windows[1]));
}

#[test]
fn drag_a_floating_window_against_the_edge() {
    let (mut layout, windows) = columns_of(2, None);
    float(&mut layout, windows[1], Proportion::ONE_HALF, 12, 30, 4);
    let mut mouse = dragging("mouse-move-edge", layout);
    let steps = mouse.drag(MouseButton::Left, (40, 6), (60, 6));
    assert_eq!(
        sent(&steps),
        [SessionAction::SetPosition {
            window: windows[1],
            col: 40,
            row: 4
        }]
    );
}

#[test]
fn gesture_sends_at_most_once_per_frame() {
    let (mut layout, windows) = columns_of(2, None);
    float(&mut layout, windows[1], Proportion::ONE_HALF, 12, 10, 4);
    let mut mouse = dragging("mouse-frame", layout);
    let now = mouse.now;
    let event = |kind, col| MouseEvent::new(kind, col, 6, Modifiers::NONE);
    let left = MouseButton::Left;
    let mut steps =
        mouse
            .controls
            .mouse(&mut mouse.display, event(MouseKind::Press(left), 20), now);
    for (col, at) in [(21, 1), (22, 2), (23, 3)] {
        steps.extend(mouse.controls.mouse(
            &mut mouse.display,
            event(MouseKind::Motion(Some(left)), col),
            now + Duration::from_millis(at),
        ));
    }
    assert_eq!(sent(&steps).len(), 1, "{steps:?}");
    let later = now + Duration::from_millis(40);
    assert_eq!(
        sent(&mouse.controls.flush(&mut mouse.display, later)),
        [SessionAction::SetPosition {
            window: windows[1],
            col: 13,
            row: 4
        }]
    );
    assert!(
        mouse
            .controls
            .flush(&mut mouse.display, later + Duration::from_millis(40))
            .is_empty()
    );
}

#[test]
fn pick_edges_by_thirds() {
    let right = Edges {
        right: true,
        ..Edges::default()
    };
    assert_eq!(pick_edges(35, 12, 40, 24), right);
    assert_eq!(
        pick_edges(2, 10, 30, 12),
        Edges {
            left: true,
            bottom: true,
            ..Edges::default()
        }
    );
    assert_eq!(
        pick_edges(14, 5, 30, 12),
        Edges {
            top: true,
            ..Edges::default()
        }
    );
    assert_eq!(
        pick_edges(15, 5, 31, 11),
        Edges {
            bottom: true,
            ..Edges::default()
        }
    );
}

#[test]
fn widen_a_column_from_its_right_edge() {
    let (layout, windows) = columns_of(2, None);
    let mut mouse = dragging("mouse-widen", layout);
    let steps = mouse.drag(MouseButton::Right, (35, 12), (43, 12));
    assert_eq!(
        sent(&steps),
        [SessionAction::SetWidth {
            window: windows[0],
            width: Proportion::new(3, 5)
        }]
    );
}

#[test]
fn grow_a_window_down() {
    let (mut layout, windows) = columns_of(2, None);
    layout.apply(
        SessionAction::ConsumeOrExpel {
            window: windows[1],
            direction: Direction::Left,
        },
        AREA,
        &LayoutOptions::default(),
    );
    let mut mouse = dragging("mouse-grow-down", layout);
    let steps = mouse.drag(MouseButton::Right, (20, 10), (20, 14));
    assert_eq!(
        sent(&steps),
        [SessionAction::SetHeight {
            window: windows[0],
            height: WindowHeight::Fixed(16)
        }]
    );
    mouse.apply(&steps);
    let heights: Vec<u16> = tiles(&mouse.layout.bands()[0], AREA)
        .iter()
        .map(|tile| tile.height)
        .collect();
    assert_eq!(heights, [16, 8]);
}

#[test]
fn resize_a_floating_window_from_its_left_edge() {
    let (mut layout, windows) = columns_of(2, None);
    float(&mut layout, windows[1], Proportion::ONE_HALF, 12, 20, 4);
    let mut mouse = dragging("mouse-left-edge", layout);
    let steps = mouse.drag(MouseButton::Right, (22, 9), (12, 9));
    assert_eq!(
        sent(&steps),
        [
            SessionAction::SetWidth {
                window: windows[1],
                width: Proportion::new(5, 8)
            },
            SessionAction::SetPosition {
                window: windows[1],
                col: 10,
                row: 4
            },
        ]
    );
    mouse.apply(&steps);
    let placed = gband_core::geometry::placed(mouse.layout.floating(windows[1]).unwrap(), AREA);
    assert_eq!((placed.x, placed.width), (10, 50));
}

#[test]
fn left_edge_of_a_tile_moves_the_camera() {
    let (layout, windows) = columns_of(3, None);
    let mut mouse = dragging("mouse-left-tile", layout);
    assert_eq!(mouse.display.camera(), Some(0));
    let steps = mouse.drag(MouseButton::Right, (42, 12), (38, 12));
    assert_eq!(
        sent(&steps),
        [SessionAction::SetWidth {
            window: windows[1],
            width: Proportion::new(11, 20)
        }]
    );
    assert_eq!(mouse.display.focused(), Some(windows[1]));
    assert_eq!(mouse.display.camera(), Some(4));
}

fn resizing_by(name: &str, edges: &str, layout: Layout) -> Mouse {
    let mut mouse = Mouse::new(
        name,
        &format!(
            "{NAVIGATION}gband.keymap.set('prefix', 'rightmouse', function() gband.action.drag_resize_window({{ edges = {{ {edges} }} }}) end)"
        ),
        layout,
    );
    mouse.key("ctrl+space");
    mouse
}

#[test]
fn named_bottom_edge_pressed_near_a_corner() {
    let (mut layout, windows) = columns_of(2, None);
    float(&mut layout, windows[1], Proportion::new(3, 8), 12, 10, 4);
    let mut mouse = resizing_by("named-bottom", "'bottom'", layout);
    let steps = mouse.drag(MouseButton::Right, (12, 15), (8, 18));
    assert_eq!(
        sent(&steps),
        [SessionAction::SetHeight {
            window: windows[1],
            height: WindowHeight::Fixed(15)
        }]
    );
    mouse.apply(&steps);
    let placed = gband_core::geometry::placed(mouse.layout.floating(windows[1]).unwrap(), AREA);
    assert_eq!(
        (placed.x, placed.y, placed.width, placed.height),
        (10, 4, 30, 15)
    );
}

#[test]
fn named_corner() {
    let (mut layout, windows) = columns_of(2, None);
    float(&mut layout, windows[1], Proportion::new(3, 8), 12, 10, 4);
    let mut mouse = resizing_by("named-corner", "'left', 'top'", layout);
    let steps = mouse.drag(MouseButton::Right, (11, 4), (6, 2));
    mouse.apply(&steps);
    let placed = gband_core::geometry::placed(mouse.layout.floating(windows[1]).unwrap(), AREA);
    assert_eq!(
        (placed.x, placed.y, placed.width, placed.height),
        (5, 2, 35, 14)
    );
}

#[test]
fn named_edge_pressed_away_from_it() {
    let (layout, windows) = columns_of(2, None);
    let mut mouse = resizing_by("named-away", "'right'", layout);
    assert_eq!(mouse.display.camera(), Some(0));
    let steps = mouse.drag(MouseButton::Right, (5, 12), (13, 12));
    assert_eq!(
        sent(&steps),
        [SessionAction::SetWidth {
            window: windows[0],
            width: Proportion::new(3, 5)
        }]
    );
    assert_eq!(mouse.display.camera(), Some(0));
}

#[test]
fn named_left_edge_of_a_tile_moves_the_camera() {
    let (layout, windows) = columns_of(3, None);
    let mut mouse = resizing_by("named-left-tile", "'left'", layout);
    assert_eq!(mouse.display.camera(), Some(0));
    let steps = mouse.drag(MouseButton::Right, (75, 12), (71, 12));
    assert_eq!(
        sent(&steps),
        [SessionAction::SetWidth {
            window: windows[1],
            width: Proportion::new(11, 20)
        }]
    );
    assert_eq!(mouse.display.camera(), Some(4));
}

#[test]
fn named_top_edge_of_a_columns_first_window() {
    let (mut layout, windows) = columns_of(2, None);
    layout.apply(
        SessionAction::ConsumeOrExpel {
            window: windows[1],
            direction: Direction::Left,
        },
        AREA,
        &LayoutOptions::default(),
    );
    let mut mouse = resizing_by("named-top-first", "'top'", layout);
    let steps = mouse.drag(MouseButton::Right, (20, 6), (20, 2));
    assert_eq!(
        sent(&steps),
        [SessionAction::SetHeight {
            window: windows[0],
            height: WindowHeight::Fixed(16)
        }]
    );
    mouse.apply(&steps);
    let placed: Vec<(u16, u16)> = tiles(&mouse.layout.bands()[0], AREA)
        .iter()
        .map(|tile| (tile.y, tile.height))
        .collect();
    assert_eq!(placed, [(0, 16), (16, 8)]);
}

#[test]
fn named_edges_from_a_key() {
    let (mut layout, windows) = columns_of(2, None);
    float(&mut layout, windows[1], Proportion::ONE_HALF, 12, 10, 4);
    let mut mouse = Mouse::new(
        "named-key",
        &format!(
            "{NAVIGATION}gband.keymap.set('prefix', 'r', function() gband.action.drag_resize_window({{ edges = {{ 'right' }} }}) end)"
        ),
        layout,
    );
    mouse.key("ctrl+space");
    let steps = mouse.key("r");
    assert!(messages(&steps).is_empty(), "{steps:?}");
    assert_eq!(mouse.display.banner(), None);
    let steps = mouse.event(MouseKind::Motion(None), 30, 8, Modifiers::NONE);
    assert!(messages(&steps).is_empty(), "{steps:?}");
}

#[test]
fn opposite_edges_send_nothing() {
    let (mut layout, windows) = columns_of(2, None);
    float(&mut layout, windows[1], Proportion::ONE_HALF, 12, 10, 4);
    let mut mouse = resizing_by("named-opposite", "'left', 'right'", layout);
    let steps = mouse.drag(MouseButton::Right, (20, 6), (25, 9));
    assert!(messages(&steps).is_empty(), "{steps:?}");
    let banner = mouse.display.banner().unwrap();
    assert!(banner.contains(":3: "), "{banner}");
    assert!(banner.contains("`left` and `right`"), "{banner}");
}

#[test]
fn slide_the_band_and_settle() {
    let (layout, windows) = columns_of(4, None);
    let mut mouse = dragging("mouse-slide", layout);
    let middle = MouseButton::Middle;
    mouse.event(MouseKind::Press(middle), 70, 5, Modifiers::NONE);
    let steps = mouse.event(MouseKind::Motion(Some(middle)), 10, 9, Modifiers::NONE);
    assert!(messages(&steps).is_empty(), "{steps:?}");
    assert_eq!(mouse.display.camera(), Some(60));
    assert_eq!(mouse.display.focused(), Some(windows[0]));
    let steps = mouse.event(MouseKind::Release(middle), 10, 9, Modifiers::NONE);
    assert!(messages(&steps).is_empty(), "{steps:?}");
    assert_eq!(mouse.display.camera(), Some(60));
    assert_eq!(mouse.display.focused(), Some(windows[2]));
}

#[test]
fn partly_shown_column_snaps_after_a_slide() {
    let (layout, windows) = columns_of(4, Some(Proportion::new(3, 4)));
    let mut mouse = dragging("mouse-slide-snap", layout);
    mouse.drag(MouseButton::Middle, (75, 5), (5, 5));
    assert_eq!(mouse.display.focused(), Some(windows[1]));
    assert_eq!(mouse.display.camera(), Some(60));
}

#[test]
fn slide_past_the_strips_end_pulls_back() {
    let (layout, windows) = columns_of(2, None);
    let mut mouse = dragging("mouse-slide-end", layout);
    let middle = MouseButton::Middle;
    mouse.event(MouseKind::Press(middle), 70, 5, Modifiers::NONE);
    mouse.event(MouseKind::Motion(Some(middle)), 40, 5, Modifiers::NONE);
    assert_eq!(mouse.display.camera(), Some(30));
    mouse.event(MouseKind::Release(middle), 40, 5, Modifiers::NONE);
    assert_eq!(mouse.display.focused(), Some(windows[1]));
    assert_eq!(mouse.display.camera(), Some(0));
}

#[test]
fn left_drag_on_empty_ribbon_slides_and_later_vertical_motion_does_not() {
    let (layout, windows) = columns_of(1, Some(Proportion::new(1, 4)));
    let mut mouse = dragging("mouse-slide-ribbon", layout);
    let left = MouseButton::Left;
    mouse.event(MouseKind::Press(left), 50, 5, Modifiers::NONE);
    mouse.event(MouseKind::Motion(Some(left)), 40, 5, Modifiers::NONE);
    assert_eq!(mouse.display.camera(), Some(10));
    mouse.event(MouseKind::Motion(Some(left)), 40, 15, Modifiers::NONE);
    assert_eq!(mouse.display.camera(), Some(10));
    assert_eq!(mouse.tops(), [(mouse.layout.bands()[0].id, 0)]);
    mouse.event(MouseKind::Release(left), 40, 15, Modifiers::NONE);
    assert_eq!(mouse.display.focused(), Some(windows[0]));
}

fn open_in(layout: &mut Layout, band: usize, width: Option<Proportion>) -> WindowId {
    let window = layout.allocate_window();
    let band = layout.bands()[band].id;
    let after = layout.band(band).and_then(|band| band.windows().last());
    layout.open(window, band, after, width, &LayoutOptions::default());
    window
}

fn two_bands(columns: usize, width: Option<Proportion>) -> (Layout, Vec<WindowId>) {
    let mut layout = Layout::new();
    let mut windows: Vec<WindowId> = (0..columns)
        .map(|_| open_in(&mut layout, 0, width))
        .collect();
    windows.push(open_in(&mut layout, 1, None));
    (layout, windows)
}

impl Mouse {
    fn tops(&mut self) -> Vec<(BandId, i64)> {
        self.display
            .present(self.now)
            .unwrap()
            .bands
            .iter()
            .map(|band| (band.band, band.top))
            .collect()
    }

    fn viewed(&self) -> BandId {
        BandId(self.display.view_state("root").band.number)
    }

    fn band(&self, index: usize) -> BandId {
        self.layout.bands()[index].id
    }

    fn middle(&mut self, kind: fn(MouseButton) -> MouseKind, col: u16, row: u16) -> Vec<Step> {
        self.event(kind(MouseButton::Middle), col, row, Modifiers::NONE)
    }
}

fn motion(button: MouseButton) -> MouseKind {
    MouseKind::Motion(Some(button))
}

#[test]
fn horizontal_axis_ignores_vertical_movement() {
    let (layout, _) = two_bands(4, None);
    let mut mouse = dragging("mouse-axis-horizontal", layout);
    let b1 = mouse.band(0);
    mouse.middle(MouseKind::Press, 70, 5);
    mouse.middle(motion, 40, 8);
    assert_eq!(mouse.display.camera(), Some(30));
    assert_eq!(mouse.tops(), [(b1, 0)]);
    mouse.middle(motion, 40, 20);
    assert_eq!(mouse.display.camera(), Some(30));
    assert_eq!(mouse.tops(), [(b1, 0)]);
    mouse.middle(MouseKind::Release, 40, 20);
    assert_eq!(mouse.viewed(), b1);
    assert_eq!(mouse.tops(), [(b1, 0)]);
}

#[test]
fn small_movement_locks_no_axis() {
    let (layout, _) = two_bands(4, None);
    let mut mouse = dragging("mouse-axis-none", layout);
    let before = mouse.tops();
    mouse.middle(MouseKind::Press, 40, 10);
    mouse.middle(motion, 41, 10);
    assert_eq!(mouse.display.camera(), Some(0));
    assert_eq!(mouse.tops(), before);
}

#[test]
fn drag_up_holds_the_band_below_until_the_release() {
    let (layout, windows) = two_bands(1, None);
    let mut mouse = dragging("mouse-band-up", layout);
    let (b1, b2) = (mouse.band(0), mouse.band(1));
    mouse.middle(MouseKind::Press, 40, 20);
    let steps = mouse.middle(motion, 40, 4);
    assert!(messages(&steps).is_empty(), "{steps:?}");
    assert_eq!(mouse.tops(), [(b2, 8), (b1, -16)]);
    assert_eq!(mouse.viewed(), b1);
    assert_eq!(mouse.display.focused(), Some(windows[0]));
    let steps = mouse.middle(MouseKind::Release, 40, 4);
    assert!(messages(&steps).is_empty(), "{steps:?}");
    assert_eq!(mouse.viewed(), b2);
    assert_eq!(mouse.display.focused(), Some(windows[1]));
    assert_eq!(mouse.tops(), [(b2, 0)]);
}

#[test]
fn bands_follow_the_pointer_with_animations_on() {
    let (layout, _) = two_bands(1, None);
    let mut mouse = Mouse::with("mouse-band-follow", DRAG, layout, Animations::On);
    mouse.key("ctrl+space");
    let (b1, b2) = (mouse.band(0), mouse.band(1));
    mouse.middle(MouseKind::Press, 40, 12);
    mouse.middle(motion, 40, 6);
    assert_eq!(mouse.tops(), [(b2, 18), (b1, -6)]);
}

#[test]
fn short_band_drag_returns() {
    let (layout, windows) = two_bands(4, None);
    let mut mouse = dragging("mouse-band-short", layout);
    dispatch(&mut mouse.display, Action::View(ViewAction::FocusRight));
    let camera = mouse.display.camera();
    mouse.drag(MouseButton::Middle, (40, 12), (40, 6));
    assert_eq!(mouse.viewed(), mouse.band(0));
    assert_eq!(mouse.display.focused(), Some(windows[1]));
    assert_eq!(mouse.display.camera(), camera);
}

#[test]
fn drag_down_to_the_band_above() {
    let (layout, _) = two_bands(1, None);
    let mut mouse = dragging("mouse-band-down", layout);
    dispatch(&mut mouse.display, Action::View(ViewAction::BandDown));
    assert_eq!(mouse.viewed(), mouse.band(1));
    mouse.drag(MouseButton::Middle, (40, 2), (40, 20));
    assert_eq!(mouse.viewed(), mouse.band(0));
}

#[test]
fn vertical_axis_ignores_sideways_movement() {
    let (layout, _) = columns_of(4, None);
    let mut mouse = dragging("mouse-axis-vertical", layout);
    let (b1, b2) = (mouse.band(0), mouse.band(1));
    mouse.middle(MouseKind::Press, 40, 10);
    mouse.middle(motion, 41, 5);
    assert_eq!(mouse.display.camera(), Some(0));
    assert_eq!(mouse.tops(), [(b2, 19), (b1, -5)]);
    mouse.middle(motion, 0, 5);
    assert_eq!(mouse.display.camera(), Some(0));
    assert_eq!(mouse.tops(), [(b2, 19), (b1, -5)]);
}

#[test]
fn first_band_stops_the_drag() {
    let (layout, windows) = two_bands(1, None);
    let mut mouse = dragging("mouse-band-first", layout);
    let b1 = mouse.band(0);
    mouse.middle(MouseKind::Press, 40, 5);
    mouse.middle(motion, 40, 15);
    assert_eq!(mouse.tops(), [(b1, 0)]);
    mouse.middle(MouseKind::Release, 40, 15);
    assert_eq!(mouse.tops(), [(b1, 0)]);
    assert_eq!(mouse.viewed(), b1);
    assert_eq!(mouse.display.focused(), Some(windows[0]));
}

#[test]
fn band_removed_above_shifts_the_held_bands() {
    let mut layout = Layout::new();
    let first = open_in(&mut layout, 0, None);
    open_in(&mut layout, 1, None);
    open_in(&mut layout, 2, None);
    let mut mouse = dragging("mouse-band-shift", layout);
    dispatch(&mut mouse.display, Action::View(ViewAction::BandDown));
    let (b2, b3) = (mouse.band(1), mouse.band(2));
    mouse.middle(MouseKind::Press, 40, 20);
    mouse.middle(motion, 40, 14);
    assert_eq!(mouse.tops(), [(b3, 18), (b2, -6)]);
    mouse.layout.remove(first);
    mouse.apply(&[]);
    assert_eq!(mouse.tops(), [(b3, 18), (b2, -6)]);
    mouse.middle(MouseKind::Release, 40, 14);
    assert_eq!(mouse.viewed(), b2);
}

#[test]
fn viewed_band_leaving_the_layout_ends_the_band_drag() {
    let (layout, windows) = two_bands(1, None);
    let mut mouse = dragging("mouse-band-removed", layout);
    let b2 = mouse.band(1);
    mouse.middle(MouseKind::Press, 40, 20);
    mouse.middle(motion, 40, 8);
    mouse.layout.remove(windows[0]);
    mouse.apply(&[]);
    assert_eq!(mouse.viewed(), b2);
    assert_eq!(mouse.tops(), [(b2, 0)]);
    let steps = mouse.middle(motion, 40, 2);
    assert!(messages(&steps).is_empty(), "{steps:?}");
    assert_eq!(mouse.tops(), [(b2, 0)]);
    mouse.middle(MouseKind::Release, 40, 2);
    assert_eq!(mouse.viewed(), b2);
    assert_eq!(mouse.tops(), [(b2, 0)]);
}

#[test]
fn left_drag_on_empty_ribbon_switches_bands() {
    let (layout, _) = two_bands(1, Some(Proportion::new(1, 4)));
    let mut mouse = dragging("mouse-band-ribbon", layout);
    let steps = mouse.drag(MouseButton::Left, (50, 22), (50, 2));
    assert!(messages(&steps).is_empty(), "{steps:?}");
    assert_eq!(mouse.viewed(), mouse.band(1));
}

#[test]
fn drag_resize_sends_set_width_from_a_mode_binding() {
    let (layout, windows) = columns_of(2, None);
    let mut mouse = dragging("mouse-resize-binding", layout);
    let steps = mouse.drag(MouseButton::Right, (38, 12), (39, 12));
    assert_eq!(
        sent(&steps),
        [SessionAction::SetWidth {
            window: windows[0],
            width: Proportion::new(41, 80)
        }]
    );
}

#[test]
fn drop_between_two_columns() {
    let (layout, windows) = columns_of(3, None);
    let mut mouse = dragging("mouse-drop-between", layout);
    for action in [
        ViewAction::FocusRight,
        ViewAction::FocusRight,
        ViewAction::FocusLeft,
        ViewAction::FocusLeft,
    ] {
        dispatch(&mut mouse.display, Action::View(action));
    }
    assert_eq!(mouse.display.focused(), Some(windows[0]));
    let steps = mouse.drag(MouseButton::Left, (10, 10), (75, 10));
    mouse.apply(&steps);
    let order: Vec<WindowId> = mouse.layout.bands()[0]
        .columns
        .iter()
        .map(|column| column.windows[0])
        .collect();
    assert_eq!(order, [windows[1], windows[0], windows[2]]);
    assert_eq!(mouse.display.focused(), Some(windows[0]));
    assert_eq!(mouse.display.camera(), Some(0));
}

const REPORTING: &[u8] = b"\x1b[?1000h\x1b[?1006h";
const BAND_WHEEL: &str =
    "gband.keymap.set('root', 'alt+wheeldown', gband.action.focus_band_down)\n";

impl Mouse {
    fn wheel(
        &mut self,
        direction: WheelDirection,
        cell: (u16, u16),
        modifiers: Modifiers,
    ) -> Vec<Step> {
        self.event(MouseKind::Wheel(direction), cell.0, cell.1, modifiers)
    }

    fn report_mouse(&mut self, window: WindowId) {
        self.display.apply(ServerMessage::Snapshot {
            window,
            cols: 38,
            rows: 22,
            contents: REPORTING.to_vec(),
        });
    }

    fn plugin_float(&mut self, id: u32) -> (u16, u16) {
        for row in 0..AREA.rows {
            for col in 0..AREA.cols {
                let hit = self.display.hit(col, row, self.now);
                if hit.target == Target::PluginFloat(id) && hit.content.is_some() {
                    return (col, row);
                }
            }
        }
        panic!("plugin window {id} is not drawn");
    }

    fn eval<T: mlua::FromLua>(&self, source: &str) -> T {
        self.controls.runtime().lua().load(source).eval().unwrap()
    }
}

fn mouse_messages(steps: &[Step]) -> Vec<(WindowId, MouseEvent)> {
    steps
        .iter()
        .filter_map(|step| match step {
            Step::Send(ClientMessage::Mouse { window, event }) => Some((*window, *event)),
            _ => None,
        })
        .collect()
}

fn three_bands() -> (Layout, Vec<WindowId>) {
    let (mut layout, mut windows) = two_bands(1, None);
    windows.push(open_in(&mut layout, 2, None));
    (layout, windows)
}

#[test]
fn bound_wheel_step_in_root() {
    let (layout, windows) = two_bands(1, None);
    let mut mouse = Mouse::new("wheel-bound", BAND_WHEEL, layout);
    mouse.report_mouse(windows[0]);
    assert_eq!(mouse.viewed(), mouse.band(0));
    let steps = mouse.wheel(WheelDirection::Down, (5, 5), Modifiers::ALT);
    assert_eq!(mouse.viewed(), mouse.band(1));
    assert!(mouse_messages(&steps).is_empty(), "{steps:?}");
}

#[test]
fn cooldown_after_a_wheel_binding() {
    let (layout, _) = three_bands();
    let mut mouse = Mouse::new("wheel-cooldown", BAND_WHEEL, layout);
    let mut steps = Vec::new();
    for _ in 0..3 {
        steps.extend(mouse.wheel(WheelDirection::Down, (5, 5), Modifiers::ALT));
    }
    assert_eq!(mouse.viewed(), mouse.band(1));
    assert!(mouse_messages(&steps).is_empty(), "{steps:?}");
}

#[test]
fn binding_runs_again_after_the_cooldown() {
    let (layout, _) = three_bands();
    let mut mouse = Mouse::new("wheel-cooldown-over", BAND_WHEEL, layout);
    mouse.wheel(WheelDirection::Down, (5, 5), Modifiers::ALT);
    mouse.now += Duration::from_millis(180);
    mouse.wheel(WheelDirection::Down, (5, 5), Modifiers::ALT);
    assert_eq!(mouse.viewed(), mouse.band(2));
}

#[test]
fn cooldown_keeps_other_wheel_names() {
    let (layout, windows) = two_bands(1, None);
    let mut mouse = Mouse::new("wheel-cooldown-other", BAND_WHEEL, layout);
    mouse.report_mouse(windows[0]);
    mouse.wheel(WheelDirection::Down, (5, 5), Modifiers::ALT);
    mouse.wheel(WheelDirection::Down, (5, 5), Modifiers::NONE);
    let steps = mouse.wheel(WheelDirection::Down, (5, 5), Modifiers::NONE);
    assert_eq!(mouse.viewed(), mouse.band(1));
    assert_eq!(mouse_messages(&steps).len(), 1, "{steps:?}");
}

#[test]
fn unbound_wheel_step_keeps_the_sequence() {
    let (layout, windows) = columns_of(2, None);
    let mut mouse = Mouse::new("wheel-sequence", "gband.keystyle.use('direct')", layout);
    mouse.report_mouse(windows[0]);
    mouse.key("ctrl+space");
    mouse.key("l");
    assert_eq!(mouse.display.focused(), Some(windows[1]));
    mouse.key("ctrl+space");
    let steps = mouse.wheel(WheelDirection::Down, (5, 5), Modifiers::NONE);
    assert_eq!(
        mouse_messages(&steps),
        [(
            windows[0],
            MouseEvent::new(
                MouseKind::Wheel(WheelDirection::Down),
                4,
                4,
                Modifiers::NONE
            )
        )]
    );
    assert_eq!(mouse.controls.active_table(), "prefix");
    mouse.key("h");
    assert_eq!(mouse.display.focused(), Some(windows[0]));
}

#[test]
fn declined_wheel_step_keeps_the_sequence() {
    let (layout, windows) = columns_of(2, None);
    let mut mouse = Mouse::new(
        "wheel-declined-sequence",
        "gband.keystyle.use('direct')\ngband.keymap.set('prefix', 'wheeldown', function() return false end)",
        layout,
    );
    mouse.report_mouse(windows[0]);
    mouse.key("ctrl+space");
    mouse.key("l");
    assert_eq!(mouse.display.focused(), Some(windows[1]));
    mouse.key("ctrl+space");
    let steps = mouse.wheel(WheelDirection::Down, (5, 5), Modifiers::NONE);
    assert_eq!(
        mouse_messages(&steps),
        [(
            windows[0],
            MouseEvent::new(
                MouseKind::Wheel(WheelDirection::Down),
                4,
                4,
                Modifiers::NONE
            )
        )]
    );
    assert_eq!(mouse.controls.active_table(), "prefix");
    mouse.key("h");
    assert_eq!(mouse.display.focused(), Some(windows[0]));
}

#[test]
fn declined_wheel_step_starts_no_cooldown() {
    let (layout, _) = three_bands();
    let mut mouse = Mouse::new(
        "wheel-declined-cooldown",
        "gband.keymap.set('root', 'alt+wheeldown', function()
  calls = (calls or 0) + 1
  if calls == 1 then return false end
  gband.action.focus_band_down()
end)",
        layout,
    );
    mouse.wheel(WheelDirection::Down, (5, 5), Modifiers::ALT);
    mouse.wheel(WheelDirection::Down, (5, 5), Modifiers::ALT);
    assert_eq!(mouse.global::<i64>("calls"), 2);
    assert_eq!(mouse.viewed(), mouse.band(1));
}

#[test]
fn declined_wheel_step_reaches_the_program() {
    let (layout, windows) = two_bands(1, None);
    let mut mouse = Mouse::new(
        "wheel-declined-program",
        "gband.keymap.set('root', 'alt+wheeldown', function() return false end)",
        layout,
    );
    mouse.report_mouse(windows[0]);
    let steps = mouse.wheel(WheelDirection::Down, (1, 1), Modifiers::ALT);
    assert_eq!(
        mouse_messages(&steps),
        [(
            windows[0],
            MouseEvent::new(MouseKind::Wheel(WheelDirection::Down), 0, 0, Modifiers::ALT)
        )]
    );
    assert_eq!(mouse.viewed(), mouse.band(0));
}

#[test]
fn wheel_during_a_gesture() {
    let (mut layout, windows) = two_bands(2, None);
    float(&mut layout, windows[1], Proportion::ONE_HALF, 12, 10, 4);
    let mut mouse = Mouse::new(
        "wheel-gesture",
        &format!("{BAND_WHEEL}gband.keymap.set('root', 'alt+leftmouse', gband.action.drag_window)"),
        layout,
    );
    mouse.event(MouseKind::Press(MouseButton::Left), 20, 6, Modifiers::ALT);
    mouse.event(motion(MouseButton::Left), 25, 6, Modifiers::ALT);
    mouse.wheel(WheelDirection::Down, (25, 6), Modifiers::ALT);
    mouse.event(MouseKind::Release(MouseButton::Left), 25, 6, Modifiers::ALT);
    assert_eq!(mouse.viewed(), mouse.band(0));
}

#[test]
fn wheel_binding_function() {
    let (layout, _) = columns_of(1, None);
    let mut mouse = Mouse::new(
        "wheel-function",
        "gband.keymap.set('root', 'ctrl+wheelup', function(e) calls = (calls or 0) + 1 got = e.direction .. ' ' .. tostring(e.ctrl) .. ' ' .. e.target end)",
        layout,
    );
    let steps = mouse.wheel(WheelDirection::Up, (60, 3), Modifiers::CTRL);
    assert_eq!(mouse.global::<String>("got"), "up true ribbon");
    assert_eq!(mouse.global::<i64>("calls"), 1);
    assert!(messages(&steps).is_empty(), "{steps:?}");
}

#[test]
fn ctrl_alt_selects_in_a_mouse_program() {
    let (layout, windows) = columns_of(1, None);
    let mut mouse = Mouse::new("ctrl-alt-select", "", layout);
    mouse.report_mouse(windows[0]);
    let ctrl_alt = Modifiers {
        ctrl: true,
        alt: true,
        shift: false,
    };
    let steps = mouse.drag_with(MouseButton::Left, (2, 3), (6, 3), ctrl_alt);
    assert!(mouse_messages(&steps).is_empty(), "{steps:?}");
    assert!(mouse.display.selection().is_some());
}

#[test]
fn own_binding_after_the_preset() {
    let (mut layout, windows) = columns_of(2, None);
    float(&mut layout, windows[1], Proportion::ONE_HALF, 12, 10, 4);
    let mut mouse = Mouse::new(
        "own-after-preset",
        "gband.keystyle.use()\ngband.keymap.set('root', 'alt+leftmouse', function() ran = true end)",
        layout,
    );
    let steps = mouse.drag_with(MouseButton::Left, (20, 6), (35, 8), Modifiers::ALT);
    assert!(mouse.global::<bool>("ran"));
    assert!(sent(&steps).is_empty(), "{steps:?}");
}

#[test]
fn unbound_alt_click_is_forwarded() {
    let (layout, windows) = columns_of(1, None);
    let mut mouse = Mouse::new("alt-forward", "", layout);
    mouse.report_mouse(windows[0]);
    let steps = mouse.event(MouseKind::Press(MouseButton::Left), 5, 3, Modifiers::ALT);
    assert_eq!(
        mouse_messages(&steps),
        [(
            windows[0],
            MouseEvent::new(MouseKind::Press(MouseButton::Left), 4, 2, Modifiers::ALT)
        )]
    );
}

#[test]
fn bound_wheel_step_over_a_plugin_window() {
    let (layout, _) = two_bands(1, None);
    let mut mouse = Mouse::new(
        "wheel-plugin-window",
        &format!(
            "{BAND_WHEEL}gband.bind('alt+o', function() win = gband.win.open({{ lines = {{ 'a', 'b' }}, on_mouse = function(_, e) got = e.kind end }}) end)"
        ),
        layout,
    );
    mouse.key("alt+o");
    let id: u32 = mouse.global("win");
    let cell = mouse.plugin_float(id);
    mouse.wheel(WheelDirection::Down, cell, Modifiers::ALT);
    assert_eq!(mouse.viewed(), mouse.band(1));
    assert_eq!(mouse.global::<Option<String>>("got"), None);
}

const RECORD_MOUSE: &str = "log = {}
function record(_, e)
  local keys = {}
  for _, k in ipairs({ 'kind', 'button', 'content_col', 'box_col', 'box_row', 'box_width', 'box_height' }) do
    keys[#keys + 1] = k .. '=' .. tostring(e[k])
  end
  log[#log + 1] = table.concat(keys, ',')
end
";

#[test]
fn declined_press_reaches_on_mouse() {
    let (layout, _) = columns_of(1, None);
    let mut mouse = Mouse::new(
        "declined-on-mouse",
        &format!(
            "{RECORD_MOUSE}gband.bind('leftmouse', function() return false end)
gband.bind('alt+o', function() win = gband.win.open({{ lines = {{ 'a', 'b', 'c' }}, focus = false, on_mouse = function(id, e) record(id, e) end }}) end)"
        ),
        layout,
    );
    mouse.key("alt+o");
    let id: u32 = mouse.global("win");
    assert!(!mouse.eval::<bool>(&format!("return gband.win.info({id}).focused")));
    let (col, row) = mouse.plugin_float(id);
    mouse.click(MouseButton::Left, (col, row + 2));
    assert!(mouse.eval::<bool>(&format!("return gband.win.info({id}).focused")));
    let kinds: Vec<String> = mouse
        .global::<Vec<String>>("log")
        .iter()
        .map(|line| line.split(',').take(2).collect::<Vec<_>>().join(","))
        .collect();
    assert_eq!(
        kinds,
        ["kind=press,button=left", "kind=release,button=left"]
    );
}

#[test]
fn declined_wheel_step_scrolls_a_plugin_window() {
    let (layout, _) = columns_of(1, None);
    let mut mouse = Mouse::new(
        "declined-wheel-plugin",
        "gband.bind('wheeldown', function() return false end)
gband.bind('alt+o', function() local lines = {} for i = 1, 25 do lines[i] = tostring(i) end win = gband.win.open({ lines = lines, height = 12, focus = false }) end)",
        layout,
    );
    mouse.key("alt+o");
    let id: u32 = mouse.global("win");
    let cell = mouse.plugin_float(id);
    mouse.wheel(WheelDirection::Down, cell, Modifiers::NONE);
    assert_eq!(
        mouse.eval::<i64>(&format!("return gband.win.info({id}).top")),
        2
    );
    assert!(!mouse.eval::<bool>(&format!("return gband.win.info({id}).focused")));
}

#[test]
fn box_cell_on_the_plugin_window_border() {
    let (layout, _) = columns_of(1, None);
    let mut mouse = Mouse::new(
        "plugin-box-border",
        &format!(
            "{RECORD_MOUSE}gband.bind('alt+o', function() win = gband.win.open({{ col = 5, row = 3, width = 20, height = 10, on_mouse = record }}) end)"
        ),
        layout,
    );
    mouse.key("alt+o");
    mouse.event(MouseKind::Press(MouseButton::Left), 24, 3, Modifiers::NONE);
    assert_eq!(
        mouse.global::<Vec<String>>("log"),
        ["kind=press,button=left,content_col=nil,box_col=19,box_row=0,box_width=20,box_height=10"]
    );
}

#[test]
fn drag_off_the_plugin_window() {
    let (layout, _) = columns_of(1, None);
    let mut mouse = Mouse::new(
        "plugin-drag-off",
        &format!(
            "{RECORD_MOUSE}gband.bind('alt+o', function() win = gband.win.open({{ col = 5, row = 3, width = 20, height = 10, on_mouse = record }}) end)"
        ),
        layout,
    );
    mouse.key("alt+o");
    mouse.event(MouseKind::Press(MouseButton::Left), 10, 6, Modifiers::NONE);
    mouse.event(motion(MouseButton::Left), 40, 6, Modifiers::NONE);
    assert_eq!(
        mouse.global::<Vec<String>>("log"),
        [
            "kind=press,button=left,content_col=4,box_col=5,box_row=3,box_width=20,box_height=10",
            "kind=drag,button=left,content_col=nil,box_col=nil,box_row=nil,box_width=nil,box_height=nil",
        ]
    );
}

#[test]
fn unbound_wheel_step_scrolls_a_plugin_window() {
    let (layout, _) = two_bands(1, None);
    let mut mouse = Mouse::new(
        "wheel-plugin-scroll",
        &format!(
            "{BAND_WHEEL}gband.bind('alt+o', function() local lines = {{}} for i = 1, 25 do lines[i] = tostring(i) end win = gband.win.open({{ lines = lines, height = 10, focus = false }}) end)"
        ),
        layout,
    );
    mouse.key("alt+o");
    let id: u32 = mouse.global("win");
    let cell = mouse.plugin_float(id);
    mouse.wheel(WheelDirection::Down, cell, Modifiers::NONE);
    assert_eq!(
        mouse.eval::<i64>(&format!("return gband.win.info({id}).top")),
        2
    );
    assert_eq!(mouse.viewed(), mouse.band(0));
}

fn toggle(window: WindowId, after: Option<WindowId>, floating: Option<bool>) -> SessionAction {
    SessionAction::ToggleFloating {
        window,
        after,
        floating,
    }
}

#[test]
fn float_twice_in_one_callback() {
    let (layout, windows) = columns_of(2, None);
    let id = windows[1].0;
    let mut mouse = Mouse::new(
        "float-twice",
        &format!(
            "gband.bind('alt+f', function()
  gband.action.toggle_window_floating({{ window = {id}, floating = true }})
  gband.action.toggle_window_floating({{ window = {id}, floating = true }})
end)"
        ),
        layout,
    );
    let steps = mouse.key("alt+f");
    let float = toggle(windows[1], None, Some(true));
    assert_eq!(sent(&steps), [float.clone(), float]);
    mouse.apply(&steps);
    let band = &mouse.layout.bands()[0];
    assert_eq!(
        band.floating
            .iter()
            .map(|floating| floating.window)
            .collect::<Vec<_>>(),
        [windows[1]]
    );
    assert_eq!(band.columns.len(), 1);
    assert_eq!(band.columns[0].windows, [windows[0]]);
}

#[test]
fn float_a_window_that_already_floats() {
    let (mut layout, windows) = columns_of(2, None);
    float(&mut layout, windows[1], Proportion::ONE_THIRD, 10, 5, 3);
    let id = windows[1].0;
    let mut mouse = Mouse::new(
        "float-floating",
        &format!(
            "gband.bind('alt+f', function() gband.action.toggle_window_floating({{ window = {id}, floating = true }}) end)"
        ),
        layout,
    );
    let steps = mouse.key("alt+f");
    assert_eq!(sent(&steps), [toggle(windows[1], None, Some(true))]);
}

#[test]
fn tile_with_floating_false_fills_after() {
    let (mut layout, windows) = columns_of(3, None);
    float(&mut layout, windows[2], Proportion::ONE_THIRD, 10, 5, 3);
    let id = windows[2].0;
    let mut mouse = Mouse::new(
        "tile-floating-false",
        &format!(
            "gband.bind('alt+t', function() gband.action.toggle_window_floating({{ window = {id}, floating = false }}) end)"
        ),
        layout,
    );
    mouse.click(MouseButton::Left, (45, 3));
    assert_eq!(mouse.display.focused(), Some(windows[1]));
    let steps = mouse.key("alt+t");
    assert_eq!(
        sent(&steps),
        [toggle(windows[2], Some(windows[1]), Some(false))]
    );
}

fn minimizing(name: &str) -> (Mouse, Vec<WindowId>) {
    let (mut layout, windows) = columns_of(3, None);
    float(&mut layout, windows[2], Proportion::ONE_HALF, 12, 10, 4);
    let mut mouse = Mouse::new(
        name,
        "gband.keymap.set('root', 'alt+m', gband.action.minimize_window)",
        layout,
    );
    dispatch(
        &mut mouse.display,
        Action::View(ViewAction::FocusWindow(windows[2])),
    );
    (mouse, windows)
}

#[test]
fn minimize_by_name_sends_no_action_and_a_shown_message_without_it() {
    let (mut mouse, windows) = minimizing("minimize-by-name");
    assert_eq!(
        mouse.display.report_shown(),
        Some(ClientMessage::Shown(vec![
            windows[0], windows[1], windows[2]
        ]))
    );
    assert_eq!(mouse.key("alt+m"), [Step::Nothing]);
    assert_eq!(mouse.display.focused(), Some(windows[0]));
    assert_eq!(
        mouse.display.report_shown(),
        Some(ClientMessage::Shown(vec![windows[0], windows[1]]))
    );
}

#[test]
fn minimized_floating_window_over_a_tile() {
    let (mut mouse, windows) = minimizing("minimize-target");
    let now = mouse.now;
    assert_eq!(
        mouse.display.hit(15, 6, now).target,
        Target::Window(windows[2])
    );
    mouse.key("alt+m");
    assert_eq!(
        mouse.display.hit(15, 6, now).target,
        Target::Window(windows[0])
    );
}

#[test]
fn minimizing_the_dragged_window_ends_the_gesture() {
    let (mut layout, windows) = columns_of(2, None);
    float(&mut layout, windows[1], Proportion::ONE_HALF, 12, 10, 4);
    let mut mouse = dragging("minimize-drag", layout);
    let left = MouseButton::Left;
    mouse.event(MouseKind::Press(left), 20, 6, Modifiers::NONE);
    mouse.event(MouseKind::Motion(Some(left)), 22, 6, Modifiers::NONE);
    mouse.now += Duration::from_millis(40);
    mouse.controls.flush(&mut mouse.display, mouse.now);
    dispatch(
        &mut mouse.display,
        Action::View(ViewAction::Minimize(Some(windows[1]))),
    );
    let mut steps = mouse.event(MouseKind::Motion(Some(left)), 30, 6, Modifiers::NONE);
    steps.extend(mouse.event(MouseKind::Release(left), 30, 6, Modifiers::NONE));
    mouse.now += Duration::from_millis(40);
    steps.extend(mouse.controls.flush(&mut mouse.display, mouse.now));
    assert!(sent(&steps).is_empty(), "{steps:?}");
}

fn two_floating(name: &str, source: &str) -> (Mouse, Vec<WindowId>) {
    let (mut layout, windows) = columns_of(3, None);
    float(&mut layout, windows[1], Proportion::ONE_HALF, 12, 10, 4);
    float(&mut layout, windows[2], Proportion::ONE_HALF, 12, 14, 6);
    let mut mouse = Mouse::new(name, source, layout);
    dispatch(
        &mut mouse.display,
        Action::View(ViewAction::FocusWindow(windows[2])),
    );
    mouse.display.report_shown();
    (mouse, windows)
}

#[test]
fn minimize_an_unfocused_floating_window_by_number() {
    let (mut mouse, windows) = two_floating(
        "minimize-unfocused",
        "gband.bind('alt+m', function() gband.window.minimize(2) end)",
    );
    assert_eq!(mouse.key("alt+m"), [Step::Nothing]);
    assert_eq!(mouse.display.focused(), Some(windows[2]));
    assert_eq!(
        mouse.display.report_shown(),
        Some(ClientMessage::Shown(vec![windows[0], windows[2]]))
    );
}

#[test]
fn minimize_the_focused_window_by_number() {
    let (mut mouse, _) = two_floating(
        "minimize-focused",
        "gband.bind('alt+m', function() gband.window.minimize(2) gband.window.minimize(3) end)",
    );
    mouse.key("alt+m");
    let view: String =
        mouse.eval("local v = gband.view() return v.window .. ' ' .. tostring(v.floating)");
    assert_eq!(view, "1 false");
}

#[test]
fn minimize_a_window_of_another_band_by_number() {
    let mut layout = Layout::new();
    let first = layout.allocate_window();
    layout.open(
        first,
        layout.bands()[0].id,
        None,
        None,
        &LayoutOptions::default(),
    );
    let other = layout.allocate_window();
    layout.open(
        other,
        layout.bands()[1].id,
        None,
        None,
        &LayoutOptions::default(),
    );
    layout.apply(
        SessionAction::ToggleFloating {
            window: other,
            after: None,
            floating: None,
        },
        AREA,
        &LayoutOptions::default(),
    );
    let mut mouse = Mouse::new(
        "minimize-other-band",
        &format!(
            "gband.bind('alt+m', function() gband.window.minimize({other}) gband.band.view(2) end)"
        ),
        layout,
    );
    mouse.key("alt+m");
    let view: String =
        mouse.eval("local v = gband.view() return v.band .. ' ' .. tostring(v.window)");
    assert_eq!(view, "2 nil");
}

#[test]
fn restore_with_focus_by_number() {
    let (mut mouse, windows) = two_floating(
        "minimize-restore",
        "gband.bind('alt+m', function() gband.window.minimize(2) end)
gband.bind('alt+f', function() gband.window.focus(2) end)",
    );
    mouse.key("alt+m");
    let minimized: bool = mouse.eval("return gband.layout().bands[1].floating[1].minimized");
    assert!(minimized);
    mouse.key("alt+f");
    assert_eq!(mouse.display.focused(), Some(windows[1]));
    let now = mouse.now;
    assert_eq!(
        mouse.display.hit(20, 8, now).target,
        Target::Window(windows[1])
    );
    let minimized: Option<bool> =
        mouse.eval("return gband.layout().bands[1].floating[1].minimized");
    assert_eq!(minimized, None);
}

#[test]
fn minimize_then_focus_in_one_function() {
    let (mut mouse, windows) = two_floating(
        "minimize-then-focus",
        "gband.bind('alt+m', function() gband.window.minimize(3) gband.window.focus(3) end)",
    );
    mouse.key("alt+m");
    assert_eq!(mouse.display.focused(), Some(windows[2]));
    assert_eq!(mouse.display.report_shown(), None);
}

const PEEK: &str = "gband.bind('alt+m', function() gband.window.minimize(2) end)
gband.bind('alt+p', function() gband.window.focus(2, { peek = true }) end)
gband.bind('alt+t', function() gband.window.focus(2, { peek = false }) end)
gband.bind('alt+f', function() gband.window.focus(2) end)
gband.bind('alt+n', function() gband.window.focus(3) end)
gband.bind('alt+1', function() gband.window.focus(1) end)
gband.keymap.set('root', 'alt+leftmouse', gband.action.drag_window)
";

#[test]
fn peeked_minimized_window_shown() {
    let (mut mouse, windows) = two_floating("peek-shown", PEEK);
    mouse.key("alt+m");
    mouse.display.report_shown();
    let steps = mouse.key("alt+p");
    assert!(sent(&steps).is_empty(), "{steps:?}");
    assert_eq!(mouse.display.focused(), Some(windows[1]));
    assert_eq!(
        mouse.display.report_shown(),
        Some(ClientMessage::Shown(windows.clone()))
    );
    mouse.key("alt+n");
    assert_eq!(
        mouse.display.report_shown(),
        Some(ClientMessage::Shown(vec![windows[0], windows[2]]))
    );
}

#[test]
fn peeked_minimized_window_over_a_tile() {
    let (mut mouse, windows) = two_floating("peek-over-tile", PEEK);
    mouse.key("alt+m");
    let now = mouse.now;
    assert_eq!(
        mouse.display.hit(11, 5, now).target,
        Target::Window(windows[0])
    );
    mouse.key("alt+p");
    assert_eq!(
        mouse.display.hit(11, 5, now).target,
        Target::Window(windows[1])
    );
}

#[test]
fn peeked_window_over_a_higher_window() {
    let (mut mouse, windows) = two_floating("peek-over-higher", PEEK);
    mouse.key("alt+f");
    mouse.key("alt+n");
    let now = mouse.now;
    assert_eq!(
        mouse.display.hit(20, 8, now).target,
        Target::Window(windows[2])
    );
    mouse.key("alt+p");
    assert_eq!(
        mouse.display.hit(20, 8, now).target,
        Target::Window(windows[1])
    );
}

#[test]
fn drag_from_a_peeked_minimized_window_restores_it() {
    let (mut mouse, windows) = two_floating("peek-drag", PEEK);
    mouse.key("alt+m");
    mouse.key("alt+p");
    mouse.drag_with(MouseButton::Left, (11, 5), (12, 5), Modifiers::ALT);
    assert_eq!(mouse.display.focused(), Some(windows[1]));
    let minimized: Option<bool> =
        mouse.eval("return gband.layout().bands[1].floating[1].minimized");
    assert_eq!(minimized, None);
    assert_eq!(mouse.eval::<Option<bool>>("return gband.view().peek"), None);
}

#[test]
fn peek_a_minimized_window_by_number() {
    let (mut mouse, windows) = two_floating("peek-by-number", PEEK);
    mouse.key("alt+f");
    mouse.key("alt+n");
    mouse.key("alt+m");
    mouse.key("alt+p");
    assert_eq!(mouse.display.focused(), Some(windows[1]));
    let now = mouse.now;
    assert_eq!(
        mouse.display.hit(20, 8, now).target,
        Target::Window(windows[1])
    );
    let minimized: bool = mouse.eval("return gband.layout().bands[1].floating[1].minimized");
    assert!(minimized);
}

#[test]
fn peek_false_focuses() {
    let (mut mouse, windows) = two_floating("peek-false", PEEK);
    mouse.key("alt+m");
    mouse.key("alt+t");
    assert_eq!(mouse.display.focused(), Some(windows[1]));
    let minimized: Option<bool> =
        mouse.eval("return gband.layout().bands[1].floating[1].minimized");
    assert_eq!(minimized, None);
}

#[test]
fn peeked_focus_in_the_view() {
    let (mut mouse, _) = two_floating("peeked-focus", PEEK);
    mouse.key("alt+1");
    mouse.key("alt+p");
    let read = "local v = gband.view() return v.window .. ' ' .. tostring(v.floating) .. ' ' .. tostring(v.peek)";
    assert_eq!(mouse.eval::<String>(read), "2 true true");
    mouse.key("alt+1");
    assert_eq!(mouse.eval::<String>(read), "1 false nil");
}

#[test]
fn peek_keeps_the_last_focus() {
    let (mut mouse, _) = two_floating("peek-last-focus", PEEK);
    mouse.key("alt+f");
    mouse.key("alt+n");
    mouse.key("alt+p");
    let newer: bool = mouse.eval(
        "local f = gband.layout().bands[1].floating return f[2].last_focus > f[1].last_focus",
    );
    assert!(newer);
}

const RECORD_MOVE: &str = "log = {}
function record(_, e)
  local keys = {}
  for _, k in ipairs({ 'kind', 'button', 'line', 'content_col', 'box_col', 'box_row' }) do
    keys[#keys + 1] = k .. '=' .. tostring(e[k])
  end
  log[#log + 1] = table.concat(keys, ',')
end
gband.on('MouseDragged', function() dragged = true end)
";

fn hovering(name: &str, options: &str, source: &str) -> (Mouse, u32) {
    let (layout, _) = columns_of(1, None);
    let mut mouse = Mouse::new(
        name,
        &format!(
            "{RECORD_MOVE}{source}gband.bind('alt+o', function() win = gband.win.open({{ col = 5, row = 3, width = 20, height = 10, lines = {{ 'a', 'b', 'c', 'd' }}, {options} }}) end)"
        ),
        layout,
    );
    mouse.key("alt+o");
    let id = mouse.global("win");
    (mouse, id)
}

impl Mouse {
    fn hover(&mut self, cell: (u16, u16)) -> Vec<Step> {
        self.event(MouseKind::Motion(None), cell.0, cell.1, Modifiers::NONE)
    }

    fn log(&self) -> Vec<String> {
        self.global("log")
    }
}

#[test]
fn hover_a_line() {
    let (mut mouse, id) = hovering("hover-line", "hover = true, on_mouse = record", "");
    let focused = mouse.display.focused();
    for cell in [(40, 12), (7, 6), (8, 6), (8, 6)] {
        mouse.hover(cell);
    }
    assert_eq!(
        mouse.log(),
        [
            "kind=move,button=nil,line=3,content_col=1,box_col=2,box_row=3",
            "kind=move,button=nil,line=3,content_col=2,box_col=3,box_row=3",
        ]
    );
    assert_eq!(mouse.display.focused(), focused);
    assert!(mouse.eval::<bool>(&format!("return gband.win.info({id}).focused")));
    assert_eq!(mouse.global::<Option<bool>>("dragged"), None);
}

#[test]
fn no_move_without_hover() {
    let (mut mouse, _) = hovering("hover-off", "on_mouse = record", "");
    mouse.hover((7, 6));
    mouse.hover((8, 6));
    assert!(mouse.log().is_empty());
}

#[test]
fn hover_in_a_mode() {
    let (mut mouse, _) = hovering(
        "hover-mode",
        "hover = true, focus = false, on_mouse = record",
        NAVIGATION,
    );
    mouse.key("ctrl+space");
    mouse.hover((7, 6));
    assert_eq!(mouse.log().len(), 1);
    assert!(mouse.log()[0].starts_with("kind=move,"));
    assert_eq!(mouse.controls.active_table(), "prefix");
}

#[test]
fn hover_off_the_plugin_window() {
    let (mut mouse, _) = hovering("hover-outside", "hover = true, on_mouse = record", "");
    mouse.hover((30, 6));
    mouse.hover((40, 6));
    assert!(mouse.log().is_empty());
}

#[test]
fn hover_without_on_mouse() {
    let (mut mouse, id) = hovering("hover-plain", "hover = true, cursorline = true", "");
    mouse.hover((7, 6));
    assert_eq!(
        mouse.eval::<i64>(&format!("return gband.win.info({id}).cursor")),
        1
    );
}
