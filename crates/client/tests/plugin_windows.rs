use std::fs;
use std::path::PathBuf;

use gband_client::animation::Animations;
use gband_client::{Controls, Display, Step};
use gband_core::geometry::Size;
use gband_core::layout::{
    BandId, Layout, LayoutOptions, PaneContent, PaneId, Proportion, SessionAction,
};
use gband_emulator::{Emulator, Grid};
use gband_lua::keys::parse_key;
use gband_lua::{Config, DEFAULTS, LoadOptions, Locations};
use gband_protocol::{ClientMessage, ServerMessage};

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "gband-client-windows-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn load(&self, source: &str) -> Config {
        let path = gband_lua::user_file(&self.0, gband_lua::Side::Client);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            format!("{DEFAULTS}\ngband.opt.statusline_position = 'off'\n{source}"),
        )
        .unwrap();
        let locations = Locations {
            config: self.0.clone(),
            plugins: None,
        };
        gband_lua::load(&locations, gband_lua::Side::Client, &LoadOptions::default()).unwrap()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Client {
    scratch: Scratch,
    display: Display,
    controls: Controls,
    layout: Layout,
    panes: Vec<PaneId>,
}

impl Client {
    fn new(name: &str, source: &str) -> Self {
        let scratch = Scratch::new(name);
        let config = scratch.load(source);
        let mut display = Display::new(Size::new(80, 24), Animations::Off);
        let mut controls = Controls::new(config, &mut display);
        let mut layout = Layout::new();
        let band = layout.bands()[0].id;
        let mut panes = Vec::new();
        for _ in 0..2 {
            let pane = layout.allocate_pane();
            layout.open(
                pane,
                band,
                panes.last().copied(),
                None,
                &LayoutOptions::default(),
            );
            panes.push(pane);
        }
        controls.attached(&mut display, "main");
        let received = controls.receive(
            &mut display,
            [ServerMessage::Layout {
                cols: 80,
                rows: 24,
                layout: layout.clone(),
            }],
        );
        assert!(received.outcome.is_none());
        Self {
            scratch,
            display,
            controls,
            layout,
            panes,
        }
    }

    fn press(&mut self, name: &str) -> Vec<Step> {
        self.controls
            .press(&mut self.display, parse_key(name).unwrap())
    }

    fn receive(&mut self, messages: Vec<ServerMessage>) -> Vec<Step> {
        let received = self.controls.receive(&mut self.display, messages);
        assert!(received.outcome.is_none());
        received.steps
    }

    fn layout_message(&self) -> ServerMessage {
        ServerMessage::Layout {
            cols: 80,
            rows: 24,
            layout: self.layout.clone(),
        }
    }

    fn open_plugin_pane(&mut self, request: u32) -> (PaneId, Vec<Step>) {
        let pane = self.layout.allocate_pane();
        let band = self.layout.bands()[0].id;
        self.layout.open(
            pane,
            band,
            Some(self.panes[0]),
            Some(Proportion::new(1, 4)),
            &LayoutOptions::default(),
        );
        let steps = self.receive(vec![
            self.layout_message(),
            ServerMessage::Snapshot {
                pane,
                cols: 18,
                rows: 22,
                contents: Vec::new(),
            },
            ServerMessage::Opened {
                request,
                pane: Some(pane),
            },
            ServerMessage::Focus(pane),
        ]);
        (pane, steps)
    }

    fn global<T: mlua::FromLua>(&self, name: &str) -> T {
        self.controls.runtime().lua().globals().get(name).unwrap()
    }
}

fn sent(steps: &[Step]) -> Vec<&ClientMessage> {
    steps
        .iter()
        .filter_map(|step| match step {
            Step::Send(message) => Some(message),
            _ => None,
        })
        .collect()
}

fn no_input(steps: &[Step]) {
    assert!(
        !sent(steps).iter().any(|message| matches!(
            message,
            ClientMessage::Key { .. } | ClientMessage::Paste { .. }
        )),
        "{steps:?}"
    );
}

const FLOAT: &str = "gband.bind('alt+o', function() win = gband.win.open({ keys = { j = function(id) pressed = id end, ['alt+h'] = function() float_alt = true end } }) end)\n";

#[test]
fn typing_into_a_focused_float_sends_nothing() {
    let mut client = Client::new("typing", FLOAT);
    client.press("alt+o");
    assert!(client.display.focused_plugin_window().is_some());
    for key in ["l", "s", "enter"] {
        no_input(&client.press(key));
    }
}

#[test]
fn paste_into_a_focused_float_is_discarded() {
    let mut client = Client::new("paste", FLOAT);
    let focused = client.display.focused().unwrap();
    assert_eq!(
        client
            .controls
            .paste(&mut client.display, "hello".to_owned()),
        [Step::Send(ClientMessage::Paste {
            pane: focused,
            text: "hello".to_owned(),
        })]
    );
    client.press("alt+o");
    assert!(
        client
            .controls
            .paste(&mut client.display, "hello".to_owned())
            .is_empty()
    );
}

