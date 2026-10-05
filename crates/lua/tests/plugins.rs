mod common;

use std::path::PathBuf;

use common::*;
use gband_core::action::Action;
use gband_core::view::ViewAction;
use gband_lua::{Binding, Chord, ConfigError, Dispatch, LoadOptions, Locations};

fn plugin_error<'a>(errors: &'a [ConfigError], plugin: &str) -> &'a ConfigError {
    errors
        .iter()
        .find(|error| error.plugin.as_deref() == Some(plugin))
        .unwrap_or_else(|| panic!("no error names {plugin}: {errors:?}"))
}

#[test]
fn default_runtimepath() {
    let scratch = Scratch::new("runtimepath");
    scratch.plugin_file("zeta", "README", "");
    scratch.plugin_file("alpha", "README", "");
    scratch.plugin_file("Beta", "README", "");
    let config = scratch.loaded();
    let runtimepath: Vec<String> = eval(&config, "return gband.runtimepath");
    let entry = |path: PathBuf| path.to_string_lossy().into_owned();
    assert_eq!(
        runtimepath,
        [
            entry(scratch.user()),
            entry(scratch.plugins().join("Beta")),
            entry(scratch.plugins().join("alpha")),
            entry(scratch.plugins().join("zeta")),
        ]
    );
}

#[test]
fn missing_plugins_directory_leaves_the_user_directory() {
    let scratch = Scratch::new("no-plugins");
    let config = scratch.loaded();
    let runtimepath: Vec<String> = eval(&config, "return gband.runtimepath");
    assert_eq!(runtimepath, [scratch.user().to_string_lossy().into_owned()]);
    let locations = Locations {
        config: scratch.dir(),
        plugins: None,
    };
    let config = gband_lua::load(&locations, &LoadOptions::default()).unwrap();
    let count: usize = eval(&config, "return #gband.runtimepath");
    assert_eq!(count, 1);
}

#[test]
fn side_and_version() {
    let scratch = Scratch::new("side");
    scratch.write("side, version = gband.side, gband.api_version");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "side"), "client");
    assert_eq!(global::<i64>(&config, "version"), 1);
}

#[test]
fn module_from_a_plugin_directory() {
    let scratch = Scratch::new("module");
    scratch.plugin_file("hello", "lua/hello/init.lua", "return { where = 'plugin' }");
    scratch.write("where = require('hello').where");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "where"), "plugin");
}

#[test]
fn earlier_entry_wins() {
    let scratch = Scratch::new("earlier");
    scratch.user_file("lua/util.lua", "return 'user'");
    scratch.plugin_file("hello", "lua/util.lua", "return 'plugin'");
    scratch.write("found = require('util')");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "found"), "user");
}

#[test]
fn submodule() {
    let scratch = Scratch::new("submodule");
    scratch.plugin_file("hello", "lua/hello/keys.lua", "return 'keys'");
    scratch.write("found = require('hello.keys')");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "found"), "keys");
}

#[test]
fn error_inside_a_module_names_its_path_and_line() {
    let scratch = Scratch::new("module-error");
    let module = scratch.plugin_file("hello", "lua/hello/init.lua", "\n\nerror('bad module')");
    scratch.write("require('hello')");
    let error = scratch.load().err().unwrap();
    assert_error_at(&error, &module, 3, "bad module");
}

#[test]
fn load_order() {
    let scratch = Scratch::new("order");
    scratch.write("order = { 'init' }");
    scratch.user_file("plugin/b.lua", "table.insert(order, 'b')");
    scratch.user_file("plugin/a.lua", "table.insert(order, 'a')");
    scratch.user_file("plugin/client/c.lua", "table.insert(order, 'c')");
    scratch.user_file("plugin/notes.txt", "table.insert(order, 'txt')");
    scratch.plugin_file("hello", "plugin/d.lua", "table.insert(order, 'd')");
    let config = scratch.loaded();
    let order: Vec<String> = global(&config, "order");
    assert_eq!(order, ["init", "a", "b", "c", "d"]);
}

#[test]
fn server_plugin_files_are_ignored() {
    let scratch = Scratch::new("server-files");
    scratch.plugin_file("hello", "plugin/server/s.lua", "error('server')");
    scratch.plugin_file("hello", "plugin/other/o.lua", "error('other')");
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
}

#[test]
fn init_file_adds_an_entry() {
    let scratch = Scratch::new("added-entry");
    let extra = scratch.0.join("opt").join("hello");
    write(&extra.join("plugin").join("hello.lua"), "sourced = true");
    scratch.write(&format!(
        "table.insert(gband.runtimepath, '{}')",
        extra.display()
    ));
    let config = scratch.loaded();
    assert!(global::<bool>(&config, "sourced"));
}

