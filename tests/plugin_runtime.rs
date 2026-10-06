mod common;

use common::*;

#[test]
fn slow_handler() {
    let env = TestEnv::new("runtime-slow-handler");
    env.write_server_config(
        "gband.on('WindowOutput', function()
  local start = os.clock()
  while os.clock() - start < 0.2 do end
end)",
    );
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.send(b"head -c 8388608 /dev/zero | tr '\\0' x; echo; echo DO''NE\r");
    client.wait_for_line("DONE");
    wait_until(
        || env.log_text("server").contains("bytes of window output"),
        "the server log to record dropped output",
    );
}

fn focused_left(screen: &Grid) -> Option<u16> {
    tiles(screen)
        .into_iter()
        .find(|tile| tile.focused)
        .map(|tile| tile.left)
}

#[test]
fn agent_status_example() {
    let env = TestEnv::new("runtime-agent-status");
    let example = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("plugins")
        .join("agent-status");
    std::fs::create_dir_all(env.plugins_dir()).unwrap();
    std::os::unix::fs::symlink(example, env.plugins_dir().join("agent-status")).unwrap();
    let mut client = Attached::start(&env, 100, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.send(b"\x00n");
    client.wait_for("the second tile focused", |screen| {
        focused_left(screen) == Some(47)
    });
    client.wait_for_prompt();
    client.send(b"printf 'Do you want to proceed?\\n'\r");
    client.wait_for("the waiting count", |screen| {
        screen
            .contents()
            .lines()
            .next()
            .is_some_and(|row| row.chars().skip(97).collect::<String>().trim() == "1")
    });
    client.send(b"\x00h");
    client.wait_for("the first tile focused", |screen| {
        focused_left(screen) == Some(1)
    });
    client.send(b"a");
    client.wait_for("the waiting tile focused", |screen| {
        focused_left(screen) == Some(47)
    });
    assert!(!env.log_text("client").contains("requires the plugin"));
    assert!(!env.log_text("server").contains("configuration error"));
}
