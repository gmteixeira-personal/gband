mod common;

use std::thread;
use std::time::Duration;

use common::*;
use gband_lua::DEFAULTS;

fn first_row(screen: &Grid) -> String {
    screen
        .contents()
        .lines()
        .next()
        .unwrap_or_default()
        .to_owned()
}

fn shows_error(screen: &Grid) -> bool {
    first_row(screen).starts_with("error ")
}

#[test]
fn server_error_cleared_in_one_client() {
    let env = TestEnv::new("errors-server-cleared");
    env.write_server_config("gband.set {}\nlocal = 1\n");
    env.write_config(&format!(
        "{DEFAULTS}\ngband.bind('alt+x', function() gband.clear_errors() end)\n"
    ));
    let expected = format!("server: {}:2:", env.server_lua().display());
    let mut first = Attached::start(&env, 120, 24);
    first.wait_for("the error item in the first client", shows_error);
    let second = Attached::start(&env, 120, 24);
    second.wait_for("the error item in the second client", shows_error);
    first.wait_for_prompt();
    first.send(b"\x1bx");
    first.wait_for("the first client to drop the error item", |screen| {
        !shows_error(screen)
    });
    thread::sleep(Duration::from_millis(300));
    assert!(shows_error(&second.screen()), "{}", second.contents());
    assert!(env.log_text("client").contains(&expected));
    let third = Attached::start(&env, 120, 24);
    third.wait_for("the error item in the third client", shows_error);
}
