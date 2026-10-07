use vt100::{MouseProtocolEncoding, MouseProtocolMode, Screen};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Record {
    Notification { title: Option<String>, body: String },
    Clipboard(String),
    Bell,
    Settle(u64),
}

const TITLE_STACK_DEPTH: usize = 10;

#[derive(Default)]
pub struct Callbacks {
    pub write_back: Vec<u8>,
    pub records: Option<Vec<Record>>,
    pub title: Option<String>,
    pub title_changed: bool,
    title_stack: Vec<Option<String>>,
}

impl Callbacks {
    fn record(&mut self, record: Record) {
        if let Some(records) = &mut self.records {
            records.push(record);
        }
    }

    fn set_title(&mut self, title: Option<String>) {
        if self.title != title {
            self.title = title;
            self.title_changed = true;
        }
    }

    fn push_title(&mut self) {
        if self.title_stack.len() == TITLE_STACK_DEPTH {
            self.title_stack.remove(0);
        }
        self.title_stack.push(self.title.clone());
    }

    fn pop_title(&mut self) {
        if let Some(title) = self.title_stack.pop() {
            self.set_title(title);
        }
    }
}

fn cleaned_title(text: &str) -> Option<String> {
    let kept: String = text.chars().filter(|c| !c.is_control()).collect();
    let trimmed = kept.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

fn title_stack_operation(params: &[&[u16]]) -> Option<bool> {
    let mut values = params.iter().map(|param| param.first().copied());
    let push = match values.next()?? {
        22 => true,
        23 => false,
        _ => return None,
    };
    matches!(values.next(), None | Some(None | Some(0 | 2))).then_some(push)
}

fn joined(params: &[&[u8]]) -> String {
    params
        .iter()
        .map(|param| String::from_utf8_lossy(param))
        .collect::<Vec<_>>()
        .join(";")
}

fn recognised(params: &[&[u8]]) -> Option<Record> {
    match params {
        [b"9", body @ ..] => Some(Record::Notification {
            title: None,
            body: joined(body),
        }),
        [b"777", b"notify", title, body @ ..] => Some(Record::Notification {
            title: Some(String::from_utf8_lossy(title).into_owned()),
            body: joined(body),
        }),
        [b"7777", b"settle", round] => std::str::from_utf8(round)
            .ok()?
            .parse()
            .ok()
            .map(Record::Settle),
        _ => None,
    }
}

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn decode_base64(text: &[u8]) -> Option<Vec<u8>> {
    let mut decoded = Vec::with_capacity(text.len() / 4 * 3);
    let mut joined = 0u32;
    let mut bits = 0;
    for &byte in text.iter().take_while(|&&byte| byte != b'=') {
        let value = BASE64.iter().position(|&known| known == byte)?;
        joined = joined << 6 | value as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            decoded.push((joined >> bits) as u8);
            joined &= (1 << bits) - 1;
        }
    }
    Some(decoded)
}

impl vt100::Callbacks for Callbacks {
    fn audible_bell(&mut self, _: &mut Screen) {
        tracing::debug!("bell");
        self.record(Record::Bell);
    }

    fn copy_to_clipboard(&mut self, _: &mut Screen, ty: &[u8], data: &[u8]) {
        if ty != b"c" {
            return;
        }
        if let Some(text) = decode_base64(data) {
            self.record(Record::Clipboard(
                String::from_utf8_lossy(&text).into_owned(),
            ));
        }
    }

    fn set_window_title(&mut self, _: &mut Screen, title: &[u8]) {
        self.set_title(cleaned_title(&String::from_utf8_lossy(title)));
    }

    fn unhandled_control(&mut self, _: &mut Screen, b: u8) {
        tracing::debug!("unhandled control {b:#04x}");
    }

    fn unhandled_escape(&mut self, _: &mut Screen, i1: Option<u8>, i2: Option<u8>, b: u8) {
        let sequence = [i1, i2, Some(b)].into_iter().flatten().map(char::from);
        tracing::debug!("unhandled escape \\e{}", sequence.collect::<String>());
    }

    fn unhandled_csi(
        &mut self,
        screen: &mut Screen,
        i1: Option<u8>,
        i2: Option<u8>,
        params: &[&[u16]],
        c: char,
    ) {
        if let Some(reply) = query_reply(screen, i1, i2, params, c) {
            self.write_back.extend_from_slice(reply.as_bytes());
            return;
        }
        if (i1, i2, c) == (None, None, 't')
            && let Some(push) = title_stack_operation(params)
        {
            if push {
                self.push_title();
            } else {
                self.pop_title();
            }
            return;
        }
        let params = params
            .iter()
            .map(|param| {
                param
                    .iter()
                    .map(u16::to_string)
                    .collect::<Vec<_>>()
                    .join(":")
            })
            .collect::<Vec<_>>()
            .join(";");
        let (private, intermediates): (Vec<u8>, Vec<u8>) = [i1, i2]
            .into_iter()
            .flatten()
            .partition(|byte| (b'<'..=b'?').contains(byte));
        let private = String::from_utf8_lossy(&private);
        let intermediates = String::from_utf8_lossy(&intermediates);
        tracing::debug!("unhandled CSI \\e[{private}{params}{intermediates}{c}");
    }

