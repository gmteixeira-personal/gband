use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;

use gband_client::synchronized;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Position, Rect};
use ratatui::{Frame, Terminal, TerminalOptions, Viewport};

const BEGIN: &[u8] = b"\x1b[?2026h";
const END: &[u8] = b"\x1b[?2026l";
const HIDE: &[u8] = b"\x1b[?25l";
const SHOW: &[u8] = b"\x1b[?25h";

#[derive(Clone, Default)]
struct Recorder {
    bytes: Rc<RefCell<Vec<u8>>>,
    fail_after: Option<usize>,
    failed: Rc<RefCell<bool>>,
}

impl Recorder {
    fn failing_after(limit: usize) -> Self {
        Self {
            fail_after: Some(limit),
            ..Self::default()
        }
    }

    fn bytes(&self) -> Vec<u8> {
        self.bytes.borrow().clone()
    }
}

impl Write for Recorder {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut bytes = self.bytes.borrow_mut();
        let mut failed = self.failed.borrow_mut();
        let accepted = match self.fail_after {
            Some(limit) if !*failed => {
                let room = limit.saturating_sub(bytes.len());
                if room == 0 {
                    *failed = true;
                    return Err(io::Error::other("terminal gone"));
                }
                buf.len().min(room)
            }
            _ => buf.len(),
        };
        bytes.extend_from_slice(&buf[..accepted]);
        Ok(accepted)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn terminal(recorder: &Recorder) -> Terminal<CrosstermBackend<Recorder>> {
    Terminal::with_options(
        CrosstermBackend::new(recorder.clone()),
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, 20, 12)),
        },
    )
    .unwrap()
}

fn set(frame: &mut Frame<'_>, x: u16, y: u16, symbol: &str) {
    frame.buffer_mut()[(x, y)].set_symbol(symbol);
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn rfind(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .rposition(|window| window == needle)
}

fn shown(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).escape_debug().to_string()
}

#[test]
fn frame_between_the_markers() {
    let recorder = Recorder::default();
    let mut terminal = terminal(&recorder);
    synchronized(&mut terminal, |frame| {
        set(frame, 1, 1, "a");
        set(frame, 2, 1, "b");
        set(frame, 7, 4, "c");
        frame.set_cursor_position(Position::new(3, 1));
    })
    .unwrap();
    let bytes = recorder.bytes();
    assert!(
        bytes.starts_with(&[BEGIN, HIDE].concat()),
        "{}",
        shown(&bytes)
    );
    assert!(bytes.ends_with(END), "{}", shown(&bytes));
    assert_eq!(find(&bytes, END), Some(bytes.len() - END.len()));
}

#[test]
fn cursor_hidden_while_cells_are_written() {
    let recorder = Recorder::default();
    let mut terminal = terminal(&recorder);
    synchronized(&mut terminal, |frame| {
        set(frame, 4, 2, "A");
        set(frame, 9, 10, "Z");
        frame.set_cursor_position(Position::new(6, 5));
    })
    .unwrap();
    let bytes = recorder.bytes();
    let last_cell = rfind(&bytes, b"Z").unwrap();
    let show = find(&bytes, SHOW).unwrap();
    assert!(show > last_cell, "{}", shown(&bytes));
    assert!(
        bytes.ends_with(&[SHOW, b"\x1b[6;7H", END].concat()),
        "{}",
        shown(&bytes)
    );
}

#[test]
fn cursor_stays_hidden() {
    let recorder = Recorder::default();
    let mut terminal = terminal(&recorder);
    synchronized(&mut terminal, |frame| set(frame, 0, 0, "x")).unwrap();
    let bytes = recorder.bytes();
    assert!(find(&bytes, HIDE).is_some(), "{}", shown(&bytes));
    assert_eq!(find(&bytes, SHOW), None, "{}", shown(&bytes));
}

#[test]
fn failed_frame_still_ends() {
    let recorder = Recorder::failing_after(64);
    let mut terminal = terminal(&recorder);
    let result = synchronized(&mut terminal, |frame| {
        for y in 0..12 {
            set(frame, 0, y, "#");
            set(frame, 19, y, "#");
        }
    });
    let bytes = recorder.bytes();
    assert!(result.is_err());
    assert!(bytes.starts_with(BEGIN), "{}", shown(&bytes));
    assert!(bytes.ends_with(END), "{}", shown(&bytes));
    assert_eq!(bytes.len(), 64 + END.len());
}
