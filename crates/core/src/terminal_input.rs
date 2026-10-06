use crate::input::{Key, KeyCode, Modifiers, MouseButton, MouseEvent, MouseKind, WheelDirection};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerminalInput {
    Key(Key),
    Mouse(MouseEvent),
    Paste(String),
    Focus(bool),
}

#[derive(Debug, Default)]
pub struct Decoder {
    held: Vec<u8>,
}

enum Step {
    Done(Vec<TerminalInput>, usize),
    Partial,
    Invalid,
}

const ESC: u8 = 0x1b;
const PASTE_END: &[u8] = b"\x1b[201~";

impl Decoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, bytes: &[u8]) -> Vec<TerminalInput> {
        self.held.extend_from_slice(bytes);
        self.decode(false)
    }

    pub fn holds(&self) -> bool {
        !self.held.is_empty()
    }

    pub fn flush(&mut self) -> Vec<TerminalInput> {
        self.decode(true)
    }

    fn decode(&mut self, last: bool) -> Vec<TerminalInput> {
        let mut inputs = Vec::new();
        let mut at = 0;
        while at < self.held.len() {
            let rest = &self.held[at..];
            match step(rest, last) {
                Step::Done(decoded, used) => {
                    inputs.extend(decoded);
                    at += used;
                }
                Step::Partial => break,
                Step::Invalid => {
                    let (resolved, used) = unfinished(rest);
                    inputs.extend(resolved);
                    at += used;
                }
            }
        }
        self.held.drain(..at);
        inputs
    }
}

fn unfinished(bytes: &[u8]) -> (Vec<TerminalInput>, usize) {
    match bytes {
        [ESC, introducer @ (b'[' | b'O'), ..] => match character(&[*introducer]) {
            Step::Done(inputs, _) => (prefixed(inputs), 2),
            _ => (Vec::new(), 2),
        },
        [ESC, ..] => (vec![key(KeyCode::Escape, Modifiers::NONE)], 1),
        _ => (Vec::new(), 1),
    }
}

fn key(code: KeyCode, modifiers: Modifiers) -> TerminalInput {
    TerminalInput::Key(Key::new(code, modifiers))
}

fn ctrl(c: char) -> TerminalInput {
    key(KeyCode::Char(c), Modifiers::CTRL)
}

fn incomplete(last: bool) -> Step {
    if last { Step::Invalid } else { Step::Partial }
}

fn step(bytes: &[u8], last: bool) -> Step {
    match bytes[0] {
        ESC => escape(bytes, last),
        b'\r' => Step::Done(vec![key(KeyCode::Enter, Modifiers::NONE)], 1),
        b'\t' => Step::Done(vec![key(KeyCode::Tab, Modifiers::NONE)], 1),
        0x7f => Step::Done(vec![key(KeyCode::Backspace, Modifiers::NONE)], 1),
        0x00 => Step::Done(vec![ctrl(' ')], 1),
        byte @ 0x01..=0x1a => Step::Done(vec![ctrl(char::from(byte - 1 + b'a'))], 1),
        byte @ 0x1c..=0x1f => Step::Done(vec![ctrl(char::from(byte - 0x1c + b'4'))], 1),
        _ => character(bytes),
    }
}

fn character(bytes: &[u8]) -> Step {
    let width = match bytes[0] {
        0x00..=0x7f => 1,
        0xc2..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf4 => 4,
        _ => return Step::Invalid,
    };
    let Some(encoded) = bytes.get(..width) else {
        return if bytes[1..].iter().all(|&byte| byte & 0xc0 == 0x80) {
            Step::Partial
        } else {
            Step::Invalid
        };
    };
    let Some(c) = std::str::from_utf8(encoded)
        .ok()
        .and_then(|text| text.chars().next())
    else {
        return Step::Invalid;
    };
    let modifiers = if c.is_uppercase() {
        Modifiers::SHIFT
    } else {
        Modifiers::NONE
    };
    Step::Done(vec![key(KeyCode::Char(c), modifiers)], width)
}

