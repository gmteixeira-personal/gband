mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use common::scratch_root;

fn state_home(name: &str) -> PathBuf {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("subcommands")
        .join(name);
    if path.exists() {
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        fs::remove_dir_all(&path).unwrap();
    }
    fs::create_dir_all(&path).unwrap();
    let runtime = runtime_home(&path);
    if runtime.exists() {
        fs::set_permissions(runtime.join("gband"), fs::Permissions::from_mode(0o700)).ok();
        fs::remove_dir_all(&runtime).unwrap();
    }
    path
}

fn runtime_home(state: &Path) -> PathBuf {
    let name = state.file_name().unwrap().to_str().unwrap();
    scratch_root("gband-subcommands", name)
}

fn gband(state: &Path, filter: Option<&str>, args: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_gband"));
    command
        .args(args)
        .env("XDG_STATE_HOME", state)
        .env("XDG_RUNTIME_DIR", runtime_home(state))
        .env("SHELL", "/bin/true")
        .env_remove("GBAND");
    match filter {
        Some(value) => command.env("GBAND_LOG", value),
        None => command.env_remove("GBAND_LOG"),
    };
    command.output().unwrap()
}

fn log_dir(state: &Path) -> PathBuf {
    state.join("gband").join("log")
}

fn log_files(state: &Path, role: &str) -> Vec<PathBuf> {
    let prefix = format!("{role}.");
    let Ok(entries) = fs::read_dir(log_dir(state)) else {
        return Vec::new();
    };
    entries
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            let name = path.file_name().unwrap().to_str().unwrap();
            name.starts_with(&prefix) && name.ends_with(".log")
        })
        .collect()
}

fn log_text(state: &Path, role: &str) -> String {
    log_files(state, role)
        .iter()
        .map(|path| fs::read_to_string(path).unwrap())
        .collect()
}

fn utc_today() -> String {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        / 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

fn assert_silent_success(output: &Output) {
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty(), "stdout: {:?}", output.stdout);
    assert!(output.stderr.is_empty(), "stderr: {:?}", output.stderr);
}

fn assert_one_line_failure(output: &Output) -> String {
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty(), "stdout: {:?}", output.stdout);
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();
    assert_eq!(stderr.lines().count(), 1, "stderr: {stderr:?}");
    stderr
}

#[test]
fn server_logs_its_start() {
    let state = state_home("server_start");
    let output = gband(&state, None, &["server"]);
    assert_silent_success(&output);
    assert!(log_text(&state, "server").contains("server started"));
}

#[test]
fn attach_without_a_terminal_is_refused() {
    let state = state_home("attach_no_terminal");
    let output = gband(&state, None, &["attach"]);
    let stderr = assert_one_line_failure(&output);
    assert!(stderr.contains("needs a terminal"), "{stderr}");
    assert!(log_text(&state, "client").contains("client started"));
    assert!(!runtime_home(&state).exists());
}

#[test]
fn attach_inside_a_pane_is_refused() {
    let state = state_home("attach_nested");
    let output = Command::new(env!("CARGO_BIN_EXE_gband"))
        .arg("attach")
        .env("XDG_STATE_HOME", &state)
        .env("XDG_RUNTIME_DIR", runtime_home(&state))
        .env("GBAND", "/somewhere/default.sock")
        .output()
        .unwrap();
    let stderr = assert_one_line_failure(&output);
    assert!(stderr.contains("inside a gband pane"), "{stderr}");
}

#[test]
fn kill_server_without_a_server_logs_to_the_client_series() {
    let state = state_home("kill_no_server");
    let output = gband(&state, None, &["kill-server"]);
    let stderr = assert_one_line_failure(&output);
    let socket = runtime_home(&state).join("gband").join("default.sock");
    assert!(stderr.contains(socket.to_str().unwrap()), "{stderr}");
    assert!(log_text(&state, "client").contains("client started"));
    assert!(log_files(&state, "server").is_empty());
    assert!(!runtime_home(&state).exists());
}

#[test]
fn server_refuses_a_runtime_directory_open_to_others() {
    let state = state_home("open_runtime_dir");
    let runtime_dir = runtime_home(&state).join("gband");
    fs::create_dir_all(&runtime_dir).unwrap();
    fs::set_permissions(&runtime_dir, fs::Permissions::from_mode(0o777)).unwrap();
    let output = gband(&state, None, &["server"]);
    let stderr = assert_one_line_failure(&output);
    assert!(stderr.contains(runtime_dir.to_str().unwrap()), "{stderr}");
    assert!(stderr.contains("0777"), "{stderr}");
    assert!(!runtime_dir.join("default.lock").exists());
}

