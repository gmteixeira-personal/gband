use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const SUBCOMMANDS: [&str; 7] = [
    "server",
    "attach",
    "list-sessions",
    "kill-session",
    "kill-server",
    "completions",
    "install-completions",
];

fn scratch(name: &str) -> PathBuf {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("completions")
        .join(name);
    if path.exists() {
        for entry in fs::read_dir(&path).unwrap() {
            fs::set_permissions(entry.unwrap().path(), fs::Permissions::from_mode(0o755)).ok();
        }
        fs::remove_dir_all(&path).unwrap();
    }
    fs::create_dir_all(path.join("state")).unwrap();
    path
}

fn gband(root: &Path, vars: &[(&str, &Path)], args: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_gband"));
    command
        .args(args)
        .env_remove("HOME")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_DATA_HOME")
        .env_remove("BASH_COMPLETION_USER_DIR")
        .env_remove("GBAND")
        .env_remove("GBAND_LOG")
        .env("XDG_STATE_HOME", root.join("state"))
        .env("XDG_RUNTIME_DIR", root.join("run"));
    for (name, value) in vars {
        command.env(name, value);
    }
    command.output().unwrap()
}

fn script(root: &Path, shell: &str) -> String {
    let output = gband(root, &[], &["completions", shell]);
    assert_eq!(output.status.code(), Some(0), "{shell}");
    assert!(output.stderr.is_empty(), "{shell}: {:?}", output.stderr);
    String::from_utf8(output.stdout).unwrap()
}

fn assert_offers_subcommands(shell: &str, text: &str) {
    for subcommand in SUBCOMMANDS {
        assert!(text.contains(subcommand), "{shell} lacks {subcommand}");
    }
}

fn on_path(binary: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|path| std::env::split_paths(&path).any(|dir| dir.join(binary).is_file()))
}

fn assert_syntax_accepted(binary: &str, flag: &str, text: &str) {
    if !on_path(binary) {
        return;
    }
    let mut child = Command::new(binary)
        .arg(flag)
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(text.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{binary} {flag}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_one_line_failure(output: &Output) -> String {
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty(), "stdout: {:?}", output.stdout);
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();
    assert_eq!(stderr.lines().count(), 1, "stderr: {stderr:?}");
    stderr
}

fn stdout(output: &Output) -> String {
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "stderr: {:?}", output.stderr);
    String::from_utf8(output.stdout.clone()).unwrap()
}

#[test]
fn fish_script() {
    let text = script(&scratch("script-fish"), "fish");
    assert!(text.contains("complete -c gband"), "{text}");
    assert_offers_subcommands("fish", &text);
    assert_syntax_accepted("fish", "--no-execute", &text);
}

#[test]
fn bash_script() {
    let text = script(&scratch("script-bash"), "bash");
    assert!(
        text.lines()
            .any(|line| line.trim_start().starts_with("complete -F _gband")
                && line.ends_with(" gband")),
        "{text}"
    );
    assert_offers_subcommands("bash", &text);
    assert_syntax_accepted("bash", "-n", &text);
}

#[test]
fn zsh_script() {
    let text = script(&scratch("script-zsh"), "zsh");
    assert!(text.starts_with("#compdef gband"), "{text}");
    assert_offers_subcommands("zsh", &text);
    assert_syntax_accepted("zsh", "-n", &text);
}

#[test]
fn shell_values_complete() {
    let text = script(&scratch("shell-values"), "fish");
    for subcommand in ["completions", "install-completions"] {
        assert!(
            text.lines().any(|line| line
                .contains(&format!("__fish_gband_using_subcommand {subcommand}\""))
                && line.contains("-a \"fish bash zsh\"")),
            "{subcommand}: {text}"
        );
    }
}

#[test]
fn script_is_stable_across_runs() {
    let root = scratch("stable");
    for shell in ["fish", "bash", "zsh"] {
        assert_eq!(script(&root, shell), script(&root, shell), "{shell}");
    }
}

