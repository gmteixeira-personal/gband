use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use gband_core::geometry::Size;
use gband_core::input::{
    Key, Modes, MouseEncoding, MouseEvent, MouseTracking, encode_key, encode_mouse, encode_paste,
};
use gband_emulator::{Emulator, Grid, Record};
use gband_protocol::Value;
use gband_protocol::test::SOCKET_VARIABLE;
use rustix::process::{Pid, Signal};

use crate::channel::{Endpoint, Listener, Waited};
use crate::env::{TestEnv, children, is_running};
use crate::screenshot;
use crate::terminal::Attached;

pub const SETTLE_TIMEOUT: Duration = Duration::from_secs(10);
const START_TIMEOUT: Duration = Duration::from_secs(10);
const SETTLE_ROUNDS: u32 = 10;
const EXIT_GRACE: Duration = Duration::from_secs(3);
const POLL: Duration = Duration::from_millis(20);

pub const DEFAULT_TIME: i64 = 1_735_732_800;

pub struct Setup {
    pub executable: PathBuf,
    pub size: Size,
    pub config: Option<String>,
    pub server_config: Option<String>,
    pub keystyle: Option<String>,
    pub files: Vec<(String, String)>,
    pub plugins: Vec<PathBuf>,
    pub env: Vec<(OsString, Option<OsString>)>,
    pub time: Option<i64>,
}

pub struct Case {
    env: TestEnv,
    terminal: Option<Attached>,
    client: Option<Endpoint>,
    server: Option<Endpoint>,
    round: u32,
    settled: BTreeSet<u32>,
    notifications: Vec<(Option<String>, String)>,
    clipboard: Vec<String>,
    bells: u64,
}

#[allow(clippy::disallowed_methods)]
fn scratch() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let next = NEXT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("gband-test-{}-{next}", std::process::id()))
}

fn opener(record: &Path) -> String {
    format!(
        "#!/bin/sh\nprintf '%s\\0' \"$1\" >> '{}'\n",
        record.display()
    )
}

pub fn plugin_name(path: &Path) -> Option<String> {
    let path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    Some(path.file_name()?.to_string_lossy().into_owned())
}

fn link_plugin(plugins: &Path, path: &Path) -> Result<(), String> {
    let name = plugin_name(path)
        .ok_or_else(|| format!("the plugin path {} has no name", path.display()))?;
    let target = fs::canonicalize(path)
        .map_err(|error| format!("cannot read the plugin {}: {error}", path.display()))?;
    let link = plugins.join(&name);
    if link.exists() {
        return Err(format!("two plugins are named `{name}`"));
    }
    std::os::unix::fs::symlink(&target, &link)
        .map_err(|error| format!("cannot install the plugin {}: {error}", path.display()))
}

fn write_file(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    fs::write(path, contents).map_err(|error| format!("cannot write {}: {error}", path.display()))
}

fn relative(base: &Path, path: &str) -> Result<PathBuf, String> {
    let relative = Path::new(path);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(format!(
            "`{path}` must be a path inside the configuration directory"
        ));
    }
    Ok(base.join(relative))
}

fn prepare(env: &mut TestEnv, setup: &Setup, socket: &Path) -> Result<(), String> {
    let io = |what: &str| {
        let what = what.to_owned();
        move |error: std::io::Error| format!("cannot prepare the case's {what}: {error}")
    };
    fs::set_permissions(&env.root, fs::Permissions::from_mode(0o700)).map_err(io("directory"))?;
    let bin = env.root.join("bin");
    fs::create_dir_all(&bin).map_err(io("programs"))?;
    let opened = env.root.join("opened");
    for program in ["xdg-open", "open"] {
        let path = bin.join(program);
        fs::write(&path, opener(&opened)).map_err(io("opener"))?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).map_err(io("opener"))?;
    }
    let plugins = env.plugins_dir();
    fs::create_dir_all(&plugins).map_err(io("plugins directory"))?;
    for plugin in &setup.plugins {
        link_plugin(&plugins, plugin)?;
    }
    let config = env.config_dir();
    fs::create_dir_all(config.join("user")).map_err(io("configuration directory"))?;
    if let Some(style) = &setup.keystyle {
        write_file(
            &config.join("user").join("keystyle.lua"),
            &format!("return \"{style}\"\n"),
        )?;
    }
    if let Some(source) = &setup.config {
        write_file(&env.user_lua(), source)?;
    }
    if let Some(source) = &setup.server_config {
        write_file(&env.server_lua(), source)?;
    }
    for (path, contents) in &setup.files {
        write_file(&relative(&config, path)?, contents)?;
    }
    let path = std::env::var_os("PATH").unwrap_or_default();
    let mut search = OsString::from(bin.as_os_str());
    if !path.is_empty() {
        search.push(":");
        search.push(&path);
    }
    let set: [(&str, Option<OsString>); 13] = [
        ("SHELL", Some("/bin/sh".into())),
        ("PS1", Some("$ ".into())),
        ("TERM", Some("xterm-256color".into())),
        ("COLORTERM", Some("truecolor".into())),
        ("TZ", Some("UTC".into())),
        ("GBAND_ANIMATIONS", Some("off".into())),
        ("INPUTRC", Some("/dev/null".into())),
        ("PATH", Some(search)),
        (SOCKET_VARIABLE, Some(socket.as_os_str().to_owned())),
        ("GBAND", None),
        ("GBAND_SESSION", None),
        ("GBAND_WINDOW", None),
        ("GBAND_LOG", None),
    ];
    for (name, value) in set {
        env.set_var(name, value.as_deref());
    }
    for (name, value) in &setup.env {
        env.set_var(name, value.as_deref());
    }
    Ok(())
}

