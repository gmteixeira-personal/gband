use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child as Process, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use portable_pty::CommandBuilder;
use rustix::process::{Pid, Signal};

pub const TIMEOUT: Duration = Duration::from_secs(10);

pub struct TestEnv {
    pub root: PathBuf,
    pub work: PathBuf,
    pub executable: PathBuf,
    vars: Vec<(OsString, Option<OsString>)>,
    shells: Mutex<Vec<i32>>,
}

impl TestEnv {
    pub fn new(root: PathBuf, executable: impl Into<PathBuf>) -> Self {
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        let work = root.join("work");
        fs::create_dir_all(&work).unwrap();
        Self {
            root,
            work,
            executable: executable.into(),
            vars: Vec::new(),
            shells: Mutex::new(Vec::new()),
        }
    }

    pub fn set_var(&mut self, name: impl AsRef<OsStr>, value: Option<&OsStr>) {
        let name = name.as_ref().to_owned();
        self.vars.retain(|(existing, _)| *existing != name);
        self.vars.push((name, value.map(OsStr::to_owned)));
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
        let mut paths: Vec<PathBuf> = entries
            .filter_map(|entry| Some(entry.ok()?.path()))
            .filter(|path| {
                path.file_name()
                    .and_then(OsStr::to_str)
                    .is_some_and(|name| name.starts_with(&format!("{role}.")))
            })
            .collect();
        paths.sort();
        paths
            .into_iter()
            .filter_map(|path| fs::read_to_string(path).ok())
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

    fn base_vars(&self) -> Vec<(OsString, Option<OsString>)> {
        let mut vars: Vec<(OsString, Option<OsString>)> = [
            ("XDG_STATE_HOME", Some(self.state_home().into_os_string())),
            (
                "XDG_RUNTIME_DIR",
                Some(self.runtime_home().into_os_string()),
            ),
            ("XDG_CONFIG_HOME", Some(self.config_home().into_os_string())),
            ("XDG_DATA_HOME", Some(self.data_home().into_os_string())),
            ("SHELL", Some("/bin/sh".into())),
            ("INPUTRC", Some("/dev/null".into())),
            ("GBAND", None),
            ("GBAND_LOG", None),
        ]
        .into_iter()
        .map(|(name, value)| (OsString::from(name), value))
        .collect();
        vars.extend(self.vars.iter().cloned());
        vars
    }

    pub fn configure(&self, command: &mut CommandBuilder) {
        command.env("TERM", "xterm-256color");
        for (name, value) in self.base_vars() {
            match value {
                Some(value) => command.env(name, value),
                None => command.env_remove(name),
            }
        }
        command.cwd(&self.work);
    }

    pub fn command(&self, executable: impl AsRef<Path>, args: &[&str]) -> Command {
        let mut command = Command::new(executable.as_ref());
        command.args(args).current_dir(&self.work);
        for (name, value) in self.base_vars() {
            match value {
                Some(value) => command.env(name, value),
                None => command.env_remove(name),
            };
        }
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
            .command(&self.executable, args)
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
                .command(&self.executable, &["kill-server"])
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
