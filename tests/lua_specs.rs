use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;

const GBAND: &str = env!("CARGO_BIN_EXE_gband");
const BUNDLED_COPIES: &str = gband_harness::runner::BUNDLED_COPIES;

fn gband_test(directory: &Path, args: &[&str]) {
    run(directory, args, false);
}

fn command(directory: &Path, args: &[&str], copies: bool) -> Command {
    let mut command = Command::new(GBAND);
    if copies {
        command.env(BUNDLED_COPIES, "1");
    } else {
        command.env_remove(BUNDLED_COPIES);
    }
    command
        .arg("test")
        .args(args)
        .current_dir(directory)
        .env_remove("GBAND")
        .env_remove("GBAND_SESSION")
        .env_remove("GBAND_WINDOW")
        .env_remove("GBAND_TEST_SOCKET");
    command
}

fn failure(directory: &Path, args: &[&str], output: &Output) -> Option<String> {
    (!output.status.success()).then(|| {
        format!(
            "gband test {} in {} exited with {:?}\n{}{}",
            args.join(" "),
            directory.display(),
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn run(directory: &Path, args: &[&str], copies: bool) {
    let output = command(directory, args, copies).output().unwrap();
    if let Some(failure) = failure(directory, args, &output) {
        panic!("{failure}");
    }
}

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn bundled_sidebar() {
    gband_test(root(), &["tests/lua/sidebar_spec.lua"]);
}

#[test]
fn mouse() {
    gband_test(root(), &["tests/lua/mouse_spec.lua"]);
}

#[test]
fn floating_target() {
    gband_test(root(), &["tests/lua/floating_target_spec.lua"]);
}

#[test]
fn minimize() {
    gband_test(root(), &["tests/lua/minimize_spec.lua"]);
}

#[test]
fn peek() {
    gband_test(root(), &["tests/lua/peek_spec.lua"]);
}

#[test]
fn floating_key_style() {
    gband_test(root(), &["tests/lua/floating_spec.lua"]);
}

#[test]
fn bundled_key_list() {
    gband_test(root(), &["tests/lua/keylist_spec.lua"]);
}

#[test]
fn bundled_error_list() {
    gband_test(root(), &["tests/lua/errors_spec.lua"]);
}

#[test]
fn bundled_lua_prompt() {
    gband_test(root(), &["tests/lua/prompt_spec.lua"]);
}

#[test]
fn key_styles() {
    gband_test(root(), &["tests/lua/keystyle_spec.lua"]);
}

#[test]
fn settings_and_themes() {
    gband_test(root(), &["tests/lua/settings_spec.lua"]);
}

#[test]
fn escape_key() {
    gband_test(root(), &["tests/lua/escape_spec.lua"]);
}

#[test]
fn looping_bands() {
    gband_test(root(), &["tests/lua/loop_bands_spec.lua"]);
}

#[test]
fn window_names() {
    gband_test(root(), &["tests/lua/window_names_spec.lua"]);
}

#[test]
fn window_decorations() {
    gband_test(root(), &["tests/lua/decorations_spec.lua"]);
}

#[test]
fn example_plugin_agent_status() {
    gband_test(&root().join("examples/plugins/agent-status"), &[]);
}

#[test]
fn example_plugin_hello() {
    gband_test(&root().join("examples/plugins/hello"), &[]);
}

#[test]
fn example_plugin_window() {
    gband_test(&root().join("examples/plugins/window"), &[]);
}

fn tutorial_directories() -> Vec<PathBuf> {
    let subdirectories = |directory: &Path| -> Vec<PathBuf> {
        let Ok(entries) = std::fs::read_dir(directory) else {
            return Vec::new();
        };
        let mut found: Vec<PathBuf> = entries
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_dir())
            .collect();
        found.sort();
        found
    };
    let chapters = ["examples/tutorial", "examples/internals"]
        .into_iter()
        .flat_map(|tutorial| subdirectories(&root().join(tutorial)));
    let mut directories = Vec::new();
    for chapter in chapters {
        let plugins = subdirectories(&chapter.join("plugins"));
        directories.push(chapter);
        directories.extend(
            plugins
                .into_iter()
                .filter(|plugin| plugin.join("tests").is_dir()),
        );
    }
    directories
}

#[test]
fn tutorial() {
    let directories = tutorial_directories();
    assert!(
        directories
            .iter()
            .any(|directory| directory.ends_with("examples/tutorial/00-setup")),
        "{directories:?}"
    );
    assert!(
        directories
            .iter()
            .any(|directory| directory.ends_with("examples/internals/12-own-prelude")),
        "{directories:?}"
    );
    let failures: Vec<String> = thread::scope(|scope| {
        let runs: Vec<_> = directories
            .iter()
            .map(|directory| {
                scope.spawn(move || {
                    let output = command(directory, &[], false).output().unwrap();
                    failure(directory, &[], &output)
                })
            })
            .collect();
        runs.into_iter()
            .filter_map(|run| run.join().unwrap())
            .collect()
    });
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn bundled_copies() {
    gband_test(root(), &["tests/lua/bundled_copies_spec.lua"]);
}

#[test]
fn every_case_with_bundled_copies() {
    let mut specs: Vec<String> = std::fs::read_dir(root().join("tests/lua"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with("_spec.lua"))
        .map(|name| format!("tests/lua/{name}"))
        .collect();
    specs.sort();
    let specs: Vec<&str> = specs.iter().map(String::as_str).collect();
    run(root(), &specs, true);
    for example in ["agent-status", "hello", "window"] {
        run(&root().join("examples/plugins").join(example), &[], true);
    }
}
