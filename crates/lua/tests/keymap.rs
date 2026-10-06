mod common;

use common::*;
use gband_core::action::{Action, ClientAction, SessionCommand};
use gband_core::input::{Modifiers, MouseButton, MouseKey};
use gband_core::view::ViewAction;
use gband_lua::{Binding, Chord, Dispatch};

#[test]
fn keymap_binding_with_a_description() {
    let scratch = Scratch::new("described");
    scratch.write(
        "gband.keymap.set('prefix', 'g', gband.action.focus_column_left, { desc = 'left' })
entry = gband.keymap.list('prefix')[1]",
    );
    let config = scratch.loaded();
    assert_eq!(
        config.keymap["prefix"],
        [(
            Chord::Key(key("g")),
            Binding::Action(Action::View(ViewAction::FocusLeft))
        )]
    );
    let entry: Vec<String> = eval(&config, "return { entry.key, entry.desc, entry.action }");
    assert_eq!(entry, ["g", "left", "focus_column_left"]);
}

#[test]
fn old_and_new_forms_agree() {
    let scratch = Scratch::new("forms");
    scratch.write("gband.bind('prefix x', gband.action.detach)\ngband.keymap.del('prefix', 'x')");
    let config = scratch.loaded();
    assert!(config.keymap.is_empty(), "{:?}", config.keymap);
    let scratch = Scratch::new("forms-reverse");
    scratch.write(
        "gband.keymap.set('root', 'alt+x', gband.action.detach)\ngband.keymap.set('prefix', 'prefix', gband.action.send_prefix)\ngband.unbind('alt+x')",
    );
    let config = scratch.loaded();
    assert_eq!(
        config.keymap["prefix"],
        [(
            Chord::Prefix,
            Binding::Action(Action::Client(ClientAction::SendPrefix))
        )]
    );
    assert!(!config.keymap.contains_key("root"));
}

#[test]
fn keymap_binds_mouse_names() {
    let scratch = Scratch::new("mouse-keymap");
    scratch.write(
        "gband.keymap.set('prefix', 'alt+rightmouse', gband.action.detach)
gband.keymap.set('prefix', 'MiddleMouse', gband.action.detach)
gband.keymap.del('prefix', 'middlemouse')
entry = gband.keymap.list('prefix')[1]",
    );
    let config = scratch.loaded();
    assert_eq!(
        config.keymap["prefix"],
        [(
            Chord::Mouse(MouseKey::new(MouseButton::Right, Modifiers::ALT)),
            Binding::Action(Action::Client(ClientAction::Detach))
        )]
    );
    let key: String = eval(&config, "return entry.key");
    assert_eq!(key, "alt+rightmouse");
}

#[test]
fn prefix_in_the_root_table() {
    let scratch = Scratch::new("root-prefix");
    let path = scratch.write("\ngband.keymap.set('root', 'prefix', gband.action.detach)");
    let error = scratch.load().err().unwrap();
    assert_error_at(&error, &path, 2, "prefix");
}