fn escape(bytes: &[u8], last: bool) -> Step {
    match bytes.get(1) {
        None if last => Step::Done(vec![key(KeyCode::Escape, Modifiers::NONE)], 1),
        None => Step::Partial,
        Some(b'[') => csi(bytes, last),
        Some(b'O') => ss3(bytes, last),
        Some(_) => match step(&bytes[1..], last) {
            Step::Done(inputs, used) => Step::Done(prefixed(inputs), used + 1),
            other => other,
        },
    }
}

fn prefixed(inputs: Vec<TerminalInput>) -> Vec<TerminalInput> {
    match inputs.as_slice() {
        [TerminalInput::Key(Key { code, modifiers })] if !modifiers.alt => vec![key(
            *code,
            Modifiers {
                alt: true,
                ..*modifiers
            },
        )],
        _ => std::iter::once(key(KeyCode::Escape, Modifiers::NONE))
            .chain(inputs)
            .collect(),
    }
}

fn ss3(bytes: &[u8], last: bool) -> Step {
    let Some(&letter) = bytes.get(2) else {
        return incomplete(last);
    };
    if !letter.is_ascii_alphabetic() {
        return Step::Invalid;
    }
    let decoded = cursor_key(letter)
        .or_else(|| function_key(letter))
        .map(|code| key(code, Modifiers::NONE));
    Step::Done(decoded.into_iter().collect(), 3)
}

fn cursor_key(letter: u8) -> Option<KeyCode> {
    Some(match letter {
        b'A' => KeyCode::Up,
        b'B' => KeyCode::Down,
        b'C' => KeyCode::Right,
        b'D' => KeyCode::Left,
        b'H' => KeyCode::Home,
        b'F' => KeyCode::End,
        _ => return None,
    })
}

fn function_key(letter: u8) -> Option<KeyCode> {
    matches!(letter, b'P'..=b'S').then(|| KeyCode::F(letter - b'P' + 1))
}

fn csi(bytes: &[u8], last: bool) -> Step {
    match bytes.get(2) {
        None => incomplete(last),
        Some(b'[') => console_function_key(bytes, last),
        Some(b'M') => default_mouse(bytes, last),
        Some(_) => {
            let parameters = bytes[2..]
                .iter()
                .position(|byte| !(0x30..=0x3f).contains(byte))
                .map_or(bytes.len(), |at| at + 2);
            let finished = bytes[parameters..]
                .iter()
                .position(|byte| !(0x20..=0x2f).contains(byte))
                .map(|at| at + parameters);
            let Some(end) = finished else {
                return incomplete(last);
            };
            if !(0x40..=0x7e).contains(&bytes[end]) {
                return Step::Invalid;
            }
            let used = end + 1;
            if end != parameters {
                return Step::Done(Vec::new(), used);
            }
            let params = &bytes[2..end];
            match (params, bytes[end]) {
                (b"200", b'~') => paste(bytes, used),
                ([b'<', sgr @ ..], end @ (b'M' | b'm')) => {
                    Step::Done(sgr_mouse(sgr, end == b'm').into_iter().collect(), used)
                }
                _ => Step::Done(sequence(params, bytes[end]).into_iter().collect(), used),
            }
        }
    }
}

fn console_function_key(bytes: &[u8], last: bool) -> Step {
    match bytes.get(3) {
        None => incomplete(last),
        Some(&letter @ b'A'..=b'E') => {
            Step::Done(vec![key(KeyCode::F(letter - b'A' + 1), Modifiers::NONE)], 4)
        }
        Some(0x40..=0x7e) => Step::Done(Vec::new(), 4),
        Some(_) => Step::Invalid,
    }
}

fn paste(bytes: &[u8], start: usize) -> Step {
    let Some(at) = bytes[start..]
        .windows(PASTE_END.len())
        .position(|window| window == PASTE_END)
    else {
        return Step::Partial;
    };
    let text = String::from_utf8_lossy(&bytes[start..start + at]).into_owned();
    Step::Done(
        vec![TerminalInput::Paste(text)],
        start + at + PASTE_END.len(),
    )
}

