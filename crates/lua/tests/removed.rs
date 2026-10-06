mod common;

use common::*;
use gband_lua::ConfigError;

fn plugin_error<'a>(errors: &'a [ConfigError], plugin: &str) -> &'a ConfigError {
    errors
        .iter()
        .find(|error| error.plugin.as_deref() == Some(plugin))
        .unwrap_or_else(|| panic!("no error names {plugin}: {errors:?}"))
}

fn job_error(name: &str, code: &str, mentions: &str) -> gband_lua::Config {
    let scratch = Scratch::new(name);
    scratch.write(JOB);
    let config = scratch.loaded();
    clean(&config.runtime.set_state(drawn(80)));
    let outcome = run_job(&config, code);
    assert!(outcome.dispatched.is_empty(), "{:?}", outcome.dispatched);
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert!(error.message.contains(mentions), "{error}");
    config
}

#[test]
fn old_action_name() {
    let scratch = Scratch::new("old-action");
    let file = scratch
        .write("local a = 1\nlocal b = 2\ngband.bind(\"prefix x\", gband.action.close_pane)\n");
    let error = scratch.load().err().unwrap();
    assert_error_at(&error, &file, 3, "`close_window`");
    assert!(error.message.contains("`close_pane`"), "{error}");
}

#[test]
fn every_old_action_names_its_replacement() {
    for (old, new) in [
        ("open_pane", "open_window"),
        ("close_pane", "close_window"),
        ("focus_pane_down", "focus_window_down"),
        ("focus_pane_up", "focus_window_up"),
        ("move_pane_down", "move_window_down"),
        ("move_pane_up", "move_window_up"),
        ("toggle_pane_floating", "toggle_window_floating"),
        ("grow_pane_height", "grow_window_height"),
        ("shrink_pane_height", "shrink_window_height"),
        ("reset_pane_height", "reset_window_height"),
    ] {
        let scratch = Scratch::new(&format!("old-{old}"));
        let file = scratch.write(&format!("local action = gband.action.{old}\n"));
        let error = scratch.load().err().unwrap();
        assert_error_at(&error, &file, 1, &format!("`{old}` is now `{new}`"));
    }
}

#[test]
fn old_api_tables() {
    for (old, new) in [
        ("gband.pane", "gband.window"),
        ("gband.pane_state", "gband.window_state"),
    ] {
        let scratch = Scratch::new(&format!("old-{old}"));
        let file = scratch.write(&format!("local a = 1\nlocal api = {old}\n"));
        let error = scratch.load().err().unwrap();
        assert_error_at(&error, &file, 2, &format!("`{old}` is now `{new}`"));
    }
}

#[test]
fn old_names_in_the_server() {
    let scratch = Scratch::new("old-server");
    let file = scratch.server("local a = gband.pane_state\n");
    let config = scratch.load_server();
    let error = config.err().unwrap();
    assert_error_at(&error, &file, 1, "`gband.window_state`");
    let scratch = Scratch::new("old-server-action");
    let file = scratch.server("\nlocal a = gband.action.toggle_pane_floating\n");
    let error = scratch.load_server().err().unwrap();
    assert_error_at(&error, &file, 2, "`toggle_window_floating`");
}

#[test]
fn old_event_name() {
    let scratch = Scratch::new("old-event");
    let file = scratch.client_plugin(
        "watcher",
        "local a = 1\nlocal b = 2\nlocal c = 3\nlocal d = 4\ngband.on(\"PaneOpened\", function() end)\n",
    );
    let config = scratch.loaded();
    let error = plugin_error(&config.errors, "watcher");
    assert_error_at(error, &file, 5, "`WindowOpened`");
}

#[test]
fn every_old_event_names_its_replacement() {
    for (old, new) in [
        ("PaneOpened", "WindowOpened"),
        ("PaneClosed", "WindowClosed"),
        ("PaneExited", "WindowExited"),
        ("PaneOutput", "WindowOutput"),
        ("PaneInput", "WindowInput"),
        ("PaneStateChanged", "WindowStateChanged"),
    ] {
        let scratch = Scratch::new(&format!("old-{old}"));
        let file = scratch.server(&format!("gband.on('{old}', function() end)\n"));
        let error = scratch.load_server().err().unwrap();
        assert_error_at(&error, &file, 1, &format!("`{old}` is now `{new}`"));
    }
}

#[test]
fn old_target_field() {
    job_error(
        "old-target",
        "gband.action.close_window({ pane = 1 })",
        "`pane` is now `window`",
    );
}

#[test]
fn old_plugin_window_kind() {
    let config = job_error(
        "old-kind-pane",
        "gband.win.open({ kind = \"pane\" })",
        "`kind = \"tiled\"`",
    );
    let open: Vec<u32> = eval(&config, "return gband.win.list()");
    assert!(open.is_empty());
}

#[test]
fn old_floating_kind() {
    let config = job_error(
        "old-kind-float",
        "gband.win.open({ kind = \"float\" })",
        "`kind = \"floating\"`",
    );
    let open: Vec<u32> = eval(&config, "return gband.win.list()");
    assert!(open.is_empty());
}

#[test]
fn filled_in_fields_are_absent() {
    let scratch = Scratch::new("old-fields");
    scratch.write(JOB);
    let config = scratch.loaded();
    clean(&config.runtime.set_state(drawn(80)));
    let outcome = run_job(&config, "old = { gband.view().pane, gband.layout().panes }");
    clean(&outcome);
    let old: mlua::Table = global(&config, "old");
    assert_eq!(old.raw_len(), 0);
}
