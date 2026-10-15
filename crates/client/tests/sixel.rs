use std::time::Instant;

use gband_client::animation::Animations;
use gband_client::{Controls, Display, Probe, Step, cell_from_window, graphics_probe};
use gband_core::geometry::{CellSize, Size};
use gband_core::layout::{Layout, LayoutOptions, WindowId};
use gband_lua::{Config, LoadOptions, Locations};
use gband_protocol::{ClientMessage, ServerMessage};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

const IMAGE: &[u8] = b"\x1bPq#0~~\x1b\\";

fn load(name: &str, source: &str) -> Config {
    let scratch = gband_scratch::Scratch::new("client", &format!("sixel-{name}"));
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

fn attached(name: &str, source: &str) -> (Display, Controls) {
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
    (display, controls)
}

fn sixel(row: u16, col: u16) -> ServerMessage {
    ServerMessage::Sixel {
        window: WindowId(1),
        col,
        row,
        data: IMAGE.to_vec(),
    }
}

fn drawn_writes(name: &str, source: &str, message: ServerMessage) -> (Vec<u8>, (i64, i64)) {
    let (mut display, mut controls) = attached(name, source);
    let received = controls.receive(&mut display, [message]);
    assert_eq!(received.outcome, None);
    assert!(
        !received
            .steps
            .iter()
            .any(|step| matches!(step, Step::Write(bytes) if bytes.ends_with(IMAGE)))
    );
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal
        .draw(|frame| display.draw(frame, Instant::now()))
        .unwrap();
    let region = display
        .regions()
        .iter()
        .find(|region| region.window == Some(WindowId(1)))
        .copied()
        .unwrap();
    (display.take_sixel_writes(), region.content_origin())
}

#[test]
fn sixel_is_drawn_at_its_position_after_the_frame() {
    let (written, (x, y)) = drawn_writes("position", "", sixel(3, 5));
    let mut expected = format!("\x1b7\x1b[{};{}H", y + 3, x + 5).into_bytes();
    expected.extend_from_slice(IMAGE);
    expected.extend_from_slice(b"\x1b8");
    assert_eq!(written, expected);
}

#[test]
fn sixel_is_not_drawn_with_the_option_off() {
    let (written, _) = drawn_writes("off", "gband.set { window_sixel = false }", sixel(3, 5));
    assert_eq!(written, b"");
}

#[test]
fn sixel_below_the_window_is_not_drawn() {
    let (written, _) = drawn_writes("outside", "", sixel(200, 5));
    assert_eq!(written, b"");
}

#[test]
fn sixel_is_drawn_once() {
    let (mut display, mut controls) = attached("once", "");
    controls.receive(&mut display, [sixel(1, 1)]);
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal
        .draw(|frame| display.draw(frame, Instant::now()))
        .unwrap();
    assert!(!display.take_sixel_writes().is_empty());
    assert!(display.take_sixel_writes().is_empty());
}

#[test]
fn terminal_advertising_sixel_supports_sixel() {
    let probe = graphics_probe(b"\x1b[?62;4;22c").unwrap();
    assert!(probe.sixel);
    assert!(!probe.graphics);
}

#[test]
fn terminal_without_attribute_four_lacks_sixel() {
    assert!(!graphics_probe(b"\x1b[?62;22c").unwrap().sixel);
    assert!(!graphics_probe(b"\x1b[?62;14;22c").unwrap().sixel);
}

#[test]
fn cell_size_from_the_window_size() {
    assert_eq!(
        cell_from_window(100, 30, 1000, 600),
        Some(CellSize::new(10, 20))
    );
}

#[test]
fn cell_size_from_the_query_reply() {
    assert_eq!(
        graphics_probe(b"\x1b[6;19;9t\x1b[?62;4;22c"),
        Some(Probe {
            graphics: false,
            sixel: true,
            cell: Some(CellSize::new(9, 19)),
            end: 20,
        })
    );
}

#[test]
fn unknown_cell_size() {
    assert_eq!(cell_from_window(80, 24, 0, 0), None);
    assert_eq!(graphics_probe(b"\x1b[?62;4;22c").unwrap().cell, None);
}

#[test]
fn resize_reports_the_cell_size() {
    let (mut display, mut controls) = attached("resize", "");
    display.set_cell_size(Some(CellSize::new(9, 19)));
    let steps = controls.resize(&mut display, Size::new(100, 30));
    assert!(steps.iter().any(|step| matches!(
        step,
        Step::Send(ClientMessage::Resize {
            cell: Some(cell),
            ..
        }) if *cell == CellSize::new(9, 19)
    )));
}
