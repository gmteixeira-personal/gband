use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use gband_client::animation::Animations;
use gband_client::color::ColorSupport;
use gband_client::{Controls, Display};
use gband_core::geometry::Size;
use gband_core::layout::{
    Layout, LayoutOptions, Proportion, SessionAction, WindowHeight, WindowId,
};
use gband_lua::keys::parse_key;
use gband_lua::{Config, LoadOptions, Locations, Side};
use gband_protocol::ServerMessage;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::{Color, Modifier};

const TERMINAL: Size = Size::new(80, 24);

fn write(path: &Path, source: &str) -> PathBuf {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
    path.to_path_buf()
}

struct Client {
    display: Display,
    controls: Controls,
    layout: Layout,
    windows: Vec<WindowId>,
    init: PathBuf,
    terminal: Size,
    _scratch: gband_scratch::Scratch,
}

impl Client {
    fn new(name: &str, source: &str, terminal: Size) -> Self {
        let scratch = gband_scratch::Scratch::new("client", &format!("decorations-{name}"));
        let config = scratch.join("config");
        let init = write(&gband_lua::user_file(&config, Side::Client), source);
        write(
            &config.join("user").join("keystyle.lua"),
            "return \"modal\"\n",
        );
        let locations = Locations {
            config,
            plugins: Some(scratch.join("plugins")),
        };
        let loaded: Config =
            gband_lua::load(&locations, Side::Client, &LoadOptions { budget: 5_000_000 })
                .unwrap_or_else(|error| panic!("{error}"));
        let mut display = Display::new(terminal, Animations::Off);
        display.set_colors(ColorSupport::TrueColor);
        let mut controls = Controls::new(loaded, &mut display);
        controls.place_bars(&mut display);
        controls.attached(&mut display, "main");
        Self {
            display,
            controls,
            layout: Layout::new(),
            windows: Vec::new(),
            init,
            terminal,
            _scratch: scratch,
        }
    }

    fn attach(&mut self, columns: usize) {
        let band = self.layout.bands()[0].id;
        for _ in 0..columns {
            let window = self.layout.allocate_window();
            self.layout.open(
                window,
                band,
                self.windows.last().copied(),
                None,
                &LayoutOptions::default(),
            );
            self.windows.push(window);
        }
        self.send_layout();
        self.focus(self.windows[0]);
    }

    fn open_in_band(&mut self, index: usize) {
        let band = self.layout.bands()[index].id;
        let window = self.layout.allocate_window();
        self.layout
            .open(window, band, None, None, &LayoutOptions::default());
        self.windows.push(window);
        self.send_layout();
    }

    fn apply(&mut self, action: SessionAction) {
        let area = self.display.reported_size();
        self.layout.apply(action, area, &LayoutOptions::default());
        self.send_layout();
    }

    fn float(&mut self, window: WindowId) {
        for action in [
            SessionAction::ToggleFloating {
                window,
                after: None,
                floating: None,
            },
            SessionAction::SetWidth {
                window,
                width: Proportion::ONE_HALF,
            },
            SessionAction::SetHeight {
                window,
                height: WindowHeight::Fixed(12),
            },
            SessionAction::SetPosition {
                window,
                col: 20,
                row: 6,
            },
        ] {
            self.apply(action);
        }
    }

    fn send_layout(&mut self) {
        let reported = self.display.reported_size();
        self.controls.receive(
            &mut self.display,
            vec![ServerMessage::Layout {
                cols: reported.cols,
                rows: reported.rows,
                layout: self.layout.clone(),
            }],
        );
    }

    fn focus(&mut self, window: WindowId) {
        self.controls
            .receive(&mut self.display, vec![ServerMessage::Focus(window)]);
    }

    fn press(&mut self, name: &str) {
        self.controls
            .press(&mut self.display, parse_key(name).unwrap());
    }

