use std::path::Path;
use std::process::Command;

const GBAND: &str = env!("CARGO_BIN_EXE_gband");

fn gband_test(directory: &Path, args: &[&str]) {
    let output = Command::new(GBAND)
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
fn bundled_status_line_segments() {
    gband_test(root(), &["tests/lua/statusline_spec.lua"]);
}

#[test]
fn bundled_key_list() {
    gband_test(root(), &["tests/lua/keylist_spec.lua"]);
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