#[test]
fn invalid_keymap_calls() {
    for (index, call) in [
        "gband.keymap.set(5, 'x', gband.action.detach)",
        "gband.keymap.set('', 'x', gband.action.detach)",
        "gband.keymap.set('move', 'hyper+x', gband.action.detach)",
        "gband.keymap.set('move', 'x', 'detach')",
        "gband.keymap.set('move', 'x', gband.action.detach, { desc = 5 })",
        "gband.keymap.del(nil, 'x')",
        "gband.keymap.list(false)",
        "gband.keymap.enter('move')",
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
fn binding_after_the_load() {
    let scratch = Scratch::new("late");
    let path = scratch.write(
        "gband.bind('alt+n', function()\n\n\n\n\n  gband.keymap.set('root', 'alt+x', function() end)\nend)",
    );
    let config = scratch.loaded();
    let Binding::Callback(callback) = config.keymap["root"][0].1 else {
        panic!("not a function");
    };
    let outcome = config.runtime.call(callback);
    assert_error_at(
        &outcome.errors[0],
        &path,
        6,
        "while the configuration loads",
    );
}

#[test]
fn list_order_and_replacement() {
    let scratch = Scratch::new("list");
    scratch.write(
        "gband.keymap.set('move', 'h', gband.action.focus_column_left, { desc = 'left' })
gband.keymap.set('move', 'l', gband.action.focus_column_right, { desc = 'right' })
gband.keymap.set('move', 'j', function() end)
gband.keymap.set('move', 'h', gband.action.focus_window_down, { desc = 'down' })",
    );
    let config = scratch.loaded();
    let listed: Vec<String> = eval(
        &config,
        "local out = {}
         for _, e in ipairs(gband.keymap.list('move')) do
           out[#out + 1] = e.key .. ':' .. tostring(e.desc) .. ':' .. tostring(e.action)
         end
         return out",
    );
    assert_eq!(
        listed,
        [
            "l:right:focus_column_right",
            "j:nil:nil",
            "h:down:focus_window_down"
        ]
    );
    let empty: usize = eval(&config, "return #gband.keymap.list('absent')");
    assert_eq!(empty, 0);
}

#[test]
fn registered_action_lists_its_full_name() {
    let scratch = Scratch::new("registered");
    scratch.write("");
    scratch.client_plugin("hello",
        "gband.action.register('greet', function() end)\ngband.keymap.set('prefix', 'g', gband.action['hello.greet'], { desc = 'Greet' })",
    );
    let config = scratch.loaded();
    let action: String = eval(&config, "return gband.keymap.list('prefix')[1].action");
    assert_eq!(action, "hello.greet");
}

#[test]
fn current_table_and_enter() {
    let scratch = Scratch::new("enter");
    scratch.write(
        "loading_table = gband.keymap.current_table()
gband.keymap.set('move', 'l', gband.action.focus_column_right)
gband.keymap.set('prefix', 'm', function()
  seen = gband.keymap.current_table()
  gband.keymap.enter('move')
end)
gband.keymap.set('prefix', 'e', function() gband.keymap.enter('empty') end)",
    );
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "loading_table"), "root");
    let prefix = &config.keymap["prefix"];
    let (Binding::Callback(enter), Binding::Callback(empty)) = (prefix[0].1, prefix[1].1) else {
        panic!("{prefix:?}");
    };
    config.runtime.set_active_table("prefix");
    let outcome = config.runtime.call(enter);
    clean(&outcome);
    assert_eq!(global::<String>(&config, "seen"), "prefix");
    assert_eq!(outcome.dispatched, [Dispatch::Enter("move".to_owned())]);
    let outcome = config.runtime.call(empty);
    assert!(outcome.dispatched.is_empty());
    assert!(
        outcome.errors[0].message.contains("empty"),
        "{:?}",
        outcome.errors
    );
}

#[test]
fn plugin_root_binding_of_the_prefix_key_is_dropped() {
    let scratch = Scratch::new("plugin-prefix");
    scratch.client_plugin("hello",
        "gband.keymap.set('root', 'ctrl+space', gband.action.detach)\ngband.keymap.set('root', 'alt+d', gband.action.detach)",
    );
    let config = scratch.loaded();
    assert_eq!(config.keymap["root"].len(), 1);
    assert_eq!(config.errors[0].plugin.as_deref(), Some("hello"));
}

#[test]
fn modes_and_labels() {
    let scratch = Scratch::new("modes");
    scratch.write(
        "gband.keymap.mode('prefix', { label = 'nav' })
gband.keymap.mode('prefix', { label = 'navigation' })
gband.keymap.mode('resize')
gband.keymap.set('move', 'h', gband.action.focus_column_left)
labels = { gband.keymap.label('prefix'), gband.keymap.label('resize'), gband.keymap.label('move') }",
    );
    let config = scratch.loaded();
    assert_eq!(
        global::<Vec<String>>(&config, "labels"),
        ["navigation", "resize", "move"]
    );
    assert_eq!(
        config.modes.iter().collect::<Vec<_>>(),
        ["prefix", "resize"]
    );
}

#[test]
fn a_mode_keeps_its_declaration_across_set_and_del() {
    let scratch = Scratch::new("mode-set-del");
    scratch.write(
        "gband.keymap.mode('resize', { label = 'RESIZE' })
gband.keymap.set('resize', '=', gband.action.grow_column_width)
gband.keymap.del('resize', '=')
gband.keymap.set('resize', '-', gband.action.shrink_column_width)",
    );
    let config = scratch.loaded();
    assert!(config.modes.contains("resize"));
    assert_eq!(
        eval::<String>(&config, "return gband.keymap.label('resize')"),
        "RESIZE"
    );
}

#[test]
fn invalid_mode_calls() {
    for (index, call) in [
        "gband.keymap.mode('root')",
        "gband.keymap.mode(5)",
        "gband.keymap.mode('')",
        "gband.keymap.mode('prefix', 3)",
        "gband.keymap.mode('prefix', { label = 3 })",
        "gband.keymap.mode('prefix', { label = '' })",
        "gband.keymap.label(nil)",
        "gband.keymap.run('prefix', 'h')",
    ]
    .into_iter()
    .enumerate()
    {
        let scratch = Scratch::new(&format!("invalid-mode-{index}"));
        let path = scratch.write(&format!("\n{call}"));
        let error = scratch
            .load()
            .err()
            .unwrap_or_else(|| panic!("{call} loaded"));
        assert_error_at(&error, &path, 2, "");
    }
}

#[test]
fn mode_declared_after_the_load() {
    let scratch = Scratch::new("late-mode");
    let path = scratch
        .write("gband.bind('alt+n', function()\n\n\n\n\n\n  gband.keymap.mode('move')\nend)");
    let config = scratch.loaded();
    let Binding::Callback(callback) = config.keymap["root"][0].1 else {
        panic!("not a function");
    };
    let outcome = config.runtime.call(callback);
    assert_error_at(
        &outcome.errors[0],
        &path,
        7,
        "while the configuration loads",
    );
}

#[test]
fn label_in_a_callback() {
    let scratch = Scratch::new("label-callback");
    scratch.write(&format!(
        "{JOB}gband.keymap.mode('prefix', {{ label = 'navigation' }})"
    ));
    let config = scratch.loaded();
    clean(&run_job(&config, "seen = gband.keymap.label('prefix')"));
    assert_eq!(global::<String>(&config, "seen"), "navigation");
}

#[test]
fn enter_root_with_no_root_binding() {
    let scratch = Scratch::new("enter-root");
    scratch.write(
        "gband.keymap.mode('prefix')
gband.keymap.set('prefix', 'escape', function() gband.keymap.enter('root') end)",
    );
    let config = scratch.loaded();
    assert!(!config.keymap.contains_key("root"));
    let Binding::Callback(callback) = config.keymap["prefix"][0].1 else {
        panic!("not a function");
    };
    config.runtime.set_active_table("prefix");
    let outcome = config.runtime.call(callback);
    clean(&outcome);
    assert_eq!(outcome.dispatched, [Dispatch::Enter("root".to_owned())]);
}

#[test]
fn run_an_action_binding() {
    let scratch = Scratch::new("run-action");
    scratch.write(&format!(
        "{JOB}gband.keymap.set('prefix', 'h', gband.action.focus_column_left)"
    ));
    let config = scratch.loaded();
    let outcome = run_job(&config, "ran = gband.keymap.run('prefix', 'h')");
    clean(&outcome);
    assert!(global::<bool>(&config, "ran"));
    assert_eq!(
        outcome.dispatched,
        [Dispatch::Action(Action::View(ViewAction::FocusLeft))]
    );
}

#[test]
fn run_a_function_binding() {
    let scratch = Scratch::new("run-function");
    scratch.write(&format!(
        "{JOB}calls = 0
gband.keymap.set('prefix', 'x', function() calls = calls + 1 end)
gband.keymap.set('prefix', 'prefix', function() prefixed = true end)"
    ));
    let config = scratch.loaded();
    let outcome = run_job(
        &config,
        "ran = gband.keymap.run('prefix', 'x')\ngband.keymap.run('prefix', 'prefix')",
    );
    clean(&outcome);
    assert!(global::<bool>(&config, "ran"));
    assert_eq!(global::<i64>(&config, "calls"), 1);
    assert!(global::<bool>(&config, "prefixed"));
}

#[test]
fn run_a_registered_action_under_its_plugin() {
    let scratch = Scratch::new("run-registered");
    scratch.write(JOB);
    scratch.client_plugin(
        "hello",
        "gband.action.register('greet', function() error('boom') end)
gband.keymap.set('prefix', 'g', gband.action['hello.greet'])",
    );
    let config = scratch.loaded();
    let outcome = run_job(&config, "ran = gband.keymap.run('prefix', 'g')");
    assert!(global::<bool>(&config, "ran"));
    assert_eq!(outcome.errors[0].plugin.as_deref(), Some("hello"));
}

#[test]
fn run_an_unbound_key() {
    let scratch = Scratch::new("run-unbound");
    scratch.write(JOB);
    let config = scratch.loaded();
    let outcome = run_job(&config, "ran = gband.keymap.run('prefix', 'z')");
    clean(&outcome);
    assert!(!global::<bool>(&config, "ran"));
    assert!(outcome.dispatched.is_empty());
}

#[test]
fn invalid_run_calls() {
    let scratch = Scratch::new("run-invalid");
    scratch.write(JOB);
    let config = scratch.loaded();
    for call in [
        "gband.keymap.run('', 'h')",
        "gband.keymap.run(3, 'h')",
        "gband.keymap.run('prefix', 'hyper+x')",
        "gband.keymap.run('prefix', 5)",
    ] {
        let outcome = run_job(&config, call);
        assert_eq!(outcome.errors.len(), 1, "{call}");
    }
}

#[test]
fn run_queues_in_the_callers_order() {
    let scratch = Scratch::new("run-order");
    scratch.write(&format!(
        "{JOB}gband.keymap.set('prefix', 'n', function()
  gband.action.open_window()
  gband.keymap.enter('root')
end)"
    ));
    let config = scratch.loaded();
    let outcome = run_job(
        &config,
        "gband.action.detach()\ngband.keymap.run('prefix', 'n')",
    );
    clean(&outcome);
    assert_eq!(
        outcome.dispatched,
        [
            Dispatch::Action(Action::Client(ClientAction::Detach)),
            Dispatch::Action(Action::Session(SessionCommand::OpenWindow)),
            Dispatch::Enter("root".to_owned()),
        ]
    );
}
