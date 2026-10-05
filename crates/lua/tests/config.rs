use std::fs;
use std::path::{Path, PathBuf};

use gband_core::action::{Action, ClientAction, SessionCommand};
use gband_core::input::{Key, KeyCode, Modifiers};
use gband_core::layout::{Direction, Program, Proportion, Step};
use gband_core::view::{CenterFocusedColumn, ViewAction};
use gband_lua::keys::parse_key;
use gband_lua::{ACTIONS, Binding, Chord, Config, ConfigError, Dispatch, Keys, call, load};

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("gband-lua-config-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn init(&self) -> PathBuf {
        self.0.join("init.lua")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn evaluate(name: &str, source: &str) -> (PathBuf, Result<Config, ConfigError>) {
    let scratch = Scratch::new(name);
    let path = scratch.init();
    fs::write(&path, source).unwrap();
    let result = load(&path);
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

fn assert_error_at(error: &ConfigError, path: &Path, line: u32, mentions: &str) {
    assert_eq!(error.location, Some((path.to_path_buf(), line)), "{error}");
    assert!(
        error
            .to_string()
            .starts_with(&format!("{}:{line}: ", path.display()))
    );
    assert!(error.message.contains(mentions), "{error}");
}

fn key(name: &str) -> Key {
    parse_key(name).unwrap()
}

fn prefixed(name: &str) -> Keys {
    Keys::Prefixed(Chord::Key(key(name)))
}

fn binding(config: &Config, keys: Keys) -> Option<&Binding> {
    config
        .bindings
        .iter()
        .find(|(bound, _)| *bound == keys)
        .map(|(_, binding)| binding)
}

fn action_of(config: &Config, keys: Keys) -> Option<Action> {
    match binding(config, keys)? {
        Binding::Action(action) => Some(*action),
        Binding::Function(_) => None,
    }
}

fn function_of(config: &Config, keys: Keys) -> &gband_lua::RegistryKey {
    match binding(config, keys) {
        Some(Binding::Function(function)) => function,
        other => panic!("{keys:?} is bound to {other:?}"),
    }
}

#[test]
fn no_configuration_file_gives_the_defaults() {
    let scratch = Scratch::new("missing");
    let config = load(&scratch.init()).unwrap();
    let defaults = gband_lua::defaults();
    assert_eq!(config.options, defaults.options);
    assert_eq!(config.bindings.len(), defaults.bindings.len());
}

#[test]
fn defaults_reproduce_the_built_in_behaviour() {
    let config = gband_lua::defaults();
    assert_eq!(
        config.options.prefix,
        Key::new(KeyCode::Char('a'), Modifiers::CTRL)
    );
    assert_eq!(config.options.layout.default_width, Proportion::ONE_HALF);
    assert_eq!(
        config.options.layout.presets,
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
    let char_key = |c| Keys::Prefixed(Chord::Key(Key::plain(KeyCode::Char(c))));
    let expected = [
        (char_key('h'), Action::View(ViewAction::FocusLeft)),
        (char_key('l'), Action::View(ViewAction::FocusRight)),
        (char_key('j'), Action::View(ViewAction::FocusDown)),
        (char_key('k'), Action::View(ViewAction::FocusUp)),
        (char_key('u'), Action::View(ViewAction::WorkspaceDown)),
        (char_key('i'), Action::View(ViewAction::WorkspaceUp)),
        (
            Keys::Prefixed(Chord::Key(Key::plain(KeyCode::Enter))),
            Action::Session(SessionCommand::OpenPane),
        ),
        (char_key('q'), Action::Session(SessionCommand::ClosePane)),
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
        (char_key('D'), Action::Client(ClientAction::Detach)),
        (
            Keys::Prefixed(Chord::Prefix),
            Action::Client(ClientAction::SendPrefix),
        ),
    ];
    let bound: Vec<(Keys, Action)> = config
        .bindings
        .iter()
        .map(|(keys, binding)| match binding {
            Binding::Action(action) => (*keys, *action),
            Binding::Function(_) => panic!("{keys:?} is bound to a function"),
        })
        .collect();
    assert_eq!(bound, expected);
}

#[test]
fn every_action_has_its_lua_name() {
    let config = gband_lua::defaults();
    let mut names: Vec<String> = config
        .lua
        .load("local names = {} for name in pairs(gband.action) do names[#names + 1] = name end return names")
        .eval()
        .unwrap();
    names.sort();
    let mut expected = [
        "focus_column_left",
        "focus_column_right",
        "focus_pane_down",
        "focus_pane_up",
        "focus_workspace_down",
        "focus_workspace_up",
        "open_pane",
        "close_pane",
        "consume_or_expel_left",
        "consume_or_expel_right",
        "cycle_column_width",
        "toggle_full_width",
        "grow_column_width",
        "shrink_column_width",
        "grow_pane_height",
        "shrink_pane_height",
        "reset_pane_height",
        "detach",
        "send_prefix",
    ];
    expected.sort();
    assert_eq!(names, expected);
    assert_eq!(ACTIONS.len(), expected.len());
}

#[test]
fn name_and_action_agree() {
    let config = loaded(
        "agree",
        "gband.bind('alt+r', gband.action.cycle_column_width)",
    );
    assert_eq!(
        action_of(&config, Keys::Direct(key("alt+r"))),
        Some(Action::Session(SessionCommand::CycleWidth))
    );
}

#[test]
fn partial_update() {
    let config = loaded("partial", "gband.set { default_column_width = 1/3 }");
    assert_eq!(config.options.layout.default_width, Proportion::ONE_THIRD);
    assert_eq!(
        config.options.layout.presets,
        gband_lua::defaults().options.layout.presets
    );
}

#[test]
fn later_call_wins() {
    let config = loaded(
        "later",
        "gband.set { default_column_width = 1/3 }\ngband.set { default_column_width = 2/3 }",
    );
    assert_eq!(config.options.layout.default_width, Proportion::TWO_THIRDS);
}

#[test]
fn presets_are_sorted() {
    let config = loaded("sorted", "gband.set { width_presets = { 2/3, 1/4, 2/3 } }");
    assert_eq!(
        config.options.layout.presets,
        [Proportion::new(1, 4), Proportion::TWO_THIRDS]
    );
}

#[test]
fn decimal_width() {
    let config = loaded("decimal", "gband.set { default_column_width = 0.35 }");
    assert_eq!(config.options.layout.default_width, Proportion::new(7, 20));
}

#[test]
fn unknown_option() {
    let (path, error) = failure("unknown", "\n\ngband.set { colum_width = 1/2 }");
    assert_error_at(&error, &path, 3, "colum_width");
}

#[test]
fn wrong_type() {
    let (path, error) = failure("type", "\ngband.set { width_presets = \"1/2\" }");
    assert_error_at(&error, &path, 2, "width_presets");
}

#[test]
fn unknown_camera_policy() {
    let (path, error) = failure(
        "policy",
        "gband.set { center_focused_column = 'sometimes' }",
    );
    assert_error_at(&error, &path, 1, "center_focused_column");
}

#[test]
fn out_of_range_width() {
    let (path, error) = failure("range", "gband.set { default_column_width = 0 }");
    assert_error_at(&error, &path, 1, "default_column_width");
}

#[test]
fn syntax_error() {
    let (path, error) = failure("syntax", "gband.set {}\n\n\nlocal = 3\n");
    assert_error_at(&error, &path, 4, "");
}

#[test]
fn runtime_error() {
    let (path, error) = failure("runtime", "local x = 1\nerror('boom')\n");
    assert_error_at(&error, &path, 2, "boom");
}

#[test]
fn unknown_key() {
    let (path, error) = failure(
        "hyper",
        "\n\n\n\ngband.bind('alt+hyper', gband.action.detach)",
    );
    assert_error_at(&error, &path, 5, "alt+hyper");
}

#[test]
fn direct_binding() {
    let config = loaded(
        "direct",
        "gband.bind('alt+h', gband.action.focus_column_left)",
    );
    assert_eq!(
        action_of(&config, Keys::Direct(key("alt+h"))),
        Some(Action::View(ViewAction::FocusLeft))
    );
}

#[test]
fn override_a_default() {
    let config = loaded("override", "gband.bind('prefix q', gband.action.detach)");
    assert_eq!(
        action_of(&config, prefixed("q")),
        Some(Action::Client(ClientAction::Detach))
    );
    assert!(config.bindings.iter().all(|(_, binding)| !matches!(
        binding,
        Binding::Action(Action::Session(SessionCommand::ClosePane))
    )));
}

#[test]
fn unbind_a_default() {
    let config = loaded("unbind", "gband.unbind('prefix q')\ngband.unbind('alt+z')");
    assert!(binding(&config, prefixed("q")).is_none());
    assert_eq!(
        config.bindings.len(),
        gband_lua::defaults().bindings.len() - 1
    );
}

#[test]
fn prefix_changed_after_binding() {
    let config = loaded("prefix", "gband.set { prefix = 'ctrl+b' }");
    assert_eq!(config.options.prefix, key("ctrl+b"));
    assert_eq!(
        action_of(&config, prefixed("h")),
        Some(Action::View(ViewAction::FocusLeft))
    );
    assert_eq!(
        action_of(&config, Keys::Prefixed(Chord::Prefix)),
        Some(Action::Client(ClientAction::SendPrefix))
    );
}

#[test]
fn direct_binding_of_the_prefix_key() {
    let (path, error) = failure(
        "prefix-direct",
        "\n\n\ngband.bind('ctrl+a', gband.action.detach)\n",
    );
    assert_error_at(&error, &path, 4, "prefix");
}

#[test]
fn direct_binding_of_a_later_prefix_key() {
    let (path, error) = failure(
        "prefix-later",
        "gband.bind('ctrl+b', gband.action.detach)\ngband.set { prefix = 'ctrl+b' }\n",
    );
    assert_error_at(&error, &path, 1, "prefix");
}

#[test]
fn key_chain_too_long() {
    let (path, error) = failure("chain", "gband.bind('prefix w q', gband.action.detach)");
    assert_error_at(&error, &path, 1, "prefix w q");
}

#[test]
fn second_key_needs_the_prefix() {
    let (path, error) = failure("pair", "gband.bind('ctrl+b h', gband.action.detach)");
    assert_error_at(&error, &path, 1, "ctrl+b h");
}

#[test]
fn not_an_action() {
    let (path, error) = failure("string", "gband.bind('alt+h', 'focus_column_left')");
    assert_error_at(&error, &path, 1, "alt+h");
}

#[test]
fn action_during_evaluation() {
    let (path, error) = failure("eval-action", "\n\n\n\n\n\ngband.action.close_pane()");
    assert_error_at(&error, &path, 7, "binding function");
}

#[test]
fn spawn_during_evaluation() {
    let (path, error) = failure("eval-spawn", "gband.spawn {}");
    assert_error_at(&error, &path, 1, "binding function");
}

#[test]
fn action_called_from_a_function() {
    let config = loaded(
        "function",
        "gband.bind('alt+w', function()\n  gband.action.focus_column_right()\n  gband.action.focus_column_right()\nend)",
    );
    let (dispatched, error) = call(
        &config.lua,
        function_of(&config, Keys::Direct(key("alt+w"))),
    );
    assert_eq!(error, None);
    assert_eq!(
        dispatched,
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
    let (dispatched, error) = call(
        &config.lua,
        function_of(&config, Keys::Direct(key("alt+e"))),
    );
    assert_eq!(
        dispatched,
        [Dispatch::Action(Action::View(ViewAction::FocusLeft))]
    );
    assert_error_at(&error.unwrap(), &path, 9, "broken");
}

#[test]
fn spawn_programs() {
    let config = loaded(
        "spawn",
        "gband.bind('alt+n', function() gband.spawn({ cmd = 'fish' }) end)\n\
         gband.bind('alt+t', function() gband.spawn({ cmd = { 'htop', '-d', '10' } }) end)\n\
         gband.bind('alt+s', function() gband.spawn({}) end)",
    );
    let spawned = |name: &str| call(&config.lua, function_of(&config, Keys::Direct(key(name))));
    assert_eq!(
        spawned("alt+n"),
        (
            vec![Dispatch::Spawn(Some(Program::CommandLine(
                "fish".to_owned()
            )))],
            None
        )
    );
    assert_eq!(
        spawned("alt+t"),
        (
            vec![Dispatch::Spawn(Some(Program::Argv(vec![
                "htop".to_owned(),
                "-d".to_owned(),
                "10".to_owned()
            ])))],
            None
        )
    );
    assert_eq!(spawned("alt+s"), (vec![Dispatch::Spawn(None)], None));
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
        let (dispatched, error) = call(
            &config.lua,
            function_of(&config, Keys::Direct(key("alt+n"))),
        );
        assert!(dispatched.is_empty(), "{request}");
        assert_error_at(&error.unwrap(), &path, 2, "");
    }
}

#[test]
fn options_cannot_change_from_a_binding_function() {
    let (path, result) = evaluate(
        "late-set",
        "gband.bind('alt+n', function()\n  gband.set { prefix = 'ctrl+b' }\nend)",
    );
    let config = result.unwrap();
    let (_, error) = call(
        &config.lua,
        function_of(&config, Keys::Direct(key("alt+n"))),
    );
    assert_error_at(&error.unwrap(), &path, 2, "while the configuration loads");
}
