use std::io::{Read, Write};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{Context, Result};
use crossterm::event::{self, DisableBracketedPaste, EnableBracketedPaste, Event};
use crossterm::execute;
use gband::logging::{self, Role};
use gband_client::input::key_from_event;
use gband_core::input::{Modes, encode_key, encode_paste};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use ratatui::DefaultTerminal;
use ratatui::layout::Position;
use tui_term::widget::{Cursor, PseudoTerminal};

type SharedParser = Arc<Mutex<vt100::Parser<LoggingCallbacks>>>;

enum Message {
    Output,
    Exited,
    Input(Event),
}

struct LoggingCallbacks;

impl vt100::Callbacks for LoggingCallbacks {
    fn audible_bell(&mut self, _: &mut vt100::Screen) {
        tracing::debug!("bell");
    }

    fn unhandled_control(&mut self, _: &mut vt100::Screen, b: u8) {
        tracing::debug!("unhandled control {b:#04x}");
    }

    fn unhandled_escape(&mut self, _: &mut vt100::Screen, i1: Option<u8>, i2: Option<u8>, b: u8) {
        let sequence = [i1, i2, Some(b)].into_iter().flatten().map(char::from);
        tracing::debug!("unhandled escape \\e{}", sequence.collect::<String>());
    }

    fn unhandled_csi(
        &mut self,
        _: &mut vt100::Screen,
        i1: Option<u8>,
        i2: Option<u8>,
        params: &[&[u16]],
        c: char,
    ) {
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

    fn unhandled_osc(&mut self, _: &mut vt100::Screen, params: &[&[u8]]) {
        let params = params
            .iter()
            .map(|param| String::from_utf8_lossy(param))
            .collect::<Vec<_>>()
            .join(";");
        tracing::debug!("unhandled OSC \\e]{params}");
    }
}

fn main() -> Result<()> {
    let _guard = logging::init(Role::Client)?;
    let mut terminal = ratatui::init();
    execute!(std::io::stdout(), EnableBracketedPaste)?;
    let result = run(&mut terminal);
    let _ = execute!(std::io::stdout(), DisableBracketedPaste);
    ratatui::restore();
    if let Err(error) = &result {
        tracing::error!("spike failed: {error:#}");
    }
    result
}

fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let area = terminal.size()?;
    let size = pty_size(area.height, area.width);
    let pair = native_pty_system()
        .openpty(size)
        .context("cannot open a PTY")?;

    let mut command = CommandBuilder::new_default_prog();
    command.env("TERM", "xterm-256color");
    command.env("COLORTERM", "truecolor");
    command.cwd(std::env::current_dir()?);
    let mut child = pair
        .slave
        .spawn_command(command)
        .context("cannot spawn the shell")?;
    drop(pair.slave);

    let parser: SharedParser = Arc::new(Mutex::new(vt100::Parser::new_with_callbacks(
        size.rows,
        size.cols,
        0,
        LoggingCallbacks,
    )));
    let (sender, receiver) = mpsc::channel();
    spawn_reader(
        pair.master.try_clone_reader()?,
        Arc::clone(&parser),
        sender.clone(),
    );
    spawn_input(sender);
    let mut writer = pair.master.take_writer()?;

    draw(terminal, &parser)?;
    for message in receiver {
        match message {
            Message::Output => draw(terminal, &parser)?,
            Message::Exited => break,
            Message::Input(Event::Resize(cols, rows)) => {
                pair.master.resize(pty_size(rows, cols))?;
                parser.lock().unwrap().screen_mut().set_size(rows, cols);
                draw(terminal, &parser)?;
            }
            Message::Input(event) => forward(event, &parser, &mut writer)?,
        }
    }

    let status = child.wait()?;
    tracing::info!("shell exited with {status}");
    Ok(())
}

fn spawn_reader(mut reader: Box<dyn Read + Send>, parser: SharedParser, sender: Sender<Message>) {
    thread::spawn(move || {
        let mut buffer = [0u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    parser.lock().unwrap().process(&buffer[..n]);
                    if sender.send(Message::Output).is_err() {
                        return;
                    }
                }
            }
        }
        let _ = sender.send(Message::Exited);
    });
}

fn spawn_input(sender: Sender<Message>) {
    thread::spawn(move || {
        while let Ok(event) = event::read() {
            if sender.send(Message::Input(event)).is_err() {
                break;
            }
        }
    });
}

fn forward(event: Event, parser: &SharedParser, writer: &mut impl Write) -> Result<()> {
    let bytes = match event {
        Event::Key(key_event) => match key_from_event(&key_event) {
            Some(key) => encode_key(key, modes(parser)),
            None => return Ok(()),
        },
        Event::Paste(text) => encode_paste(&text, modes(parser)),
        _ => return Ok(()),
    };
    writer.write_all(&bytes)?;
    writer.flush()?;
    Ok(())
}

fn modes(parser: &SharedParser) -> Modes {
    let parser = parser.lock().unwrap();
    let screen = parser.screen();
    Modes {
        application_cursor: screen.application_cursor(),
        bracketed_paste: screen.bracketed_paste(),
    }
}

fn draw(terminal: &mut DefaultTerminal, parser: &SharedParser) -> Result<()> {
    let parser = parser.lock().unwrap();
    let screen = parser.screen();
    terminal.draw(|frame| {
        frame.render_widget(
            PseudoTerminal::new(screen).cursor(Cursor::default().visibility(false)),
            frame.area(),
        );
        if !screen.hide_cursor() {
            let (row, col) = screen.cursor_position();
            frame.set_cursor_position(Position::new(col, row));
        }
    })?;
    Ok(())
}

fn pty_size(rows: u16, cols: u16) -> PtySize {
    PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    }
}