#[test]
fn default_configuration_with_a_plugin() {
    let scratch = Scratch::new("defaults-plugin");
    scratch.plugin_file(
        "keys",
        "plugin/keys.lua",
        "gband.keymap.set('root', 'alt+g', gband.action.focus_column_left)",
    );
    let config = scratch.loaded();
    assert_eq!(config.keymap["prefix"].len(), 19);
    assert_eq!(
        config.keymap["root"],
        [(
            Chord::Key(key("alt+g")),
            Binding::Action(Action::View(ViewAction::FocusLeft))
        )]
    );
}

#[test]
fn error_in_a_plugin_file() {
    let scratch = Scratch::new("plugin-file-error");
    let a = scratch.plugin_file("hello", "plugin/a.lua", "\nerror('boom')");
    scratch.plugin_file("hello", "plugin/b.lua", "b_sourced = true");
    scratch.plugin_file("other", "plugin/c.lua", "c_sourced = true");
    let config = scratch.loaded();
    assert!(global::<Option<bool>>(&config, "b_sourced").is_none());
    assert!(global::<bool>(&config, "c_sourced"));
    let error = plugin_error(&config.errors, "hello");
    assert_error_at(error, &a, 2, "boom");
    assert_eq!(error.to_string(), format!("hello: {}:2: boom", a.display()));
}

const HELLO: &str = "return {
  name = 'hello',
  api = 1,
  setup = function(opts)
    stored = opts.greeting
    received = opts
  end,
}";

#[test]
fn setup_receives_the_options() {
    let scratch = Scratch::new("setup");
    scratch.plugin_file("hello", "lua/hello/init.lua", HELLO);
    scratch.write(
        "result = gband.plugin('hello', { greeting = 'hi' })\nempty = gband.plugin('hello')",
    );
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "stored"), "hi");
    assert!(global::<bool>(&config, "result"));
    assert!(!global::<bool>(&config, "empty"));
    assert!(
        plugin_error(&config.errors, "hello")
            .message
            .contains("already set up")
    );
}

#[test]
fn setup_without_options_receives_an_empty_table() {
    let scratch = Scratch::new("setup-empty");
    scratch.plugin_file("hello", "lua/hello/init.lua", HELLO);
    scratch.write("gband.plugin('hello')");
    let config = scratch.loaded();
    let kind: String = eval(&config, "return type(received)");
    assert_eq!(kind, "table");
}

#[test]
fn api_mismatch_still_sets_up() {
    let scratch = Scratch::new("api");
    scratch.plugin_file(
        "hello",
        "lua/hello/init.lua",
        "return { api = 2, setup = function() ran = true end }",
    );
    scratch.write("result = gband.plugin('hello')");
    let config = scratch.loaded();
    assert!(global::<bool>(&config, "ran"));
    assert!(global::<bool>(&config, "result"));
    assert!(config.errors.is_empty(), "{:?}", config.errors);
}

#[test]
fn missing_module() {
    let scratch = Scratch::new("missing-module");
    let init = scratch.write("result = gband.plugin('absent')\nafter = true");
    let config = scratch.loaded();
    assert!(!global::<bool>(&config, "result"));
    assert!(global::<bool>(&config, "after"));
    let error = plugin_error(&config.errors, "absent");
    assert_error_at(error, &init, 1, "absent");
}

#[test]
fn wrong_shape() {
    let scratch = Scratch::new("shape");
    scratch.plugin_file("hello", "lua/hello/init.lua", "return { name = 'hello' }");
    scratch.write("result = gband.plugin('hello')");
    let config = scratch.loaded();
    assert!(!global::<bool>(&config, "result"));
    assert!(
        plugin_error(&config.errors, "hello")
            .message
            .contains("hello")
    );
}

#[test]
fn setup_error_disables_the_plugin() {
    let scratch = Scratch::new("setup-error");
    let module = scratch.plugin_file(
        "broken",
        "lua/broken/init.lua",
        "return { setup = function()
  gband.action.register('go', function() gband.action.focus_column_left() end)
  gband.keymap.set('root', 'alt+b', gband.action['broken.go'])
  gband.on('FocusChanged', function() handled = true end)
  gband.cmd.register('run', function() end)
  error('setup failed')
end }",
    );
    scratch.write(
        "result = gband.plugin('broken')\ngband.bind('alt+h', gband.action.focus_column_left)\ngband.opt.default_column_width = 1/3",
    );
    let config = scratch.loaded();
    assert!(!global::<bool>(&config, "result"));
    assert_error_at(
        plugin_error(&config.errors, "broken"),
        &module,
        6,
        "setup failed",
    );
    assert_eq!(config.options.layout.default_width.den, 3);
    let root = &config.keymap["root"];
    assert_eq!(root.len(), 2);
    let Binding::Callback(go) = root[0].1 else {
        panic!("{root:?}");
    };
    let outcome = config.runtime.call(go);
    assert!(outcome.disabled);
    assert!(outcome.dispatched.is_empty());
    clean(&config.runtime.emit(&gband_lua::Event::FocusChanged {
        pane: None,
        previous: None,
    }));
    assert!(global::<Option<bool>>(&config, "handled").is_none());
}