impl Case {
    pub fn start(setup: Setup) -> Result<Self, String> {
        let root = scratch();
        let mut env = TestEnv::new(root.clone(), &setup.executable);
        let socket = root.join("channel.sock");
        let listener = prepare(&mut env, &setup, &socket)
            .and_then(|()| {
                Listener::bind(&socket)
                    .map_err(|error| format!("cannot listen on the test channel: {error}"))
            })
            .inspect_err(|_| {
                let _ = fs::remove_dir_all(&root);
            })?;
        let mut grid = Grid::new(setup.size);
        grid.record();
        let terminal = Attached::spawn(&env, &setup.executable, &["attach"], grid, |_| {})
            .map_err(|error| format!("cannot start gband attach: {error}"))?;
        let mut case = Self {
            env,
            terminal: Some(terminal),
            client: None,
            server: None,
            round: 0,
            settled: BTreeSet::new(),
            notifications: Vec::new(),
            clipboard: Vec::new(),
            bells: 0,
        };
        let deadline = Instant::now() + START_TIMEOUT;
        let (client, server) = listener.accept(setup.time, deadline, || case.alive())?;
        case.client = Some(client);
        case.server = Some(server);
        loop {
            if case.terminal().screen().alternate_screen() {
                break;
            }
            case.alive()?;
            if Instant::now() >= deadline {
                return Err(format!(
                    "the client did not take over its terminal in time\n{}",
                    case.screenshot(true)
                ));
            }
            thread::sleep(POLL);
        }
        case.settle()?;
        Ok(case)
    }

    fn terminal(&self) -> &Attached {
        self.terminal
            .as_ref()
            .expect("the terminal lives as long as the case")
    }

    fn alive(&mut self) -> Result<(), String> {
        let terminal = self
            .terminal
            .as_mut()
            .expect("the terminal lives as long as the case");
        match terminal.child.try_wait() {
            Ok(Some(status)) => {
                let screen = terminal.contents();
                Err(format!(
                    "gband attach exited with status {}\n{screen}",
                    status.exit_code()
                ))
            }
            _ => Ok(()),
        }
    }

    pub fn env(&self) -> &TestEnv {
        &self.env
    }

    pub fn screen(&self) -> vt100::Screen {
        self.terminal().screen().screen().clone()
    }

    pub fn screenshot(&self, styles: bool) -> String {
        screenshot::render(self.terminal().screen().screen(), styles)
    }

    fn write(&self, bytes: &[u8]) -> Result<(), String> {
        self.terminal()
            .try_send(bytes)
            .map_err(|error| format!("cannot write to the client's terminal: {error}"))
    }

    pub fn modes(&self) -> gband_core::input::Modes {
        self.terminal().screen().modes()
    }

    pub fn keys(&mut self, keys: &[Key]) -> Result<(), String> {
        for &key in keys {
            let bytes = encode_key(key, self.modes());
            self.write(&bytes)?;
            if bytes.last() == Some(&0x1b) {
                self.input_read(Instant::now() + START_TIMEOUT)
                    .map_err(|_| "the client did not read an escape key in time".to_owned())?;
            }
        }
        Ok(())
    }

    fn input_read(&mut self, deadline: Instant) -> Result<u64, Waited> {
        let written = self.terminal().written();
        let round = self.next_round();
        self.endpoint(false).settle(round, Some(written), deadline)
    }

