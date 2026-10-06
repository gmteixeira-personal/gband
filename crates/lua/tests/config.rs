mod common;

use std::fs;
use std::path::PathBuf;

use common::*;
use gband_core::action::{Action, ClientAction, SessionCommand};
use gband_core::input::{Key, KeyCode, Modifiers};
use gband_core::layout::{Direction, Program, Proportion, Step, Vertical};
use gband_core::view::{CenterFocusedColumn, ViewAction};
use gband_lua::{
    ACTIONS, Binding, CallbackId, Chord, Config, ConfigError, DEFAULTS, Dispatch, Options,
    defaults_file, prepare,
};

type Keys = (&'static str, Chord);

fn evaluate(name: &str, source: &str) -> (PathBuf, Result<Config, ConfigError>) {
    let scratch = Scratch::new(name);
    let path = scratch.write(source);
    let result = scratch.load();
    (path, result)
}

fn loaded(name: &str, source: &str) -> Config {
    let (_, result) = evaluate(name, source);
    result.unwrap_or_else(|error| panic!("{error}"))
}

fn failure(name: &str, source: &str) -> (PathBuf, ConfigError) {
    let (path, result) = evaluate(name, source);
    match result {
        Ok(_) => panic!("{name} loaded"),
        Err(error) => (path, error),
    }
}

fn server_evaluate(name: &str, source: &str) -> (PathBuf, Result<Config, ConfigError>) {
    let scratch = Scratch::new(name);
    let path = scratch.server(source);
    let result = scratch.load_server();
    (path, result)
}

fn server_loaded(name: &str, source: &str) -> Config {
    let (_, result) = server_evaluate(name, source);
    result.unwrap_or_else(|error| panic!("{error}"))
}

fn server_failure(name: &str, source: &str) -> (PathBuf, ConfigError) {
    let (path, result) = server_evaluate(name, source);
    match result {
        Ok(_) => panic!("{name} loaded"),
        Err(error) => (path, error),
    }
}

fn assert_failure_at(error: &ConfigError, path: &std::path::Path, line: u32, mentions: &str) {
    assert_error_at(error, path, line, mentions);
    assert!(
        error
            .to_string()
            .starts_with(&format!("{}:{line}: ", path.display()))
    );
}

fn direct(name: &str) -> Keys {
    ("root", Chord::Key(key(name)))
}

fn prefixed(name: &str) -> Keys {
    ("prefix", Chord::Key(key(name)))
}

fn binding(config: &Config, (table, chord): Keys) -> Option<Binding> {
    config
        .keymap
        .get(table)?
        .iter()
        .find(|(bound, _)| *bound == chord)
        .map(|(_, binding)| *binding)
}

fn action_of(config: &Config, keys: Keys) -> Option<Action> {
    match binding(config, keys)? {
        Binding::Action(action) => Some(action),
        Binding::Callback(_) => None,
    }
}

fn actions(config: &Config) -> Vec<(String, Chord, Action)> {
    config
        .keymap
        .iter()
        .flat_map(|(table, bindings)| {
            bindings
                .iter()
                .filter_map(move |(chord, binding)| match binding {
                    Binding::Action(action) => Some((table.clone(), *chord, *action)),
                    Binding::Callback(_) => None,
                })
        })
        .collect()
}

fn bound_keys(config: &Config) -> Vec<(String, String)> {
    let mut keys: Vec<(String, String)> = Vec::new();
    for table in config.keymap.keys() {
        let entries: Vec<(String, Option<String>)> =
            eval::<Vec<mlua::Table>>(config, &format!("return gband.keymap.list('{table}')"))
                .into_iter()
                .map(|entry| (entry.get("key").unwrap(), entry.get("action").unwrap()))
                .collect();
        keys.extend(
            entries
                .into_iter()
                .map(|(key, action)| (key, action.unwrap_or_else(|| "function".to_owned()))),
        );
    }
    keys
}

fn function_of(config: &Config, keys: Keys) -> CallbackId {
    match binding(config, keys) {
        Some(Binding::Callback(callback)) => callback,
        other => panic!("{keys:?} is bound to {other:?}"),
    }
}

fn binding_count(config: &Config) -> usize {
    config.keymap.values().map(Vec::len).sum()
}

#[test]
fn no_configuration_file_gives_the_defaults() {
    let scratch = Scratch::new("missing");
    let config = scratch.loaded();
    let defaults = gband_lua::defaults(gband_lua::Side::Client);
    assert_eq!(config.options, defaults.options);
    assert_eq!(actions(&config), actions(&defaults));
    assert_eq!(bound_keys(&config), bound_keys(&defaults));
    assert_eq!(config.modes, defaults.modes);
    assert!(config.errors.is_empty());
}

#[test]
fn user_file_replaces_the_defaults() {
    let config = loaded(
        "replace",
        "gband.bind('alt+h', gband.action.focus_column_left)",
    );
    assert_eq!(
        actions(&config),
        [(
            "root".to_owned(),
            Chord::Key(key("alt+h")),
            Action::View(ViewAction::FocusLeft)
        )]
    );
    assert_eq!(config.options, Options::default());
}

#[test]
fn copied_defaults_load_unchanged() {
    let scratch = Scratch::new("copied");
    prepare(&scratch.dir()).unwrap();
    let copy = fs::read_to_string(defaults_file(&scratch.dir(), gband_lua::Side::Client)).unwrap();
    scratch.write(&copy);
    let config = scratch.loaded();
    let defaults = gband_lua::defaults(gband_lua::Side::Client);
    assert_eq!(config.options, defaults.options);
    assert_eq!(actions(&config), actions(&defaults));
    assert_eq!(bound_keys(&config), bound_keys(&defaults));
    assert_eq!(config.modes, defaults.modes);
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    assert_eq!(component_ids(&config), component_ids(&defaults));
}

#[test]
fn defaults_reproduce_the_built_in_behaviour() {
    let config = gband_lua::defaults(gband_lua::Side::Client);
    assert_eq!(
        config.options.prefix,
        Key::new(KeyCode::Char(' '), Modifiers::CTRL)
    );
    let server = gband_lua::defaults(gband_lua::Side::Server);
    assert!(server.errors.is_empty(), "{:?}", server.errors);
    assert_eq!(server.options, Options::default());
    assert_eq!(server.options.layout.default_width, Proportion::ONE_HALF);
    assert_eq!(
        server.options.layout.presets,
        [
            Proportion::ONE_THIRD,
            Proportion::ONE_HALF,
            Proportion::TWO_THIRDS
        ]
    );
    assert_eq!(
        config.options.center_focused_column,
        CenterFocusedColumn::Never
    );
    assert_eq!(config.options, Options::default());
    let char_key = |c| ("prefix", Chord::Key(Key::plain(KeyCode::Char(c))));
    let ctrl_key = |code| ("prefix", Chord::Key(Key::new(code, Modifiers::CTRL)));
    let expected = [
        (char_key('h'), Action::View(ViewAction::FocusLeft)),
        (char_key('l'), Action::View(ViewAction::FocusRight)),
        (char_key('j'), Action::View(ViewAction::FocusDown)),
        (char_key('k'), Action::View(ViewAction::FocusUp)),
        (char_key('u'), Action::View(ViewAction::BandDown)),
        (char_key('i'), Action::View(ViewAction::BandUp)),
        (char_key('c'), Action::View(ViewAction::CenterColumn)),
        (char_key('q'), Action::Session(SessionCommand::CloseWindow)),
        (
            char_key('['),
            Action::Session(SessionCommand::ConsumeOrExpel(Direction::Left)),
        ),
        (
            char_key(']'),
            Action::Session(SessionCommand::ConsumeOrExpel(Direction::Right)),
        ),
        (char_key('r'), Action::Session(SessionCommand::CycleWidth)),
        (
            char_key('f'),
            Action::Session(SessionCommand::ToggleFullWidth),
        ),
        (
            char_key('-'),
            Action::Session(SessionCommand::StepWidth(Step::Shrink)),
        ),
        (
            char_key('='),
            Action::Session(SessionCommand::StepWidth(Step::Grow)),
        ),
        (
            char_key('_'),
            Action::Session(SessionCommand::StepHeight(Step::Shrink)),
        ),
        (
            char_key('+'),
            Action::Session(SessionCommand::StepHeight(Step::Grow)),
        ),
        (char_key('R'), Action::Session(SessionCommand::ResetHeight)),
        (
            char_key('v'),
            Action::Session(SessionCommand::ToggleFloating),
        ),
        (char_key('V'), Action::View(ViewAction::SwitchLayer)),
        (
            ctrl_key(KeyCode::Char('h')),
            Action::Session(SessionCommand::MoveColumn(Direction::Left)),
        ),
        (
            ctrl_key(KeyCode::Char('l')),
            Action::Session(SessionCommand::MoveColumn(Direction::Right)),
        ),
        (
            ctrl_key(KeyCode::Char('j')),
            Action::Session(SessionCommand::MoveWindow(Vertical::Down)),
        ),
        (
            ctrl_key(KeyCode::Char('k')),
            Action::Session(SessionCommand::MoveWindow(Vertical::Up)),
        ),
        (
            ctrl_key(KeyCode::Left),
            Action::Session(SessionCommand::MoveColumn(Direction::Left)),
        ),
        (
            ctrl_key(KeyCode::Right),
            Action::Session(SessionCommand::MoveColumn(Direction::Right)),
        ),
        (
            ctrl_key(KeyCode::Down),
            Action::Session(SessionCommand::MoveWindow(Vertical::Down)),
        ),
        (
            ctrl_key(KeyCode::Up),
            Action::Session(SessionCommand::MoveWindow(Vertical::Up)),
        ),
        (char_key('D'), Action::Client(ClientAction::Detach)),
        (
            ("prefix", Chord::Key(Key::plain(KeyCode::Left))),
            Action::View(ViewAction::FocusLeft),
        ),
        (
            ("prefix", Chord::Key(Key::plain(KeyCode::Right))),
            Action::View(ViewAction::FocusRight),
        ),
        (
            ("prefix", Chord::Key(Key::plain(KeyCode::Down))),
            Action::View(ViewAction::FocusDown),
        ),
        (
            ("prefix", Chord::Key(Key::plain(KeyCode::Up))),
            Action::View(ViewAction::FocusUp),
        ),
    ]
    .map(|((table, chord), action)| (table.to_owned(), chord, action));
    assert_eq!(actions(&config), expected);
    assert_eq!(config.modes.iter().collect::<Vec<_>>(), ["prefix"]);
    let label: String = eval(&config, "return gband.keymap.label('prefix')");
    assert_eq!(label, "navigation");
    let root = || Dispatch::Enter("root".to_owned());
    let functions = [
        (
            prefixed("n"),
            vec![
                Dispatch::Action(Action::Session(SessionCommand::OpenWindow)),
                root(),
            ],
        ),
        (prefixed("escape"), vec![root()]),
        (prefixed("enter"), vec![root()]),
        (
            ("prefix", Chord::Prefix),
            vec![
                Dispatch::Action(Action::Client(ClientAction::SendPrefix)),
                root(),
            ],
        ),
    ];
    for (keys, dispatched) in functions {
        config.runtime.set_active_table("prefix");
        let outcome = config.runtime.call(function_of(&config, keys));
        clean(&outcome);
        assert_eq!(outcome.dispatched, dispatched, "{keys:?}");
    }
    assert_eq!(
        component_ids(&config),
        ["band", "hints", "mode", "position"]
    );
}

fn component_ids(config: &Config) -> Vec<String> {
    eval(
        config,
        "local ids = {} for _, c in ipairs(gband.ui.statusline.list()) do ids[#ids + 1] = c.id end return ids",
    )
}

#[test]
fn every_default_binding_is_described() {
    let config = gband_lua::defaults(gband_lua::Side::Client);
    let undescribed: Vec<String> = eval(
        &config,
        "local descs = {}
         for _, action in ipairs(gband.action.list()) do descs[action.name] = action.desc end
         local functions = {
           n = 'open a window',
           escape = 'interactive mode',
           enter = 'interactive mode',
           prefix = 'send the prefix key',
         }
         local wrong = {}
         for _, entry in ipairs(gband.keymap.list('prefix')) do
           local expected = entry.action and descs[entry.action] or functions[entry.key]
           if entry.desc == nil or entry.desc ~= expected then
             wrong[#wrong + 1] = entry.key
           end
         end
         return wrong",
    );
    assert!(undescribed.is_empty(), "{undescribed:?}");
    let count: usize = eval(&config, "return #gband.keymap.list('prefix')");
    assert_eq!(count, 37);
}

#[test]
fn key_list_set_up_by_the_defaults() {
    let config = gband_lua::defaults(gband_lua::Side::Client);
    let keys: Vec<String> = eval(
        &config,
        "local keys = {} for _, entry in ipairs(gband.keymap.list('prefix')) do keys[#keys + 1] = entry.key end return keys",
    );
    let position = |key: &str| keys.iter().position(|bound| bound == key).unwrap();
    assert!(position("R") < position("?"));
    assert!(position("?") < position("D"));
    assert!(position("ctrl+up") < position("?"));
    assert!(matches!(
        binding(&config, prefixed("?")),
        Some(Binding::Callback(_))
    ));
    let action: String = eval(
        &config,
        "for _, entry in ipairs(gband.keymap.list('prefix')) do if entry.key == '?' then return entry.action end end",
    );
    assert_eq!(action, "keylist.open");
}

#[test]
fn every_action_is_named() {
    let config = gband_lua::defaults(gband_lua::Side::Client);
    let mut names: Vec<String> = eval(
        &config,
        "local names = {} for _, action in ipairs(gband.action.list()) do names[#names + 1] = action.name end return names",
    );
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
    names.sort();
    let mut expected = [
        "focus_column_left",
        "focus_column_right",
        "focus_window_down",
        "focus_window_up",
        "focus_band_down",
        "focus_band_up",
        "center_column",
        "switch_focus_floating_tiled",
        "open_window",
        "close_window",
        "consume_or_expel_left",
        "consume_or_expel_right",
        "move_column_left",
        "move_column_right",
        "move_window_down",
        "move_window_up",
        "toggle_window_floating",
        "cycle_column_width",
        "toggle_full_width",
        "grow_column_width",
        "shrink_column_width",
        "grow_window_height",
        "shrink_window_height",
        "reset_window_height",
        "detach",
        "send_prefix",
        "keylist.open",
    ];
    expected.sort();
    assert_eq!(names, expected);
    assert_eq!(ACTIONS.len() + 1, expected.len());
}

#[test]
fn center_column_is_a_client_action_on_the_server() {
    let (path, error) = server_failure("center", "\nlocal _ = gband.action.center_column");
    assert_failure_at(&error, &path, 2, "center_column");
    assert!(
        error
            .message
            .contains("`gband.action.center_column` is a client action; this is the server"),
        "{error}"
    );
}

#[test]
fn built_in_descriptions() {
    let config = gband_lua::defaults(gband_lua::Side::Client);
    let desc: String = eval(
        &config,
        "for _, action in ipairs(gband.action.list()) do if action.name == 'cycle_column_width' then return action.desc end end",
    );
    assert_eq!(desc, "cycle the width of the window's column");
}

#[test]
fn name_and_action_agree() {
    let config = loaded(
        "agree",
        "gband.bind('alt+r', gband.action.cycle_column_width)",
    );
    assert_eq!(
        action_of(&config, direct("alt+r")),
        Some(Action::Session(SessionCommand::CycleWidth))
    );
}

#[test]
fn toggle_floating_by_name() {
    let config = loaded(
        "toggle-floating",
        "gband.bind('alt+v', gband.action.toggle_window_floating)\ngband.bind('alt+s', gband.action.switch_focus_floating_tiled)",
    );
    assert_eq!(
        action_of(&config, direct("alt+v")),
        Some(Action::Session(SessionCommand::ToggleFloating))
    );
    assert_eq!(
        action_of(&config, direct("alt+s")),
        Some(Action::View(ViewAction::SwitchLayer))
    );
}

#[test]
fn partial_update() {
    let config = server_loaded("partial", "gband.set { default_column_width = 1/3 }");
    assert_eq!(config.options.layout.default_width, Proportion::ONE_THIRD);
    assert_eq!(
        config.options.layout.presets,
        Options::default().layout.presets
    );
}

#[test]
fn later_call_wins() {
    let config = server_loaded(
        "later",
        "gband.set { default_column_width = 1/3 }\ngband.set { default_column_width = 2/3 }",
    );
    assert_eq!(config.options.layout.default_width, Proportion::TWO_THIRDS);
}

#[test]
fn presets_are_sorted() {
    let config = server_loaded("sorted", "gband.set { width_presets = { 2/3, 1/4, 2/3 } }");
    assert_eq!(
        config.options.layout.presets,
        [Proportion::new(1, 4), Proportion::TWO_THIRDS]
    );
}

#[test]
fn decimal_width() {
    let config = server_loaded("decimal", "gband.set { default_column_width = 0.35 }");
    assert_eq!(config.options.layout.default_width, Proportion::new(7, 20));
}

#[test]
fn unknown_option() {
    let (path, error) = failure("unknown", "\n\ngband.set { colum_width = 1/2 }");
    assert_failure_at(&error, &path, 3, "colum_width");
}

#[test]
fn wrong_type() {
    let (path, error) = server_failure("type", "\ngband.set { width_presets = \"1/2\" }");
    assert_failure_at(&error, &path, 2, "width_presets");
    assert!(error.message.contains("invalid value"), "{error}");
}

#[test]
fn server_option_in_the_client_file() {
    let (path, error) = failure(
        "server-option",
        "\ngband.set { default_column_width = 1/3 }",
    );
    assert_failure_at(&error, &path, 2, "default_column_width");
    assert!(error.message.contains("server"), "{error}");
}

#[test]
fn client_option_in_the_server_file() {
    let (path, error) = server_failure("client-option", "gband.set { prefix = 'ctrl+b' }");
    assert_failure_at(&error, &path, 1, "prefix");
    assert!(error.message.contains("client"), "{error}");
}

#[test]
fn unknown_camera_policy() {
    let (path, error) = failure(
        "policy",
        "gband.set { center_focused_column = 'sometimes' }",
    );
    assert_failure_at(&error, &path, 1, "center_focused_column");
}

#[test]
fn out_of_range_width() {
    let (path, error) = server_failure("range", "gband.set { default_column_width = 0 }");
    assert_failure_at(&error, &path, 1, "default_column_width");
}

#[test]
fn syntax_error() {
    let (path, error) = failure("syntax", "gband.set {}\n\n\nlocal = 3\n");
    assert_failure_at(&error, &path, 4, "");
}

#[test]
fn runtime_error() {
    let (path, error) = failure("runtime", "local x = 1\nerror('boom')\n");
    assert_failure_at(&error, &path, 2, "boom");
}

#[test]
fn unknown_key() {
    let (path, error) = failure(
        "hyper",
        "\n\n\n\ngband.bind('alt+hyper', gband.action.detach)",
    );
    assert_failure_at(&error, &path, 5, "alt+hyper");
}

#[test]
fn direct_binding() {
    let config = loaded(
        "direct",
        "gband.bind('alt+h', gband.action.focus_column_left)",
    );
    assert_eq!(
        action_of(&config, direct("alt+h")),
        Some(Action::View(ViewAction::FocusLeft))
    );
}

#[test]
fn default_prefix() {
    let config = loaded(
        "default-prefix",
        "gband.bind('prefix q', gband.action.close_window)",
    );
    assert_eq!(config.options.prefix, key("ctrl+space"));
    assert_eq!(
        actions(&config),
        [(
            "prefix".to_owned(),
            Chord::Key(key("q")),
            Action::Session(SessionCommand::CloseWindow)
        )]
    );
}

#[test]
fn override_a_default() {
    let config = loaded(
        "override",
        &format!("{DEFAULTS}\ngband.bind('prefix q', gband.action.detach)"),
    );
    assert_eq!(config.options.prefix, key("ctrl+space"));
    assert_eq!(
        action_of(&config, prefixed("q")),
        Some(Action::Client(ClientAction::Detach))
    );
    assert!(
        actions(&config)
            .iter()
            .all(|(_, _, action)| *action != Action::Session(SessionCommand::CloseWindow))
    );
}

#[test]
fn unbind_a_default() {
    let config = loaded(
        "unbind",
        &format!("{DEFAULTS}\ngband.unbind('prefix q')\ngband.unbind('alt+z')"),
    );
    assert_eq!(config.options.prefix, key("ctrl+space"));
    assert!(binding(&config, prefixed("q")).is_none());
    assert_eq!(
        binding_count(&config),
        binding_count(&gband_lua::defaults(gband_lua::Side::Client)) - 1
    );
}

#[test]
fn prefix_changed_after_binding() {
    let config = loaded(
        "prefix",
        "gband.bind('prefix h', gband.action.focus_column_left)\ngband.set { prefix = 'ctrl+b' }",
    );
    assert_eq!(config.options.prefix, key("ctrl+b"));
    assert_eq!(
        actions(&config),
        [(
            "prefix".to_owned(),
            Chord::Key(key("h")),
            Action::View(ViewAction::FocusLeft)
        )]
    );
}

#[test]
fn direct_binding_of_the_prefix_key() {
    let (path, error) = failure(
        "prefix-direct",
        "\n\n\ngband.bind('ctrl+space', gband.action.detach)\n",
    );
    assert_failure_at(&error, &path, 4, "prefix");
}

#[test]
fn direct_binding_of_a_later_prefix_key() {
    let (path, error) = failure(
        "prefix-later",
        "gband.bind('ctrl+b', gband.action.detach)\ngband.set { prefix = 'ctrl+b' }\n",
    );
    assert_failure_at(&error, &path, 1, "prefix");
}

#[test]
fn key_chain_too_long() {
    let (path, error) = failure("chain", "gband.bind('prefix w q', gband.action.detach)");
    assert_failure_at(&error, &path, 1, "prefix w q");
}

#[test]
fn second_key_needs_the_prefix() {
    let (path, error) = failure("pair", "gband.bind('ctrl+b h', gband.action.detach)");
    assert_failure_at(&error, &path, 1, "ctrl+b h");
}

#[test]
fn not_an_action() {
    let (path, error) = failure("string", "gband.bind('alt+h', 'focus_column_left')");
    assert_failure_at(&error, &path, 1, "alt+h");
}

#[test]
fn action_during_evaluation() {
    let (path, error) = failure("eval-action", "\n\n\n\n\n\ngband.action.close_window()");
    assert_failure_at(&error, &path, 7, "binding function");
}

#[test]
fn spawn_during_evaluation() {
    let (path, error) = failure("eval-spawn", "gband.spawn {}");
    assert_failure_at(&error, &path, 1, "binding function");
}

#[test]
fn action_called_from_a_function() {
    let config = loaded(
        "function",
        "gband.bind('alt+w', function()\n  gband.action.focus_column_right()\n  gband.action.focus_column_right()\nend)",
    );
    let outcome = config.runtime.call(function_of(&config, direct("alt+w")));
    clean(&outcome);
    assert_eq!(
        outcome.dispatched,
        [
            Dispatch::Action(Action::View(ViewAction::FocusRight)),
            Dispatch::Action(Action::View(ViewAction::FocusRight)),
        ]
    );
}

#[test]
fn error_in_a_binding_function() {
    let (path, result) = evaluate(
        "function-error",
        "gband.bind('alt+e', function()\n  gband.action.focus_column_left()\n\n\n\n\n\n\n  error('broken')\nend)",
    );
    let config = result.unwrap();
    let outcome = config.runtime.call(function_of(&config, direct("alt+e")));
    assert_eq!(
        outcome.dispatched,
        [Dispatch::Action(Action::View(ViewAction::FocusLeft))]
    );
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert_failure_at(error, &path, 9, "broken");
}

#[test]
fn spawn_programs() {
    let config = loaded(
        "spawn",
        "gband.bind('alt+n', function() gband.spawn({ cmd = 'fish' }) end)\n\
         gband.bind('alt+t', function() gband.spawn({ cmd = { 'htop', '-d', '10' } }) end)\n\
         gband.bind('alt+s', function() gband.spawn({}) end)",
    );
    let spawned = |name: &str| {
        let outcome = config.runtime.call(function_of(&config, direct(name)));
        clean(&outcome);
        outcome.dispatched
    };
    assert_eq!(
        spawned("alt+n"),
        [Dispatch::Spawn(Some(Program::CommandLine(
            "fish".to_owned()
        )))]
    );
    assert_eq!(
        spawned("alt+t"),
        [Dispatch::Spawn(Some(Program::Argv(vec![
            "htop".to_owned(),
            "-d".to_owned(),
            "10".to_owned()
        ])))]
    );
    assert_eq!(spawned("alt+s"), [Dispatch::Spawn(None)]);
}

#[test]
fn invalid_spawn_requests_fail() {
    for (index, request) in [
        "{ cmd = 5 }",
        "{ cmd = {} }",
        "{ cmd = { 'a', 5 } }",
        "{ cmd = 'a', cwd = '/' }",
        "'fish'",
    ]
    .into_iter()
    .enumerate()
    {
        let (path, result) = evaluate(
            &format!("bad-spawn-{index}"),
            &format!("gband.bind('alt+n', function()\n  gband.spawn({request})\nend)"),
        );
        let config = result.unwrap();
        let outcome = config.runtime.call(function_of(&config, direct("alt+n")));
        assert!(outcome.dispatched.is_empty(), "{request}");
        assert_failure_at(&outcome.errors[0], &path, 2, "");
    }
}

#[test]
fn options_cannot_change_from_a_binding_function() {
    let (path, result) = evaluate(
        "late-set",
        "gband.bind('alt+n', function()\n  gband.set { prefix = 'ctrl+b' }\nend)",
    );
    let config = result.unwrap();
    let outcome = config.runtime.call(function_of(&config, direct("alt+n")));
    assert_failure_at(
        &outcome.errors[0],
        &path,
        2,
        "while the configuration loads",
    );
}

#[test]
fn registered_action_bound_to_a_key() {
    let config = loaded(
        "registered",
        "gband.action.register('twice', function()\n  gband.action.focus_column_right()\n  gband.action.focus_column_right()\nend)\ngband.bind('alt+t', gband.action.twice)",
    );
    let outcome = config.runtime.call(function_of(&config, direct("alt+t")));
    clean(&outcome);
    assert_eq!(
        outcome.dispatched,
        [
            Dispatch::Action(Action::View(ViewAction::FocusRight)),
            Dispatch::Action(Action::View(ViewAction::FocusRight)),
        ]
    );
}

#[test]
fn user_names_are_not_namespaced() {
    let config = loaded(
        "user-names",
        "gband.action.register('greet', function() end, { desc = 'Greet' })",
    );
    let kind: String = eval(&config, "return type(gband.action.greet)");
    assert_eq!(kind, "userdata");
    let desc: String = eval(
        &config,
        "for _, a in ipairs(gband.action.list()) do if a.name == 'greet' then return a.desc end end",
    );
    assert_eq!(desc, "Greet");
}

#[test]
fn built_in_name_taken() {
    let (path, error) = failure(
        "taken",
        "\n\ngband.action.register('detach', function() end)",
    );
    assert_failure_at(&error, &path, 3, "detach");
    for name in ["register", "list"] {
        let (path, error) = failure(
            &format!("reserved-{name}"),
            &format!("gband.action.register('{name}', function() end)"),
        );
        assert_failure_at(&error, &path, 1, name);
    }
    let (path, error) = failure(
        "duplicate-action",
        "gband.action.register('a', function() end)\ngband.action.register('a', function() end)",
    );
    assert_failure_at(&error, &path, 2, "`a`");
}

#[test]
fn registering_after_the_load_fails() {
    let (path, result) = evaluate(
        "late-register",
        "gband.bind('alt+n', function()\n  gband.action.register('x', function() end)\nend)",
    );
    let config = result.unwrap();
    let outcome = config.runtime.call(function_of(&config, direct("alt+n")));
    assert_failure_at(
        &outcome.errors[0],
        &path,
        2,
        "while the configuration loads",
    );
}

#[test]
fn registered_action_cannot_be_called_while_loading() {
    let (path, error) = failure(
        "eval-registered",
        "gband.action.register('x', function() end)\ngband.action.x()",
    );
    assert_failure_at(&error, &path, 2, "binding function");
}
