const ESC: u8 = 0x1b;
const CAN: u8 = 0x18;
const SUB: u8 = 0x1a;

const INTRODUCER: &[u8] = b"\x1b_G";
const TERMINATOR: &[u8] = b"\x1b\\";

pub const GRAPHICS_LIMIT: usize = 16 << 20;

#[derive(Clone, Copy, Default)]
enum State {
    #[default]
    Ground,
    Escape,
    Introducer,
    Graphics,
    GraphicsEscape,
    Skip,
}

#[derive(Default)]
pub struct Scanner {
    state: State,
    pending: Vec<u8>,
}

fn after_escape(byte: u8) -> State {
    match byte {
        b'_' => State::Introducer,
        ESC => State::Escape,
        _ => State::Ground,
    }
}

fn string_end(bytes: &[u8]) -> usize {
    bytes
        .iter()
        .position(|&byte| matches!(byte, ESC | CAN | SUB))
        .unwrap_or(bytes.len())
}

impl Scanner {
    pub fn scan(&mut self, bytes: &[u8]) -> (usize, Option<Vec<u8>>) {
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
                State::Escape => after_escape(byte),
                State::Introducer => match byte {
                    b'G' => {
                        self.pending.extend_from_slice(INTRODUCER);
                        State::Graphics
                    }
                    ESC => State::Escape,
                    CAN | SUB => State::Ground,
                    _ => State::Skip,
                },
                State::Graphics => match byte {
                    ESC => {
                        self.pending.push(ESC);
                        State::GraphicsEscape
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
                        if self.pending.len() > GRAPHICS_LIMIT {
                            self.pending = Vec::new();
                            State::Skip
                        } else {
                            State::Graphics
                        }
                    }
                },
                State::GraphicsEscape if byte == b'\\' => {
                    self.pending.push(byte);
                    self.state = State::Ground;
                    return (total - bytes.len(), Some(std::mem::take(&mut self.pending)));
                }
                State::GraphicsEscape => {
                    self.pending.clear();
                    after_escape(byte)
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