#[test]
fn plugin_error_keeps_the_rest() {
    let scratch = Scratch::new("keeps-rest");
    scratch.plugin_file(
        "broken",
        "lua/broken/init.lua",
        "return { setup = function() error('nope') end }",
    );
    scratch.write("gband.bind('alt+h', gband.action.focus_column_left)\ngband.plugin('broken')");
    let config = scratch.loaded();
    assert_eq!(
        config.keymap["root"],
        [(
            Chord::Key(key("alt+h")),
            Binding::Action(Action::View(ViewAction::FocusLeft))
        )]
    );
    assert_eq!(config.errors.len(), 1);
}

#[test]
fn plugin_action_is_namespaced() {
    let scratch = Scratch::new("namespaced");
    scratch.plugin_file(
        "hello",
        "plugin/hello.lua",
        "gband.action.register('greet', function() end)\ngband.cmd.register('hello.say', function() end)",
    );
    let config = scratch.loaded();
    let kinds: Vec<String> = eval(
        &config,
        "return { type(gband.action['hello.greet']), type(gband.action.greet) }",
    );
    assert_eq!(kinds, ["userdata", "nil"]);
    let command: String = eval(&config, "return gband.cmd.list()[1].name");
    assert_eq!(command, "hello.say");
}

#[test]
fn foreign_namespace() {
    let scratch = Scratch::new("foreign");
    let file = scratch.plugin_file(
        "hello",
        "plugin/hello.lua",
        "\n\n\ngband.action.register('other.greet', function() end)",
    );
    let config = scratch.loaded();
    assert_error_at(
        plugin_error(&config.errors, "hello"),
        &file,
        4,
        "other.greet",
    );
}

