mod common;

use std::time::Duration;

use common::*;
use gband_core::input::{Key, KeyCode};
use gband_core::layout::WindowId;
use gband_lua::DEFAULTS;
use gband_protocol::ClientMessage;
use gband_test_support::{TestClient, TestServer};
use portable_pty::CommandBuilder;

const TITLES: &str = "GBAND_WINDOW_TITLES";
const RELOADED: &str = "configuration reloaded";

fn automatic(client: &TestClient, window: WindowId) -> Option<&str> {
    client
        .names
        .get(&window)
        .map(|(automatic, _)| automatic.as_str())
}

fn manual(client: &TestClient, window: WindowId) -> Option<&str> {
    client
        .names
        .get(&window)
        .and_then(|(_, manual)| manual.as_deref())
}

async fn rename(client: &mut TestClient, window: WindowId, name: Option<&str>) {
    client
        .send(&ClientMessage::Rename {
            window,
            name: name.map(str::to_owned),
        })
        .await;
}

async fn started(name: &str) -> (TestServer, TestClient) {
    let server = TestServer::start(name, &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let first = client.first();
    client.wait_for_prompt(first).await;
    (server, client)
}

#[tokio::test(flavor = "multi_thread")]
async fn automatic_name_is_the_command_at_first() {
    let (_server, mut client) = started("names-first").await;
    let first = client.first();
    client
        .wait_until(|client| automatic(client, first) == Some("sh"))
        .await;
    assert_eq!(manual(&client, first), None);
}

#[tokio::test(flavor = "multi_thread")]
async fn title_set_by_the_program() {
    let (_server, mut client) = started("names-title").await;
    let first = client.first();
    client.type_line("printf '\\033]2;build\\007'").await;
    client
        .wait_until(|client| automatic(client, first) == Some("build"))
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn command_in_the_foreground() {
    let (_server, mut client) = started("names-foreground").await;
    let first = client.first();
    client.type_line("/bin/sleep 100").await;
    client
        .wait_until(|client| automatic(client, first) == Some("sleep"))
        .await;
    client
        .key(Key::new(
            KeyCode::Char('c'),
            gband_core::input::Modifiers::CTRL,
        ))
        .await;
    client
        .wait_until(|client| automatic(client, first) == Some("sh"))
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn prompt_title_wins_over_the_command() {
    let (_server, mut client) = started("names-prompt-title").await;
    let first = client.first();
    client
        .type_line("PS1=\"$(printf '\\033]2;user@host:~\\007')\\$ \"")
        .await;
    client
        .wait_until(|client| automatic(client, first) == Some("user@host:~"))
        .await;
    client.type_line("sleep 100").await;
    client.pump(Duration::from_millis(1500)).await;
    assert_eq!(automatic(&client, first), Some("user@host:~"));
}

#[tokio::test(flavor = "multi_thread")]
async fn empty_title_clears_it() {
    let (_server, mut client) = started("names-empty-title").await;
    let first = client.first();
    client.type_line("printf '\\033]2;build\\007'").await;
    client
        .wait_until(|client| automatic(client, first) == Some("build"))
        .await;
    client.type_line("printf '\\033]2;\\007'; sleep 100").await;
    client
        .wait_until(|client| automatic(client, first) == Some("sleep"))
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn manual_name_overrides_the_title() {
    let (_server, mut client) = started("names-manual").await;
    let first = client.first();
    client.type_line("printf '\\033]2;build\\007'").await;
    client
        .wait_until(|client| automatic(client, first) == Some("build"))
        .await;
    rename(&mut client, first, Some("  logs ")).await;
    client
        .wait_until(|client| manual(client, first) == Some("logs"))
        .await;
    client.type_line("printf '\\033]2;test\\007'").await;
    client
        .wait_until(|client| automatic(client, first) == Some("test"))
        .await;
    assert_eq!(manual(&client, first), Some("logs"));
    rename(&mut client, first, Some("  ")).await;
    client
        .wait_until(|client| manual(client, first).is_none())
        .await;
    assert_eq!(automatic(&client, first), Some("test"));
}

#[tokio::test(flavor = "multi_thread")]
async fn rename_of_an_unknown_window_is_ignored() {
    let (_server, mut client) = started("names-unknown").await;
    let first = client.first();
    rename(&mut client, WindowId(99), Some("x")).await;
    rename(&mut client, first, Some("logs")).await;
    client
        .wait_until(|client| manual(client, first) == Some("logs"))
        .await;
    assert!(!client.names.contains_key(&WindowId(99)));
}

#[tokio::test(flavor = "multi_thread")]
async fn names_shared_by_every_client() {
    let (server, mut first_client) = started("names-shared").await;
    let mut second_client = server.attach(80, 24).await;
    let first = first_client.first();
    rename(&mut first_client, first, Some("logs")).await;
    second_client
        .wait_until(|client| manual(client, first) == Some("logs"))
        .await;
    drop(first_client);
    drop(second_client);
    let later = server.attach(80, 24).await;
    assert_eq!(manual(&later, first), Some("logs"));
}

fn titled(env: &TestEnv) -> Attached {
    let mut client = Attached::start_with(env, &env.executable, &["attach"], 80, 24, |command| {
        command.env_remove(TITLES)
    });
    client.wait_for_prompt();
    client.shell_pid(env);
    client
}

fn untitled(env: &TestEnv, adjust: impl FnOnce(&mut CommandBuilder)) -> Attached {
    let mut client = Attached::start_with(env, &env.executable, &["attach"], 80, 24, adjust);
    client.wait_for_prompt();
    client.shell_pid(env);
    client
}

fn top_border(screen: &Grid, tile: &Tile) -> String {
    let row = screen
        .contents()
        .lines()
        .nth(usize::from(tile.top))
        .unwrap_or_default()
        .to_owned();
    row.chars()
        .skip(usize::from(tile.left) + 1)
        .take(usize::from(tile.right - tile.left - 1))
        .collect()
}

fn titles(screen: &Grid) -> Vec<String> {
    let mut found = tiles(screen);
    found.sort_by_key(|tile| tile.left);
    found
        .iter()
        .map(|tile| top_border(screen, tile).trim_end_matches('─').to_owned())
        .collect()
}

fn reloads(env: &TestEnv) -> usize {
    env.log_text("client").matches(RELOADED).count()
}

#[test]
fn title_on_the_top_border() {
    let env = TestEnv::new("names-border");
    let client = titled(&env);
    client.wait_for("the title sh", |screen| titles(screen) == ["sh"]);
}

#[test]
fn title_follows_the_program() {
    let env = TestEnv::new("names-border-follows");
    let mut client = titled(&env);
    client.wait_for("the title sh", |screen| titles(screen) == ["sh"]);
    client.run("sleep 100");
    client.wait_for("the title sleep", |screen| titles(screen) == ["sleep"]);
}

#[test]
fn number_follows_the_layout() {
    let env = TestEnv::new("names-numbers");
    let mut client = titled(&env);
    client.send(b"\x00n");
    client.wait_for("two numbered shells", |screen| {
        titles(screen) == ["sh #1", "sh #2"]
    });
    client.wait_for_prompt();
    client.run("printf '\\033]2;second\\007'");
    client.wait_for("the second titled", |screen| {
        titles(screen) == ["sh", "second"]
    });
    client.run("printf '\\033]2;\\007'");
    client.wait_for("two numbered shells again", |screen| {
        titles(screen) == ["sh #1", "sh #2"]
    });
    client.send(b"\x00\x08");
    client.wait_for("the moved column first", |screen| {
        let mut found = tiles(screen);
        found.sort_by_key(|tile| tile.left);
        found.first().is_some_and(|tile| tile.focused) && titles(screen) == ["sh #1", "sh #2"]
    });
}

#[test]
fn no_titles_in_end_to_end_tests_by_default() {
    let env = TestEnv::new("names-default-off");
    let client = untitled(&env, |_| {});
    client.wait_for("a bare top border", |screen| titles(screen) == [""]);
}

#[test]
fn option_turns_titles_off() {
    let env = TestEnv::new("names-option-off");
    env.write_config(&format!(
        "{DEFAULTS}\ngband.set({{ window_titles = false }})\n"
    ));
    let client = titled(&env);
    client.wait_for("a bare top border", |screen| titles(screen) == [""]);
}

#[test]
fn environment_turns_titles_off() {
    let env = TestEnv::new("names-env-off");
    env.write_config(&format!(
        "{DEFAULTS}\ngband.set({{ window_titles = true }})\n"
    ));
    let client = untitled(&env, |command| command.env(TITLES, "off"));
    client.wait_for("a bare top border", |screen| titles(screen) == [""]);
}

#[test]
fn unknown_environment_value() {
    let env = TestEnv::new("names-env-unknown");
    let client = untitled(&env, |command| command.env(TITLES, "no"));
    client.wait_for("the title sh", |screen| titles(screen) == ["sh"]);
    let log = env.log_text("client");
    assert!(log.contains("GBAND_WINDOW_TITLES value \"no\""), "{log}");
}

#[test]
fn turned_back_on_by_a_reload() {
    let env = TestEnv::new("names-reload");
    env.write_config(&format!(
        "{DEFAULTS}\ngband.set({{ window_titles = false }})\n"
    ));
    let client = titled(&env);
    client.wait_for("a bare top border", |screen| titles(screen) == [""]);
    let seen = reloads(&env);
    env.write_config(&format!(
        "{DEFAULTS}\ngband.set({{ window_titles = true }})\n"
    ));
    wait_until(|| reloads(&env) > seen, "a configuration reload");
    client.wait_for("the title sh", |screen| titles(screen) == ["sh"]);
}
