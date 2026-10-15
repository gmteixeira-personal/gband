mod callbacks;
mod graphics;

use gband_core::geometry::{CellSize, Size};
use gband_core::input::{Modes, MouseEncoding, MouseTracking};

use crate::callbacks::Callbacks;
pub use crate::callbacks::{Record, decode_base64};
use crate::graphics::{Found, Scanner};
pub use crate::graphics::{GRAPHICS_LIMIT, query_reply};

pub trait Emulator: Send + 'static {
    type Checkpoint: Clone + Send + Sync + 'static;
    type Screen;

    fn new(size: Size) -> Self;
    fn process(&mut self, bytes: &[u8]);
    fn resize(&mut self, size: Size);
    fn size(&self) -> Size;
    fn modes(&self) -> Modes;
    fn cursor(&self) -> Option<(u16, u16)>;
    fn alternate_screen(&self) -> bool;
    fn contents(&self) -> String;
    fn text_between(&self, start: (u16, u16), end: (u16, u16)) -> String;
    fn bottom_lines(&self, count: usize) -> Vec<String>;
    fn screen(&self) -> &Self::Screen;
    fn checkpoint(&self) -> Self::Checkpoint;
    fn snapshot(&self) -> Vec<u8>;
    fn diff(&self, since: &Self::Checkpoint) -> Vec<u8>;
    fn take_write_back(&mut self) -> Vec<u8>;
}

pub type Grid = Vt100;

const FULL_RESET: &[u8] = b"\x1bc";

const SEGMENT_ENDS: [&[u8]; 7] = [
    FULL_RESET,
    b"\x1b[?47h",
    b"\x1b[?1047h",
    b"\x1b[?1049h",
    b"\x1b[?47l",
    b"\x1b[?1047l",
    b"\x1b[?1049l",
];

fn segment_end(bytes: &[u8]) -> Option<usize> {
    (0..bytes.len())
        .filter(|&at| bytes[at] == 0x1b)
        .find_map(|at| {
            SEGMENT_ENDS
                .iter()
                .find(|end| bytes[at..].starts_with(end))
                .map(|end| at + end.len())
        })
}

pub struct Vt100 {
    parser: vt100::Parser<Callbacks>,
    escape_pending: bool,
    scanner: Scanner,
    graphics: Vec<Vec<u8>>,
    graphics_replies: bool,
    sixel_start: (u16, u16),
    sixel: Vec<(u16, u16, Vec<u8>)>,
}

impl Vt100 {
    pub fn record(&mut self) {
        self.parser
            .callbacks_mut()
            .records
            .get_or_insert_with(Vec::new);
    }

    pub fn take_records(&mut self) -> Vec<Record> {
        self.parser
            .callbacks_mut()
            .records
            .as_mut()
            .map(std::mem::take)
            .unwrap_or_default()
    }

    pub fn title(&self) -> Option<&str> {
        self.parser.callbacks().title.as_deref()
    }

    pub fn take_clipboard(&mut self) -> Option<String> {
        self.parser.callbacks_mut().clipboard.take()
    }

    pub fn take_title_changed(&mut self) -> bool {
        std::mem::take(&mut self.parser.callbacks_mut().title_changed)
    }

    pub fn take_graphics(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.graphics)
    }

    pub fn answer_graphics_queries(&mut self, answer: bool) {
        self.graphics_replies = answer;
    }

    pub fn take_sixel(&mut self) -> Vec<(u16, u16, Vec<u8>)> {
        std::mem::take(&mut self.sixel)
    }

    pub fn set_sixel_supported(&mut self, supported: bool) {
        self.parser.callbacks_mut().sixel_supported = supported;
    }

    pub fn set_cell_size(&mut self, cell_size: Option<CellSize>) {
        self.parser.callbacks_mut().cell_size = cell_size;
    }

    fn feed(&mut self, bytes: &[u8]) {
        let mut rest = bytes;
        while !rest.is_empty() {
            let split_reset = self.escape_pending && rest[0] == b'c';
            let end = if split_reset {
                1
            } else {
                segment_end(rest).unwrap_or(rest.len())
            };
            let reset = split_reset || rest[..end].ends_with(FULL_RESET);
            let was_alternate = self.parser.screen().alternate_screen();
            self.parser.process(&rest[..end]);
            self.escape_pending = rest[end - 1] == 0x1b;
            let entered_alternate = !was_alternate && self.parser.screen().alternate_screen();
            let callbacks = self.parser.callbacks_mut();
            if reset {
                callbacks.modify_other_keys = 0;
                callbacks.clear_keyboard_flags(false);
                callbacks.clear_keyboard_flags(true);
            }
            if entered_alternate {
                callbacks.clear_keyboard_flags(true);
            }
            rest = &rest[end..];
        }
    }
}

impl Emulator for Vt100 {
    type Checkpoint = vt100::Screen;
    type Screen = vt100::Screen;