#[test]
fn unbound_key_goes_to_the_focused_plugin_window() {
    let mut client = Client::new("window-key", FLOAT);
    client.press("alt+o");
    no_input(&client.press("j"));
    assert_eq!(client.global::<u32>("pressed"), client.global::<u32>("win"));
}

#[test]
fn root_binding_runs_before_the_plugin_window() {
    let mut client = Client::new(
        "root-first",
        &format!("gband.bind('alt+h', function() root_alt = true end)\n{FLOAT}"),
    );
    client.press("alt+o");
    client.press("alt+h");
    assert!(client.global::<bool>("root_alt"));
    assert!(client.global::<Option<bool>>("float_alt").is_none());
}

#[test]
fn bindings_still_win() {
    let mut client = Client::new("bindings-win", FLOAT);
    client.press("alt+o");
    assert!(client.press("ctrl+space").is_empty());
    let focused = client.display.focused().unwrap();
    assert_eq!(
        client.press("q"),
        [Step::Send(ClientMessage::Action(SessionAction::ClosePane(
            focused
        )))]
    );
    assert!(client.global::<Option<u32>>("pressed").is_none());
}

#[test]
fn moving_focus_leaves_the_float() {
    let mut client = Client::new("leave-float", FLOAT);
    client.press("alt+o");
    client.press("ctrl+space");
    client.press("l");
    assert_eq!(client.display.focused(), Some(client.panes[1]));
    assert_eq!(client.display.plugin_windows().floats().len(), 1);
    assert_eq!(client.display.plugin_windows().focused_float(), None);
    assert_eq!(client.display.focused_plugin_window(), None);
    let steps = client.press("x");
    assert_eq!(
        sent(&steps),
        [&ClientMessage::Key {
            pane: client.panes[1],
            key: parse_key("x").unwrap(),
        }]
    );
}

#[test]
fn escape_closes_a_float() {
    let mut client = Client::new("escape", FLOAT);
    client.press("alt+o");
    no_input(&client.press("escape"));
    assert!(client.display.plugin_windows().floats().is_empty());
    assert_eq!(client.display.focused_plugin_window(), None);
}

const PANE: &str = "gband.bind('alt+p', function() win = gband.win.open({ kind = 'tiled', lines = { 'hello' }, column_width = 1/4, keys = { j = function() pressed = true end }, on_close = function(id) closed = id end }) end)\n";

#[test]
fn tiled_plugin_window_opens_and_sends_its_contents() {
    let mut client = Client::new("pane-window", PANE);
    let steps = client.press("alt+p");
    let plugin_window: u32 = client.global("win");
    assert_eq!(
        sent(&steps),
        [&ClientMessage::Action(SessionAction::OpenPane {
            band: BandId(1),
            after: Some(client.panes[0]),
            width: Some(Proportion::new(1, 4)),
            floating: false,
            focus: true,
            content: PaneContent::Plugin {
                request: plugin_window
            },
        })]
    );
    let (pane, steps) = client.open_plugin_pane(plugin_window);
    let [
        ClientMessage::Content {
            pane: target,
            output,
        },
    ] = sent(&steps)[..]
    else {
        panic!("{steps:?}");
    };
    assert_eq!(*target, pane);
    let mut grid = Grid::new(Size::new(18, 22));
    grid.process(output);
    assert_eq!(grid.contents().lines().next().unwrap().trim_end(), "hello");
    assert_eq!(grid.cursor(), None);
    assert_eq!(client.display.focused(), Some(pane));
    assert_eq!(client.display.focused_plugin_window(), Some(plugin_window));
    no_input(&client.press("j"));
    assert!(client.global::<bool>("pressed"));
}

#[test]
fn resized_tiled_plugin_window_is_drawn_again() {
    let mut client = Client::new("pane-resized", PANE);
    client.press("alt+p");
    let plugin_window: u32 = client.global("win");
    let (pane, _) = client.open_plugin_pane(plugin_window);
    let steps = client.receive(vec![ServerMessage::Snapshot {
        pane,
        cols: 30,
        rows: 22,
        contents: Vec::new(),
    }]);
    let [ClientMessage::Content { output, .. }] = sent(&steps)[..] else {
        panic!("{steps:?}");
    };
    let mut grid = Grid::new(Size::new(30, 22));
    grid.process(output);
    assert_eq!(grid.contents().lines().next().unwrap().trim_end(), "hello");
    assert!(
        client
            .receive(vec![ServerMessage::Update {
                pane,
                contents: b"x".to_vec(),
            }])
            .is_empty()
    );
}

#[test]
fn tiled_plugin_window_closed_by_the_user() {
    let mut client = Client::new("pane-closed", PANE);
    client.press("alt+p");
    let plugin_window: u32 = client.global("win");
    let (pane, _) = client.open_plugin_pane(plugin_window);
    client.press("ctrl+space");
    assert_eq!(
        client.press("q"),
        [Step::Send(ClientMessage::Action(SessionAction::ClosePane(
            pane
        )))]
    );
    client.layout.remove(pane);
    let message = client.layout_message();
    client.receive(vec![message]);
    assert_eq!(client.global::<u32>("closed"), plugin_window);
    assert_eq!(client.display.plugin_windows().plugin_window_of(pane), None);
}

