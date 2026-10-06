mod common;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use common::*;
use gband_core::action::Action;
use gband_core::view::ViewAction;
use gband_lua::{Binding, Chord, ConfigError, Dispatch, LoadOptions, Locations};
use gband_protocol::Value as Data;

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
    let config =
        gband_lua::load(&locations, gband_lua::Side::Client, &LoadOptions::default()).unwrap();
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
fn server_side() {
    let scratch = Scratch::new("server-side");
    scratch.server("side, version = gband.side, gband.api_version");
    let config = scratch.loaded_server();
    assert_eq!(global::<String>(&config, "side"), "server");
    assert_eq!(global::<i64>(&config, "version"), 1);
}

#[test]
fn load_order() {
    let scratch = Scratch::new("order");
    scratch.write("order = { 'init' }");
    scratch.client_plugin("beta", "table.insert(order, 'beta')");
    scratch.client_plugin("alpha", "table.insert(order, 'alpha')");
    let config = scratch.loaded();
    let order: Vec<String> = global(&config, "order");
    assert_eq!(order, ["init", "alpha", "beta"]);
}

#[test]
fn valid_manifest() {
    let scratch = Scratch::new("manifest");
    scratch.plugin(
        "agent-status",
        "return { name = 'agent-status', version = '0.1.0', client = '>= 0.1' }",
    );
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    let listed: Vec<String> = eval(
        &config,
        "local out = {}
         for _, p in ipairs(gband.plugins()) do
           out[#out + 1] = table.concat({ p.name, p.version, p.client, tostring(p.failed) }, '|')
         end
         return out",
    );
    assert_eq!(listed, ["agent-status|0.1.0|>= 0.1|false"]);
    assert_eq!(
        config.plugins,
        [gband_lua::PluginManifest {
            name: "agent-status".to_owned(),
            version: Some("0.1.0".to_owned()),
            client: Some(">= 0.1".to_owned()),
            failed: false,
        }]
    );
}

#[test]
fn name_mismatch() {
    let scratch = Scratch::new("mismatch");
    scratch.plugin("agent-status", "return { name = 'agents', version = '1' }");
    scratch.plugin_file("agent-status", "client.lua", "sourced = true");
    let config = scratch.loaded();
    let error = plugin_error(&config.errors, "agent-status");
    assert!(error.message.contains("`agents`"), "{error}");
    assert!(global::<Option<bool>>(&config, "sourced").is_none());
    let failed: bool = eval(&config, "return gband.plugins()[1].failed");
    assert!(failed);
}

#[test]
fn invalid_manifests() {
    for (manifest, mentions) in [
        ("return 3", "must return a table"),
        ("return { name = 'hello' }", "`version`"),
        (
            "return { name = 'hello', version = 'one' }",
            "not a version",
        ),
        (
            "return { name = 'hello', version = '1', client = '~1' }",
            "does not start",
        ),
    ] {
        let scratch = Scratch::new("invalid-manifest");
        scratch.plugin("hello", manifest);
        scratch.plugin_file("hello", "client.lua", "sourced = true");
        let config = scratch.loaded();
        let error = plugin_error(&config.errors, "hello");
        assert!(error.message.contains(mentions), "{manifest}: {error}");
        assert!(global::<Option<bool>>(&config, "sourced").is_none());
    }
}

#[test]
fn manifest_cannot_reach_gband() {
    let scratch = Scratch::new("manifest-env");
    let manifest = scratch.plugin(
        "hello",
        "gband.keymap.set('root', 'alt+x', function() end)\nreturn { name = 'hello', version = '1' }",
    );
    let config = scratch.loaded();
    let error = plugin_error(&config.errors, "hello");
    assert_eq!(error.location, Some((manifest, 1)), "{error}");
    assert!(!config.keymap.contains_key("root"));
}

#[test]
fn side_file_without_a_manifest() {
    let scratch = Scratch::new("orphan");
    scratch.plugin_file("hello", "client.lua", "sourced = true");
    let config = scratch.loaded();
    let error = plugin_error(&config.errors, "hello");
    assert!(error.message.contains("plugin.lua"), "{error}");
    assert!(global::<Option<bool>>(&config, "sourced").is_none());
}

#[test]
fn one_file_per_side() {
    let scratch = Scratch::new("per-side");
    scratch.plugin("hello", &manifest("hello"));
    scratch.plugin_file("hello", "server.lua", "server_sourced = true");
    let client = scratch.plugin_file("hello", "client.lua", "error('client')");
    let server = scratch.loaded_server();
    assert!(server.errors.is_empty(), "{:?}", server.errors);
    assert!(global::<bool>(&server, "server_sourced"));
    let config = scratch.loaded();
    assert!(global::<Option<bool>>(&config, "server_sourced").is_none());
    assert_eq!(
        plugin_error(&config.errors, "hello").location,
        Some((client, 1))
    );
}

#[test]
fn client_only_plugin() {
    let scratch = Scratch::new("client-only");
    scratch.client_plugin("hello", "error('client')");
    let server = scratch.loaded_server();
    assert!(server.errors.is_empty(), "{:?}", server.errors);
}

#[test]
fn legacy_plugin_files_are_ignored() {
    let scratch = Scratch::new("legacy");
    scratch.plugin_file("hello", "plugin/keys.lua", "error('keys')");
    scratch.plugin_file("hello", "plugin/client/k.lua", "error('k')");
    scratch.user_file("plugin/u.lua", "error('user')");
    scratch.user_file("client.lua", "error('user client')");
    assert!(scratch.loaded().errors.is_empty());
    assert!(scratch.loaded_server().errors.is_empty());
}

#[test]
fn server_plugin_files_are_ignored() {
    let scratch = Scratch::new("server-files");
    scratch.plugin("hello", &manifest("hello"));
    scratch.plugin_file("hello", "plugin/server/s.lua", "error('server')");
    assert!(scratch.loaded().errors.is_empty());
    assert!(scratch.loaded_server().errors.is_empty());
}

#[test]
fn init_file_adds_an_entry() {
    let scratch = Scratch::new("added-entry");
    let extra = scratch.0.join("opt").join("hello");
    write(&extra.join("plugin.lua"), &manifest("hello"));
    write(&extra.join("client.lua"), "sourced = true");
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
    scratch.client_plugin(
        "keys",
        "gband.keymap.set('root', 'alt+g', gband.action.focus_column_left)",
    );
    let config = scratch.loaded();
    assert_eq!(config.keymap["prefix"].len(), 38);
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
    let file = scratch.client_plugin(
        "hello",
        "gband.bind('alt+b', function() pressed = true end)\nerror('boom')",
    );
    scratch.client_plugin("other", "other_sourced = true");
    let config = scratch.loaded();
    assert!(global::<bool>(&config, "other_sourced"));
    let error = plugin_error(&config.errors, "hello");
    assert_error_at(error, &file, 2, "boom");
    assert_eq!(
        error.to_string(),
        format!("hello: {}:2: boom", file.display())
    );
    let failed: bool = eval(&config, "return gband.plugins()[1].failed");
    assert!(failed);
    let Some((_, Binding::Callback(callback))) = config.keymap["root"].first() else {
        panic!("alt+b is bound");
    };
    assert!(config.runtime.call(*callback).disabled);
    assert!(global::<Option<bool>>(&config, "pressed").is_none());
}

#[test]
fn client_api_in_the_server() {
    let scratch = Scratch::new("client-api");
    scratch.plugin("hello", &manifest("hello"));
    let file = scratch.plugin_file(
        "hello",
        "server.lua",
        "local a = 1\nlocal b = 2\nlocal k = gband.keymap",
    );
    let config = scratch.loaded_server();
    let error = plugin_error(&config.errors, "hello");
    assert_error_at(error, &file, 3, "`gband.keymap`");
    assert!(error.message.contains("client"), "{error}");
}

#[test]
fn bars_in_the_server() {
    for field in ["bar", "errors"] {
        let scratch = Scratch::new(&format!("{field}-in-the-server"));
        scratch.plugin("tabs", &manifest("tabs"));
        let file = scratch.plugin_file(
            "tabs",
            "server.lua",
            &format!("local a = 1\nlocal b = gband.{field}"),
        );
        let config = scratch.loaded_server();
        let error = plugin_error(&config.errors, "tabs");
        assert_error_at(error, &file, 2, &format!("`gband.{field}`"));
        assert!(error.message.contains("client"), "{error}");
    }
}

#[test]
fn clearing_errors_in_the_server() {
    let scratch = Scratch::new("clear-errors-in-the-server");
    scratch.plugin("tabs", &manifest("tabs"));
    let file = scratch.plugin_file(
        "tabs",
        "server.lua",
        "local a = 1\nlocal b = 2\nlocal c = 3\ngband.clear_errors()",
    );
    let config = scratch.loaded_server();
    let error = plugin_error(&config.errors, "tabs");
    assert_error_at(error, &file, 4, "`gband.clear_errors`");
    assert!(error.message.contains("client"), "{error}");
}

#[test]
fn server_api_in_the_client() {
    let scratch = Scratch::new("server-api");
    let file = scratch.write("local a = 1\ngband.sessions()");
    let error = scratch.load().err().unwrap();
    assert_error_at(&error, &file, 2, "`gband.sessions`");
    assert!(error.message.contains("server"), "{error}");
}

#[test]
fn view_action_in_the_server() {
    let scratch = Scratch::new("view-action");
    let file = scratch.server("local a = gband.action.focus_column_left");
    let error = scratch.load_server().err().unwrap();
    assert_error_at(&error, &file, 1, "focus_column_left");
    assert!(error.message.contains("client"), "{error}");
    let scratch = Scratch::new("session-action");
    scratch.server("kind = type(gband.action.close_window)\nmissing = gband.action.absent == nil");
    let config = scratch.loaded_server();
    assert_eq!(global::<String>(&config, "kind"), "userdata");
    assert!(global::<bool>(&config, "missing"));
}

#[test]
fn every_field_belongs_to_one_side() {
    let scratch = Scratch::new("fields");
    let shared = [
        "on",
        "augroup",
        "emit",
        "cmd",
        "opt",
        "set",
        "plugin",
        "plugins",
        "runtimepath",
        "side",
        "api_version",
        "window_state",
        "action",
    ];
    let list = |names: &[&str]| {
        names
            .iter()
            .map(|name| format!("'{name}'"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let check = format!(
        "missing = {{}}
         for _, name in ipairs({{ {} }}) do
           if rawget(gband, name) == nil then missing[#missing + 1] = name end
         end",
        list(&shared)
    );
    scratch.write(&check);
    scratch.server(&check);
    let missing: Vec<String> = global(&scratch.loaded(), "missing");
    assert!(missing.is_empty(), "client lacks {missing:?}");
    let missing: Vec<String> = global(&scratch.loaded_server(), "missing");
    assert!(missing.is_empty(), "server lacks {missing:?}");
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
        "result = gband.plugin('broken')\ngband.bind('alt+h', gband.action.focus_column_left)\ngband.opt.width_step = 1/4",
    );
    let config = scratch.loaded();
    assert!(!global::<bool>(&config, "result"));
    assert_error_at(
        plugin_error(&config.errors, "broken"),
        &module,
        6,
        "setup failed",
    );
    assert_eq!(
        config.options.steps.width,
        gband_core::layout::Proportion::new(1, 4)
    );
    let root = &config.keymap["root"];
    assert_eq!(root.len(), 2);
    let Binding::Callback(go) = root[0].1 else {
        panic!("{root:?}");
    };
    let outcome = config.runtime.call(go);
    assert!(outcome.disabled);
    assert!(outcome.dispatched.is_empty());
    clean(&config.runtime.emit(&gband_lua::Event::FocusChanged {
        window: None,
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
    scratch.client_plugin("hello",
        "gband.action.register('greet', function() end)\ngband.cmd.register('hello.say', function() end)",
    );
    let config = scratch.loaded();
    let kinds: Vec<String> = eval(
        &config,
        "return { type(gband.action['hello.greet']), type(gband.action.greet) }",
    );
    assert_eq!(kinds, ["userdata", "nil"]);
    let commands: Vec<String> = eval(
        &config,
        "local names = {} for _, c in ipairs(gband.cmd.list()) do names[#names + 1] = c.name end return names",
    );
    assert!(commands.contains(&"hello.say".to_owned()), "{commands:?}");
}

#[test]
fn foreign_namespace() {
    let scratch = Scratch::new("foreign");
    let file = scratch.client_plugin(
        "hello",
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
    scratch.client_plugin(
        "hello",
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
    let file = scratch.client_plugin(
        "hello",
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
    scratch.client_plugin("spin", "require('spin').setup()");
    scratch.write("gband.bind('alt+h', gband.action.focus_column_left)");
    let config = scratch.load_with_budget(100_000).unwrap();
    let error = plugin_error(&config.errors, "spin");
    assert!(error.message.contains("instruction limit"), "{error}");
    assert_eq!(config.keymap["root"].len(), 1);
}

#[test]
fn infinite_loop_in_a_callback_disables_its_plugin() {
    let scratch = Scratch::new("callback-loop");
    scratch.client_plugin(
        "spin",
        "gband.action.register('go', function() while true do end end)
gband.cmd.register('cmd', function() end)
gband.bind('alt+s', gband.action['spin.go'])",
    );
    scratch.client_plugin(
        "zz",
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
        window: Some(gband_core::layout::WindowId(2)),
        previous: None,
    }));
    assert_eq!(global::<Vec<String>>(&config, "log"), ["focused window 2"]);
}

#[test]
fn bundled_module() {
    let scratch = Scratch::new("bundled");
    scratch.write("name = require('gband.statusline.band').name");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "name"), "band");
}

#[test]
fn bundled_key_form() {
    let scratch = Scratch::new("bundled-keyform");
    scratch.write("form = require('gband.keyform')('ctrl+space')");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "form"), "C-space");
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

fn test_side() -> (gband_lua::Lua, Arc<Mutex<String>>) {
    let printed = Arc::new(Mutex::new(String::new()));
    let output = Arc::clone(&printed);
    let lua = gband_lua::Lua::new();
    gband_lua::install_test(&lua, move |line| {
        let mut output = output.lock().unwrap();
        output.push_str(line);
        output.push('\n');
    })
    .unwrap();
    (lua, printed)
}

#[test]
fn test_side_holds_only_the_side_and_version() {
    let (lua, printed) = test_side();
    lua.load("print(gband.side, gband.api_version)")
        .exec()
        .unwrap();
    assert_eq!(*printed.lock().unwrap(), "test\t1\n");
    let fields: Vec<String> = lua
        .load(
            "local names = {} for name in pairs(gband) do names[#names + 1] = name end \
             table.sort(names) return names",
        )
        .eval()
        .unwrap();
    assert_eq!(fields, ["api_version", "side"]);
}

#[test]
fn client_api_in_a_test_file() {
    let (lua, _) = test_side();
    let error = lua
        .load("local a = 1\nlocal b = 2\nlocal c = 3\nreturn gband.opt")
        .set_name("@spec_test.lua")
        .exec()
        .unwrap_err()
        .to_string();
    assert!(error.contains("spec_test.lua:4:"), "{error}");
    assert!(
        error.contains("`gband.opt` is a client and server API"),
        "{error}"
    );
    let error = lua
        .load("return gband.keymap")
        .exec()
        .unwrap_err()
        .to_string();
    assert!(error.contains("`gband.keymap` is a client API"), "{error}");
    let error = lua
        .load("return gband.sessions")
        .exec()
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("`gband.sessions` is a server API"),
        "{error}"
    );
    let unknown: Option<String> = lua.load("return gband.nothing").eval().unwrap();
    assert_eq!(unknown, None);
}

#[test]
fn frozen_time() {
    let scratch = Scratch::new("frozen-time");
    gband_lua::freeze_time(None);
    let before = scratch.loaded();
    let real: i64 = eval(&before, "return os.time()");
    assert!(real > 1_735_733_100, "{real}");
    gband_lua::freeze_time(Some(1_735_732_800));
    let now: i64 = eval(&before, "return os.time()");
    assert_eq!(now, 1_735_732_800, "a state created earlier was not frozen");
    let reloaded = scratch.loaded();
    let now: i64 = eval(&reloaded, "return os.time()");
    assert_eq!(now, 1_735_732_800);
    let year: String = eval(&reloaded, "return os.date('!%Y-%m-%d %H:%M')");
    assert_eq!(year, "2025-01-01 12:00");
    let explicit: String = eval(&reloaded, "return os.date('%Y', 0)");
    assert_eq!(explicit, "1970");
    let table: i64 = eval(
        &reloaded,
        "return os.time({ year = 2000, month = 1, day = 1, hour = 0 })",
    );
    assert_ne!(table, 1_735_732_800);
    gband_lua::freeze_time(Some(1_735_733_100));
    let moved: String = eval(&reloaded, "return os.date('!%H:%M')");
    assert_eq!(moved, "12:05");
    gband_lua::freeze_time(None);
    let real: i64 = eval(&reloaded, "return os.time()");
    assert!(real > 1_735_733_100, "{real}");
}

#[test]
fn chunks_answer_with_plain_data() {
    let config = Scratch::new("eval").loaded();
    let (answer, outcome) = config.runtime.eval(
        "return gband.side, select('#', ...)",
        &[Data::Int(1), Data::Int(2)],
    );
    assert_eq!(
        answer,
        Ok(vec![Data::Bytes(b"client".to_vec()), Data::Int(2)])
    );
    assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
    let (answer, outcome) = config.runtime.eval("error('boom')", &[]);
    assert!(answer.unwrap_err().contains("boom"));
    assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
    let (answer, _) = config.runtime.eval("return function() end", &[]);
    assert!(
        answer.as_ref().unwrap_err().contains("not plain data"),
        "{answer:?}"
    );
    let (answer, _) = config.runtime.eval("while true do end", &[]);
    assert!(
        answer.as_ref().unwrap_err().contains("instruction limit"),
        "{answer:?}"
    );
    let (answer, _) = config.runtime.eval("return (", &[]);
    assert!(answer.is_err(), "{answer:?}");
    let (_, outcome) = config.runtime.eval("gband.action.focus_column_left()", &[]);
    assert_eq!(outcome.dispatched.len(), 1, "{:?}", outcome.dispatched);
}

#[test]
fn print_in_a_test() {
    let (lua, printed) = test_side();
    lua.load("print('step', 2) print(nil, true)")
        .exec()
        .unwrap();
    assert_eq!(*printed.lock().unwrap(), "step\t2\nnil\ttrue\n");
}
