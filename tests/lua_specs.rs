use std::path::Path;
use std::process::Command;

const GBAND: &str = env!("CARGO_BIN_EXE_gband");
const BUNDLED_COPIES: &str = gband_harness::runner::BUNDLED_COPIES;

fn gband_test(directory: &Path, args: &[&str]) {
    run(directory, args, false);
}

fn run(directory: &Path, args: &[&str], copies: bool) {
    let mut command = Command::new(GBAND);
    if copies {
        command.env(BUNDLED_COPIES, "1");
    } else {
        command.env_remove(BUNDLED_COPIES);
    }
    let output = command
        .arg("test")
        .args(args)
        .current_dir(directory)
        .env_remove("GBAND")
        .env_remove("GBAND_SESSION")
        .env_remove("GBAND_WINDOW")
        .env_remove("GBAND_TEST_SOCKET")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "gband test {} in {} exited with {:?}\n{}{}",
        args.join(" "),
        directory.display(),
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
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
