mod common;

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Output, Stdio};

use common::*;
use gband_lua::DEFAULTS;
use serde_json::Value as Json;

fn gband(env: &TestEnv, args: &[&str]) -> Output {
    env.command(GBAND, args)
        .env_remove("GBAND_SESSION")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn gband_with_input(env: &TestEnv, args: &[&str], input: &str) -> Output {
    let mut child = env
        .command(GBAND, args)
        .env_remove("GBAND_SESSION")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

fn report(output: &Output) -> String {
    format!(
        "status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        stdout(output),
        stderr(output)
    )
}

fn json(output: &Output) -> Vec<Json> {
    let Json::Array(entries) = serde_json::from_slice(&output.stdout).unwrap() else {
        panic!("not an array: {}", report(output));
    };
    entries
}

fn errors_json(env: &TestEnv, session: &str) -> Vec<Json> {
    json(&gband(env, &["errors", "--json", "-s", session]))
}

fn clients(env: &TestEnv, session: &str) -> Vec<u64> {
    errors_json(env, session)
        .iter()
        .filter_map(|entry| entry["client"].as_u64())
        .collect()
}

fn load_of(env: &TestEnv, session: &str, process: &str) -> u64 {
    errors_json(env, session)
        .iter()
        .find(|entry| name(entry) == process)
        .and_then(|entry| entry["load"].as_u64())
        .unwrap_or_else(|| panic!("no load for {process}"))
}

fn name(entry: &Json) -> String {
    match entry["client"].as_u64() {
        Some(number) => format!("client {number}"),
        None => "server".to_owned(),
    }
}

fn wait_for_load(env: &TestEnv, session: &str, process: &str, load: u64) {
    wait_until(
        || load_of(env, session, process) == load,
        &format!("load {load} of {process}"),
    );
}

fn attach(env: &TestEnv, args: &[&str], cols: u16) -> Attached {
    let mut client = Attached::start_with(env, GBAND, args, cols, 24, |_| {});
    client.wait_for_prompt();
    client.shell_pid(env);
    client
}

fn attached(env: &TestEnv) -> Attached {
    attach(env, &["attach"], 80)
}

fn focused_window(env: &TestEnv) -> String {
    let output = gband(env, &["eval", "return gband.core.state().window"]);
    assert!(output.status.success(), "{}", report(&output));
    stdout(&output).trim().to_owned()
}

fn two_windows_first_focused(env: &TestEnv) -> Attached {
    let mut client = attached(env);
    client.send(b"\x00n\r");
    client.wait_for("the second of two tiles focused", |screen| {
        let found = tiles(screen);
        found.len() == 2 && found[1].focused
    });
    client.wait_for_prompt();
    client.shell_pid(env);
    client.send(b"\x00h\r");
    client.wait_for("the first tile focused", |screen| {
        tiles(screen).first().is_some_and(|tile| tile.focused)
    });
    client
}

fn in_window(env: &TestEnv, client: &mut Attached, args: &str, file: &str) -> Vec<Json> {
    let path = out_file(env, file);
    client.run(&format!(
        "'{GBAND}' {args} > '{}'; echo \"done-\"{file}",
        path.display()
    ));
    client.wait_for_line(&format!("done-{file}"));
    serde_json::from_str::<Vec<Json>>(&fs::read_to_string(path).unwrap()).unwrap()
}

fn stop(env: &TestEnv, server: &mut Child) {
    assert!(gband(env, &["kill-server"]).status.success());
    wait_process_exit(server);
}

fn out_file(env: &TestEnv, file: &str) -> PathBuf {
    env.config_home().parent().unwrap().join(file)
}

#[test]
fn no_server() {
    let env = TestEnv::new("control-no-server");
    let output = gband(&env, &["errors"]);
    assert_eq!(output.status.code(), Some(1), "{}", report(&output));
    let stderr = stderr(&output);
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(
        stderr.contains(&env.socket().display().to_string()),
        "{stderr}"
    );
}

#[test]
fn run_reload_without_a_server_logs_to_the_client_series() {
    let env = TestEnv::new("control-run-reload");
    let output = gband(&env, &["reload"]);
    assert_eq!(output.status.code(), Some(1), "{}", report(&output));
    assert!(env.log_text("client").contains("client started"));
    let logs: Vec<String> = fs::read_dir(env.state_home().join("gband").join("log"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        logs.iter()
            .any(|log| log.starts_with("client.") && log.ends_with(".log")),
        "{logs:?}"
    );
    assert!(
        !logs.iter().any(|log| log.starts_with("server.")),
        "{logs:?}"
    );
}

#[test]
fn no_such_session() {
    let env = TestEnv::new("control-no-session");
    let _client = attached(&env);
    let output = gband(&env, &["reload", "-s", "absent"]);
    assert_eq!(output.status.code(), Some(1), "{}", report(&output));
    let stderr = stderr(&output);
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(stderr.contains("absent"), "{stderr}");
}

#[test]
fn from_a_window_and_the_window_session() {
    let env = TestEnv::new("control-window");
    let _other = attach(&env, &["-S", "feature", "attach", "-s", "other"], 80);
    let mut work = attach(&env, &["-S", "feature", "attach", "-s", "work"], 80);
    let other_number = json(&gband(
        &env,
        &["-S", "feature", "errors", "--json", "-s", "other"],
    ))[1]["client"]
        .as_u64()
        .unwrap();
    let work_number = json(&gband(
        &env,
        &["-S", "feature", "errors", "--json", "-s", "work"],
    ))[1]["client"]
        .as_u64()
        .unwrap();
    let reported = in_window(&env, &mut work, "errors --json", "errors.json");
    assert_eq!(reported.len(), 2, "{reported:?}");
    assert_eq!(reported[0]["process"], "server");
    assert_eq!(reported[1]["client"].as_u64(), Some(work_number));
    let before = load_of_named(&env, "other", other_number);
    let reloaded = in_window(&env, &mut work, "reload --json -s other", "reload.json");
    assert_eq!(reloaded.len(), 2, "{reloaded:?}");
    assert_eq!(reloaded[1]["client"].as_u64(), Some(other_number));
    assert_eq!(reloaded[1]["load"].as_u64(), Some(before + 1));
}

fn load_of_named(env: &TestEnv, session: &str, client: u64) -> u64 {
    json(&gband(
        env,
        &["-S", "feature", "errors", "--json", "-s", session],
    ))
    .iter()
    .find(|entry| entry["client"].as_u64() == Some(client))
    .and_then(|entry| entry["load"].as_u64())
    .unwrap()
}

#[test]
fn outside_a_window_the_session_is_default() {
    let env = TestEnv::new("control-outside");
    let _default = attached(&env);
    let _work = attach(&env, &["attach", "-s", "work"], 80);
    let reported = json(&gband(&env, &["errors", "--json"]));
    assert_eq!(reported.len(), 2, "{reported:?}");
    assert_eq!(
        clients(&env, "default"),
        [reported[1]["client"].as_u64().unwrap()]
    );
    assert_ne!(clients(&env, "default"), clients(&env, "work"));
}

#[test]
fn without_a_terminal() {
    let env = TestEnv::new("control-pipes");
    let _client = attached(&env);
    let output = env
        .command(GBAND, &["errors"])
        .env_remove("GBAND_SESSION")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", report(&output));
    assert!(output.stdout.is_empty(), "{}", report(&output));
}

#[test]
fn load_after_a_file_change() {
    let env = TestEnv::new("control-file-change");
    let _client = attached(&env);
    let client = format!("client {}", clients(&env, "default")[0]);
    assert_eq!(load_of(&env, "default", &client), 1);
    env.write_config(DEFAULTS);
    wait_for_load(&env, "default", &client, 2);
}

#[test]
fn everything_loads() {
    let env = TestEnv::new("control-reload-all");
    let _client = attached(&env);
    let number = clients(&env, "default")[0];
    let client = format!("client {number}");
    let server = load_of(&env, "default", "server");
    let loaded = load_of(&env, "default", &client);
    let output = gband(&env, &["reload"]);
    assert_eq!(output.status.code(), Some(0), "{}", report(&output));
    assert_eq!(
        stdout(&output),
        format!(
            "server\t{}\tok\nclient {number}\t{}\tok\n",
            server + 1,
            loaded + 1
        )
    );
    let output = gband(&env, &["reload", "--json"]);
    let reloaded = json(&output);
    assert_eq!(
        reloaded,
        [
            serde_json::json!({"process": "server", "answered": true, "load": server + 2, "error": null}),
            serde_json::json!({"process": "client", "client": number, "answered": true, "load": loaded + 2, "error": null}),
        ]
    );
}

#[test]
fn a_client_fails_and_keeps_its_configuration() {
    let env = TestEnv::new("control-reload-fails");
    let mut client = attached(&env);
    let number = clients(&env, "default")[0];
    env.write_config("error('bad')\n");
    let output = gband(&env, &["reload"]);
    assert_eq!(output.status.code(), Some(1), "{}", report(&output));
    let line = stdout(&output)
        .lines()
        .find(|line| line.starts_with(&format!("client {number}\t")))
        .unwrap()
        .to_owned();
    assert!(line.contains("\terror\t"), "{line}");
    assert!(line.contains("bad"), "{line}");
    client.send(b"\x00n\r");
    client.wait_for("a second tile from the kept bindings", |screen| {
        tiles(screen).len() == 2
    });
}

#[test]
fn plugin_change_takes_effect() {
    let env = TestEnv::new("control-plugin-change");
    env.write_client_plugin(
        "label",
        "gband.bar.add({ side = 'right', size = 6, lines = { 'old' } })\n",
    );
    let client = attached(&env);
    client.wait_for_text("old");
    env.write_client_plugin(
        "label",
        "gband.bar.add({ side = 'right', size = 6, lines = { 'new' } })\n",
    );
    let output = gband(&env, &["reload"]);
    assert_eq!(output.status.code(), Some(0), "{}", report(&output));
    client.wait_for_text("new");
}

#[test]
fn no_client_attached() {
    let env = TestEnv::new("control-no-client");
    let mut server = env.start_server("/bin/sh");
    let output = gband(&env, &["reload"]);
    assert_eq!(output.status.code(), Some(0), "{}", report(&output));
    assert_eq!(stdout(&output), "server\t2\tok\n");
    stop(&env, &mut server);
}

#[test]
fn clients_of_another_session_keep_theirs() {
    let env = TestEnv::new("control-other-session");
    let _work = attach(&env, &["attach", "-s", "work"], 80);
    let _other = attach(&env, &["attach", "-s", "other"], 80);
    let work = clients(&env, "work")[0];
    let other = format!("client {}", clients(&env, "other")[0]);
    let output = gband(&env, &["reload", "-s", "work"]);
    assert_eq!(output.status.code(), Some(0), "{}", report(&output));
    let lines: Vec<&str> = output
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| std::str::from_utf8(line).unwrap())
        .collect();
    assert_eq!(lines.len(), 2, "{}", report(&output));
    assert!(lines[0].starts_with("server\t"));
    assert!(lines[1].starts_with(&format!("client {work}\t")));
    assert_eq!(load_of(&env, "other", &other), 1);
}

#[test]
fn no_errors() {
    let env = TestEnv::new("control-no-errors");
    let _client = attached(&env);
    let output = gband(&env, &["errors"]);
    assert_eq!(output.status.code(), Some(0), "{}", report(&output));
    assert!(output.stdout.is_empty(), "{}", report(&output));
    let reported = errors_json(&env, "default");
    assert_eq!(reported.len(), 2);
    for entry in &reported {
        assert_eq!(entry["errors"], serde_json::json!([]), "{entry}");
        assert_eq!(entry["answered"], true);
    }
}

#[test]
fn error_after_a_save_then_cleared() {
    let env = TestEnv::new("control-error-save");
    let _client = attached(&env);
    let number = clients(&env, "default")[0];
    let client = format!("client {number}");
    env.write_config("\nerror('bad')\n");
    wait_for_load(&env, "default", &client, 2);
    let output = gband(&env, &["errors"]);
    assert_eq!(output.status.code(), Some(1), "{}", report(&output));
    let text = stdout(&output);
    assert_eq!(text.lines().count(), 1, "{text}");
    assert!(text.starts_with(&format!("client {number}\t2\t")), "{text}");
    assert!(text.contains("user/init.lua:2"), "{text}");
    assert!(text.contains("bad"), "{text}");
    env.write_config(DEFAULTS);
    wait_for_load(&env, "default", &client, 3);
    let output = gband(&env, &["errors"]);
    assert_eq!(output.status.code(), Some(0), "{}", report(&output));
    assert!(output.stdout.is_empty(), "{}", report(&output));
}

#[test]
fn server_error_listed_once() {
    let env = TestEnv::new("control-server-error");
    env.write_server_plugin(
        "broken",
        "gband.on('WindowOpened', function() error('boom') end)\n",
    );
    let mut client = attached(&env);
    client.send(b"\x00n\r");
    client.wait_for("the error marker", sidebar_error);
    let output = gband(&env, &["errors"]);
    assert_eq!(output.status.code(), Some(1), "{}", report(&output));
    let text = stdout(&output);
    assert!(text.contains("boom"), "{text}");
    for line in text.lines() {
        assert!(line.starts_with("server\t"), "{text}");
    }
}

#[test]
fn eval_reads_the_client_and_the_server() {
    let env = TestEnv::new("control-eval-read");
    let _client = attached(&env);
    let output = gband(
        &env,
        &["eval", "return gband.side, select('#', ...)", "a", "b"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", report(&output));
    assert_eq!(stdout(&output), "client\n2\n");
    let output = gband(&env, &["eval", "--on-server", "return gband.side"]);
    assert_eq!(stdout(&output), "server\n", "{}", report(&output));
    let output = gband(&env, &["eval", "--json", "return { 1, 2 }, { a = true }"]);
    assert_eq!(stdout(&output), "[[1,2],{\"a\":true}]\n");
    let output = gband(
        &env,
        &[
            "eval",
            "--json",
            "return { 'a', 'b' }, { [1] = 'a', x = 2 }",
        ],
    );
    let values: Json = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(values, serde_json::json!([["a", "b"], {"1": "a", "x": 2}]));
    let output = gband_with_input(&env, &["eval", "-"], "return 40 + 2");
    assert_eq!(stdout(&output), "42\n", "{}", report(&output));
}

#[test]
fn eval_acts_in_the_client() {
    let env = TestEnv::new("control-eval-act");
    let client = two_windows_first_focused(&env);
    let output = gband(&env, &["eval", "gband.action.focus_column_right()"]);
    assert_eq!(output.status.code(), Some(0), "{}", report(&output));
    assert!(output.stdout.is_empty(), "{}", report(&output));
    client.wait_for("the second tile focused", |screen| {
        tiles(screen).get(1).is_some_and(|tile| tile.focused)
    });
}

#[test]
fn eval_error_answered() {
    let env = TestEnv::new("control-eval-error");
    let client = attached(&env);
    let output = gband(&env, &["eval", "error('boom')"]);
    assert_eq!(output.status.code(), Some(1), "{}", report(&output));
    let stderr = stderr(&output);
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(stderr.contains("boom"), "{stderr}");
    assert!(!sidebar_error(&client.screen()), "{}", client.contents());
    let output = gband(&env, &["errors"]);
    assert_eq!(output.status.code(), Some(0), "{}", report(&output));
}

#[test]
fn cmd_runs_a_client_command() {
    let env = TestEnv::new("control-cmd-client");
    env.write_config(&format!(
        "{DEFAULTS}\ngband.cmd.register('greet', function(args) return 'hi ' .. args.who end)\n\
         gband.cmd.register('types', function(args) return type(args.n) .. ' ' .. type(args.list) end)\n"
    ));
    let _client = attached(&env);
    let output = gband(&env, &["cmd", "greet", r#"{"who":"you"}"#]);
    assert_eq!(output.status.code(), Some(0), "{}", report(&output));
    assert_eq!(stdout(&output), "hi you\n");
    let output = gband(&env, &["cmd", "types", r#"{"n":1,"list":[1,2]}"#]);
    assert_eq!(stdout(&output), "number table\n", "{}", report(&output));
    let output = gband(&env, &["cmd", "absent"]);
    assert_eq!(output.status.code(), Some(1), "{}", report(&output));
    let stderr = stderr(&output);
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(stderr.contains("absent"), "{stderr}");
}

#[test]
fn cmd_runs_a_server_command_for_the_chosen_client() {
    let env = TestEnv::new("control-cmd-server");
    env.write_server_config(
        "gband.cmd.register('focus', function(args, ctx) ctx.focus(args.window) end)\n",
    );
    let mut client = two_windows_first_focused(&env);
    let first = focused_window(&env);
    client.send(b"\x00l\r");
    client.wait_for("the second tile focused", |screen| {
        tiles(screen).get(1).is_some_and(|tile| tile.focused)
    });
    let output = gband(
        &env,
        &[
            "cmd",
            "--on-server",
            "focus",
            &format!(r#"{{"window":{first}}}"#),
        ],
    );
    assert_eq!(output.status.code(), Some(0), "{}", report(&output));
    client.wait_for("the first tile focused", |screen| {
        tiles(screen).first().is_some_and(|tile| tile.focused)
    });
}

#[test]
fn cmd_arguments_that_are_not_json() {
    let env = TestEnv::new("control-cmd-json");
    let output = gband(&env, &["cmd", "greet", "{who"]);
    assert_eq!(output.status.code(), Some(2), "{}", report(&output));
    assert!(!env.socket().exists());
}

#[test]
fn chosen_client() {
    let env = TestEnv::new("control-chosen");
    let mut first = attach(&env, &["attach"], 100);
    let _second = attach(&env, &["attach"], 120);
    let numbers = clients(&env, "default");
    let width = |args: &[&str]| {
        let output = gband(&env, args);
        assert_eq!(output.status.code(), Some(0), "{}", report(&output));
        stdout(&output)
    };
    first.run("echo typed");
    first.wait_for_line("typed");
    assert_eq!(width(&["eval", "return gband.core.state().width"]), "100\n");
    let second = numbers[1].to_string();
    assert_eq!(
        width(&[
            "eval",
            "--client",
            &second,
            "return gband.core.state().width"
        ]),
        "120\n"
    );
    let output = gband(&env, &["eval", "--client", "9999", "return 1"]);
    assert_eq!(output.status.code(), Some(1), "{}", report(&output));
    assert!(stderr(&output).contains("9999"), "{}", report(&output));
}

#[test]
fn chosen_client_attached_last_until_one_types() {
    let env = TestEnv::new("control-chosen-last");
    let mut first = Attached::start_with(&env, GBAND, &["attach"], 100, 24, |_| {});
    first.wait_for_prompt();
    let second = Attached::start_with(&env, GBAND, &["attach"], 120, 24, |_| {});
    second.wait_for_prompt();
    let width = || stdout(&gband(&env, &["eval", "return gband.core.state().width"]));
    assert_eq!(width(), "120\n");
    first.shell_pid(&env);
    assert_eq!(width(), "100\n");
}

#[test]
fn no_client_to_choose() {
    let env = TestEnv::new("control-chosen-none");
    let mut server = env.start_server("/bin/sh");
    let output = gband(&env, &["eval", "return 1"]);
    assert_eq!(output.status.code(), Some(1), "{}", report(&output));
    let stderr = stderr(&output);
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(stderr.contains("has no client"), "{stderr}");
    stop(&env, &mut server);
}