    fn frame(&mut self) -> Buffer {
        let now = Instant::now();
        self.controls.decorate(&mut self.display, now);
        let mut backend =
            Terminal::new(TestBackend::new(self.terminal.cols, self.terminal.rows)).unwrap();
        backend.draw(|frame| self.display.draw(frame, now)).unwrap();
        backend.backend().buffer().clone()
    }

    fn global<T: mlua::FromLua>(&self, name: &str) -> T {
        self.controls.runtime().lua().globals().get(name).unwrap()
    }

    fn errors(&self) -> Vec<String> {
        self.display.errors().to_vec()
    }
}

fn cells(screen: &Buffer, row: u16, columns: std::ops::Range<u16>) -> String {
    columns.map(|col| screen[(col, row)].symbol()).collect()
}

fn row_text(screen: &Buffer, row: u16) -> String {
    cells(screen, row, 0..screen.area.width)
}

const BUTTONS_ON_FLOATING: &str = "gband.core.provide('decorations', function(info)\n\
     if info.floating then return { '[_]', '[□]', '[X]' } end\n\
     end)\n";

#[test]
fn buttons_on_floating_windows_only() {
    let mut client = Client::new("floating", BUTTONS_ON_FLOATING, TERMINAL);
    client.attach(2);
    client.float(client.windows[1]);
    let screen = client.frame();
    assert_eq!(cells(&screen, 6, 49..60), "[_][□][X]─╮");
    assert!(
        !row_text(&screen, 0).contains('['),
        "{}",
        row_text(&screen, 0)
    );
}

#[test]
fn info_table() {
    let mut client = Client::new(
        "info",
        "seen = {}\n\
         gband.core.provide('decorations', function(info)\n\
           seen[#seen + 1] = string.format('%d %s %s %d', info.window, info.floating, info.focused, info.width)\n\
         end)\n",
        TERMINAL,
    );
    client.attach(2);
    client.focus(client.windows[1]);
    client.frame();
    let mut seen: Vec<String> = client.global("seen");
    seen.sort();
    assert_eq!(
        seen,
        [
            format!("{} false false 40", client.windows[0].0),
            format!("{} false true 40", client.windows[1].0),
        ]
    );
}

#[test]
fn control_characters_removed() {
    let mut client = Client::new(
        "controls",
        "gband.core.provide('decorations', function() return { 'a\\tb' } end)\n",
        Size::new(40, 12),
    );
    client.attach(1);
    let screen = client.frame();
    assert_eq!(cells(&screen, 0, 16..20), "ab─╮");
}

#[test]
fn result_kept_between_frames() {
    let mut client = Client::new(
        "kept",
        "calls = 0\n\
         gband.core.provide('decorations', function() calls = calls + 1 end)\n",
        TERMINAL,
    );
    client.attach(1);
    for _ in 0..3 {
        client.frame();
    }
    assert_eq!(client.global::<i64>("calls"), 1);
}

#[test]
fn called_again_after_a_callback() {
    let mut client = Client::new(
        "again",
        "mark = '[a]'\n\
         gband.core.provide('decorations', function() return { mark } end)\n\
         gband.bind('alt+m', function() mark = '[b]' end)\n",
        TERMINAL,
    );
    client.attach(1);
    assert_eq!(cells(&client.frame(), 0, 35..38), "[a]");
    client.press("alt+m");
    assert_eq!(cells(&client.frame(), 0, 35..38), "[b]");
}

#[test]
fn no_call_without_a_top_side() {
    let mut client = Client::new(
        "no-top",
        "calls = 0\n\
         gband.opt.tile_border_sides = { 'left', 'right', 'bottom' }\n\
         gband.core.provide('decorations', function() calls = calls + 1 end)\n",
        TERMINAL,
    );
    client.attach(1);
    client.frame();
    assert_eq!(client.global::<i64>("calls"), 0);
}

