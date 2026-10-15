const ESC: u8 = 0x1b;
const CAN: u8 = 0x18;
const SUB: u8 = 0x1a;

const INTRODUCER: &[u8] = b"\x1b_G";
const DCS: &[u8] = b"\x1bP";
const TERMINATOR: &[u8] = b"\x1b\\";

pub const GRAPHICS_LIMIT: usize = 16 << 20;

#[derive(Clone, Copy)]
enum Kind {
    Graphics,
    Sixel,
}

#[derive(Clone, Copy, Default)]
enum State {
    #[default]
    Ground,
    Escape,
    Introducer,
    DcsParams,
    Payload(Kind),
    PayloadEscape(Kind),
    Skip,
}

pub enum Found {
    Graphics(Vec<u8>),
    SixelStart,
    Sixel(Vec<u8>),
}

#[derive(Default)]
pub struct Scanner {
    state: State,
    pending: Vec<u8>,
}

fn string_end(bytes: &[u8]) -> usize {
    bytes
        .iter()
        .position(|&byte| matches!(byte, ESC | CAN | SUB))
        .unwrap_or(bytes.len())
}

impl Scanner {
    fn after_escape(&mut self, byte: u8) -> State {
        match byte {
            b'_' => State::Introducer,
            b'P' => {
                self.pending.extend_from_slice(DCS);
                State::DcsParams
            }
            ESC => State::Escape,
            _ => State::Ground,
        }
    }

    fn overflowed(&mut self) -> bool {
        let over = self.pending.len() > GRAPHICS_LIMIT;
        if over {
            self.pending = Vec::new();
        }
        over
    }

    pub fn scan(&mut self, bytes: &[u8]) -> (usize, Option<Found>) {
        let total = bytes.len();
        let mut bytes = bytes;
        while let Some((&byte, rest)) = bytes.split_first() {
            bytes = rest;
            self.state = match self.state {
                State::Ground if byte == ESC => State::Escape,
                State::Ground => {
                    let skip = bytes.iter().position(|&b| b == ESC).unwrap_or(bytes.len());
                    bytes = &bytes[skip..];
                    State::Ground
                }
                State::Escape => self.after_escape(byte),
                State::Introducer => match byte {
                    b'G' => {
                        self.pending.extend_from_slice(INTRODUCER);
                        State::Payload(Kind::Graphics)
                    }
                    ESC => State::Escape,
                    CAN | SUB => State::Ground,
                    _ => State::Skip,
                },
                State::DcsParams => match byte {
                    b'q' => {
                        self.pending.push(byte);
                        self.state = State::Payload(Kind::Sixel);
                        return (total - bytes.len(), Some(Found::SixelStart));
                    }
                    b'0'..=b'9' | b';' => {
                        self.pending.push(byte);
                        if self.overflowed() {
                            State::Skip
                        } else {
                            State::DcsParams
                        }
                    }
                    ESC => {
                        self.pending.clear();
                        State::Escape
                    }
                    CAN | SUB => {
                        self.pending.clear();
                        State::Ground
                    }
                    _ => {
                        self.pending.clear();
                        State::Skip
                    }
                },
                State::Payload(kind) => match byte {
                    ESC => {
                        self.pending.push(ESC);
                        State::PayloadEscape(kind)
                    }
                    CAN | SUB => {
                        self.pending.clear();
                        State::Ground
                    }
                    _ => {
                        let run = string_end(bytes);
                        self.pending.push(byte);
                        self.pending.extend_from_slice(&bytes[..run]);
                        bytes = &bytes[run..];
                        if self.overflowed() {
                            State::Skip
                        } else {
                            State::Payload(kind)
                        }
                    }
                },
                State::PayloadEscape(kind) if byte == b'\\' => {
                    self.pending.push(byte);
                    self.state = State::Ground;
                    let sequence = std::mem::take(&mut self.pending);
                    let found = match kind {
                        Kind::Graphics => Found::Graphics(sequence),
                        Kind::Sixel => Found::Sixel(sequence),
                    };
                    return (total - bytes.len(), Some(found));
                }
                State::PayloadEscape(_) => {
                    self.pending.clear();
                    self.after_escape(byte)
                }
                State::Skip => match byte {
                    ESC => State::Escape,
                    CAN | SUB => State::Ground,
                    _ => {
                        bytes = &bytes[string_end(bytes)..];
                        State::Skip
                    }
                },
            };
        }
        (total, None)
    }
}

pub fn query_reply(sequence: &[u8]) -> Option<Vec<u8>> {
    let body = sequence
        .strip_prefix(INTRODUCER)?
        .strip_suffix(TERMINATOR)?;
    let control = body.split(|&byte| byte == b';').next()?;
    let mut query = false;
    let mut id = 0u32;
    for key in control.split(|&byte| byte == b',') {
        if key == b"a=q" {
            query = true;
        } else if let Some(value) = key.strip_prefix(b"i=") {
            id = std::str::from_utf8(value)
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(0);
        }
    }
    query.then(|| format!("\x1b_Gi={id};OK\x1b\\").into_bytes())
}