fn sequence(params: &[u8], end: u8) -> Option<TerminalInput> {
    let numbers = numbers(params)?;
    match (end, numbers.as_slice()) {
        (b'Z', []) => Some(key(KeyCode::BackTab, Modifiers::NONE)),
        (b'I', []) => Some(TerminalInput::Focus(true)),
        (b'O', []) => Some(TerminalInput::Focus(false)),
        (letter, []) => cursor_key(letter).map(|code| key(code, Modifiers::NONE)),
        (b'~', [number]) => tilde_key(*number).map(|code| key(code, Modifiers::NONE)),
        (b'~', [number, parameter]) => {
            tilde_key(*number).map(|code| key(code, modifiers(*parameter)))
        }
        (letter, [1, parameter]) => cursor_key(letter)
            .or_else(|| function_key(letter))
            .map(|code| key(code, modifiers(*parameter))),
        _ => None,
    }
}

fn numbers(params: &[u8]) -> Option<Vec<u32>> {
    if params.is_empty() {
        return Some(Vec::new());
    }
    params
        .split(|&byte| byte == b';')
        .map(|field| std::str::from_utf8(field).ok()?.parse().ok())
        .collect()
}

fn modifiers(parameter: u32) -> Modifiers {
    let bits = parameter.saturating_sub(1);
    Modifiers {
        shift: bits & 1 != 0,
        alt: bits & 2 != 0,
        ctrl: bits & 4 != 0,
    }
}

fn tilde_key(number: u32) -> Option<KeyCode> {
    Some(match number {
        1 | 7 => KeyCode::Home,
        2 => KeyCode::Insert,
        3 => KeyCode::Delete,
        4 | 8 => KeyCode::End,
        5 => KeyCode::PageUp,
        6 => KeyCode::PageDown,
        11..=15 => KeyCode::F((number - 10) as u8),
        17..=21 => KeyCode::F((number - 11) as u8),
        23 | 24 => KeyCode::F((number - 12) as u8),
        _ => return None,
    })
}

fn default_mouse(bytes: &[u8], last: bool) -> Step {
    let Some(&[code, col, row]) = bytes.get(3..6) else {
        return incomplete(last);
    };
    let [code, col, row] = [code, col, row].map(|value| u32::from(value.saturating_sub(32)));
    Step::Done(mouse(code, col, row, None).into_iter().collect(), 6)
}

fn sgr_mouse(params: &[u8], release: bool) -> Option<TerminalInput> {
    let params = params.strip_suffix(b";").unwrap_or(params);
    match numbers(params)?.as_slice() {
        &[code, col, row] => mouse(code, col, row, Some(release)),
        _ => None,
    }
}

fn mouse(code: u32, col: u32, row: u32, release: Option<bool>) -> Option<TerminalInput> {
    let button = match code & 0b11 {
        0 => Some(MouseButton::Left),
        1 => Some(MouseButton::Middle),
        2 => Some(MouseButton::Right),
        _ => None,
    };
    let kind = match (code & 0b1100_0000, code & 32 != 0, release) {
        (0, true, _) => MouseKind::Motion(button),
        (0, false, Some(true)) => MouseKind::Release(button?),
        (0, false, Some(false)) => MouseKind::Press(button?),
        (0, false, None) => match button {
            Some(pressed) => MouseKind::Press(pressed),
            None => MouseKind::Release(MouseButton::Left),
        },
        (64, false, _) => MouseKind::Wheel(match code & 0b11 {
            0 => WheelDirection::Up,
            1 => WheelDirection::Down,
            2 => WheelDirection::Left,
            _ => WheelDirection::Right,
        }),
        _ => return None,
    };
    let cell = |value: u32| u16::try_from(value.saturating_sub(1)).unwrap_or(u16::MAX);
    Some(TerminalInput::Mouse(MouseEvent::new(
        kind,
        cell(col),
        cell(row),
        Modifiers {
            shift: code & 4 != 0,
            alt: code & 8 != 0,
            ctrl: code & 16 != 0,
        },
    )))
}