    fn unhandled_osc(&mut self, _: &mut Screen, params: &[&[u8]]) {
        match params {
            [b"0" | b"2", text @ ..] => {
                self.set_title(cleaned_title(&joined(text)));
                return;
            }
            [b"1", ..] => return,
            _ => {}
        }
        if self.records.is_some()
            && let Some(record) = recognised(params)
        {
            self.record(record);
            return;
        }
        tracing::debug!("unhandled OSC \\e]{}", joined(params));
    }
}

fn query_reply(
    screen: &Screen,
    i1: Option<u8>,
    i2: Option<u8>,
    params: &[&[u16]],
    c: char,
) -> Option<String> {
    let first = params.first().and_then(|param| param.first()).copied();
    match (i1, i2, c) {
        (None, None, 'c') if matches!(first, None | Some(0)) => Some("\x1b[?62;22c".to_owned()),
        (None, None, 'n') => match first? {
            5 => Some("\x1b[0n".to_owned()),
            6 => {
                let (row, col) = screen.cursor_position();
                Some(format!("\x1b[{};{}R", row + 1, col + 1))
            }
            _ => None,
        },
        (Some(b'>'), None, 'q') if matches!(first, None | Some(0)) => {
            Some(format!("\x1bP>|gband {}\x1b\\", env!("CARGO_PKG_VERSION")))
        }
        (Some(b'?'), Some(b'$'), 'p') => {
            let mode = first?;
            Some(format!(
                "\x1b[?{mode};{}$y",
                private_mode_state(screen, mode)
            ))
        }
        (Some(b'$'), None, 'p') => Some(format!("\x1b[{};0$y", first?)),
        _ => None,
    }
}

fn private_mode_state(screen: &Screen, mode: u16) -> u8 {
    let set = match mode {
        1 => screen.application_cursor(),
        9 => screen.mouse_protocol_mode() == MouseProtocolMode::Press,
        25 => !screen.hide_cursor(),
        47 | 1049 => screen.alternate_screen(),
        1000 => screen.mouse_protocol_mode() == MouseProtocolMode::PressRelease,
        1002 => screen.mouse_protocol_mode() == MouseProtocolMode::ButtonMotion,
        1003 => screen.mouse_protocol_mode() == MouseProtocolMode::AnyMotion,
        1005 => screen.mouse_protocol_encoding() == MouseProtocolEncoding::Utf8,
        1006 => screen.mouse_protocol_encoding() == MouseProtocolEncoding::Sgr,
        2004 => screen.bracketed_paste(),
        _ => return 0,
    };
    if set { 1 } else { 2 }
}

#[cfg(test)]
mod tests {
    use gband_core::geometry::Size;

    use crate::{Emulator, Grid, Record};

    fn recorded(output: &[u8]) -> Vec<Record> {
        let mut grid = Grid::new(Size::new(80, 24));
        grid.record();
        grid.process(output);
        grid.take_records()
    }

    #[test]
    fn osc_9_is_a_notification_without_a_title() {
        assert_eq!(
            recorded(b"\x1b]9;build; done\x07"),
            [Record::Notification {
                title: None,
                body: "build; done".to_owned()
            }]
        );
    }

    #[test]
    fn osc_777_notify_carries_a_title() {
        assert_eq!(
            recorded(b"\x1b]777;notify;ci;build done\x1b\\"),
            [Record::Notification {
                title: Some("ci".to_owned()),
                body: "build done".to_owned()
            }]
        );
    }

    #[test]
    fn osc_52_is_decoded() {
        assert_eq!(
            recorded(b"\x1b]52;c;aGVsbG8gd29ybGQ=\x07"),
            [Record::Clipboard("hello world".to_owned())]
        );
        assert_eq!(recorded(b"\x1b]52;p;aGk=\x07"), []);
        assert_eq!(recorded(b"\x1b]52;c;?\x07"), []);
    }

    #[test]
    fn bells_are_counted() {
        assert_eq!(recorded(b"a\x07b\x07"), [Record::Bell, Record::Bell]);
    }

