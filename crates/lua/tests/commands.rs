mod common;

use common::*;
use gband_core::action::Action;
use gband_core::view::ViewAction;
use gband_lua::{Binding, CallbackId, Config, Dispatch};

fn root_callback(config: &Config, index: usize) -> CallbackId {
    config.keymap["root"]
        .iter()
        .filter_map(|(_, binding)| match binding {
            Binding::Callback(callback) => Some(*callback),
            Binding::Action(_) => None,
        })
        .nth(index)
        .unwrap_or_else(|| panic!("{:?}", config.keymap["root"]))
}

#[test]
fn register_and_list() {
    let scratch = Scratch::new("list");
    scratch.write(
        "result = gband.cmd.register('greet', function() end, { desc = 'Say hello', args = { 'who' } })",
    );
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "result"), "greet");
    let entry: Vec<String> = eval(
        &config,
        "local list = gband.cmd.list()
         assert(#list == 1)
         return { list[1].name, list[1].desc, list[1].args[1], tostring(#list[1].args) }",
    );
    assert_eq!(entry, ["greet", "Say hello", "who", "1"]);
}

#[test]
fn duplicate_name() {
    let scratch = Scratch::new("duplicate");
    let path = scratch.write(
        "gband.cmd.register('greet', function() end)\n\ngband.cmd.register('greet', function() end)",
    );
    let error = scratch.load().err().unwrap();
    assert_error_at(&error, &path, 3, "greet");
}

#[test]
fn invalid_registrations() {
    for (index, call) in [
        "gband.cmd.register('', function() end)",
        "gband.cmd.register(5, function() end)",
        "gband.cmd.register('a', 'not a function')",
        "gband.cmd.register('a', function() end, { desc = 5 })",
        "gband.cmd.register('a', function() end, { args = { 1 } })",
        "gband.cmd.register('a', function() end, 'opts')",
    ]
    .into_iter()
    .enumerate()
    {
        let scratch = Scratch::new(&format!("invalid-{index}"));
        let path = scratch.write(&format!("\n{call}"));
        let error = scratch
            .load()
            .err()
            .unwrap_or_else(|| panic!("{call} loaded"));
        assert_error_at(&error, &path, 2, "");
    }
}

#[test]
fn run_with_arguments() {
    let scratch = Scratch::new("run");
    scratch.write(
        "gband.cmd.register('greet', function(args) received = args.who end)
gband.bind('alt+g', function() result = gband.cmd.run('greet', { who = 'you' }) end)",
    );
    let config = scratch.loaded();
    clean(&config.runtime.call(root_callback(&config, 0)));
    assert_eq!(global::<String>(&config, "received"), "you");
    assert!(global::<bool>(&config, "result"));
}

#[test]
fn run_without_arguments_passes_an_empty_table() {
    let scratch = Scratch::new("run-empty");
    scratch.write(
        "gband.cmd.register('greet', function(args) kind = type(args) end)
gband.bind('alt+g', function() gband.cmd.run('greet') end)",
    );
    let config = scratch.loaded();
    clean(&config.runtime.call(root_callback(&config, 0)));
    assert_eq!(global::<String>(&config, "kind"), "table");
}

#[test]
fn command_dispatches_an_action() {
    let scratch = Scratch::new("dispatch");
    scratch.write(
        "gband.cmd.register('right', function() gband.action.focus_column_right() end)
gband.bind('alt+r', function() gband.cmd.run('right') end)",
    );
    let config = scratch.loaded();
    let outcome = config.runtime.call(root_callback(&config, 0));
    clean(&outcome);
    assert_eq!(
        outcome.dispatched,
        [Dispatch::Action(Action::View(ViewAction::FocusRight))]
    );
}

#[test]
fn failing_command() {
    let scratch = Scratch::new("failing");
    let path = scratch.write(
        "gband.cmd.register('broken', function()\n  error('broken command')\nend)
gband.bind('alt+b', function() result = gband.cmd.run('broken') end)",
    );
    let config = scratch.loaded();
    let outcome = config.runtime.call(root_callback(&config, 0));
    assert!(!global::<bool>(&config, "result"));
    assert_error_at(&outcome.errors[0], &path, 2, "broken command");
}

#[test]
fn unknown_command() {
    let scratch = Scratch::new("unknown");
    let path = scratch
        .write("gband.bind('alt+u', function()\n\n\n\n\n\n\n  gband.cmd.run('absent')\nend)");
    let config = scratch.loaded();
    let outcome = config.runtime.call(root_callback(&config, 0));
    assert_error_at(&outcome.errors[0], &path, 8, "absent");
}

#[test]
fn order() {
    let scratch = Scratch::new("order");
    scratch
        .write("gband.cmd.register('b', function() end)\ngband.cmd.register('a', function() end)");
    let config = scratch.loaded();
    let names: Vec<String> = eval(
        &config,
        "local names = {} for _, c in ipairs(gband.cmd.list()) do names[#names + 1] = c.name end return names",
    );
    assert_eq!(names, ["a", "b"]);
}

#[test]
fn listed_tables_are_copies() {
    let scratch = Scratch::new("copies");
    scratch.write("gband.cmd.register('a', function() end, { args = { 'x' } })");
    let config = scratch.loaded();
    let name: String = eval(
        &config,
        "local list = gband.cmd.list()
         list[1].name = 'changed'
         list[1].args[1] = 'changed'
         return gband.cmd.list()[1].name .. gband.cmd.list()[1].args[1]",
    );
    assert_eq!(name, "ax");
}

#[test]
fn already_namespaced() {
    let scratch = Scratch::new("namespaced");
    scratch.client_plugin("hello",
        "full = gband.cmd.register('hello.say', function() end)\nshort = gband.cmd.register('wave', function() end)",
    );
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "full"), "hello.say");
    assert_eq!(global::<String>(&config, "short"), "hello.wave");
}

#[test]
fn disabled_command_returns_false() {
    let scratch = Scratch::new("disabled");
    scratch.client_plugin(
        "broken",
        "gband.cmd.register('go', function() ran = true end)\nerror('broken plugin')",
    );
    scratch.client_plugin(
        "user",
        "gband.bind('alt+g', function() result = gband.cmd.run('broken.go') end)",
    );
    let config = scratch.loaded();
    clean(&config.runtime.call(root_callback(&config, 0)));
    assert!(!global::<bool>(&config, "result"));
    assert!(global::<Option<bool>>(&config, "ran").is_none());
}

#[test]
fn registering_after_the_load_fails() {
    let scratch = Scratch::new("late");
    let path = scratch
        .write("gband.bind('alt+g', function()\n  gband.cmd.register('x', function() end)\nend)");
    let config = scratch.loaded();
    let outcome = config.runtime.call(root_callback(&config, 0));
    assert_error_at(
        &outcome.errors[0],
        &path,
        2,
        "while the configuration loads",
    );
}
