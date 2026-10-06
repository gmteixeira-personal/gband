use std::fs;
use std::path::Path;
use std::time::Instant;

use gband_client::animation::Animations;
use gband_client::color::ColorSupport;
use gband_client::{Controls, Display};
use gband_core::geometry::Size;
use gband_core::layout::{Layout, LayoutOptions};
use gband_lua::keys::parse_key;
use gband_lua::{Config, LoadOptions, Locations, Side};
use gband_protocol::ServerMessage;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::{Color, Modifier};

const TERMINAL: Size = Size::new(80, 24);

fn load(name: &str, source: &str) -> (gband_scratch::Scratch, Config) {
    let scratch = gband_scratch::Scratch::new("client", &format!("look-{name}"));
    let config = scratch.join("config");
    for (path, text) in [
        (gband_lua::user_file(&config, Side::Client), source),
        (
            config.join("user").join("keystyle.lua"),
            "return \"modal\"\n",
        ),
    ] {
        write(&path, text);
    }
    let locations = Locations {
        config,
        plugins: Some(scratch.join("plugins")),
    };
    let loaded = gband_lua::load(&locations, Side::Client, &LoadOptions::default());
    (scratch, loaded.unwrap_or_else(|error| panic!("{error}")))
}

fn write(path: &Path, source: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
}

struct Client {
    display: Display,
    controls: Controls,
    _scratch: gband_scratch::Scratch,
}

impl Client {
    fn attached((scratch, config): (gband_scratch::Scratch, Config)) -> Self {
        let mut display = Display::new(TERMINAL, Animations::Off);
        display.set_colors(ColorSupport::TrueColor);
        let mut controls = Controls::new(config, &mut display);
        controls.place_bars(&mut display);
        controls.attached(&mut display, "main");
        let mut layout = Layout::new();
        let band = layout.bands()[0].id;
        let window = layout.allocate_window();
        layout.open(window, band, None, None, &LayoutOptions::default());
        let reported = display.reported_size();
        controls.receive(
            &mut display,
            vec![
                ServerMessage::Layout {
                    cols: reported.cols,
                    rows: reported.rows,
                    layout,
                },
                ServerMessage::Focus(window),
            ],
        );
        Self {
            display,
            controls,
            _scratch: scratch,
        }
    }

    fn press(&mut self, name: &str) {
        self.controls
            .press(&mut self.display, parse_key(name).unwrap());
    }

    fn screen(&mut self) -> Buffer {
        let mut backend = Terminal::new(TestBackend::new(TERMINAL.cols, TERMINAL.rows)).unwrap();
        backend
            .draw(|frame| self.display.draw(frame, Instant::now()))
            .unwrap();
        backend.backend().buffer().clone()
    }
}

#[test]
fn palette_set_by_a_binding_shows_in_the_next_frame() {
    let config = load(
        "palette",
        "gband.bind('alt+p', function() gband.palette.set({ bg = '#101010' }) end)",
    );
    let mut client = Client::attached(config);
    assert_eq!(client.screen()[(60, 10)].bg, Color::Reset);
    client.press("alt+p");
    assert_eq!(client.screen()[(60, 10)].bg, Color::Rgb(0x10, 0x10, 0x10));
}

#[test]
fn palette_set_by_the_init_file() {
    let config = load("palette-init", "gband.palette.set({ bg = '#101010' })");
    let mut client = Client::attached(config);
    let screen = client.screen();
    assert_eq!(screen[(60, 10)].bg, Color::Rgb(0x10, 0x10, 0x10));
    assert_eq!(screen[(60, 10)].fg, Color::Reset);
}

#[test]
fn colored_focused_border_from_a_binding() {
    let config = load(
        "border",
        "gband.bind('alt+b', function() gband.hl.set('WindowBorderFocused', { fg = 2, bold = true }) end)",
    );
    let mut client = Client::attached(config);
    let before = &client.screen()[(0, 0)];
    assert_eq!(before.fg, Color::Rgb(0xb1, 0xb9, 0xf9));
    client.press("alt+b");
    let screen = client.screen();
    assert_eq!(screen[(0, 0)].fg, Color::Indexed(2));
    assert!(screen[(0, 0)].modifier.contains(Modifier::BOLD));
}
