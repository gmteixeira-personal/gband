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

fn bottom_row(screen: &Grid) -> String {
    screen
        .contents()
        .lines()
        .nth(usize::from(screen.size().rows) - 1)
        .unwrap_or_default()
        .to_owned()
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
    client.send(b"\x00\r");
    client.wait_for("two tiles with the second focused", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 2 && tiles[1].focused
    });
    client.wait_for_prompt();
    client.send(b"printf 'Do you want to proceed?\\n'\r");
    client.wait_for("the waiting count", |screen| {
        bottom_row(screen).contains("agents waiting: 1")
    });
    client.send(b"\x00h");
    client.wait_for("the first tile focused", |screen| tiles(screen)[0].focused);
    client.send(b"\x00a");
    client.wait_for("the waiting tile focused", |screen| {
        tiles(screen)[1].focused
    });
    assert!(!env.log_text("client").contains("requires the plugin"));
    assert!(!env.log_text("server").contains("configuration error"));
}
