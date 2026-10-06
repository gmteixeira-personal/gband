use std::fs;

use gband_client::animation::Animations;
use gband_client::{Controls, Display, Step};
use gband_core::geometry::Size;
use gband_core::layout::{
    BandId, Layout, LayoutOptions, Proportion, SessionAction, WindowContent, WindowId,
};
use gband_emulator::{Emulator, Grid};
use gband_lua::keys::parse_key;
use gband_lua::{Config, DEFAULTS, LoadOptions, Locations};
use gband_protocol::{ClientMessage, ServerMessage};

struct Scratch(gband_scratch::Scratch);

impl Scratch {
    fn new(name: &str) -> Self {
        Self(gband_scratch::Scratch::new(
            "client",
            &format!("windows-{name}"),
        ))
    }

    fn load(&self, source: &str) -> Config {
        let path = gband_lua::user_file(&self.0, gband_lua::Side::Client);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path.with_file_name("keystyle.lua"), "return \"modal\"\n").unwrap();
        fs::write(
            &path,
            format!(
                "{}\n{source}",
                DEFAULTS.replace("gband.plugin(\"gband.sidebar\")", "")
            ),
        )
        .unwrap();
        let locations = Locations {
            config: self.0.to_path_buf(),
            plugins: None,
        };
        gband_lua::load(&locations, gband_lua::Side::Client, &LoadOptions::default()).unwrap()
    }
}

struct Client {
    scratch: Scratch,
    display: Display,
    controls: Controls,
    layout: Layout,
    windows: Vec<WindowId>,
}