#[test]
fn not_a_callback() {
    let mut client = Client::new(
        "dispatching",
        "gband.core.provide('decorations', function() dispatching = gband.core.dispatching() end)\n",
        TERMINAL,
    );
    client.attach(1);
    client.frame();
    assert!(!client.global::<bool>("dispatching"));
}

#[test]
fn error_in_the_function() {
    let mut client = Client::new(
        "error",
        "gband.core.provide('decorations', function()\n\
         local a = 1\n\
         error('boom')\n\
         end)\n\
         gband.bind('alt+j', gband.action.focus_band_down)\n",
        TERMINAL,
    );
    client.attach(2);
    client.open_in_band(1);
    let screen = client.frame();
    assert_eq!(
        client.errors(),
        [format!("decorations: {}:3: boom", client.init.display())]
    );
    assert!(!row_text(&screen, 0).contains('['));
    client.focus(client.windows[1]);
    client.frame();
    client.press("alt+j");
    assert_eq!(client.display.focused(), Some(client.windows[2]));
    let screen = client.frame();
    assert_eq!(client.errors().len(), 1);
    assert!(!row_text(&screen, 0).contains('['));
}

#[test]
fn wrong_span() {
    let mut client = Client::new(
        "wrong-span",
        "gband.core.provide('decorations', function() return { '[_]', 7 } end)\n",
        TERMINAL,
    );
    client.attach(1);
    let screen = client.frame();
    let errors = client.errors();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].starts_with("decorations: "), "{errors:?}");
    assert!(errors[0].contains("span 2"), "{errors:?}");
    assert!(!row_text(&screen, 0).contains("[_]"));
}

#[test]
fn action_in_the_function() {
    let mut client = Client::new(
        "action",
        "gband.core.provide('decorations', function() gband.action.focus_column_left() end)\n",
        TERMINAL,
    );
    client.attach(2);
    client.focus(client.windows[1]);
    client.frame();
    let errors = client.errors();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].starts_with("decorations: "), "{errors:?}");
    assert_eq!(client.display.focused(), Some(client.windows[1]));
}

#[test]
fn endless_loop() {
    let mut client = Client::new(
        "endless",
        "gband.core.provide('decorations', function() while true do end end)\n\
         gband.bind('alt+k', function() handled = true end)\n",
        TERMINAL,
    );
    client.attach(1);
    client.frame();
    let errors = client.errors();
    assert_eq!(errors.len(), 1);
    assert!(errors[0].starts_with("decorations: "), "{errors:?}");
    assert!(errors[0].contains("instruction limit"), "{errors:?}");
    client.press("alt+k");
    assert!(client.global::<bool>("handled"));
}

#[test]
fn registered_again() {
    let mut client = Client::new(
        "registered",
        "gband.core.provide('decorations', function() error('boom') end)\n\
         gband.bind('alt+r', function()\n\
           gband.core.provide('decorations', function() return { '[X]' } end)\n\
         end)\n",
        TERMINAL,
    );
    client.attach(2);
    client.frame();
    assert_eq!(client.errors().len(), 1);
    client.press("alt+r");
    let screen = client.frame();
    assert_eq!(cells(&screen, 0, 35..38), "[X]");
    assert_eq!(cells(&screen, 0, 75..78), "[X]");
}

#[test]
fn highlight_group_as_a_style() {
    let mut client = Client::new(
        "group",
        "gband.hl.set('CloseButton', { fg = 'red' })\n\
         gband.core.provide('decorations', function(info)\n\
           if info.focused then\n\
             return { { text = '[X]', style = require('gband.hl').drawn('CloseButton') } }\n\
           end\n\
         end)\n",
        TERMINAL,
    );
    client.attach(1);
    let screen = client.frame();
    let span = &screen[(36, 0)];
    assert_eq!(span.symbol(), "X");
    assert_eq!(span.fg, Color::Indexed(1));
    assert!(span.modifier.contains(Modifier::BOLD));
}
