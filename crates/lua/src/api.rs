use gband_core::action::Action;
use gband_core::input::{Key, MouseKey};
use gband_core::layout::{BandId, Program, Proportion, SessionAction, WindowContent, WindowId};
use gband_protocol::Value as Data;
use mlua::{Lua, Table, Value};

use crate::Side;
use crate::callbacks::CallbackId;
use crate::error::ConfigError;
use crate::keymap;
use crate::keys::{Pressed, parse_pressed};
use crate::options;
use crate::runtime::is_loading;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chord {
    Key(Key),
    Mouse { key: MouseKey, uses_mod: bool },
    Prefix,
}

impl From<Pressed> for Chord {
    fn from(pressed: Pressed) -> Self {
        match pressed {
            Pressed::Key(key) => Chord::Key(key),
            Pressed::Mouse { key, uses_mod } => Chord::Mouse { key, uses_mod },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Binding {
    Action(Action),
    Callback(CallbackId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowInput {
    Key(Key),
    Paste(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginWindowRequest {
    Open {
        plugin_window: u32,
        target: Option<(BandId, Option<WindowId>)>,
        width: Option<Proportion>,
        focus: bool,
    },
    Close {
        plugin_window: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Dispatch {
    Action(Action),
    Spawn(Option<Program>),
    Enter(String),
    Session(SessionAction),
    Input {
        window: WindowId,
        input: WindowInput,
    },
    PluginWindow(PluginWindowRequest),
    Write(Vec<u8>),
    Call {
        call: u64,
        name: String,
        args: Data,
    },
    Targeted {
        session: String,
        action: SessionAction,
    },
    ClearErrors,
    Rename {
        window: WindowId,
        name: Option<String>,
    },
}

#[derive(Default)]
pub(crate) struct Queue(pub Option<Vec<Dispatch>>);

pub(crate) fn in_callback(lua: &Lua) -> bool {
    lua.app_data_ref::<Queue>()
        .is_some_and(|queue| queue.0.is_some())
}

pub(crate) fn outside_callback(lua: &Lua, what: &str) -> mlua::Error {
    ConfigError::raise(
        lua,
        format!("{what} can only be called inside a binding function or another callback"),
    )
}

pub(crate) fn queue(lua: &Lua, entry: Dispatch, what: &str) -> mlua::Result<()> {
    let mut queue = lua
        .app_data_mut::<Queue>()
        .expect("the queue is installed with the runtime");
    match &mut queue.0 {
        Some(entries) => {
            entries.push(entry);
            Ok(())
        }
        None => {
            drop(queue);
            Err(outside_callback(lua, what))
        }
    }
}

pub(crate) fn require_loading(lua: &Lua, what: &str) -> mlua::Result<()> {
    if is_loading(lua) {
        Ok(())
    } else {
        Err(ConfigError::raise(
            lua,
            format!("{what} can only be called while the configuration loads"),
        ))
    }
}

pub(crate) fn install(lua: &Lua, gband: &Table, side: Side) -> mlua::Result<()> {
    lua.set_app_data(Queue::default());
    gband.set("set", lua.create_function(set)?)?;
    if side == Side::Server {
        return Ok(());
    }
    gband.set("bind", lua.create_function(bind)?)?;
    gband.set("unbind", lua.create_function(unbind)?)?;
    gband.set("spawn", lua.create_function(spawn)?)?;
    gband.set(
        "errors",
        lua.create_function(|lua, ()| {
            lua.create_sequence_from(crate::ui::current_state(lua).errors)
        })?,
    )?;
    gband.set("clear_errors", lua.create_function(clear_errors)?)?;
    Ok(())
}

fn clear_errors(lua: &Lua, (): ()) -> mlua::Result<()> {
    if !in_callback(lua) {
        return Err(outside_callback(lua, "gband.clear_errors"));
    }
    crate::ui::clear_errors(lua);
    queue(lua, Dispatch::ClearErrors, "gband.clear_errors")
}

fn set(lua: &Lua, options: Value) -> mlua::Result<()> {
    let Value::Table(options) = options else {
        return Err(ConfigError::raise(
            lua,
            "gband.set expects a table of options",
        ));
    };
    let mut patches = Vec::new();
    for pair in options.pairs::<Value, Value>() {
        let (name, value) = pair?;
        let name = match &name {
            Value::String(name) => name.to_str()?.to_owned(),
            other => {
                return Err(ConfigError::raise(
                    lua,
                    format!("option names must be strings, found {}", other.type_name()),
                ));
            }
        };
        options::check_name(lua, &name).map_err(|message| ConfigError::raise(lua, message))?;
        let patch = options::patch(lua, &name, value)
            .map_err(|reason| ConfigError::raise(lua, options::invalid(&name, &reason)))?;
        patches.push(patch);
    }
    require_loading(lua, "gband.set")?;
    for patch in patches {
        options::merge(lua, patch);
    }
    Ok(())
}

pub(crate) fn parse_keys(text: &str) -> Result<(&'static str, Chord), String> {
    let words: Vec<&str> = text.split(' ').collect();
    let chord = |name: &str| {
        parse_pressed(name)
            .map(Chord::from)
            .map_err(|error| error.to_string())
    };
    match words.as_slice() {
        [name] => chord(name).map(|chord| (keymap::ROOT, chord)),
        ["prefix", "prefix"] => Ok((keymap::PREFIX, Chord::Prefix)),
        ["prefix", name] => chord(name).map(|chord| (keymap::PREFIX, chord)),
        _ => Err(format!("invalid key list `{text}`")),
    }
}

fn keys_argument(
    lua: &Lua,
    keys: &Value,
    function: &str,
) -> mlua::Result<(String, &'static str, Chord)> {
    let Value::String(text) = keys else {
        return Err(ConfigError::raise(
            lua,
            format!("{function} expects key names as a string"),
        ));
    };
    let text = text.to_str()?.to_owned();
    let (table, chord) = parse_keys(&text).map_err(|message| ConfigError::raise(lua, message))?;
    Ok((text, table, chord))
}

fn bind(lua: &Lua, (keys, action): (Value, Value)) -> mlua::Result<()> {
    let (text, table, chord) = keys_argument(lua, &keys, "gband.bind")?;
    let target = keymap::target(lua, &action)?.ok_or_else(|| {
        ConfigError::raise(
            lua,
            format!("the binding of `{text}` must be a gband.action value or a function"),
        )
    })?;
    require_loading(lua, "gband.bind")?;
    let key = text.rsplit(' ').next().unwrap_or(&text).to_owned();
    keymap::insert(lua, table, key, chord, target, None)
}

fn unbind(lua: &Lua, keys: Value) -> mlua::Result<()> {
    let (_, table, chord) = keys_argument(lua, &keys, "gband.unbind")?;
    require_loading(lua, "gband.unbind")?;
    keymap::remove(lua, table, chord);
    Ok(())
}

fn spawn(lua: &Lua, request: Value) -> mlua::Result<()> {
    let Value::Table(request) = request else {
        return Err(ConfigError::raise(lua, "gband.spawn expects a table"));
    };
    if !in_callback(lua) {
        return Err(outside_callback(lua, "gband.spawn"));
    }
    let mut program = None;
    let (mut band, mut after) = (Value::Nil, Value::Nil);
    for pair in request.pairs::<Value, Value>() {
        let (name, value) = pair?;
        match &name {
            Value::String(name) if name == "cmd" => program = command(lua, value)?,
            Value::String(name) if name == "band" => band = value,
            Value::String(name) if name == "after" => after = value,
            _ => {
                return Err(ConfigError::raise(
                    lua,
                    format!(
                        "gband.spawn takes only the fields `cmd`, `band` and `after`, found `{}`",
                        crate::control::field_name(&name)
                    ),
                ));
            }
        }
    }
    let entry = match crate::control::open_target(lua, &band, &after)
        .map_err(|message| ConfigError::raise(lua, message))?
    {
        None => Dispatch::Spawn(program),
        Some((band, after)) => Dispatch::Session(SessionAction::OpenWindow {
            band,
            after,
            width: None,
            floating: false,
            focus: true,
            content: WindowContent::Program(program),
        }),
    };
    queue(lua, entry, "gband.spawn")
}

fn command(lua: &Lua, value: Value) -> mlua::Result<Option<Program>> {
    let invalid = || ConfigError::raise(lua, "`cmd` must be a string or a list of strings");
    match value {
        Value::Nil => Ok(None),
        Value::String(line) => Ok(Some(Program::CommandLine(line.to_str()?.to_owned()))),
        Value::Table(list) => {
            let argv = list_of_strings(&list).ok_or_else(invalid)?;
            if argv.is_empty() {
                return Err(ConfigError::raise(lua, "`cmd` must not be an empty list"));
            }
            Ok(Some(Program::Argv(argv)))
        }
        _ => Err(invalid()),
    }
}

pub(crate) fn list_of_strings(list: &Table) -> Option<Vec<String>> {
    let length = list.raw_len();
    if list.pairs::<Value, Value>().count() != length {
        return None;
    }
    (1..=length)
        .map(|index| match list.raw_get::<Value>(index).ok()? {
            Value::String(text) => text.to_str().ok().map(|text| text.to_owned()),
            _ => None,
        })
        .collect()
}