impl Client {
    fn new(name: &str, source: &str) -> Self {
        let scratch = Scratch::new(name);
        let config = scratch.load(source);
        let mut display = Display::new(Size::new(80, 24), Animations::Off);
        let mut controls = Controls::new(config, &mut display);
        let mut layout = Layout::new();
        let band = layout.bands()[0].id;
        let mut windows = Vec::new();
        for _ in 0..2 {
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
            windows,
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

    fn open_plugin_window(&mut self, request: u32) -> (WindowId, Vec<Step>) {
        let window = self.layout.allocate_window();
        let band = self.layout.bands()[0].id;
        self.layout.open(
            window,
            band,
            Some(self.windows[0]),
            Some(Proportion::new(1, 4)),
            &LayoutOptions::default(),
        );
        let steps = self.receive(vec![
            self.layout_message(),
            ServerMessage::Snapshot {
                window,
                cols: 18,
                rows: 22,
                contents: Vec::new(),
            },
            ServerMessage::Opened {
                request,
                window: Some(window),
            },
            ServerMessage::Focus(window),
        ]);
        (window, steps)
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
            window: focused,
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

const TAKES_TEXT: &str = "gband.bind('alt+o', function() win = gband.win.open({ on_input = function(id, text) got = { id, text } end }) end)\n";

impl Client {
    fn paste(&mut self, text: &str) -> Vec<Step> {
        self.controls.paste(&mut self.display, text.to_owned())
    }

    fn got(&self) -> (u32, String) {
        let got: mlua::Table = self.global("got");
        (got.get(1).unwrap(), got.get(2).unwrap())
    }
}

#[test]
fn paste_into_a_plugin_window_that_takes_text() {
    let mut client = Client::new("paste-text", TAKES_TEXT);
    client.press("alt+o");
    let steps = client.paste("hello");
    assert!(sent(&steps).is_empty(), "{steps:?}");
    assert_eq!(client.got(), (client.global("win"), "hello".to_owned()));
}

#[test]
fn paste_on_a_band_with_no_window() {
    let mut client = Client::new("paste-empty-band", TAKES_TEXT);
    client.press("ctrl+space");
    client.press("u");
    client.press("escape");
    assert_eq!(client.display.focused(), None);
    client.press("alt+o");
    let steps = client.paste("hi");
    assert!(sent(&steps).is_empty(), "{steps:?}");
    assert_eq!(client.got(), (client.global("win"), "hi".to_owned()));
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
fn navigation_mode_before_the_plugin_window() {
    let mut client = Client::new(
        "navigation-first",
        "gband.bind('alt+o', function() gband.win.open({ keys = { x = function() pressed = true end } }) end)\n",
    );
    client.press("alt+o");
    client.press("ctrl+space");
    no_input(&client.press("x"));
    assert!(client.global::<Option<bool>>("pressed").is_none());
    assert_eq!(client.controls.active_table(), "prefix");
}

#[test]
fn bindings_still_win() {
    let mut client = Client::new(
        "bindings-win",
        "gband.bind('alt+o', function() gband.win.open({ keys = { h = function() pressed = true end } }) end)\n",
    );
    client.press("ctrl+space");
    client.press("l");
    client.press("escape");
    client.press("alt+o");
    assert!(client.press("ctrl+space").is_empty());
    no_input(&client.press("h"));
    assert_eq!(client.display.focused(), Some(client.windows[0]));
    assert!(client.global::<Option<bool>>("pressed").is_none());
}

const CLOSABLE: &str = "gband.bind('alt+o', function() win = gband.win.open({ keys = { q = function(id) pressed = id end }, on_close = function(id) closed = id end }) end)\ngband.bind('alt+n', function() win = gband.win.open({ on_close = function(id) closed = id end }) end)\ngband.bind('alt+c', function() gband.action.close_window() end)\ngband.bind('alt+t', function() gband.action.close_window({ window = target }) end)\n";

#[test]
fn q_closes_a_float() {
    let mut client = Client::new("q-closes", CLOSABLE);
    client.press("alt+n");
    let plugin_window: u32 = client.global("win");
    let steps = client.press("q");
    assert!(sent(&steps).is_empty(), "{steps:?}");
    assert!(client.display.plugin_windows().floats().is_empty());
    assert_eq!(client.global::<u32>("closed"), plugin_window);
}

#[test]
fn keys_entry_for_q_wins() {
    let mut client = Client::new("q-entry", CLOSABLE);
    client.press("alt+o");
    let plugin_window: u32 = client.global("win");
    no_input(&client.press("q"));
    assert_eq!(client.global::<u32>("pressed"), plugin_window);
    assert_eq!(client.display.plugin_windows().floats().len(), 1);
}

#[test]
fn prefix_q_closes_the_focused_float() {
    let mut client = Client::new("prefix-q", CLOSABLE);
    let focused = client.display.focused();
    client.press("alt+o");
    let plugin_window: u32 = client.global("win");
    client.press("ctrl+space");
    let steps = client.press("q");
    assert!(sent(&steps).is_empty(), "{steps:?}");
    assert!(client.display.plugin_windows().floats().is_empty());
    assert_eq!(client.display.focused(), focused);
    assert_eq!(client.global::<u32>("closed"), plugin_window);
    assert!(client.global::<Option<u32>>("pressed").is_none());
}

#[test]
fn close_window_from_a_binding_function_closes_the_focused_float() {
    let mut client = Client::new("close-action", CLOSABLE);
    client.press("alt+n");
    let plugin_window: u32 = client.global("win");
    let steps = client.press("alt+c");
    assert!(sent(&steps).is_empty(), "{steps:?}");
    assert!(client.display.plugin_windows().floats().is_empty());
    assert_eq!(client.global::<u32>("closed"), plugin_window);
    assert_eq!(
        client.press("alt+c"),
        [Step::Send(ClientMessage::Action(
            SessionAction::CloseWindow(client.windows[0])
        ))]
    );
}

#[test]
fn prefix_q_closes_a_float_on_the_empty_band() {
    let mut client = Client::new("prefix-q-empty", CLOSABLE);
    client.press("ctrl+space");
    client.press("u");
    client.press("escape");
    assert_eq!(client.display.focused(), None);
    client.press("alt+n");
    assert_eq!(client.display.plugin_windows().floats().len(), 1);
    client.press("ctrl+space");
    let steps = client.press("q");
    assert!(sent(&steps).is_empty(), "{steps:?}");
    assert!(client.display.plugin_windows().floats().is_empty());
    assert!(sent(&client.press("q")).is_empty());
}

#[test]
fn targeted_close_window_leaves_the_focused_float_open() {
    let mut client = Client::new("targeted-close", CLOSABLE);
    let target = client.windows[1];
    client
        .controls
        .runtime()
        .lua()
        .globals()
        .set("target", target.0)
        .unwrap();
    client.press("alt+n");
    assert_eq!(
        sent(&client.press("alt+t")),
        [&ClientMessage::Action(SessionAction::CloseWindow(target))]
    );
    assert_eq!(client.display.plugin_windows().floats().len(), 1);
    assert!(client.global::<Option<u32>>("closed").is_none());
}

#[test]
fn moving_focus_leaves_the_float() {
    let mut client = Client::new("leave-float", FLOAT);
    client.press("alt+o");
    client.press("ctrl+space");
    client.press("l");
    assert_eq!(client.display.focused(), Some(client.windows[1]));
    assert_eq!(client.display.plugin_windows().floats().len(), 1);
    assert_eq!(client.display.plugin_windows().focused_float(), None);
    assert_eq!(client.display.focused_plugin_window(), None);
    client.press("escape");
    let steps = client.press("x");
    assert_eq!(
        sent(&steps),
        [&ClientMessage::Key {
            window: client.windows[1],
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

const WINDOW: &str = "gband.bind('alt+p', function() win = gband.win.open({ kind = 'tiled', lines = { 'hello' }, column_width = 1/4, keys = { j = function() pressed = true end }, on_close = function(id) closed = id end }) end)\n";

#[test]
fn q_in_a_tiled_plugin_window_is_discarded() {
    let mut client = Client::new("tiled-q", WINDOW);
    client.press("alt+p");
    let plugin_window: u32 = client.global("win");
    let (window, _) = client.open_plugin_window(plugin_window);
    assert_eq!(client.display.focused_plugin_window(), Some(plugin_window));
    let steps = client.press("q");
    assert!(sent(&steps).is_empty(), "{steps:?}");
    assert!(client.global::<Option<u32>>("closed").is_none());
    assert_eq!(client.display.focused(), Some(window));
    assert_eq!(client.display.focused_plugin_window(), Some(plugin_window));
}

#[test]
fn tiled_plugin_window_opens_and_sends_its_contents() {
    let mut client = Client::new("window-window", WINDOW);
    let steps = client.press("alt+p");
    let plugin_window: u32 = client.global("win");
    assert_eq!(
        sent(&steps),
        [&ClientMessage::Action(SessionAction::OpenWindow {
            band: BandId(1),
            after: Some(client.windows[0]),
            width: Some(Proportion::new(1, 4)),
            floating: false,
            focus: true,
            content: WindowContent::Plugin {
                request: plugin_window
            },
        })]
    );
    let (window, steps) = client.open_plugin_window(plugin_window);
    let [
        ClientMessage::Content {
            window: target,
            output,
        },
    ] = sent(&steps)[..]
    else {
        panic!("{steps:?}");
    };
    assert_eq!(*target, window);
    let mut grid = Grid::new(Size::new(18, 22));
    grid.process(output);
    assert_eq!(grid.contents().lines().next().unwrap().trim_end(), "hello");
    assert_eq!(grid.cursor(), None);
    assert_eq!(client.display.focused(), Some(window));
    assert_eq!(client.display.focused_plugin_window(), Some(plugin_window));
    no_input(&client.press("j"));
    assert!(client.global::<bool>("pressed"));
}

#[test]
fn resized_tiled_plugin_window_is_drawn_again() {
    let mut client = Client::new("window-resized", WINDOW);
    client.press("alt+p");
    let plugin_window: u32 = client.global("win");
    let (window, _) = client.open_plugin_window(plugin_window);
    let steps = client.receive(vec![ServerMessage::Snapshot {
        window,
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
                window,
                contents: b"x".to_vec(),
            }])
            .is_empty()
    );
}

#[test]
fn tiled_plugin_window_closed_by_the_user() {
    let mut client = Client::new("window-closed", WINDOW);
    client.press("alt+p");
    let plugin_window: u32 = client.global("win");
    let (window, _) = client.open_plugin_window(plugin_window);
    client.press("ctrl+space");
    assert_eq!(
        client.press("q"),
        [Step::Send(ClientMessage::Action(
            SessionAction::CloseWindow(window)
        ))]
    );
    client.layout.remove(window);
    let message = client.layout_message();
    client.receive(vec![message]);
    assert_eq!(client.global::<u32>("closed"), plugin_window);
    assert_eq!(
        client.display.plugin_windows().plugin_window_of(window),
        None
    );
}

#[test]
fn closing_a_tiled_plugin_window_closes_its_window() {
    let mut client = Client::new(
        "window-close",
        &format!("{WINDOW}gband.bind('alt+c', function() gband.win.close(win) end)\n"),
    );
    client.press("alt+p");
    let plugin_window: u32 = client.global("win");
    let (window, _) = client.open_plugin_window(plugin_window);
    assert_eq!(
        sent(&client.press("alt+c")),
        [&ClientMessage::Action(SessionAction::CloseWindow(window))]
    );
    assert_eq!(client.global::<u32>("closed"), plugin_window);
}

#[test]
fn plugin_window_closed_before_the_window_is_known() {
    let mut client = Client::new(
        "window-early-close",
        &format!("{WINDOW}gband.bind('alt+c', function() gband.win.close(win) end)\n"),
    );
    client.press("alt+p");
    let plugin_window: u32 = client.global("win");
    assert!(sent(&client.press("alt+c")).is_empty());
    let (window, steps) = client.open_plugin_window(plugin_window);
    assert_eq!(
        sent(&steps),
        [&ClientMessage::Action(SessionAction::CloseWindow(window))]
    );
}

#[test]
fn opened_with_no_window_closes_the_plugin_window() {
    let mut client = Client::new("window-none", WINDOW);
    client.press("alt+p");
    let plugin_window: u32 = client.global("win");
    let steps = client.receive(vec![ServerMessage::Opened {
        request: plugin_window,
        window: None,
    }]);
    assert!(sent(&steps).is_empty());
    assert_eq!(client.global::<u32>("closed"), plugin_window);
}

#[test]
fn reload_closes_every_plugin_window() {
    let mut client = Client::new("reload", &format!("{FLOAT}{WINDOW}"));
    client.press("alt+o");
    client.press("alt+p");
    let plugin_window: u32 = client.global("win");
    let (window, _) = client.open_plugin_window(plugin_window);
    let config = client.scratch.load(&format!("{FLOAT}{WINDOW}"));
    let steps = client.controls.reload(&mut client.display, Ok(config));
    assert!(
        sent(&steps).contains(&&ClientMessage::Action(SessionAction::CloseWindow(window))),
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
fn orphaned_opened_reply_is_answered_with_close_window() {
    let mut client = Client::new("orphan", WINDOW);
    client.press("alt+p");
    let plugin_window: u32 = client.global("win");
    let config = client.scratch.load(WINDOW);
    client.controls.reload(&mut client.display, Ok(config));
    let (window, steps) = client.open_plugin_window(plugin_window);
    assert_eq!(
        sent(&steps),
        [&ClientMessage::Action(SessionAction::CloseWindow(window))]
    );
}

const RUNNER: &str = "gband.bind('alt+o', function() win = gband.win.open({ keys = { enter = function() gband.action.focus_column_right() end, o = function() gband.action.open_window() end } }) end)\n";

#[test]
fn action_from_the_floats_own_key_keeps_it_focused() {
    let mut client = Client::new("own-key", RUNNER);
    client.press("alt+o");
    let plugin_window: u32 = client.global("win");
    no_input(&client.press("enter"));
    assert_eq!(client.display.focused(), Some(client.windows[1]));
    assert_eq!(client.display.focused_plugin_window(), Some(plugin_window));
    client.press("ctrl+space");
    client.press("h");
    assert_eq!(client.display.focused(), Some(client.windows[0]));
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
            Some(client.windows[0]),
            None
        ))]
    );
    let opened = client.layout.allocate_window();
    let band = client.layout.bands()[0].id;
    client.layout.open(
        opened,
        band,
        Some(client.windows[0]),
        None,
        &LayoutOptions::default(),
    );
    let layout = client.layout_message();
    client.receive(vec![layout, ServerMessage::Focus(opened)]);
    assert_eq!(client.display.focused(), Some(opened));
    assert_eq!(client.display.focused_plugin_window(), Some(plugin_window));
    no_input(&client.press("down"));
    client.receive(vec![ServerMessage::Focus(client.windows[1])]);
    assert_eq!(client.display.plugin_windows().focused_float(), None);
}