    #[test]
    fn settle_markers_carry_their_round() {
        assert_eq!(recorded(b"\x1b]7777;settle;3\x07"), [Record::Settle(3)]);
        assert_eq!(recorded(b"\x1b]7777;settle;x\x07"), []);
    }

    #[test]
    fn nothing_is_recorded_unless_asked() {
        let mut grid = Grid::new(Size::new(80, 24));
        grid.process(b"\x07\x1b]9;hi\x07\x1b]52;c;aGk=\x07\x1b]7777;settle;1\x07");
        assert_eq!(grid.take_records(), []);
    }

    #[test]
    fn base64_round_trips() {
        for text in ["", "h", "hi", "hey", "hello world"] {
            let encoded = gband_base64(text.as_bytes());
            assert_eq!(
                crate::callbacks::decode_base64(encoded.as_bytes()).unwrap(),
                text.as_bytes()
            );
        }
        assert_eq!(crate::callbacks::decode_base64(b"a*b="), None);
    }

    fn gband_base64(bytes: &[u8]) -> String {
        const TABLE: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut encoded = String::new();
        for chunk in bytes.chunks(3) {
            let joined = chunk
                .iter()
                .enumerate()
                .fold(0u32, |joined, (index, &byte)| {
                    joined | u32::from(byte) << (16 - 8 * index)
                });
            for index in 0..4 {
                if index <= chunk.len() {
                    encoded.push(char::from(
                        TABLE[(joined >> (18 - 6 * index) & 63) as usize],
                    ));
                } else {
                    encoded.push('=');
                }
            }
        }
        encoded
    }

    fn replies(output: &str) -> String {
        let mut grid = Grid::new(Size::new(80, 24));
        grid.process(output.as_bytes());
        String::from_utf8(grid.take_write_back()).unwrap()
    }

    #[test]
    fn device_attributes_claim_a_vt220_with_colour() {
        assert_eq!(replies("\x1b[c"), "\x1b[?62;22c");
        assert_eq!(replies("\x1b[0c"), "\x1b[?62;22c");
    }

    #[test]
    fn operating_status_is_ok() {
        assert_eq!(replies("\x1b[5n"), "\x1b[0n");
    }

    #[test]
    fn cursor_position_is_read_at_the_query() {
        assert_eq!(replies("\x1b[6n"), "\x1b[1;1R");
        assert_eq!(replies("\x1b[5;10H\x1b[6n\x1b[1;1H"), "\x1b[5;10R");
    }

    #[test]
    fn version_names_gband() {
        let expected = format!("\x1bP>|gband {}\x1b\\", env!("CARGO_PKG_VERSION"));
        assert_eq!(replies("\x1b[>q"), expected);
        assert_eq!(replies("\x1b[>0q"), expected);
    }

    #[test]
    fn private_modes_report_their_state() {
        assert_eq!(replies("\x1b[?1$p"), "\x1b[?1;2$y");
        assert_eq!(replies("\x1b[?1h\x1b[?1$p"), "\x1b[?1;1$y");
        assert_eq!(replies("\x1b[?25$p"), "\x1b[?25;1$y");
        assert_eq!(replies("\x1b[?25l\x1b[?25$p"), "\x1b[?25;2$y");
        assert_eq!(
            replies("\x1b[?1049h\x1b[?47$p\x1b[?1049$p"),
            "\x1b[?47;1$y\x1b[?1049;1$y"
        );
        assert_eq!(
            replies("\x1b[?1002h\x1b[?9$p\x1b[?1000$p\x1b[?1002$p\x1b[?1003$p"),
            "\x1b[?9;2$y\x1b[?1000;2$y\x1b[?1002;1$y\x1b[?1003;2$y"
        );
        assert_eq!(
            replies("\x1b[?1006h\x1b[?1005$p\x1b[?1006$p"),
            "\x1b[?1005;2$y\x1b[?1006;1$y"
        );
        assert_eq!(replies("\x1b[?2004$p"), "\x1b[?2004;2$y");
        assert_eq!(replies("\x1b[?2004h\x1b[?2004$p"), "\x1b[?2004;1$y");
    }

    #[test]
    fn unrecognised_modes_report_zero() {
        assert_eq!(replies("\x1b[?7727$p"), "\x1b[?7727;0$y");
        assert_eq!(replies("\x1b[?6$p"), "\x1b[?6;0$y");
        assert_eq!(replies("\x1b[4$p"), "\x1b[4;0$y");
    }

    #[test]
    fn unsupported_queries_stay_unanswered() {
        assert_eq!(replies("\x1b[?u"), "");
        assert_eq!(replies("\x1b]11;?\x07"), "");
        assert_eq!(replies("\x1b[999z"), "");
    }
}
