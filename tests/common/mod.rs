#![allow(dead_code)]

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child as Process, Command, Stdio};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use gband_core::geometry::Size;
pub use gband_emulator::{Emulator, Grid};
use gband_test_support::cell;
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use ratatui::style::Modifier;
use rustix::process::{Pid, Signal};

pub const TIMEOUT: Duration = Duration::from_secs(10);
pub const GBAND: &str = env!("CARGO_BIN_EXE_gband");

const FNV_OFFSET: u32 = 0x811c_9dc5;
const FNV_PRIME: u32 = 0x0100_0193;

pub fn scratch_root(prefix: &str, name: &str) -> PathBuf {
    let worktree = env!("CARGO_MANIFEST_DIR")
        .bytes()
        .fold(FNV_OFFSET, |hash, byte| {
            (hash ^ u32::from(byte)).wrapping_mul(FNV_PRIME)
        });
    std::env::temp_dir().join(format!("{prefix}-{worktree:08x}-{name}"))
}

pub struct TestEnv {
    pub root: PathBuf,
    pub work: PathBuf,
    shells: Mutex<Vec<i32>>,
}

impl TestEnv {
    pub fn new(name: &str) -> Self {
        let root = scratch_root("gband-e2e", name);
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        let work = root.join("work");
        fs::create_dir_all(&work).unwrap();
        Self {
            root,
            work,
            shells: Mutex::new(Vec::new()),
        }
    }

    pub fn state_home(&self) -> PathBuf {
        self.root.join("state")
    }

    pub fn runtime_home(&self) -> PathBuf {
        self.root.join("run")
    }

    pub fn config_home(&self) -> PathBuf {
        self.root.join("config")
    }

    pub fn config_dir(&self) -> PathBuf {
        self.config_home().join("gband")
    }

    pub fn data_home(&self) -> PathBuf {
        self.root.join("data")
    }

    pub fn plugins_dir(&self) -> PathBuf {
        self.data_home().join("gband").join("plugins")
    }

