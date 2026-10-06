use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use gband_core::geometry::Size;
use gband_emulator::{Emulator, Grid};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use rustix::process::{Pid, Signal};

use crate::env::{TIMEOUT, TestEnv};

type SharedWriter = Arc<Mutex<Input>>;

struct Input {
    writer: Box<dyn Write + Send>,
    written: u64,
}

impl Input {
    fn send(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(bytes)?;
        self.written += bytes.len() as u64;
        self.writer.flush()
    }
}

pub struct Attached {
    pub child: Box<dyn Child + Send + Sync>,
    pub master: Box<dyn MasterPty + Send>,
    writer: SharedWriter,
    grid: Arc<Mutex<Grid>>,
}

impl Attached {
    pub fn start(env: &TestEnv, cols: u16, rows: u16) -> Self {
        Self::start_with(env, &env.executable, &["attach"], cols, rows, |_| {})
    }

    pub fn start_with(
        env: &TestEnv,
        executable: impl AsRef<Path>,
        args: &[&str],
        cols: u16,
        rows: u16,
        adjust: impl FnOnce(&mut CommandBuilder),
    ) -> Self {
        Self::spawn(
            env,
            executable,
            args,
            Grid::new(Size::new(cols, rows)),
            adjust,
        )
        .unwrap()
    }

    pub fn spawn(
        env: &TestEnv,
        executable: impl AsRef<Path>,
        args: &[&str],
        grid: Grid,
        adjust: impl FnOnce(&mut CommandBuilder),
    ) -> std::io::Result<Self> {
        let Size { cols, rows } = grid.size();
        let pair = native_pty_system()
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(std::io::Error::other)?;
        let mut command = CommandBuilder::new(executable.as_ref());
        command.args(args);
        env.configure(&mut command);
        adjust(&mut command);
        let child = pair
            .slave
            .spawn_command(command)
            .map_err(std::io::Error::other)?;
        drop(pair.slave);

        let grid = Arc::new(Mutex::new(grid));
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(std::io::Error::other)?;
        let writer: SharedWriter = Arc::new(Mutex::new(Input {
            writer: pair.master.take_writer().map_err(std::io::Error::other)?,
            written: 0,
        }));
        let output = Arc::clone(&grid);
        let replies = Arc::clone(&writer);
        thread::spawn(move || {
            let mut buffer = [0u8; 8192];
            while let Ok(n) = reader.read(&mut buffer) {
                if n == 0 {
                    break;
                }
                let reply = {
                    let mut grid = output.lock().unwrap();
                    grid.process(&buffer[..n]);
                    grid.take_write_back()
                };
                if !reply.is_empty() {
                    let _ = replies.lock().unwrap().send(&reply);
                }
            }
        });
        Ok(Self {
            child,
            master: pair.master,
            writer,
            grid,
        })
    }