    pub fn mouse(&self, event: MouseEvent) -> Result<(), String> {
        let xterm = Modes {
            mouse_tracking: MouseTracking::AnyMotion,
            mouse_encoding: MouseEncoding::Sgr,
            ..Modes::DEFAULT
        };
        self.write(&encode_mouse(event, xterm))
    }

    pub fn type_text(&self, text: &str) -> Result<(), String> {
        self.write(text.as_bytes())
    }

    pub fn paste(&self, text: &str) -> Result<(), String> {
        let modes = self.modes();
        if modes.bracketed_paste {
            self.write(&encode_paste(text, modes))
        } else {
            self.write(text.as_bytes())
        }
    }

    fn view_size(&mut self, deadline: Instant) -> Result<Vec<Value>, String> {
        self.eval(
            false,
            "local view = gband.view() return view.cols, view.rows",
            Vec::new(),
            deadline,
        )?
        .map_err(|reason| format!("cannot read the client's view: {reason}"))
    }

    pub fn resize(&mut self, size: Size) -> Result<(), String> {
        let deadline = Instant::now() + START_TIMEOUT;
        if self.terminal().screen().size() == size {
            return Ok(());
        }
        let before = self.view_size(deadline)?;
        self.terminal()
            .try_resize(size.cols, size.rows)
            .map_err(|error| format!("cannot resize the client's terminal: {error}"))?;
        while self.view_size(deadline)? == before {
            if Instant::now() >= deadline {
                return Err("the client did not take the new size in time".to_owned());
            }
            thread::sleep(Duration::from_millis(5));
        }
        Ok(())
    }

    pub fn write_file(&self, path: &str, contents: &str) -> Result<(), String> {
        write_file(&relative(&self.env.config_dir(), path)?, contents)
    }

    fn endpoint(&mut self, server: bool) -> &mut Endpoint {
        let endpoint = if server {
            &mut self.server
        } else {
            &mut self.client
        };
        endpoint.as_mut().expect("both processes joined")
    }

    fn waited(&self, side: &str, waited: Waited, what: &str) -> String {
        match waited {
            Waited::TimedOut => format!("the {side} did not answer {what} in time"),
            Waited::Closed => format!("the {side} left the test channel while {what}"),
        }
    }

    pub fn eval(
        &mut self,
        server: bool,
        source: &str,
        args: Vec<Value>,
        deadline: Instant,
    ) -> Result<Result<Vec<Value>, String>, String> {
        let side = if server { "server" } else { "client" };
        self.endpoint(server)
            .eval(source, args, deadline)
            .map_err(|waited| self.waited(side, waited, "evaluating a chunk"))
    }

    pub fn reload(&mut self, deadline: Instant) -> Result<Option<String>, String> {
        let client = self
            .endpoint(false)
            .reload(deadline)
            .map_err(|waited| self.waited("client", waited, "reloading"))?;
        let server = self
            .endpoint(true)
            .reload(deadline)
            .map_err(|waited| self.waited("server", waited, "reloading"))?;
        Ok(client.or(server))
    }

    pub fn set_time(&mut self, time: Option<i64>) -> Result<(), String> {
        for server in [false, true] {
            let side = if server { "server" } else { "client" };
            self.endpoint(server)
                .set_time(time)
                .map_err(|waited| self.waited(side, waited, "setting the time"))?;
        }
        Ok(())
    }

    fn collect(&mut self) {
        let records = self
            .terminal
            .as_ref()
            .expect("the terminal lives as long as the case")
            .screen()
            .take_records();
        for record in records {
            match record {
                Record::Notification { title, body } => self.notifications.push((title, body)),
                Record::Clipboard(text) => self.clipboard.push(text),
                Record::Bell => self.bells += 1,
                Record::Settle(round) => {
                    if let Ok(round) = u32::try_from(round) {
                        self.settled.insert(round);
                    }
                }
            }
        }
    }

    pub fn notifications(&mut self) -> Vec<(Option<String>, String)> {
        self.collect();
        self.notifications.clone()
    }

    pub fn clipboard(&mut self) -> Vec<String> {
        self.collect();
        self.clipboard.clone()
    }

    pub fn bells(&mut self) -> u64 {
        self.collect();
        self.bells
    }

