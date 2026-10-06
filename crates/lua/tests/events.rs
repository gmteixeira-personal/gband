mod common;

use common::*;
use gband_core::action::Action;
use gband_core::layout::{BandId, WindowId};
use gband_core::view::ViewAction;
use gband_lua::{Binding, Config, Dispatch, Event};

fn focus(window: u32, previous: u32) -> Event {
    Event::FocusChanged {
        window: Some(WindowId(window)),
        previous: Some(WindowId(previous)),
    }
}

fn log(config: &Config) -> Vec<String> {
    global(config, "log")
}

#[test]
fn handler_receives_the_payload() {
    let scratch = Scratch::new("payload");
    scratch.write(
        "log = {}\ngband.on('TerminalResized', function(e) log[#log + 1] = e.cols .. 'x' .. e.rows end)",
    );
    let config = scratch.loaded();
    clean(&config.runtime.emit(&Event::TerminalResized {
        cols: 100,
        rows: 30,
    }));
    assert_eq!(log(&config), ["100x30"]);
}

#[test]
fn every_built_in_payload() {
    let scratch = Scratch::new("payloads");
    scratch.write(
        "log = {}
local function record(e)
  local keys = {}
  for k, v in pairs(e) do keys[#keys + 1] = k .. '=' .. tostring(v) end
  table.sort(keys)
  log[#log + 1] = table.concat(keys, ',')
end
for _, name in ipairs({ 'Attached', 'FocusChanged', 'BandChanged', 'WindowOpened', 'WindowClosed', 'ConfigReloaded', 'KeyTableChanged' }) do
  gband.on(name, record)
end",
    );
    let config = scratch.loaded();
    for event in [
        Event::Attached {
            session: "main".to_owned(),
        },
        Event::FocusChanged {
            window: None,
            previous: Some(WindowId(2)),
        },
        Event::BandChanged {
            band: BandId(2),
            previous: BandId(1),
        },
        Event::WindowOpened {
            window: WindowId(3),
            band: BandId(1),
        },
        Event::WindowClosed {
            window: WindowId(3),
            band: BandId(1),
        },
        Event::ConfigReloaded,
        Event::KeyTableChanged {
            table: "prefix".to_owned(),
            previous: "root".to_owned(),
        },
    ] {
        clean(&config.runtime.emit(&event));
    }
    assert_eq!(
        log(&config),
        [
            "session=main",
            "previous=2",
            "band=2,previous=1",
            "band=1,window=3",
            "band=1,window=3",
            "",
            "previous=root,table=prefix",
        ]
    );
}

#[test]
fn once() {
    let scratch = Scratch::new("once");
    scratch.write("log = {}\ngband.on('FocusChanged', function(e) log[#log + 1] = 'ran' end, { once = true })");
    let config = scratch.loaded();
    clean(&config.runtime.emit(&focus(1, 2)));
    clean(&config.runtime.emit(&focus(2, 1)));
    assert_eq!(log(&config), ["ran"]);
}

#[test]
fn payload_copies_are_separate() {
    let scratch = Scratch::new("copies");
    scratch.write(
        "log = {}
gband.on('FocusChanged', function(e) e.window = nil end)
gband.on('FocusChanged', function(e) log[#log + 1] = tostring(e.window) end)",
    );
    let config = scratch.loaded();
    clean(&config.runtime.emit(&focus(1, 2)));
    assert_eq!(log(&config), ["1"]);
}

#[test]
fn handlers_run_in_registration_order() {
    let scratch = Scratch::new("order");
    scratch.write(
        "log = {}
gband.on('FocusChanged', function() log[#log + 1] = 'a' end)
gband.on('FocusChanged', function() log[#log + 1] = 'b' end)
gband.on('FocusChanged', function() log[#log + 1] = 'c' end)",
    );
    let config = scratch.loaded();
    clean(&config.runtime.emit(&focus(1, 2)));
    assert_eq!(log(&config), ["a", "b", "c"]);
}

#[test]
fn unknown_event() {
    let scratch = Scratch::new("unknown");
    let path = scratch.write("\ngband.on('FocusChange', function() end)");
    let error = scratch.load().err().unwrap();
    assert_error_at(&error, &path, 2, "FocusChange");
}

#[test]
fn invalid_handlers() {
    for (index, call) in [
        "gband.on('FocusChanged', 'not a function')",
        "gband.on('FocusChanged', function() end, { pattern = 'x' })",
        "gband.on('FocusChanged', function() end, { group = 'absent' })",
        "gband.on('FocusChanged', function() end, { group = 42 })",
        "gband.on('FocusChanged', function() end, { once = 'yes' })",
        "gband.on(5, function() end)",
        "gband.augroup('')",
        "gband.augroup('g', { clear = 'no' })",
    ]
    .into_iter()
    .enumerate()
    {
        let scratch = Scratch::new(&format!("invalid-{index}"));
        let path = scratch.write(&format!("\n\n{call}"));
        let error = scratch
            .load()
            .err()
            .unwrap_or_else(|| panic!("{call} loaded"));
        assert_error_at(&error, &path, 3, "");
    }
}

#[test]
fn clear_on_redefinition() {
    let scratch = Scratch::new("clear");
    scratch.write(
        "log = {}
first = gband.augroup('g')
gband.on('FocusChanged', function() log[#log + 1] = 'first' end, { group = first })
second = gband.augroup('g')
gband.on('FocusChanged', function() log[#log + 1] = 'second' end, { group = second })",
    );
    let config = scratch.loaded();
    clean(&config.runtime.emit(&focus(1, 2)));
    assert_eq!(log(&config), ["second"]);
    assert_eq!(
        global::<i64>(&config, "first"),
        global::<i64>(&config, "second")
    );
}

#[test]
fn keep_without_clear() {
    let scratch = Scratch::new("keep");
    scratch.write(
        "log = {}
gband.augroup('g')
gband.on('FocusChanged', function() log[#log + 1] = 'first' end, { group = 'g' })
gband.augroup('g', { clear = false })
gband.on('FocusChanged', function() log[#log + 1] = 'second' end, { group = 'g' })",
    );
    let config = scratch.loaded();
    clean(&config.runtime.emit(&focus(1, 2)));
    assert_eq!(log(&config), ["first", "second"]);
}

#[test]
fn group_by_name() {
    let scratch = Scratch::new("group-name");
    scratch.write(
        "log = {}
gband.augroup('other')
id = gband.augroup('g')
gband.on('FocusChanged', function() log[#log + 1] = 'named' end, { group = 'g' })
gband.augroup(id == gband.augroup('g', { clear = false }) and 'g' or 'mismatch')",
    );
    let config = scratch.loaded();
    clean(&config.runtime.emit(&focus(1, 2)));
    assert!(log(&config).is_empty());
}

#[test]
fn pattern_match() {
    let scratch = Scratch::new("pattern");
    scratch.write(
        "log = {}
gband.on('User', function(e) log[#log + 1] = 'first ' .. e.name .. ' ' .. e.data.n end, { pattern = 'hello.ready' })
gband.on('User', function(e) log[#log + 1] = 'second' end, { pattern = 'other' })
gband.on('User', function(e) log[#log + 1] = 'any ' .. e.name end)
gband.bind('alt+e', function()
  gband.emit('hello.ready', { n = 1 })
  log[#log + 1] = 'after'
end)",
    );
    let config = scratch.loaded();
    let Binding::Callback(callback) = config.keymap["root"][0].1 else {
        panic!("not a function");
    };
    clean(&config.runtime.call(callback));
    assert_eq!(
        log(&config),
        ["first hello.ready 1", "any hello.ready", "after"]
    );
}

#[test]
fn emit_outside_a_callback_fails() {
    let scratch = Scratch::new("emit-loading");
    let path = scratch.write("\ngband.emit('x')");
    let error = scratch.load().err().unwrap();
    assert_error_at(&error, &path, 2, "callback");
}

#[test]
fn spawn_from_an_event_handler() {
    let scratch = Scratch::new("spawn");
    scratch.write(
        "gband.on('User', function() gband.spawn({ cmd = 'fish' }) end, { pattern = 'go' })
gband.bind('alt+e', function() gband.emit('go') end)",
    );
    let config = scratch.loaded();
    let Binding::Callback(callback) = config.keymap["root"][0].1 else {
        panic!("not a function");
    };
    let outcome = config.runtime.call(callback);
    clean(&outcome);
    assert_eq!(
        outcome.dispatched,
        [Dispatch::Spawn(Some(
            gband_core::layout::Program::CommandLine("fish".to_owned())
        ))]
    );
}

#[test]
fn handler_dispatches_are_returned() {
    let scratch = Scratch::new("dispatch");
    scratch.write("gband.on('WindowOpened', function() gband.action.focus_column_left() end)");
    let config = scratch.loaded();
    let outcome = config.runtime.emit(&Event::WindowOpened {
        window: WindowId(2),
        band: BandId(1),
    });
    clean(&outcome);
    assert_eq!(
        outcome.dispatched,
        [Dispatch::Action(Action::View(ViewAction::FocusLeft))]
    );
}

#[test]
fn error_in_a_callback() {
    let scratch = Scratch::new("handler-error");
    let file = scratch.client_plugin("first",
        "log = {}\ngband.on('FocusChanged', function()\n  log[#log + 1] = 'first'\n  error('handler failed')\nend)",
    );
    scratch.client_plugin(
        "second",
        "gband.on('FocusChanged', function() log[#log + 1] = 'second' end)",
    );
    let config = scratch.loaded();
    let outcome = config.runtime.emit(&focus(1, 2));
    assert_eq!(outcome.errors.len(), 1, "{:?}", outcome.errors);
    assert_eq!(outcome.errors[0].plugin.as_deref(), Some("first"));
    assert_error_at(&outcome.errors[0], &file, 4, "handler failed");
    let again = config.runtime.emit(&focus(2, 1));
    assert_eq!(again.errors.len(), 1);
    assert_eq!(log(&config), ["first", "second", "first", "second"]);
}