    pub fn write_plugin_file(&self, plugin: &str, relative: &str, source: &str) -> PathBuf {
        let path = self.plugins_dir().join(plugin).join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, source).unwrap();
        path
    }

    pub fn write_manifest(&self, plugin: &str, extra: &str) -> PathBuf {
        self.write_plugin_file(
            plugin,
            "plugin.lua",
            &format!("return {{ name = '{plugin}', version = '0.1.0'{extra} }}"),
        )
    }

    pub fn write_client_plugin(&self, plugin: &str, source: &str) -> PathBuf {
        self.write_manifest(plugin, "");
        self.write_plugin_file(plugin, "client.lua", source)
    }

    pub fn write_server_plugin(&self, plugin: &str, source: &str) -> PathBuf {
        self.write_manifest(plugin, "");
        self.write_plugin_file(plugin, "server.lua", source)
    }

    pub fn user_lua(&self) -> PathBuf {
        self.config_dir().join("user").join("init.lua")
    }

    pub fn server_lua(&self) -> PathBuf {
        self.config_dir().join("user").join("server.lua")
    }

    pub fn write_server_config(&self, source: &str) {
        let path = self.server_lua();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }

    pub fn defaults_lua(&self) -> PathBuf {
        self.config_dir().join("defaults").join("init.lua")
    }

    pub fn write_config(&self, source: &str) {
        let path = self.user_lua();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }

    pub fn runtime_dir(&self) -> PathBuf {
        self.runtime_home().join("gband")
    }

    pub fn socket(&self) -> PathBuf {
        self.socket_named("default")
    }

    pub fn socket_named(&self, name: &str) -> PathBuf {
        self.runtime_dir().join(format!("{name}.sock"))
    }

    pub fn lock(&self) -> PathBuf {
        self.runtime_dir().join("default.lock")
    }

    pub fn log_text(&self, role: &str) -> String {
        let dir = self.state_home().join("gband").join("log");
        let Ok(entries) = fs::read_dir(dir) else {
            return String::new();
        };
        entries
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with(&format!("{role}."))
            })
            .map(|path| fs::read_to_string(path).unwrap())
            .collect()
    }

    pub fn server_pid(&self) -> i32 {
        let deadline = Instant::now() + TIMEOUT;
        loop {
            if let Ok(record) = fs::read_to_string(self.lock())
                && let Ok(pid) = record.trim().parse()
            {
                return pid;
            }
            assert!(Instant::now() < deadline, "no pid record appeared");
            thread::sleep(Duration::from_millis(20));
        }
    }

    pub fn track_shell(&self, pid: i32) {
        self.shells.lock().unwrap().push(pid);
    }

    pub fn configure(&self, command: &mut CommandBuilder) {
        command.env("XDG_STATE_HOME", self.state_home());
        command.env("XDG_RUNTIME_DIR", self.runtime_home());
        command.env("XDG_CONFIG_HOME", self.config_home());
        command.env("XDG_DATA_HOME", self.data_home());
        command.env("SHELL", "/bin/sh");
        command.env("TERM", "xterm-256color");
        command.env("INPUTRC", "/dev/null");
        command.env_remove("GBAND");
        command.env_remove("GBAND_LOG");
        command.cwd(&self.work);
    }

    pub fn command(&self, executable: impl AsRef<Path>, args: &[&str]) -> Command {
        let mut command = Command::new(executable.as_ref());
        command
            .args(args)
            .env("XDG_STATE_HOME", self.state_home())
            .env("XDG_RUNTIME_DIR", self.runtime_home())
            .env("XDG_CONFIG_HOME", self.config_home())
            .env("XDG_DATA_HOME", self.data_home())
            .env("SHELL", "/bin/sh")
            .env("INPUTRC", "/dev/null")
            .env_remove("GBAND")
            .env_remove("GBAND_LOG")
            .current_dir(&self.work);
        command
    }

    pub fn start_server(&self, shell: &str) -> Process {
        self.spawn_server(&["server"], shell, &self.socket())
    }

    pub fn start_named_server(&self, name: &str, shell: &str) -> Process {
        self.spawn_server(&["-S", name, "server"], shell, &self.socket_named(name))
    }

    fn spawn_server(&self, args: &[&str], shell: &str, socket: &Path) -> Process {
        let process = self
            .command(GBAND, args)
            .env("SHELL", shell)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        wait_until(|| socket.exists(), "the server socket");
        process
    }

    pub fn shell_of(&self, server: &Process) -> i32 {
        let server = server.id() as i32;
        wait_until(|| !children(server).is_empty(), "the server's shell");
        let shells = children(server);
        assert_eq!(shells.len(), 1, "{shells:?}");
        self.track_shell(shells[0]);
        shells[0]
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        if self.socket().exists() {
            let _ = self
                .command(GBAND, &["kill-server"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        for pid in self.shells.lock().unwrap().drain(..) {
            if let Some(pid) = Pid::from_raw(pid) {
                let _ = rustix::process::kill_process(pid, Signal::KILL);
            }
        }
    }
}

pub struct Attached {
    pub child: Box<dyn Child + Send + Sync>,
    pub master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    grid: Arc<Mutex<Grid>>,
}

impl Attached {
    pub fn start(env: &TestEnv, cols: u16, rows: u16) -> Self {
        Self::start_with(env, GBAND, &["attach"], cols, rows, |_| {})
    }

    pub fn start_with(
        env: &TestEnv,
        executable: impl AsRef<Path>,
        args: &[&str],
        cols: u16,
        rows: u16,
        adjust: impl FnOnce(&mut CommandBuilder),
    ) -> Self {
        let pair = native_pty_system()
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        let mut command = CommandBuilder::new(executable.as_ref());
        command.args(args);
        env.configure(&mut command);
        adjust(&mut command);
        let child = pair.slave.spawn_command(command).unwrap();
        drop(pair.slave);

        let grid = Arc::new(Mutex::new(Grid::new(Size::new(cols, rows))));
        let mut reader = pair.master.try_clone_reader().unwrap();
        let output = Arc::clone(&grid);
        thread::spawn(move || {
            let mut buffer = [0u8; 8192];
            while let Ok(n) = reader.read(&mut buffer) {
                if n == 0 {
                    break;
                }
                output.lock().unwrap().process(&buffer[..n]);
            }
        });
        let writer = pair.master.take_writer().unwrap();
        Self {
            child,
            master: pair.master,
            writer,
            grid,
        }
    }

    pub fn screen(&self) -> MutexGuard<'_, Grid> {
        self.grid.lock().unwrap()
    }

    pub fn contents(&self) -> String {
        self.screen().contents()
    }

    pub fn send(&mut self, bytes: &[u8]) {
        self.writer.write_all(bytes).unwrap();
        self.writer.flush().unwrap();
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
            .expect("no pid line in the focused pane")
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

    pub fn resize(&self, cols: u16, rows: u16) {
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
    }

    pub fn kill(&mut self) {
        let pid = self.child.process_id().unwrap() as i32;
        rustix::process::kill_process(Pid::from_raw(pid).unwrap(), Signal::KILL).unwrap();
        self.child.wait().unwrap();
    }
}

impl Drop for Attached {
    fn drop(&mut self) {
        if let Ok(None) = self.child.try_wait() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
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

impl Tile {
    pub fn lines(&self, screen: &Grid) -> Vec<String> {
        (self.top + 1..self.bottom)
            .map(|row| {
                (self.left + 1..self.right)
                    .map(|col| {
                        cell(screen, row, col)
                            .map(|cell| cell.symbol().to_string())
                            .unwrap_or_else(|| " ".to_string())
                    })
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect()
    }
}

fn symbol_at(screen: &Grid, row: u16, col: u16) -> Option<(String, bool)> {
    cell(screen, row, col).map(|cell| {
        (
            cell.symbol().to_string(),
            cell.modifier.contains(Modifier::BOLD),
        )
    })
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

pub fn wait_process_exit(process: &mut Process) -> std::process::ExitStatus {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        if let Some(status) = process.try_wait().unwrap() {
            return status;
        }
        assert!(Instant::now() < deadline, "the process did not exit");
        thread::sleep(Duration::from_millis(20));
    }
}

pub fn wait_until(condition: impl Fn() -> bool, what: &str) {
    let deadline = Instant::now() + TIMEOUT;
    while !condition() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        thread::sleep(Duration::from_millis(20));
    }
}

pub fn is_running(pid: i32) -> bool {
    let Some(pid) = Pid::from_raw(pid) else {
        return false;
    };
    if rustix::process::test_kill_process(pid).is_err() {
        return false;
    }
    fs::read_to_string(format!("/proc/{}/stat", pid.as_raw_pid()))
        .map(|stat| {
            let state = stat.rsplit(')').next().unwrap_or("").trim_start();
            !state.starts_with('Z')
        })
        .unwrap_or(false)
}

pub fn session_of(pid: i32) -> i32 {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).unwrap();
    let fields: Vec<&str> = stat
        .rsplit(')')
        .next()
        .unwrap()
        .split_whitespace()
        .collect();
    fields[3].parse().unwrap()
}

pub fn children(pid: i32) -> Vec<i32> {
    let Ok(tasks) = fs::read_dir(format!("/proc/{pid}/task")) else {
        return Vec::new();
    };
    tasks
        .filter_map(|task| fs::read_to_string(task.ok()?.path().join("children")).ok())
        .flat_map(|children| {
            children
                .split_whitespace()
                .filter_map(|pid| pid.parse().ok())
                .collect::<Vec<_>>()
        })
        .collect()
}