#[test]
fn unsupported_shell_is_rejected() {
    let output = gband(&scratch("powershell"), &[], &["completions", "powershell"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    for word in ["powershell", "fish", "bash", "zsh"] {
        assert!(stderr.contains(word), "{word}: {stderr}");
    }
}

#[test]
fn missing_shell_is_rejected_with_usage() {
    for subcommand in ["completions", "install-completions"] {
        let output = gband(&scratch("missing-shell"), &[], &[subcommand]);
        assert_eq!(output.status.code(), Some(2), "{subcommand}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("Usage"), "{subcommand}: {stderr}");
    }
}

#[test]
fn install_for_fish() {
    let root = scratch("install-fish");
    let config = root.join("c");
    let output = gband(
        &root,
        &[("XDG_CONFIG_HOME", &config)],
        &["install-completions", "fish"],
    );
    let target = config.join("fish/completions/gband.fish");
    assert_eq!(stdout(&output), format!("{}\n", target.display()));
    assert_eq!(fs::read_to_string(target).unwrap(), script(&root, "fish"));
}

#[test]
fn fish_default_directory() {
    let root = scratch("fish-default");
    let home = root.join("h");
    let output = gband(&root, &[("HOME", &home)], &["install-completions", "fish"]);
    let target = home.join(".config/fish/completions/gband.fish");
    assert_eq!(stdout(&output), format!("{}\n", target.display()));
    assert!(target.is_file());
}

#[test]
fn install_for_bash() {
    let root = scratch("install-bash");
    let data = root.join("d");
    let output = gband(
        &root,
        &[("XDG_DATA_HOME", &data)],
        &["install-completions", "bash"],
    );
    let target = data.join("bash-completion/completions/gband");
    assert_eq!(stdout(&output), format!("{}\n", target.display()));
    assert_eq!(fs::read_to_string(target).unwrap(), script(&root, "bash"));
}

#[test]
fn bash_user_directory_override() {
    let root = scratch("bash-override");
    let user = root.join("b");
    let data = root.join("d");
    let output = gband(
        &root,
        &[
            ("BASH_COMPLETION_USER_DIR", &user),
            ("XDG_DATA_HOME", &data),
        ],
        &["install-completions", "bash"],
    );
    let target = user.join("completions/gband");
    assert_eq!(stdout(&output), format!("{}\n", target.display()));
    assert!(target.is_file());
    assert!(!data.exists());
}

#[test]
fn install_for_zsh() {
    let root = scratch("install-zsh");
    let data = root.join("d");
    let output = gband(
        &root,
        &[("XDG_DATA_HOME", &data)],
        &["install-completions", "zsh"],
    );
    let directory = data.join("zsh/site-functions");
    let target = directory.join("_gband");
    let text = stdout(&output);
    let lines: Vec<_> = text.lines().collect();
    assert_eq!(lines.len(), 2, "{text}");
    assert_eq!(lines[0], target.to_str().unwrap());
    for word in [directory.to_str().unwrap(), "fpath", "compinit"] {
        assert!(lines[1].contains(word), "{word}: {text}");
    }
    assert_eq!(fs::read_to_string(target).unwrap(), script(&root, "zsh"));
}

#[test]
fn relative_variable_ignored() {
    let root = scratch("relative");
    let home = root.join("h");
    let output = gband(
        &root,
        &[("XDG_CONFIG_HOME", Path::new("relative")), ("HOME", &home)],
        &["install-completions", "fish"],
    );
    let target = home.join(".config/fish/completions/gband.fish");
    assert_eq!(stdout(&output), format!("{}\n", target.display()));
    assert!(target.is_file());
}

#[test]
fn replace_an_existing_file() {
    let root = scratch("replace");
    let config = root.join("c");
    let target = config.join("fish/completions/gband.fish");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, "other text").unwrap();
    let output = gband(
        &root,
        &[("XDG_CONFIG_HOME", &config)],
        &["install-completions", "fish"],
    );
    stdout(&output);
    assert_eq!(fs::read_to_string(target).unwrap(), script(&root, "fish"));
}

#[test]
fn no_home() {
    let output = gband(&scratch("no-home"), &[], &["install-completions", "fish"]);
    let stderr = assert_one_line_failure(&output);
    assert!(
        stderr.contains("cannot determine the install directory"),
        "{stderr}"
    );
}

#[test]
fn unwritable_directory() {
    if rustix::process::geteuid().is_root() {
        return;
    }
    let root = scratch("unwritable");
    let config = root.join("c");
    fs::create_dir_all(&config).unwrap();
    fs::set_permissions(&config, fs::Permissions::from_mode(0o500)).unwrap();
    let output = gband(
        &root,
        &[("XDG_CONFIG_HOME", &config)],
        &["install-completions", "fish"],
    );
    let stderr = assert_one_line_failure(&output);
    assert!(stderr.contains(config.to_str().unwrap()), "{stderr}");
    assert_eq!(fs::read_dir(&config).unwrap().count(), 0);
    fs::set_permissions(&config, fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn no_log_file() {
    let root = scratch("no-log");
    stdout(&gband(&root, &[], &["completions", "bash"]));
    assert_eq!(fs::read_dir(root.join("state")).unwrap().count(), 0);
    assert!(!root.join("run/gband").exists());
}

#[test]
fn session_option_rejected() {
    let root = scratch("session-option");
    let output = gband(&root, &[], &["completions", "fish", "-s", "work"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("'-s' does not apply to 'completions'"),
        "{stderr}"
    );
    assert_eq!(fs::read_dir(root.join("state")).unwrap().count(), 0);
}

#[test]
fn server_option_rejected() {
    let root = scratch("server-option");
    let data = root.join("d");
    for args in [
        ["-S", "feature", "install-completions", "zsh"],
        ["install-completions", "zsh", "-S", "feature"],
    ] {
        let output = gband(&root, &[("XDG_DATA_HOME", &data)], &args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(
            stderr.contains("'-S' does not apply to 'install-completions'"),
            "{args:?}: {stderr}"
        );
    }
    assert!(!data.exists());
}

#[test]
fn socket_option_rejected() {
    let root = scratch("socket-option");
    let output = gband(&root, &[], &["-p", "/tmp/x.sock", "completions", "bash"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("'-p' does not apply to 'completions'"),
        "{stderr}"
    );
}

#[test]
fn help_lists_the_completion_subcommands() {
    let text = stdout(&gband(&scratch("help"), &[], &["--help"]));
    for subcommand in ["completions", "install-completions"] {
        assert!(
            text.lines()
                .any(|line| line.trim_start().starts_with(&format!("{subcommand} "))),
            "{subcommand}: {text}"
        );
    }
}
