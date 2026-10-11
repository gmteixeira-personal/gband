mod common;

use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};
use std::thread;
use std::time::Duration;

use common::*;
use gband_protocol::{HelloReply, PROTOCOL_VERSION, encode};
use rustix::process::{Pid, Signal};

const FAKE_SSH: &str = r#"#!/bin/sh
printf '%s\n' "$*" >> "$SSH_RECORD"
if [ -n "$SSH_STATUS" ]; then
    [ -n "$SSH_ERROR" ] && printf '%s\n' "$SSH_ERROR" >&2
    exit "$SSH_STATUS"
fi
shift 3
unset GBAND GBAND_SESSION
export XDG_STATE_HOME="$SSH_HOME/state"
cd "$SSH_HOME" || exit 1
exec sh -c "$*"
"#;

struct Remote {
    env: TestEnv,
    home: PathBuf,
    record: PathBuf,
}

impl std::ops::Deref for Remote {
    type Target = TestEnv;

    fn deref(&self) -> &TestEnv {
        &self.env
    }
}

impl Remote {
    fn new(name: &str) -> Self {
        let mut env = TestEnv::new(name);
        let bin = env.root.join("bin");
        fs::create_dir_all(&bin).unwrap();
        let ssh = bin.join("ssh");
        fs::write(&ssh, FAKE_SSH).unwrap();
        fs::set_permissions(&ssh, fs::Permissions::from_mode(0o755)).unwrap();
        std::os::unix::fs::symlink(GBAND, bin.join("gband")).unwrap();
        let home = env.root.join("home");
        fs::create_dir_all(&home).unwrap();
        let record = env.root.join("ssh.record");
        let path = format!(
            "{}:{}",
            bin.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        env.set_var("PATH", Some(path.as_ref()));
        env.set_var("SSH_RECORD", Some(record.as_os_str()));
        env.set_var("SSH_HOME", Some(home.as_os_str()));
        Self { env, home, record }
    }

    fn failing(name: &str, status: &str, error: &str) -> Self {
        let mut remote = Self::new(name);
        remote.env.set_var("SSH_STATUS", Some(status.as_ref()));
        remote.env.set_var("SSH_ERROR", Some(error.as_ref()));
        remote
    }

    fn recorded(&self) -> Vec<String> {
        fs::read_to_string(&self.record)
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }

    fn attach(&self, args: &[&str]) -> Attached {
        Attached::start_with(&self.env, GBAND, args, 80, 24, |_| {})
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command(GBAND, args)
            .stdin(Stdio::null())
            .output()
            .unwrap()
    }

    fn run_with_input(&self, args: &[&str], input: &str) -> Output {
        let mut child = self
            .command(GBAND, args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }

    fn list(&self, args: &[&str]) -> String {
        let output = self.run(&[args, &["list-sessions"]].concat());
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        String::from_utf8(output.stdout).unwrap()
    }

    fn kill_server(&self, args: &[&str]) {
        let output = self.run(&[args, &["kill-server"]].concat());
        assert_eq!(output.status.code(), Some(0), "{output:?}");
    }

    fn client_events(&self) -> Vec<String> {
        self.log_text("client")
            .lines()
            .filter(|line| !line.contains("client started"))
            .map(str::to_owned)
            .collect()
    }
}

fn detach(client: &mut Attached) {
    client.send(b"\x00D");
    assert_eq!(client.wait_exit(), 0);
    client.wait_for_text("[detached]");
}

fn in_home(client: &mut Attached, remote: &Remote) {
    client.run(&format!(
        "[ \"$(pwd)\" = '{}' ] && echo home-$((6 * 7))",
        remote.home.display()
    ));
    client.wait_for_line("home-42");
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

fn install_copy(path: &Path) {
    fs::copy(GBAND, path).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn window_script(env: &TestEnv, body: &str) -> PathBuf {
    let path = env.root.join("window.sh");
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

#[test]
fn remote_command_line() {
    let remote = Remote::new("remote-command-line");
    let mut client = remote.attach(&["-r", "u@h", "-S", "feature", "-s", "work"]);
    client.wait_for_prompt();
    client.shell_pid(&remote);
    assert_eq!(
        remote.recorded(),
        ["-T -- u@h gband '-S' 'feature' 'relay' '-s' 'work' '--size' '79x24'"]
    );
    detach(&mut client);
    remote.kill_server(&["-S", "feature"]);
}

#[test]
fn remote_gband_with_a_tilde() {
    let remote = Remote::new("remote-tilde");
    let cargo_bin = remote.home.join(".cargo").join("bin");
    fs::create_dir_all(&cargo_bin).unwrap();
    std::os::unix::fs::symlink(GBAND, cargo_bin.join("gband")).unwrap();
    let home = remote.home.clone();
    let mut client = Attached::start_with(
        &remote,
        GBAND,
        &["-r", "u@h", "--remote-gband", "~/.cargo/bin/gband"],
        80,
        24,
        move |command| command.env("HOME", home),
    );
    client.wait_for_prompt();
    client.shell_pid(&remote);
    let recorded = remote.recorded();
    assert!(
        recorded[0].starts_with("-T -- u@h ~/.cargo/bin/gband 'relay'"),
        "{recorded:?}"
    );
    detach(&mut client);
}

#[test]
fn socket_path_with_a_space() {
    let remote = Remote::new("remote-socket-space");
    let socket = remote.root.join("my socket");
    let path = socket.to_str().unwrap();
    let mut client = remote.attach(&["-r", "u@h", "-p", path]);
    client.wait_for_prompt();
    client.shell_pid(&remote);
    let recorded = remote.recorded();
    assert!(
        recorded[0].contains(&format!("'-p' '{path}'")),
        "{recorded:?}"
    );
    assert!(socket.exists());
    detach(&mut client);
    remote.kill_server(&["-p", path]);
}

#[test]
fn attach_through_ssh() {
    let remote = Remote::new("remote-attach");
    let mut client = remote.attach(&["-r", "u@h"]);
    client.wait_for_prompt();
    client.shell_pid(&remote);
    client.run(r"printf 'relayed\n'");
    client.wait_for_line("relayed");
    detach(&mut client);
    assert_eq!(remote.list(&[]), "default\t1\t0\n");
}

#[test]
fn remote_named_session() {
    let remote = Remote::new("remote-named");
    let mut client = remote.attach(&["-r", "u@h", "-s", "work"]);
    client.wait_for_prompt();
    client.shell_pid(&remote);
    assert_eq!(remote.list(&[]), "work\t1\t1\n");
    detach(&mut client);
}

#[test]
fn remote_session_starts_in_the_login_directory() {
    let remote = Remote::new("remote-login-dir");
    let mut client = remote.attach(&["-r", "u@h"]);
    client.wait_for_prompt();
    client.shell_pid(&remote);
    in_home(&mut client, &remote);
    detach(&mut client);
}

#[test]
fn new_remote_session_on_a_running_server() {
    let remote = Remote::new("remote-new-session");
    let mut server = remote
        .command(GBAND, &["server"])
        .current_dir(&remote.home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    remote.shell_of(&server);
    let mut client = remote.attach(&["-r", "u@h", "-s", "work"]);
    client.wait_for_prompt();
    client.shell_pid(&remote);
    in_home(&mut client, &remote);
    detach(&mut client);
    remote.kill_server(&[]);
    wait_process_exit(&mut server);
}

#[test]
fn local_bindings_over_ssh() {
    let remote = Remote::new("remote-bindings");
    remote.write_config(
        "gband.bind('alt+h', gband.action.focus_column_left)\n\
         gband.bind('alt+n', gband.action.open_window)\n",
    );
    let mut client = remote.attach(&["-r", "u@h"]);
    client.wait_for_prompt();
    let first = client.shell_pid(&remote);
    client.run("clear; cat -v");
    client.wait_for_child(first, "cat");
    client.send(b"\x1bn");
    client.wait_for("the second of two tiles focused", |screen| {
        let found = tiles(screen);
        found.len() == 2 && found[1].focused
    });
    client.wait_for_prompt();
    let second = client.shell_pid(&remote);
    client.run("clear; cat -v");
    client.wait_for_child(second, "cat");
    client.send(b"\x1bh");
    client.wait_for("the first tile focused", |screen| {
        tiles(screen).first().is_some_and(|tile| tile.focused)
    });
    thread::sleep(Duration::from_millis(300));
    let contents = client.contents();
    assert!(!contents.contains("^[h"), "{contents}");
    client.kill();
}

#[test]
fn remote_attach_from_a_gband_window() {
    let remote = Remote::new("remote-from-window");
    let mut server = remote.start_server("/bin/sh");
    remote.shell_of(&server);
    let socket = remote.socket();
    let mut client = Attached::start_with(&remote, GBAND, &["-r", "u@h"], 80, 24, |command| {
        command.env("GBAND", socket)
    });
    client.wait_for_prompt();
    assert_eq!(remote.list(&[]), "default\t1\t1\n");
    detach(&mut client);
    remote.kill_server(&[]);
    wait_process_exit(&mut server);
}

#[test]
fn remote_attach_without_a_terminal() {
    let remote = Remote::new("remote-no-terminal");
    let output = remote.run(&["-r", "u@h"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(stderr(&output).contains("needs a terminal"), "{output:?}");
    assert!(remote.recorded().is_empty());
}

#[test]
fn remote_list_sessions() {
    let remote = Remote::new("remote-list");
    let mut server = remote.start_named_server("feature", "/bin/sh");
    remote.shell_of(&server);
    let output = remote.run(&["-r", "u@h", "-S", "feature", "list-sessions"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "default\t1\t0\n");
    assert_eq!(
        remote.recorded(),
        ["-T -- u@h gband '-S' 'feature' 'list-sessions'"]
    );
    let output = remote.run(&["-r", "u@h", "list-sessions"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    remote.kill_server(&["-S", "feature"]);
    wait_process_exit(&mut server);
}

#[test]
fn remote_eval_from_standard_input() {
    let remote = Remote::new("remote-eval-stdin");
    let mut server = remote.start_server("/bin/sh");
    remote.shell_of(&server);
    let output = remote.run_with_input(&["-r", "u@h", "eval", "--on-server", "-"], "return 1 + 1");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "2\n");
    remote.kill_server(&[]);
    wait_process_exit(&mut server);
}

#[test]
fn window_variables_stay_local() {
    let remote = Remote::new("remote-window-vars");
    let mut server = remote.start_server("/bin/sh");
    remote.shell_of(&server);
    let output = remote
        .command(GBAND, &["-r", "u@h", "errors", "--json"])
        .env("GBAND_SESSION", "work")
        .env("GBAND", remote.root.join("elsewhere.sock"))
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(remote.recorded(), ["-T -- u@h gband 'errors' '--json'"]);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("work"), "{stdout}");
    remote.kill_server(&[]);
    wait_process_exit(&mut server);
}

#[test]
fn relay_starts_the_server() {
    let remote = Remote::new("remote-relay-starts");
    let size = remote.root.join("size");
    let cwd = remote.root.join("cwd");
    let window = window_script(
        &remote,
        &format!(
            "pwd > '{}'\nstty size > '{}'\nsleep 100",
            cwd.display(),
            size.display()
        ),
    );
    let output = remote
        .command(GBAND, &["relay", "-s", "work", "--size", "100x30"])
        .current_dir(&remote.home)
        .env("SHELL", &window)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    wait_until(
        || fs::read_to_string(&size).is_ok_and(|text| text.ends_with('\n')),
        "the window's size",
    );
    assert_eq!(fs::read_to_string(&size).unwrap(), "28 48\n");
    assert_eq!(
        fs::read_to_string(&cwd).unwrap().trim_end(),
        remote.home.to_str().unwrap()
    );
    assert_eq!(remote.list(&[]), "work\t1\t0\n");
    remote.kill_server(&[]);
}

#[test]
fn run_relay() {
    let remote = Remote::new("remote-run-relay");
    remote
        .command(GBAND, &["relay"])
        .env("SHELL", "/bin/true")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(remote.log_text("client").contains("client started"));
}

#[test]
fn remote_events() {
    let remote = Remote::new("remote-events");
    let output = remote.run(&["-r", "u@h", "list-sessions"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let events = remote.client_events();
    assert!(!events.is_empty());
    assert!(events.iter().all(|line| line.contains("u@h")), "{events:?}");
}

#[test]
fn remote_server_from_another_build() {
    let remote = Remote::new("remote-other-build");
    let other = remote.root.join("gband-other");
    install_copy(&other);
    let other = other.to_str().unwrap();
    let mut client = remote.attach(&["-r", "u@h", "--remote-gband", other]);
    client.wait_for_prompt();
    let shell = client.shell_pid(&remote);
    let server = remote.server_pid();
    detach(&mut client);
    thread::sleep(Duration::from_millis(300));
    let contents = client.contents();
    assert!(!contents.contains("different gband build"), "{contents}");
    assert!(is_running(shell));
    assert!(is_running(server));
}

fn failed_attach(remote: &Remote, args: &[&str], line: &str) {
    let mut client = remote.attach(args);
    assert_eq!(client.wait_exit(), 1);
    client.wait_for_text(line);
    assert!(!client.screen().alternate_screen());
}

#[test]
fn unreachable_host() {
    let error = "ssh: Could not resolve hostname nowhere: Name or service not known";
    let remote = Remote::failing("remote-unreachable", "255", error);
    failed_attach(
        &remote,
        &["-r", "nowhere"],
        &format!("gband: cannot reach nowhere: {error}"),
    );
}

#[test]
fn gband_missing_on_the_remote_host() {
    let remote = Remote::failing("remote-missing", "127", "sh: gband: command not found");
    failed_attach(
        &remote,
        &["-r", "u@h"],
        "gband: gband was not found on u@h; install gband there or pass --remote-gband",
    );
}

#[test]
fn ssh_fails_for_a_subcommand() {
    let error = "ssh: connect to host nowhere port 22: Connection refused";
    let remote = Remote::failing("remote-subcommand-fails", "255", error);
    let output = remote.run(&["-r", "nowhere", "kill-server"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = stderr(&output);
    let ours: Vec<&str> = stderr
        .lines()
        .filter(|line| line.starts_with("gband:"))
        .collect();
    assert_eq!(ours.len(), 1, "{stderr}");
    assert!(ours[0].contains("nowhere"), "{stderr}");
}

#[test]
fn connection_drops() {
    let remote = Remote::new("remote-drops");
    let mut client = remote.attach(&["-r", "u@h"]);
    client.wait_for_prompt();
    client.shell_pid(&remote);
    let ssh = children(client.child.process_id().unwrap() as i32);
    assert_eq!(ssh.len(), 1, "{ssh:?}");
    rustix::process::kill_process(Pid::from_raw(ssh[0]).unwrap(), Signal::KILL).unwrap();
    assert_eq!(client.wait_exit(), 1);
    client.wait_for_text("[lost server]");
    assert!(!client.screen().alternate_screen());
}

#[test]
fn incompatible_remote_server() {
    let remote = Remote::new("remote-incompatible");
    let socket = remote.socket_named("feature");
    fs::create_dir_all(socket.parent().unwrap()).unwrap();
    let listener = UnixListener::bind(&socket).unwrap();
    let version = PROTOCOL_VERSION + 1;
    let server = thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut hello = [0; 64];
            let _ = stream.read(&mut hello);
            let reply = encode(&HelloReply::Rejected { version }).unwrap();
            let _ = stream.write_all(&reply);
        }
    });
    failed_attach(
        &remote,
        &["-r", "u@h", "-S", "feature"],
        "gband -r u@h -S feature kill-server",
    );
    assert!(!server.is_finished());
    assert!(socket.exists());
}
