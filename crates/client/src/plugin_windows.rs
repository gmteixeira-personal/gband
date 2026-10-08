use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;

use gband_core::geometry::Size;
use gband_core::layout::{BandId, Proportion, SessionAction, WindowContent, WindowId};
use gband_lua::plugin_windows::{FloatingFrame, Frame, Run, TiledFrame};
use gband_lua::{Color, Style};
use gband_protocol::ClientMessage;

pub struct OpenRequest {
    pub plugin_window: u32,
    pub band: BandId,
    pub after: Option<WindowId>,
    pub width: Option<Proportion>,
    pub focus: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Opened {
    PluginWindow(u32),
    Close(Option<WindowId>),
}

pub struct PluginWindows {
    counter: Arc<AtomicU32>,
    floats: BTreeMap<u32, FloatingFrame>,
    hovers: BTreeSet<u32>,
    pending: BTreeMap<u32, bool>,
    windows: BTreeMap<WindowId, u32>,
    sizes: HashMap<WindowId, Size>,
}

impl Default for PluginWindows {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginWindows {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(AtomicU32::new(1)),
            floats: BTreeMap::new(),
            hovers: BTreeSet::new(),
            pending: BTreeMap::new(),
            windows: BTreeMap::new(),
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

    pub fn floats(&self) -> Vec<(u32, &FloatingFrame)> {
        let mut floats: Vec<(u32, &FloatingFrame)> =
            self.floats.iter().map(|(&id, frame)| (id, frame)).collect();
        floats.sort_by_key(|(_, frame)| frame.z);
        floats
    }

    pub fn hovers(&self, plugin_window: u32) -> bool {
        self.hovers.contains(&plugin_window)
    }

    pub fn plugin_window_of(&self, window: WindowId) -> Option<u32> {
        self.windows.get(&window).copied()
    }

    pub fn windows(&self) -> Vec<(WindowId, u32)> {
        self.windows
            .iter()
            .map(|(&window, &plugin_window)| (window, plugin_window))
            .collect()
    }

    pub fn open(&mut self, request: OpenRequest) -> ClientMessage {
        self.pending.insert(request.plugin_window, false);
        ClientMessage::Action(SessionAction::OpenWindow {
            band: request.band,
            after: request.after,
            width: request.width,
            floating: false,
            focus: request.focus,
            content: WindowContent::Plugin {
                request: request.plugin_window,
            },
        })
    }

    pub fn close(&mut self, plugin_window: u32) -> Option<ClientMessage> {
        if let Some(orphaned) = self.pending.get_mut(&plugin_window) {
            *orphaned = true;
            return None;
        }
        let window = self
            .windows
            .iter()
            .find(|&(_, &candidate)| candidate == plugin_window)
            .map(|(&window, _)| window)?;
        self.forget(window);
        Some(ClientMessage::Action(SessionAction::CloseWindow(window)))
    }

    pub fn opened(&mut self, request: u32, window: Option<WindowId>) -> Opened {
        match self.pending.remove(&request) {
            Some(false) => {
                if let Some(window) = window {
                    self.windows.insert(window, request);
                }
                Opened::PluginWindow(request)
            }
            _ => Opened::Close(window),
        }
    }

    pub fn forget(&mut self, window: WindowId) -> Option<u32> {
        self.sizes.remove(&window);
        self.windows.remove(&window)
    }

    pub fn resized(&mut self, window: WindowId, size: Size) -> bool {
        self.sizes.insert(window, size) != Some(size)
    }

    pub fn close_all(&mut self) -> Vec<ClientMessage> {
        self.floats.clear();
        self.hovers.clear();
        self.sizes.clear();
        for orphaned in self.pending.values_mut() {
            *orphaned = true;
        }
        std::mem::take(&mut self.windows)
            .into_keys()
            .map(|window| ClientMessage::Action(SessionAction::CloseWindow(window)))
            .collect()
    }

    pub fn present(&mut self, frames: Vec<(u32, Option<Frame>)>) -> Vec<ClientMessage> {
        let mut contents = Vec::new();
        for (plugin_window, frame) in frames {
            let hover = match &frame {
                Some(Frame::Floating(frame)) => frame.hover,
                Some(Frame::Tiled(frame)) => frame.hover,
                None => false,
            };
            if hover {
                self.hovers.insert(plugin_window);
            } else {
                self.hovers.remove(&plugin_window);
            }
            match frame {
                None => {
                    self.floats.remove(&plugin_window);
                }
                Some(Frame::Floating(frame)) => {
                    self.floats.insert(plugin_window, frame);
                }
                Some(Frame::Tiled(frame)) => {
                    let window = self
                        .windows
                        .iter()
                        .find(|&(_, &candidate)| candidate == plugin_window)
                        .map(|(&window, _)| window);
                    if let Some(window) = window {
                        contents.push(ClientMessage::Content {
                            window,
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

pub fn encode(frame: &TiledFrame) -> Vec<u8> {
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
        let frame = TiledFrame {
            cols: 10,
            rows: 3,
            base,
            lines: vec![
                vec![run("C-h", key), run(" left", base), run("  ", base)],
                vec![run("next", cursor), run("      ", cursor)],
            ],
            hover: false,
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
        let mut plugin_windows = PluginWindows::new();
        let open = |plugin_window| OpenRequest {
            plugin_window,
            band: BandId(1),
            after: None,
            width: None,
            focus: true,
        };
        plugin_windows.open(open(1));
        plugin_windows.open(open(2));
        assert_eq!(plugin_windows.close(2), None);
        assert_eq!(
            plugin_windows.opened(1, Some(WindowId(5))),
            Opened::PluginWindow(1)
        );
        assert_eq!(
            plugin_windows.opened(2, Some(WindowId(6))),
            Opened::Close(Some(WindowId(6)))
        );
        assert_eq!(plugin_windows.plugin_window_of(WindowId(5)), Some(1));
        assert_eq!(
            plugin_windows.close(1),
            Some(ClientMessage::Action(SessionAction::CloseWindow(WindowId(
                5
            ))))
        );
        assert_eq!(plugin_windows.plugin_window_of(WindowId(5)), None);
    }
}
