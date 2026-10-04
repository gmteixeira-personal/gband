use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: Modifiers,
}

impl Key {
    pub fn new(code: KeyCode, modifiers: Modifiers) -> Self {
        Self { code, modifiers }
    }

    pub fn plain(code: KeyCode) -> Self {
        Self::new(code, Modifiers::NONE)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyCode {
    Char(char),
    Enter,
    Tab,
    BackTab,
    Backspace,
    Escape,
    Up,
    Down,
    Right,
    Left,
    Home,
    End,
    Insert,
    Delete,
    PageUp,
    PageDown,
    F(u8),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Modifiers {
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

impl Modifiers {
    pub const NONE: Self = Self {
        shift: false,
        alt: false,
        ctrl: false,
    };
    pub const SHIFT: Self = Self {
        shift: true,
        ..Self::NONE
    };
    pub const ALT: Self = Self {
        alt: true,
        ..Self::NONE
    };
    pub const CTRL: Self = Self {
        ctrl: true,
        ..Self::NONE
    };

    pub fn is_empty(self) -> bool {
        self == Self::NONE
    }

    fn parameter(self) -> u8 {
        1 + u8::from(self.shift) + 2 * u8::from(self.alt) + 4 * u8::from(self.ctrl)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modes {
    pub application_cursor: bool,
    pub bracketed_paste: bool,
}

const ESC: u8 = 0x1b;
const PASTE_START: &[u8] = b"\x1b[200~";
const PASTE_END: &[u8] = b"\x1b[201~";

pub fn encode_key(key: Key, modes: Modes) -> Vec<u8> {
    let modifiers = key.modifiers;
    match key.code {
        KeyCode::Char(c) => alt_prefixed(modifiers, encode_char(c, modifiers.ctrl)),
        KeyCode::Enter => alt_prefixed(modifiers, b"\r".to_vec()),
        KeyCode::Tab => alt_prefixed(modifiers, b"\t".to_vec()),
        KeyCode::BackTab => alt_prefixed(modifiers, b"\x1b[Z".to_vec()),
        KeyCode::Backspace => {
            let byte = if modifiers.ctrl { 0x08 } else { 0x7f };
            alt_prefixed(modifiers, vec![byte])
        }
        KeyCode::Escape => alt_prefixed(modifiers, vec![ESC]),
        KeyCode::Up => cursor(b'A', modifiers, modes),
        KeyCode::Down => cursor(b'B', modifiers, modes),
        KeyCode::Right => cursor(b'C', modifiers, modes),
        KeyCode::Left => cursor(b'D', modifiers, modes),
        KeyCode::Home => cursor(b'H', modifiers, modes),
        KeyCode::End => cursor(b'F', modifiers, modes),
        KeyCode::Insert => tilde(2, modifiers),
        KeyCode::Delete => tilde(3, modifiers),
        KeyCode::PageUp => tilde(5, modifiers),
        KeyCode::PageDown => tilde(6, modifiers),
        KeyCode::F(n @ 1..=4) => {
            let letter = b'P' + (n - 1);
            if modifiers.is_empty() {
                vec![ESC, b'O', letter]
            } else {
                modified_letter(letter, modifiers)
            }
        }
        KeyCode::F(n @ 5..=12) => {
            const NUMBERS: [u8; 8] = [15, 17, 18, 19, 20, 21, 23, 24];
            tilde(NUMBERS[usize::from(n - 5)], modifiers)
        }
        KeyCode::F(_) => Vec::new(),
    }
}

pub fn encode_paste(text: &str, modes: Modes) -> Vec<u8> {
    let filtered: String = text.chars().filter(|&c| !is_filtered_control(c)).collect();
    if modes.bracketed_paste {
        [PASTE_START, filtered.as_bytes(), PASTE_END].concat()
    } else {
        filtered
            .replace("\r\n", "\r")
            .replace('\n', "\r")
            .into_bytes()
    }
}

fn is_filtered_control(c: char) -> bool {
    matches!(c, '\0'..='\x1f' | '\u{80}'..='\u{9f}') && !matches!(c, '\t' | '\n' | '\r')
}

fn encode_char(c: char, ctrl: bool) -> Vec<u8> {
    match ctrl.then(|| control_code(c)).flatten() {
        Some(byte) => vec![byte],
        None => c.to_string().into_bytes(),
    }
}

fn control_code(c: char) -> Option<u8> {
    match c {
        'a'..='z' => Some(c as u8 - b'a' + 1),
        'A'..='Z' => Some(c as u8 - b'A' + 1),
        ' ' | '@' | '2' => Some(0x00),
        '[' | '3' => Some(0x1b),
        '\\' | '4' => Some(0x1c),
        ']' | '5' => Some(0x1d),
        '^' | '6' => Some(0x1e),
        '_' | '/' | '7' => Some(0x1f),
        '?' | '8' => Some(0x7f),
        _ => None,
    }
}

fn alt_prefixed(modifiers: Modifiers, bytes: Vec<u8>) -> Vec<u8> {
    if modifiers.alt {
        [vec![ESC], bytes].concat()
    } else {
        bytes
    }
}

fn cursor(letter: u8, modifiers: Modifiers, modes: Modes) -> Vec<u8> {
    if !modifiers.is_empty() {
        modified_letter(letter, modifiers)
    } else if modes.application_cursor {
        vec![ESC, b'O', letter]
    } else {
        vec![ESC, b'[', letter]
    }
}

fn modified_letter(letter: u8, modifiers: Modifiers) -> Vec<u8> {
    let mut bytes = format!("\x1b[1;{}", modifiers.parameter()).into_bytes();
    bytes.push(letter);
    bytes
}

fn tilde(number: u8, modifiers: Modifiers) -> Vec<u8> {
    if modifiers.is_empty() {
        format!("\x1b[{number}~").into_bytes()
    } else {
        format!("\x1b[{number};{}~", modifiers.parameter()).into_bytes()
    }
}
