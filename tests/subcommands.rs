use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn state_home(name: &str) -> PathBuf {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("subcommands")
        .join(name);
    if path.exists() {
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        fs::remove_dir_all(&path).unwrap();
    }
    fs::create_dir_all(&path).unwrap();
    path
}

fn gband(state: &Path, filter: Option<&str>, args: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_gband"));
    command.args(args).env("XDG_STATE_HOME", state);
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

#[test]
fn server_stub_logs_its_start() {
    let state = state_home("server_stub");
    let output = gband(&state, None, &["server"]);
    assert_silent_success(&output);
    assert!(log_text(&state, "server").contains("server started"));
}

#[test]
fn attach_stub_logs_its_start() {
    let state = state_home("attach_stub");
    let output = gband(&state, None, &["attach"]);
    assert_silent_success(&output);
    assert!(log_text(&state, "client").contains("client started"));
}

#[test]
fn help_lists_subcommands() {
    let state = state_home("help");
    let output = gband(&state, None, &["--help"]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("server"));
    assert!(stdout.contains("attach"));
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
fn no_subcommand_prints_usage() {
    let state = state_home("no_subcommand");
    let output = gband(&state, None, &[]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Usage"));
    assert!(stderr.contains("server"));
    assert!(stderr.contains("attach"));
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
    assert_silent_success(&gband(&state, None, &["attach"]));
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
