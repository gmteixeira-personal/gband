use gband_client::animation::Animations;
use gband_client::color::ColorSupport;
use gband_client::render::apply_palette;
use gband_client::{Controls, Display, Probe, Step, graphics_probe};
use gband_core::geometry::Size;
use gband_core::layout::{Layout, LayoutOptions, WindowId};
use gband_emulator::{Emulator, Grid};
use gband_lua::{Config, LoadOptions, Locations, Palette};
use gband_protocol::ServerMessage;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::Widget;
use tui_term::widget::PseudoTerminal;

const IMAGE: &[u8] = b"\x1b_Ga=T;AAAA\x1b\\";

fn load(name: &str, source: &str) -> Config {
    let scratch = gband_scratch::Scratch::new("client", &format!("graphics-{name}"));
    let config = scratch.join("config");
    let user = gband_lua::user_file(&config, gband_lua::Side::Client);
    std::fs::create_dir_all(user.parent().unwrap()).unwrap();
    std::fs::write(&user, source).unwrap();
    gband_lua::load(
        &Locations {
            config,
            plugins: None,
        },
        gband_lua::Side::Client,
        &LoadOptions {
            budget: gband_lua::BUDGET,
        },
    )
    .unwrap()
}

fn writes(name: &str, source: &str, messages: Vec<ServerMessage>) -> Vec<Vec<u8>> {
    let mut display = Display::new(Size::new(80, 24), Animations::Off);
    let mut controls = Controls::new(load(name, source), &mut display);
    let mut layout = Layout::new();
    let window = layout.allocate_window();
    let band = layout.bands()[0].id;
    layout.open(window, band, None, None, &LayoutOptions::default());
    let shown = ServerMessage::Layout {
        cols: 80,
        rows: 24,
        layout,
    };
    controls.receive(&mut display, [shown]);
    let received = controls.receive(&mut display, messages);
    assert_eq!(received.outcome, None);
    received
        .steps
        .into_iter()
        .filter_map(|step| match step {
            Step::Write(bytes) => Some(bytes),
            _ => None,
        })
        .collect()
}

fn graphics(data: &[u8]) -> ServerMessage {
    ServerMessage::Graphics {
        window: WindowId(1),
        data: data.to_vec(),
    }
}

#[test]
fn forwarded_sequence_is_written_by_default() {
    let written = writes(
        "default",
        "",
        vec![graphics(IMAGE), graphics(b"\x1b_Ga=T;Qg==\x1b\\")],
    );
    assert_eq!(written, [IMAGE.to_vec(), b"\x1b_Ga=T;Qg==\x1b\\".to_vec()]);
}

#[test]
fn forwarded_sequence_is_not_written_with_the_option_off() {
    let written = writes(
        "off",
        "gband.set { window_graphics = false }",
        vec![graphics(IMAGE)],
    );
    assert_eq!(written, Vec::<Vec<u8>>::new());
}

fn rendered(bytes: &[u8], palette: &Palette) -> Buffer {
    let mut grid = Grid::new(Size::new(4, 1));
    grid.process(bytes);
    let area = Rect::new(0, 0, 4, 1);
    let mut buffer = Buffer::empty(area);
    PseudoTerminal::new(grid.screen()).render(area, &mut buffer);
    apply_palette(&mut buffer, palette, ColorSupport::TrueColor);
    buffer
}

fn colorscheme() -> Palette {
    let mut palette = Palette::default();
    palette.colors[3] = Some((0xd7, 0x99, 0x21));
    palette
}

#[test]
fn placeholder_foreground_is_not_remapped() {
    let buffer = rendered("\x1b[38;5;3m\u{10EEEE}x".as_bytes(), &colorscheme());
    assert_eq!(buffer[(0, 0)].fg, Color::Indexed(3));
    assert_eq!(buffer[(1, 0)].fg, Color::Rgb(0xd7, 0x99, 0x21));
}

#[test]
fn placeholder_cell_is_one_column_wide() {
    let buffer = rendered(
        "\u{10EEEE}\u{0305}\u{030D}x".as_bytes(),
        &Palette::default(),
    );
    assert_eq!(buffer[(0, 0)].symbol(), "\u{10EEEE}\u{0305}\u{030D}");
    assert_eq!(buffer[(1, 0)].symbol(), "x");
}

#[test]
fn terminal_answering_the_graphics_query_supports_graphics() {
    assert_eq!(
        graphics_probe(b"\x1b_Gi=1;OK\x1b\\\x1b[?62;22c"),
        Some(Probe {
            graphics: true,
            end: 20,
            ..Probe::default()
        })
    );
}

#[test]
fn terminal_answering_only_device_attributes_lacks_graphics() {
    assert_eq!(
        graphics_probe(b"\x1b[?62;22c"),
        Some(Probe {
            end: 9,
            ..Probe::default()
        })
    );
}

#[test]
fn probe_waits_for_the_device_attributes_reply() {
    assert_eq!(graphics_probe(b"\x1b_Gi=1;OK\x1b\\\x1b[?62;2"), None);
}