#[test]
fn closing_a_tiled_plugin_window_closes_its_pane() {
    let mut client = Client::new(
        "pane-close",
        &format!("{PANE}gband.bind('alt+c', function() gband.win.close(win) end)\n"),
    );
    client.press("alt+p");
    let plugin_window: u32 = client.global("win");
    let (pane, _) = client.open_plugin_pane(plugin_window);
    assert_eq!(
        sent(&client.press("alt+c")),
        [&ClientMessage::Action(SessionAction::ClosePane(pane))]
    );
    assert_eq!(client.global::<u32>("closed"), plugin_window);
}

#[test]
fn plugin_window_closed_before_the_pane_is_known() {
    let mut client = Client::new(
        "pane-early-close",
        &format!("{PANE}gband.bind('alt+c', function() gband.win.close(win) end)\n"),
    );
    client.press("alt+p");
    let plugin_window: u32 = client.global("win");
    assert!(sent(&client.press("alt+c")).is_empty());
    let (pane, steps) = client.open_plugin_pane(plugin_window);
    assert_eq!(
        sent(&steps),
        [&ClientMessage::Action(SessionAction::ClosePane(pane))]
    );
}

#[test]
fn opened_with_no_pane_closes_the_plugin_window() {
    let mut client = Client::new("pane-none", PANE);
    client.press("alt+p");
    let plugin_window: u32 = client.global("win");
    let steps = client.receive(vec![ServerMessage::Opened {
        request: plugin_window,
        pane: None,
    }]);
    assert!(sent(&steps).is_empty());
    assert_eq!(client.global::<u32>("closed"), plugin_window);
}

#[test]
fn reload_closes_every_plugin_window() {
    let mut client = Client::new("reload", &format!("{FLOAT}{PANE}"));
    client.press("alt+o");
    client.press("alt+p");
    let plugin_window: u32 = client.global("win");
    let (pane, _) = client.open_plugin_pane(plugin_window);
    let config = client.scratch.load(&format!("{FLOAT}{PANE}"));
    let steps = client.controls.reload(&mut client.display, Ok(config));
    assert!(
        sent(&steps).contains(&&ClientMessage::Action(SessionAction::ClosePane(pane))),
        "{steps:?}"
    );
    assert!(client.display.plugin_windows().floats().is_empty());
    assert!(client.global::<Option<u32>>("closed").is_none());
    let listed: Vec<u32> = client
        .controls
        .runtime()
        .lua()
        .load("return gband.win.list()")
        .eval()
        .unwrap();
    assert!(listed.is_empty());
    client.press("alt+o");
    assert!(client.global::<u32>("win") > plugin_window);
}

#[test]
fn orphaned_opened_reply_is_answered_with_close_pane() {
    let mut client = Client::new("orphan", PANE);
    client.press("alt+p");
    let plugin_window: u32 = client.global("win");
    let config = client.scratch.load(PANE);
    client.controls.reload(&mut client.display, Ok(config));
    let (pane, steps) = client.open_plugin_pane(plugin_window);
    assert_eq!(
        sent(&steps),
        [&ClientMessage::Action(SessionAction::ClosePane(pane))]
    );
}

const RUNNER: &str = "gband.bind('alt+o', function() win = gband.win.open({ keys = { enter = function() gband.action.focus_column_right() end, o = function() gband.action.open_pane() end } }) end)\n";

#[test]
fn action_from_the_floats_own_key_keeps_it_focused() {
    let mut client = Client::new("own-key", RUNNER);
    client.press("alt+o");
    let plugin_window: u32 = client.global("win");
    no_input(&client.press("enter"));
    assert_eq!(client.display.focused(), Some(client.panes[1]));
    assert_eq!(client.display.focused_plugin_window(), Some(plugin_window));
    client.press("ctrl+space");
    client.press("h");
    assert_eq!(client.display.focused(), Some(client.panes[0]));
    assert_eq!(client.display.plugin_windows().focused_float(), None);
}

#[test]
fn focus_from_the_server_after_the_floats_own_key_keeps_it_focused() {
    let mut client = Client::new("own-key-server", RUNNER);
    client.press("alt+o");
    let plugin_window: u32 = client.global("win");
    let steps = client.press("o");
    assert_eq!(
        sent(&steps),
        [&ClientMessage::Action(SessionAction::open(
            BandId(1),
            Some(client.panes[0]),
            None
        ))]
    );
    let opened = client.layout.allocate_pane();
    let band = client.layout.bands()[0].id;
    client.layout.open(
        opened,
        band,
        Some(client.panes[0]),
        None,
        &LayoutOptions::default(),
    );
    let layout = client.layout_message();
    client.receive(vec![layout, ServerMessage::Focus(opened)]);
    assert_eq!(client.display.focused(), Some(opened));
    assert_eq!(client.display.focused_plugin_window(), Some(plugin_window));
    no_input(&client.press("down"));
    client.receive(vec![ServerMessage::Focus(client.panes[1])]);
    assert_eq!(client.display.plugin_windows().focused_float(), None);
}