    pub fn opened(&self) -> Vec<String> {
        fs::read(self.env.root.join("opened"))
            .map(|record| {
                record
                    .split(|&byte| byte == 0)
                    .filter(|entry| !entry.is_empty())
                    .map(|entry| String::from_utf8_lossy(entry).into_owned())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn log(&self, side: &str) -> Vec<String> {
        self.env.log_text(side).lines().map(str::to_owned).collect()
    }

    fn next_round(&mut self) -> u32 {
        self.round += 1;
        self.round
    }

    pub fn settle(&mut self) -> Result<(), String> {
        let deadline = Instant::now() + SETTLE_TIMEOUT;
        let failed = |step: &str| format!("settle did not finish in 10 seconds: {step}");
        self.input_read(deadline)
            .map_err(|_| failed("the client did not read the input written before it"))?;
        for _ in 0..SETTLE_ROUNDS {
            let round = self.next_round();
            let server = self
                .endpoint(true)
                .settle(round, None, deadline)
                .map_err(|_| failed("the server did not handle the messages the client sent"))?;
            let client = self
                .endpoint(false)
                .settle(round, None, deadline)
                .map_err(|_| failed("the client did not handle the server's messages and draw"))?;
            loop {
                self.collect();
                if self.settled.remove(&round) {
                    break;
                }
                if Instant::now() >= deadline {
                    return Err(failed("the terminal did not read the client's frame"));
                }
                thread::sleep(Duration::from_millis(2));
            }
            if server == 0 && client == 0 {
                return Ok(());
            }
        }
        Err(format!(
            "settle did not finish: the client and the server still sent messages after \
             {SETTLE_ROUNDS} rounds, as handlers that keep triggering each other do"
        ))
    }

    fn processes(&self) -> Vec<i32> {
        let mut found = Vec::new();
        let mut pending: Vec<i32> = self
            .terminal
            .as_ref()
            .and_then(|terminal| terminal.child.process_id())
            .map(|pid| pid as i32)
            .into_iter()
            .chain(
                fs::read_to_string(self.env.lock())
                    .ok()
                    .and_then(|record| record.trim().parse().ok()),
            )
            .collect();
        while let Some(pid) = pending.pop() {
            if found.contains(&pid) {
                continue;
            }
            found.push(pid);
            pending.extend(children(pid));
        }
        found
    }

    pub fn finish(mut self) {
        self.stop();
    }

    fn stop(&mut self) {
        let processes = self.processes();
        self.client = None;
        self.server = None;
        let deadline = Instant::now() + EXIT_GRACE;
        while processes.iter().any(|&pid| is_running(pid)) && Instant::now() < deadline {
            thread::sleep(POLL);
        }
        for pid in processes {
            if is_running(pid)
                && let Some(pid) = Pid::from_raw(pid)
            {
                let _ = rustix::process::kill_process(pid, Signal::KILL);
            }
        }
        if let Some(mut terminal) = self.terminal.take() {
            terminal.finish();
        }
        let _ = fs::remove_dir_all(&self.env.root);
    }
}

impl Drop for Case {
    fn drop(&mut self) {
        if self.terminal.is_some() {
            self.stop();
        }
    }
}

pub fn parse_size(text: &str) -> Option<Size> {
    let (cols, rows) = text.split_once('x')?;
    let size = Size::new(cols.parse().ok()?, rows.parse().ok()?);
    (size.cols > 0 && size.rows > 0).then_some(size)
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let of_era = year - era * 400;
    let of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let of_cycle = of_era * 365 + of_era / 4 - of_era / 100 + of_year;
    era * 146_097 + of_cycle - 719_468
}

pub fn parse_time(text: &str) -> Option<i64> {
    let (date, time) = text.split_once(' ')?;
    let date: Vec<i64> = date
        .split('-')
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    let time: Vec<i64> = time
        .split(':')
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    let (&[year, month, day], &[hour, minute, second]) = (date.as_slice(), time.as_slice()) else {
        return None;
    };
    let valid = (1..=12).contains(&month)
        && (1..=31).contains(&day)
        && (0..24).contains(&hour)
        && (0..60).contains(&minute)
        && (0..61).contains(&second);
    valid.then(|| days_from_civil(year, month, day) * 86_400 + hour * 3600 + minute * 60 + second)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_parse_as_utc() {
        assert_eq!(parse_time("2025-01-01 12:00:00"), Some(DEFAULT_TIME));
        assert_eq!(parse_time("1970-01-01 00:00:00"), Some(0));
        assert_eq!(parse_time("2000-03-01 00:00:01"), Some(951_868_801));
        assert_eq!(parse_time("2025-13-01 00:00:00"), None);
        assert_eq!(parse_time("2025-01-01"), None);
    }

    #[test]
    fn sizes_parse() {
        assert_eq!(parse_size("80x24"), Some(Size::new(80, 24)));
        assert_eq!(parse_size("0x24"), None);
        assert_eq!(parse_size("80 x 24"), None);
    }
}