    fn new(size: Size) -> Self {
        Self {
            parser: vt100::Parser::new_with_callbacks(
                size.rows,
                size.cols,
                0,
                Callbacks::default(),
            ),
            escape_pending: false,
            scanner: Scanner::default(),
            graphics: Vec::new(),
            graphics_replies: false,
            sixel_start: (1, 1),
            sixel: Vec::new(),
        }
    }

    fn process(&mut self, bytes: &[u8]) {
        let mut rest = bytes;
        while !rest.is_empty() {
            let (end, found) = self.scanner.scan(rest);
            self.feed(&rest[..end]);
            match found {
                Some(Found::Graphics(sequence)) => {
                    if self.graphics_replies
                        && let Some(reply) = query_reply(&sequence)
                    {
                        self.parser
                            .callbacks_mut()
                            .write_back
                            .extend_from_slice(&reply);
                    }
                    self.graphics.push(sequence);
                }
                Some(Found::SixelStart) => {
                    let (row, col) = self.parser.screen().cursor_position();
                    self.sixel_start = (row + 1, col + 1);
                }
                Some(Found::Sixel(sequence)) => {
                    let (row, col) = self.sixel_start;
                    self.sixel.push((row, col, sequence));
                }
                None => {}
            }
            rest = &rest[end..];
        }
    }

    fn resize(&mut self, size: Size) {
        self.parser.screen_mut().set_size(size.rows, size.cols);
    }

    fn size(&self) -> Size {
        let (rows, cols) = self.parser.screen().size();
        Size::new(cols, rows)
    }

    fn modes(&self) -> Modes {
        let screen = self.parser.screen();
        Modes {
            application_cursor: screen.application_cursor(),
            bracketed_paste: screen.bracketed_paste(),
            mouse_tracking: match screen.mouse_protocol_mode() {
                vt100::MouseProtocolMode::None => MouseTracking::None,
                vt100::MouseProtocolMode::Press => MouseTracking::Press,
                vt100::MouseProtocolMode::PressRelease => MouseTracking::PressRelease,
                vt100::MouseProtocolMode::ButtonMotion => MouseTracking::ButtonMotion,
                vt100::MouseProtocolMode::AnyMotion => MouseTracking::AnyMotion,
            },
            mouse_encoding: match screen.mouse_protocol_encoding() {
                vt100::MouseProtocolEncoding::Default => MouseEncoding::Default,
                vt100::MouseProtocolEncoding::Utf8 => MouseEncoding::Utf8,
                vt100::MouseProtocolEncoding::Sgr => MouseEncoding::Sgr,
            },
            modify_other_keys: self.parser.callbacks().modify_other_keys,
            keyboard_flags: self
                .parser
                .callbacks()
                .keyboard_flags(screen.alternate_screen()),
        }
    }

    fn cursor(&self) -> Option<(u16, u16)> {
        let screen = self.parser.screen();
        (!screen.hide_cursor()).then(|| screen.cursor_position())
    }

    fn alternate_screen(&self) -> bool {
        self.parser.screen().alternate_screen()
    }

    fn contents(&self) -> String {
        self.parser.screen().contents()
    }

    fn text_between(&self, start: (u16, u16), end: (u16, u16)) -> String {
        let screen = self.parser.screen();
        let (rows, cols) = screen.size();
        let mut text = String::new();
        for row in start.1..=end.1.min(rows.saturating_sub(1)) {
            let from = if row == start.1 { start.0 } else { 0 }.min(cols);
            let to = if row == end.1 { end.0 + 1 } else { cols }.min(cols);
            let line = screen
                .rows(from, to.saturating_sub(from))
                .nth(usize::from(row))
                .unwrap_or_default();
            if row != end.1 && screen.row_wrapped(row) {
                text.push_str(&line);
            } else {
                text.push_str(line.trim_end_matches(' '));
                if row != end.1 {
                    text.push('\n');
                }
            }
        }
        text
    }

    fn bottom_lines(&self, count: usize) -> Vec<String> {
        let screen = self.parser.screen();
        let (_, cols) = screen.size();
        let rows: Vec<String> = screen.rows(0, cols).collect();
        let mut lines: Vec<String> = rows
            .into_iter()
            .rev()
            .map(|row| row.trim_end_matches(' ').to_owned())
            .filter(|row| !row.is_empty())
            .take(count)
            .collect();
        lines.reverse();
        lines
    }

    fn screen(&self) -> &vt100::Screen {
        self.parser.screen()
    }

    fn checkpoint(&self) -> vt100::Screen {
        self.parser.screen().clone()
    }

    fn snapshot(&self) -> Vec<u8> {
        self.parser.screen().state_formatted()
    }

    fn diff(&self, since: &vt100::Screen) -> Vec<u8> {
        self.parser.screen().state_diff(since)
    }

    fn take_write_back(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.parser.callbacks_mut().write_back)
    }
}