#[test]
fn registered_action_called_from_another_callback() {
    let scratch = Scratch::new("nested-action");
    scratch.plugin_file(
        "hello",
        "plugin/hello.lua",
        "gband.action.register('greet', function()
  log[#log + 1] = 'greet'
  gband.action.focus_column_left()
end)",
    );
    scratch.write(
        "log = {}
gband.bind('alt+g', function()
  gband.action['hello.greet']()
  log[#log + 1] = 'after'
  gband.action.focus_column_right()
end)",
    );
    let config = scratch.loaded();
    let Binding::Callback(callback) = config.keymap["root"][0].1 else {
        panic!("not a function");
    };
    let outcome = config.runtime.call(callback);
    clean(&outcome);
    assert_eq!(global::<Vec<String>>(&config, "log"), ["greet", "after"]);
    assert_eq!(
        outcome.dispatched,
        [
            Dispatch::Action(Action::View(ViewAction::FocusLeft)),
            Dispatch::Action(Action::View(ViewAction::FocusRight)),
        ]
    );
}

#[test]
fn error_in_a_registered_action_lets_the_caller_continue() {
    let scratch = Scratch::new("nested-error");
    let file = scratch.plugin_file(
        "hello",
        "plugin/hello.lua",
        "gband.action.register('greet', function()\n  error('greet failed')\nend)",
    );
    scratch.write(
        "gband.bind('alt+g', function()\n  gband.action['hello.greet']()\n  after = true\nend)",
    );
    let config = scratch.loaded();
    let Binding::Callback(callback) = config.keymap["root"][0].1 else {
        panic!("not a function");
    };
    let outcome = config.runtime.call(callback);
    assert!(global::<bool>(&config, "after"));
    assert_error_at(
        plugin_error(&outcome.errors, "hello"),
        &file,
        2,
        "greet failed",
    );
    let again = config.runtime.call(callback);
    assert_eq!(again.errors.len(), 1);
}

#[test]
fn infinite_loop_in_setup() {
    let scratch = Scratch::new("setup-loop");
    scratch.plugin_file(
        "spin",
        "lua/spin/init.lua",
        "return { setup = function() while true do end end }",
    );
    scratch.plugin_file("spin", "plugin/spin.lua", "require('spin').setup()");
    scratch.write("gband.bind('alt+h', gband.action.focus_column_left)");
    let config = scratch.load_with_budget(100_000).unwrap();
    let error = plugin_error(&config.errors, "spin");
    assert!(error.message.contains("instruction limit"), "{error}");
    assert_eq!(config.keymap["root"].len(), 1);
}

#[test]
fn infinite_loop_in_a_callback_disables_its_plugin() {
    let scratch = Scratch::new("callback-loop");
    scratch.plugin_file(
        "spin",
        "plugin/spin.lua",
        "gband.action.register('go', function() while true do end end)
gband.cmd.register('cmd', function() end)
gband.bind('alt+s', gband.action['spin.go'])",
    );
    scratch.plugin_file(
        "zz",
        "plugin/keys.lua",
        "gband.bind('alt+h', function() ran = gband.cmd.run('spin.cmd') end)",
    );
    let config = scratch.load_with_budget(100_000).unwrap();
    let root = &config.keymap["root"];
    let (Binding::Callback(spin), Binding::Callback(other)) = (root[0].1, root[1].1) else {
        panic!("{root:?}");
    };
    let outcome = config.runtime.call(spin);
    assert!(
        plugin_error(&outcome.errors, "spin")
            .message
            .contains("instruction limit")
    );
    assert!(config.runtime.call(spin).disabled);
    clean(&config.runtime.call(other));
    assert!(!global::<bool>(&config, "ran"));
}

#[test]
fn infinite_loop_in_the_init_file_fails_the_load() {
    let scratch = Scratch::new("init-loop");
    scratch.write("while true do end");
    let error = scratch.load_with_budget(100_000).err().unwrap();
    assert!(error.message.contains("instruction limit"), "{error}");
    assert_eq!(error.plugin, None);
}

#[test]
fn coroutines_are_limited_too() {
    let scratch = Scratch::new("coroutine-loop");
    scratch.write("coroutine.wrap(function() while true do pcall(function() while true do end end) end end)()");
    let error = scratch.load_with_budget(100_000).err().unwrap();
    assert!(error.message.contains("instruction limit"), "{error}");
}

#[test]
fn sample_plugin() {
    let scratch = Scratch::new("sample");
    let example = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/plugins/hello");
    std::fs::create_dir_all(scratch.plugins()).unwrap();
    std::os::unix::fs::symlink(&example, scratch.plugins().join("hello")).unwrap();
    scratch.write(
        "log = {}
local print = print
_G.print = function(...) log[#log + 1] = table.concat({ ... }, ' ') print(...) end
ok = gband.plugin('hello', { greeting = 'hi' })
gband.keymap.set('prefix', 'g', gband.action['hello.greet'], { desc = 'Greet' })",
    );
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    assert!(global::<bool>(&config, "ok"));
    let greeting: String = eval(&config, "return gband.opt['hello.greeting']");
    assert_eq!(greeting, "hi");
    let command: Vec<String> = eval(
        &config,
        "local c = gband.cmd.list()[1] return { c.name, c.args[1] }",
    );
    assert_eq!(command, ["hello.say", "who"]);
    let Binding::Callback(greet) = config.keymap["prefix"][0].1 else {
        panic!("{:?}", config.keymap);
    };
    let outcome = config.runtime.call(greet);
    clean(&outcome);
    let [Dispatch::Spawn(Some(gband_core::layout::Program::Argv(argv)))] =
        outcome.dispatched.as_slice()
    else {
        panic!("{:?}", outcome.dispatched);
    };
    assert_eq!(argv.first().map(String::as_str), Some("sh"));
    assert_eq!(argv.last().map(String::as_str), Some("hi"));
    clean(&config.runtime.emit(&gband_lua::Event::FocusChanged {
        pane: Some(gband_core::layout::PaneId(2)),
        previous: None,
    }));
    assert_eq!(global::<Vec<String>>(&config, "log"), ["focused pane 2"]);
}

#[test]
fn bundled_module() {
    let scratch = Scratch::new("bundled");
    scratch.write("name = require('gband.statusline.band').name");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "name"), "band");
}

#[test]
fn bundled_module_shadowed() {
    let scratch = Scratch::new("bundled-shadowed");
    scratch.user_file("lua/gband/statusline/band.lua", "return { name = 'mine' }");
    scratch.write("name = require('gband.statusline.band').name");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "name"), "mine");
}

#[test]
fn api_chunks_are_not_modules() {
    let scratch = Scratch::new("bundled-api");
    scratch.write("found = pcall(require, 'gband.hl')");
    let config = scratch.loaded();
    assert!(!global::<bool>(&config, "found"));
}

#[test]
fn errors_in_a_bundled_module_name_its_path() {
    let scratch = Scratch::new("bundled-error");
    scratch.write("gband.plugin('gband.statusline.band', { align = 'middle' })");
    let config = scratch.loaded();
    let error = plugin_error(&config.errors, "band");
    let (path, _) = error.location.clone().unwrap();
    assert_eq!(path, PathBuf::from("gband/statusline/band.lua"));
    assert!(error.message.contains("align"), "{error}");
}
