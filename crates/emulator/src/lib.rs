mod callbacks;

use gband_core::geometry::Size;
use gband_core::input::{Modes, MouseEncoding, MouseTracking};

use crate::callbacks::Callbacks;
pub use crate::callbacks::{Record, decode_base64};

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
    fn screen(&self) -> &Self::Screen;
    fn checkpoint(&self) -> Self::Checkpoint;
    fn snapshot(&self) -> Vec<u8>;
    fn diff(&self, since: &Self::Checkpoint) -> Vec<u8>;
    fn take_write_back(&mut self) -> Vec<u8>;
}

pub type Grid = Vt100;

pub struct Vt100 {
    parser: vt100::Parser<Callbacks>,
    escape_pending: bool,
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

    pub fn take_title_changed(&mut self) -> bool {
        std::mem::take(&mut self.parser.callbacks_mut().title_changed)
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
        }
    }

    fn process(&mut self, bytes: &[u8]) {
        let mut rest = bytes;
        while !rest.is_empty() {
            let reset = if self.escape_pending && rest[0] == b'c' {
                Some(1)
            } else {
                rest.windows(2)
                    .position(|pair| pair == b"\x1bc")
                    .map(|at| at + 2)
            };
            let end = reset.unwrap_or(rest.len());
            self.parser.process(&rest[..end]);
            self.escape_pending = rest[end - 1] == 0x1b;
            if reset.is_some() {
                self.parser.callbacks_mut().modify_other_keys = 0;
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
