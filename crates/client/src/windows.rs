use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;

use gband_core::geometry::Size;
use gband_core::layout::{BandId, PaneContent, PaneId, Proportion, SessionAction};
use gband_lua::windows::{FloatFrame, Frame, PaneFrame, Run};
use gband_lua::{Color, Style};
use gband_protocol::ClientMessage;

pub struct OpenRequest {
    pub window: u32,
    pub band: BandId,
    pub after: Option<PaneId>,
    pub width: Option<Proportion>,
    pub focus: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Opened {
    Window(u32),
    Close(Option<PaneId>),
}

pub struct Windows {
    counter: Arc<AtomicU32>,
    floats: BTreeMap<u32, FloatFrame>,
    pending: BTreeMap<u32, bool>,
    panes: BTreeMap<PaneId, u32>,
    sizes: HashMap<PaneId, Size>,
}

impl Default for Windows {
    fn default() -> Self {
        Self::new()
    }
}

impl Windows {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(AtomicU32::new(1)),
            floats: BTreeMap::new(),
            pending: BTreeMap::new(),
            panes: BTreeMap::new(),
            sizes: HashMap::new(),
        }
    }

    pub fn counter(&self) -> Arc<AtomicU32> {
        Arc::clone(&self.counter)
    }

    pub fn focused_float(&self) -> Option<u32> {
        self.floats
            .iter()
            .find(|(_, frame)| frame.focused)
            .map(|(&id, _)| id)
    }

    pub fn floats(&self) -> Vec<&FloatFrame> {
        let mut floats: Vec<&FloatFrame> = self.floats.values().collect();
        floats.sort_by_key(|frame| frame.z);
        floats
    }

    pub fn window_of(&self, pane: PaneId) -> Option<u32> {
        self.panes.get(&pane).copied()
    }

    pub fn panes(&self) -> Vec<(PaneId, u32)> {
        self.panes
            .iter()
            .map(|(&pane, &window)| (pane, window))
            .collect()
    }

    pub fn open(&mut self, request: OpenRequest) -> ClientMessage {
        self.pending.insert(request.window, false);
        ClientMessage::Action(SessionAction::OpenPane {
            band: request.band,
            after: request.after,
            width: request.width,
            focus: request.focus,
            content: PaneContent::Plugin {
                request: request.window,
            },
        })
    }

    pub fn close(&mut self, window: u32) -> Option<ClientMessage> {
        if let Some(orphaned) = self.pending.get_mut(&window) {
            *orphaned = true;
            return None;
        }
        let pane = self
            .panes
            .iter()
            .find(|&(_, &candidate)| candidate == window)
            .map(|(&pane, _)| pane)?;
        self.forget(pane);
        Some(ClientMessage::Action(SessionAction::ClosePane(pane)))
    }

    pub fn opened(&mut self, request: u32, pane: Option<PaneId>) -> Opened {
        match self.pending.remove(&request) {
            Some(false) => {
                if let Some(pane) = pane {
                    self.panes.insert(pane, request);
                }
                Opened::Window(request)
            }
            _ => Opened::Close(pane),
        }
    }

    pub fn forget(&mut self, pane: PaneId) -> Option<u32> {
        self.sizes.remove(&pane);
        self.panes.remove(&pane)
    }

    pub fn resized(&mut self, pane: PaneId, size: Size) -> bool {
        self.sizes.insert(pane, size) != Some(size)
    }

    pub fn close_all(&mut self) -> Vec<ClientMessage> {
        self.floats.clear();
        self.sizes.clear();
        for orphaned in self.pending.values_mut() {
            *orphaned = true;
        }
        std::mem::take(&mut self.panes)
            .into_keys()
            .map(|pane| ClientMessage::Action(SessionAction::ClosePane(pane)))
            .collect()
    }

    pub fn present(&mut self, frames: Vec<(u32, Option<Frame>)>) -> Vec<ClientMessage> {
        let mut contents = Vec::new();
        for (window, frame) in frames {
            match frame {
                None => {
                    self.floats.remove(&window);
                }
                Some(Frame::Float(frame)) => {
                    self.floats.insert(window, frame);
                }
                Some(Frame::Pane(frame)) => {
                    let pane = self
                        .panes
                        .iter()
                        .find(|&(_, &candidate)| candidate == window)
                        .map(|(&pane, _)| pane);
                    if let Some(pane) = pane {
                        contents.push(ClientMessage::Content {
                            pane,
                            output: encode(&frame),
                        });
                    }
                }
            }
        }
        contents
    }
}