    pub fn screen(&self) -> MutexGuard<'_, Grid> {
        self.grid.lock().unwrap()
    }

    pub fn contents(&self) -> String {
        self.screen().contents()
    }

    pub fn try_send(&self, bytes: &[u8]) -> std::io::Result<()> {
        self.writer.lock().unwrap().send(bytes)
    }

    pub fn written(&self) -> u64 {
        self.writer.lock().unwrap().written
    }

    pub fn send(&mut self, bytes: &[u8]) {
        self.try_send(bytes).unwrap();
    }

    pub fn run(&mut self, line: &str) {
        self.send(line.as_bytes());
        self.send(b"\r");
    }

    pub fn wait_for(&self, what: &str, predicate: impl Fn(&Grid) -> bool) {
        let deadline = Instant::now() + TIMEOUT;
        loop {
            {
                let screen = self.screen();
                if predicate(&screen) {
                    return;
                }
                assert!(
                    Instant::now() < deadline,
                    "timed out waiting for {what}; screen:\n{}",
                    screen.contents()
                );
            }
            thread::sleep(Duration::from_millis(20));
        }
    }

    pub fn wait_for_text(&self, text: &str) {
        self.wait_for(text, |screen| screen.contents().contains(text));
    }

    pub fn tiles(&self) -> Vec<Tile> {
        tiles(&self.screen())
    }

    pub fn focused_lines(&self) -> Vec<String> {
        focused_lines(&self.screen())
    }

    pub fn wait_for_focused(&self, what: &str, predicate: impl Fn(&[String]) -> bool) {
        self.wait_for(what, |screen| predicate(&focused_lines(screen)));
    }

    pub fn wait_for_line(&self, expected: &str) {
        self.wait_for_focused(expected, |lines| lines.iter().any(|line| line == expected));
    }

    pub fn wait_for_prompt(&self) {
        self.wait_for_focused("a prompt", |lines| {
            lines.iter().any(|line| line.ends_with('$'))
        });
    }

    pub fn shell_pid(&mut self, env: &TestEnv) -> i32 {
        self.run("echo pid=$$");
        self.wait_for_focused("the shell pid", |lines| {
            lines.iter().any(|line| parse_pid(line).is_some())
        });
        let pid = self.last_pid();
        env.track_shell(pid);
        pid
    }

    pub fn last_pid(&self) -> i32 {
        self.focused_lines()
            .iter()
            .filter_map(|line| parse_pid(line))
            .next_back()
            .expect("no pid line in the focused window")
    }

    pub fn wait_exit(&mut self) -> u32 {
        let deadline = Instant::now() + TIMEOUT;
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                thread::sleep(Duration::from_millis(100));
                return status.exit_code();
            }
            assert!(
                Instant::now() < deadline,
                "client did not exit; screen:\n{}",
                self.contents()
            );
            thread::sleep(Duration::from_millis(20));
        }
    }

    pub fn try_resize(&self, cols: u16, rows: u16) -> std::io::Result<()> {
        self.screen().resize(Size::new(cols, rows));
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(std::io::Error::other)
    }

    pub fn resize(&self, cols: u16, rows: u16) {
        self.try_resize(cols, rows).unwrap();
    }

    pub fn kill(&mut self) {
        let pid = self.child.process_id().unwrap() as i32;
        rustix::process::kill_process(Pid::from_raw(pid).unwrap(), Signal::KILL).unwrap();
        self.child.wait().unwrap();
    }

    pub fn finish(&mut self) {
        if let Ok(None) = self.child.try_wait() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

impl Drop for Attached {
    fn drop(&mut self) {
        self.finish();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tile {
    pub top: u16,
    pub left: u16,
    pub bottom: u16,
    pub right: u16,
    pub focused: bool,
}

fn symbol(screen: &Grid, row: u16, col: u16) -> Option<String> {
    let cell = screen.screen().cell(row, col)?;
    Some(if cell.has_contents() {
        cell.contents().to_owned()
    } else {
        " ".to_owned()
    })
}

impl Tile {
    pub fn lines(&self, screen: &Grid) -> Vec<String> {
        (self.top + 1..self.bottom)
            .map(|row| {
                (self.left + 1..self.right)
                    .map(|col| symbol(screen, row, col).unwrap_or_else(|| " ".to_owned()))
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect()
    }
}

fn symbol_at(screen: &Grid, row: u16, col: u16) -> Option<(String, bool)> {
    let bold = screen.screen().cell(row, col)?.bold();
    Some((symbol(screen, row, col)?, bold))
}

pub fn tiles(screen: &Grid) -> Vec<Tile> {
    let Size { cols, rows } = screen.size();
    let mut found = Vec::new();
    for top in 0..rows {
        for left in 0..cols {
            let Some((symbol, focused)) = symbol_at(screen, top, left) else {
                continue;
            };
            if symbol != "┌" {
                continue;
            }
            let right = (left + 1..cols)
                .find(|&col| symbol_at(screen, top, col).is_some_and(|(s, _)| s == "┐"));
            let bottom = (top + 1..rows)
                .find(|&row| symbol_at(screen, row, left).is_some_and(|(s, _)| s == "└"));
            if let (Some(right), Some(bottom)) = (right, bottom) {
                found.push(Tile {
                    top,
                    left,
                    bottom,
                    right,
                    focused,
                });
            }
        }
    }
    found
}

pub fn focused_lines(screen: &Grid) -> Vec<String> {
    tiles(screen)
        .into_iter()
        .find(|tile| tile.focused)
        .map(|tile| tile.lines(screen))
        .unwrap_or_default()
}

fn parse_pid(line: &str) -> Option<i32> {
    line.trim_end().strip_prefix("pid=")?.parse().ok()
}