#[test]
fn help_lists_subcommands() {
    let state = state_home("help");
    let mut listings = Vec::new();
    for flag in ["-h", "--help"] {
        let output = gband(&state, None, &[flag]);
        assert_eq!(output.status.code(), Some(0), "{flag}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        let commands: Vec<String> = stdout
            .lines()
            .skip_while(|line| *line != "Commands:")
            .skip(1)
            .take_while(|line| !line.is_empty())
            .map(str::to_owned)
            .collect();
        let names: Vec<&str> = commands
            .iter()
            .filter_map(|line| line.split_whitespace().next())
            .collect();
        assert_eq!(
            names,
            [
                "server",
                "attach",
                "list-sessions",
                "kill-session",
                "kill-server",
                "completions",
                "install-completions"
            ],
            "{flag}"
        );
        let attach = commands
            .iter()
            .find(|line| line.trim_start().starts_with("attach "))
            .unwrap();
        assert!(attach.contains("when no command is given"), "{attach}");
        assert!(stdout.contains("-s, --session <NAME>"), "{flag}: {stdout}");
        listings.push(commands);
    }
    assert_eq!(listings[0], listings[1]);
    assert!(!state.join("gband").exists());
}

#[test]
fn invalid_session_name_is_rejected_without_a_log() {
    let state = state_home("invalid_session");
    for args in [["attach", "-s", "a/b"], ["-s", "a/b", "kill-session"]] {
        let output = gband(&state, None, &args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("'a/b'"), "{stderr}");
        assert!(stderr.contains("session name"), "{stderr}");
        assert!(stderr.contains("ASCII letters, digits"), "{stderr}");
    }
    assert!(!state.join("gband").exists());
    assert!(!runtime_home(&state).exists());
}

#[test]
fn session_option_on_a_subcommand_without_sessions_is_rejected() {
    let state = state_home("session_not_applicable");
    for (args, subcommand) in [
        (["kill-server", "-s", "work"], "kill-server"),
        (["-s", "work", "kill-server"], "kill-server"),
        (["list-sessions", "-s", "work"], "list-sessions"),
    ] {
        let output = gband(&state, None, &args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("'-s'"), "{stderr}");
        assert!(stderr.contains(subcommand), "{stderr}");
    }
    assert!(!state.join("gband").exists());
    assert!(!runtime_home(&state).exists());
}

#[test]
fn session_requests_without_a_server_fail_and_create_nothing() {
    for (name, args) in [
        ("requests_no_server_list", &["list-sessions"][..]),
        ("requests_no_server_kill", &["kill-session"]),
        (
            "requests_no_server_kill_named",
            &["-s", "work", "kill-session"],
        ),
    ] {
        let state = state_home(name);
        let socket = runtime_home(&state).join("gband").join("default.sock");
        let output = gband(&state, None, args);
        let stderr = assert_one_line_failure(&output);
        assert!(
            stderr.contains("no server is running"),
            "{args:?}: {stderr}"
        );
        assert!(
            stderr.contains(socket.to_str().unwrap()),
            "{args:?}: {stderr}"
        );
        assert!(
            log_text(&state, "client").contains("client started"),
            "{args:?}"
        );
        assert!(log_files(&state, "server").is_empty(), "{args:?}");
        assert!(!runtime_home(&state).exists(), "{args:?}");
    }
}

#[test]
fn session_request_events_name_their_server() {
    for (name, subcommand) in [
        ("request_socket_list", "list-sessions"),
        ("request_socket_kill", "kill-session"),
    ] {
        let state = state_home(name);
        let socket = runtime_home(&state).join("gband").join("a.sock");
        assert_one_line_failure(&gband(&state, None, &["-S", "a", subcommand]));
        let text = log_text(&state, "client");
        let events: Vec<&str> = text
            .lines()
            .filter(|line| !line.contains("client started"))
            .collect();
        assert!(!events.is_empty(), "{subcommand}: {text}");
        let field = format!("socket={}", socket.display());
        for event in events {
            assert!(event.contains(&field), "{subcommand}: {event}");
        }
    }
}

#[test]
fn help_lists_the_server_selection_options() {
    let state = state_home("help_selection");
    let output = gband(&state, None, &["--help"]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("-S, --server <NAME>"), "{stdout}");
    assert!(stdout.contains("-p, --socket <PATH>"), "{stdout}");
}

#[test]
fn both_selection_options_are_rejected_with_usage() {
    let state = state_home("both_options");
    for args in [
        ["-S", "a", "-p", "/tmp/b.sock", "attach"],
        ["attach", "-S", "a", "-p", "/tmp/b.sock"],
        ["-S", "a", "attach", "-p", "/tmp/b.sock"],
    ] {
        let output = gband(&state, None, &args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("Usage"), "{args:?}: {stderr}");
    }
}

#[test]
fn invalid_server_name_is_rejected() {
    let state = state_home("invalid_name");
    let output = gband(&state, None, &["-S", "a.b", "attach"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("'a.b'"), "{stderr}");
    assert!(stderr.contains("server name"), "{stderr}");
}

#[test]
fn socket_path_over_the_limit_is_refused() {
    let state = state_home("long_socket");
    let socket = format!("/tmp/{}", "x".repeat(115));
    assert_eq!(socket.len(), 120);
    let output = gband(&state, None, &["server", "-p", &socket]);
    let stderr = assert_one_line_failure(&output);
    assert!(stderr.contains(&socket), "{stderr}");
    assert!(stderr.contains("107 bytes"), "{stderr}");
    assert!(!Path::new(&format!("{socket}.lock")).exists());
}

#[test]
fn missing_parent_of_an_explicit_socket_is_refused() {
    let state = state_home("missing_parent");
    let parent = runtime_home(&state).join("missing");
    let socket = parent.join("s.sock");
    let output = gband(&state, None, &["server", "-p", socket.to_str().unwrap()]);
    let stderr = assert_one_line_failure(&output);
    assert!(stderr.contains(parent.to_str().unwrap()), "{stderr}");
    assert!(!parent.exists());
}

#[test]
fn version_names_the_package_version() {
    let state = state_home("version");
    let output = gband(&state, None, &["--version"]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim(),
        format!("gband {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn no_subcommand_runs_attach() {
    let state = state_home("no_subcommand");
    let output = gband(&state, None, &[]);
    let stderr = assert_one_line_failure(&output);
    assert!(stderr.contains("needs a terminal"), "{stderr}");
    assert!(log_text(&state, "client").contains("client started"));
    assert!(!runtime_home(&state).exists());
}

#[test]
fn global_options_reach_the_default_attach() {
    let state = state_home("no_subcommand_options");
    let output = gband(&state, None, &["-s", "a/b"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("'a/b'"), "{stderr}");
    assert!(stderr.contains("session name"), "{stderr}");
    assert!(stderr.contains("ASCII letters, digits"), "{stderr}");
    assert!(!state.join("gband").exists());
}

#[test]
fn unknown_option_without_a_subcommand_is_rejected_without_a_log() {
    let state = state_home("unknown_option");
    let output = gband(&state, None, &["--frobnicate"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("--frobnicate"), "{stderr}");
    assert!(!state.join("gband").exists());
}

#[test]
fn unknown_subcommand_is_rejected_without_a_log() {
    let state = state_home("unknown_subcommand");
    let output = gband(&state, None, &["frobnicate"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("frobnicate")
    );
    assert!(!state.join("gband").exists());
}

#[test]
fn server_logs_to_its_own_series_under_xdg_state_home() {
    let state = state_home("server_series");
    assert_silent_success(&gband(&state, None, &["server"]));
    assert_eq!(log_files(&state, "server").len(), 1);
    assert!(log_files(&state, "client").is_empty());
}

#[test]
fn log_file_is_named_by_role_and_date() {
    let state = state_home("file_name");
    let before = utc_today();
    assert_silent_success(&gband(&state, None, &["server"]));
    let after = utc_today();
    let dir = log_dir(&state);
    assert!(
        dir.join(format!("server.{before}.log")).is_file()
            || dir.join(format!("server.{after}.log")).is_file()
    );
}

#[test]
fn missing_directory_is_created() {
    let state = state_home("missing_directory");
    assert!(!log_dir(&state).exists());
    assert_one_line_failure(&gband(&state, None, &["attach"]));
    assert!(log_dir(&state).is_dir());
    assert_eq!(log_files(&state, "client").len(), 1);
}

#[test]
fn log_has_no_colour_codes() {
    let state = state_home("no_colour");
    assert_silent_success(&gband(&state, Some("debug"), &["server"]));
    let text = log_text(&state, "server");
    assert!(!text.is_empty());
    assert!(!text.contains('\u{1b}'));
}

#[test]
fn default_level_is_info() {
    let state = state_home("default_level");
    assert_silent_success(&gband(&state, None, &["server"]));
    let text = log_text(&state, "server");
    assert!(text.contains(" INFO "));
    assert!(!text.contains(" DEBUG "));
    assert!(!text.contains(" TRACE "));
}

#[test]
fn raised_level_logs_debug_events() {
    let state = state_home("raised_level");
    assert_silent_success(&gband(&state, Some("debug"), &["server"]));
    assert!(
        log_text(&state, "server")
            .lines()
            .any(|line| line.contains(" DEBUG gband"))
    );
}

#[test]
fn unparseable_filter_falls_back_with_a_warning() {
    let state = state_home("unparseable_filter");
    assert_silent_success(&gband(&state, Some("[[["), &["server"]));
    assert!(
        log_text(&state, "server")
            .lines()
            .any(|line| line.contains(" WARN ") && line.contains("\"[[[\""))
    );
}

#[test]
fn unwritable_state_directory_fails_setup() {
    let state = state_home("unwritable");
    fs::set_permissions(&state, fs::Permissions::from_mode(0o555)).unwrap();
    let output = gband(&state, None, &["server"]);
    fs::set_permissions(&state, fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(stderr.lines().count(), 1);
    assert!(stderr.contains(log_dir(&state).to_str().unwrap()));
    assert!(stderr.contains("Permission denied"));
}