fn color(codes: &mut Vec<String>, layer: u8, color: Option<Color>) {
    match color {
        Some(Color::Rgb(r, g, b)) => codes.push(format!("{layer}8;2;{r};{g};{b}")),
        Some(Color::Index(index)) => codes.push(format!("{layer}8;5;{index}")),
        None => {}
    }
}

fn sgr(style: &Style) -> String {
    let mut codes = vec!["0".to_owned()];
    for (set, code) in [
        (style.bold, "1"),
        (style.dim, "2"),
        (style.italic, "3"),
        (style.underline, "4"),
        (style.reverse, "7"),
    ] {
        if set {
            codes.push(code.to_owned());
        }
    }
    color(&mut codes, 3, style.fg);
    color(&mut codes, 4, style.bg);
    format!("\x1b[{}m", codes.join(";"))
}

pub fn encode(frame: &PaneFrame) -> Vec<u8> {
    let mut output = String::from("\x1b[0m\x1b[2J\x1b[?25l");
    let blank = |output: &mut String, cells: usize| {
        output.push_str(&sgr(&frame.base));
        output.push_str(&" ".repeat(cells));
    };
    for row in 0..frame.rows {
        let _ = write!(output, "\x1b[{};1H", row + 1);
        let runs: &[Run] = frame.lines.get(usize::from(row)).map_or(&[], Vec::as_slice);
        let mut used = 0;
        for run in runs {
            output.push_str(&sgr(&run.style));
            output.push_str(&run.text);
            used += gband_lua::ui::width(&run.text);
        }
        if used < usize::from(frame.cols) {
            blank(&mut output, usize::from(frame.cols) - used);
        }
    }
    output.push_str("\x1b[0m");
    output.into_bytes()
}

#[cfg(test)]
mod tests {
    use gband_emulator::{Emulator, Grid};
    use vt100::Color as CellColor;

    use super::*;

    fn run(text: &str, style: Style) -> Run {
        Run {
            text: text.to_owned(),
            style,
        }
    }

    #[test]
    fn encoded_frame_reproduces_the_cells() {
        let base = Style {
            fg: Some(Color::Index(7)),
            ..Style::default()
        };
        let key = Style {
            fg: Some(Color::Rgb(0xff, 0x80, 0)),
            bold: true,
            ..Style::default()
        };
        let cursor = Style {
            reverse: true,
            ..base
        };
        let frame = PaneFrame {
            cols: 10,
            rows: 3,
            base,
            lines: vec![
                vec![run("C-h", key), run(" left", base), run("  ", base)],
                vec![run("next", cursor), run("      ", cursor)],
            ],
        };
        let mut grid = Grid::new(Size::new(10, 3));
        grid.process(b"old contents\r\nmore");
        grid.process(&encode(&frame));
        let contents = grid.contents();
        let rows: Vec<&str> = contents.lines().map(str::trim_end).collect();
        assert_eq!(rows, ["C-h left", "next", ""]);
        assert_eq!(grid.cursor(), None);
        let screen = grid.screen();
        let cell = |row, col| screen.cell(row, col).unwrap();
        assert_eq!(cell(0, 0).fgcolor(), CellColor::Rgb(0xff, 0x80, 0));
        assert!(cell(0, 0).bold());
        assert_eq!(cell(0, 4).fgcolor(), CellColor::Idx(7));
        assert!(!cell(0, 4).bold());
        assert!(cell(1, 9).inverse());
        assert_eq!(cell(2, 5).fgcolor(), CellColor::Idx(7));
        assert!(!cell(2, 5).inverse());
    }

    #[test]
    fn pending_requests_and_orphans() {
        let mut windows = Windows::new();
        let open = |window| OpenRequest {
            window,
            band: BandId(1),
            after: None,
            width: None,
            focus: true,
        };
        windows.open(open(1));
        windows.open(open(2));
        assert_eq!(windows.close(2), None);
        assert_eq!(windows.opened(1, Some(PaneId(5))), Opened::Window(1));
        assert_eq!(
            windows.opened(2, Some(PaneId(6))),
            Opened::Close(Some(PaneId(6)))
        );
        assert_eq!(windows.window_of(PaneId(5)), Some(1));
        assert_eq!(
            windows.close(1),
            Some(ClientMessage::Action(SessionAction::ClosePane(PaneId(5))))
        );
        assert_eq!(windows.window_of(PaneId(5)), None);
    }
}
