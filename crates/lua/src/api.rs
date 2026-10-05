use std::path::PathBuf;

use gband_core::action::{Action, ClientAction, SessionCommand};
use gband_core::input::Key;
use gband_core::layout::{Direction, Program, Step};
use gband_core::view::ViewAction;
use mlua::{
    Function, Lua, LuaSerdeExt, MetaMethod, RegistryKey, Table, UserData, UserDataMethods, Value,
};

use crate::error::{ConfigError, caller};
use crate::keys::parse_key;
use crate::options::{NAMES, OptionsPatch, PartialOptions};

pub const ACTIONS: [(&str, Action); 19] = [
    ("focus_column_left", Action::View(ViewAction::FocusLeft)),
    ("focus_column_right", Action::View(ViewAction::FocusRight)),
    ("focus_pane_down", Action::View(ViewAction::FocusDown)),
    ("focus_pane_up", Action::View(ViewAction::FocusUp)),
    (
        "focus_workspace_down",
        Action::View(ViewAction::WorkspaceDown),
    ),
    ("focus_workspace_up", Action::View(ViewAction::WorkspaceUp)),
    ("open_pane", Action::Session(SessionCommand::OpenPane)),
    ("close_pane", Action::Session(SessionCommand::ClosePane)),
    (
        "consume_or_expel_left",
        Action::Session(SessionCommand::ConsumeOrExpel(Direction::Left)),
    ),
    (
        "consume_or_expel_right",
        Action::Session(SessionCommand::ConsumeOrExpel(Direction::Right)),
    ),
    (
        "cycle_column_width",
        Action::Session(SessionCommand::CycleWidth),
    ),
    (
        "toggle_full_width",
        Action::Session(SessionCommand::ToggleFullWidth),
    ),
    (
        "grow_column_width",
        Action::Session(SessionCommand::StepWidth(Step::Grow)),
    ),
    (
        "shrink_column_width",
        Action::Session(SessionCommand::StepWidth(Step::Shrink)),
    ),
    (
        "grow_pane_height",
        Action::Session(SessionCommand::StepHeight(Step::Grow)),
    ),
    (
        "shrink_pane_height",
        Action::Session(SessionCommand::StepHeight(Step::Shrink)),
    ),
    (
        "reset_pane_height",
        Action::Session(SessionCommand::ResetHeight),
    ),
    ("detach", Action::Client(ClientAction::Detach)),
    ("send_prefix", Action::Client(ClientAction::SendPrefix)),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chord {
    Key(Key),
    Prefix,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keys {
    Direct(Key),
    Prefixed(Chord),
}

#[derive(Debug)]
pub enum Binding {
    Action(Action),
    Function(RegistryKey),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Dispatch {
    Action(Action),
    Spawn(Option<Program>),
}

pub(crate) struct Bound {
    pub keys: Keys,
    pub binding: Binding,
    pub location: Option<(PathBuf, u32)>,
}

#[derive(Default)]
pub(crate) struct Loading {
    pub options: PartialOptions,
    pub bindings: Vec<Bound>,
}

pub(crate) struct Source(pub Option<PathBuf>);

struct Queue(Vec<Dispatch>);

struct LuaAction(Action);

impl UserData for LuaAction {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(MetaMethod::Call, |lua, this, ()| {
            queue(lua, Dispatch::Action(this.0), "an action")
        });
    }
}

fn queue(lua: &Lua, entry: Dispatch, what: &str) -> mlua::Result<()> {
    match lua.app_data_mut::<Queue>() {
        Some(mut queue) => {
            queue.0.push(entry);
            Ok(())
        }
        None => Err(ConfigError::raise(
            lua,
            format!("{what} can only be called inside a binding function"),
        )),
    }
}

fn loading<'lua>(lua: &'lua Lua, what: &str) -> mlua::Result<mlua::AppDataRefMut<'lua, Loading>> {
    lua.app_data_mut::<Loading>().ok_or_else(|| {
        ConfigError::raise(
            lua,
            format!("{what} can only be called while the configuration loads"),
        )
    })
}

pub(crate) fn install(lua: &Lua) -> mlua::Result<()> {
    let gband = lua.create_table()?;
    gband.set("set", lua.create_function(set)?)?;
    gband.set("bind", lua.create_function(bind)?)?;
    gband.set("unbind", lua.create_function(unbind)?)?;
    gband.set("spawn", lua.create_function(spawn)?)?;
    let actions = lua.create_table()?;
    for (name, action) in ACTIONS {
        actions.set(name, LuaAction(action))?;
    }
    gband.set("action", actions)?;
    lua.globals().set("gband", gband)
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
        if !NAMES.contains(&name.as_str()) {
            return Err(ConfigError::raise(lua, format!("unknown option `{name}`")));
        }
        let single = lua.create_table()?;
        single.set(name.as_str(), value)?;
        let patch: OptionsPatch = lua.from_value(Value::Table(single)).map_err(|error| {
            ConfigError::raise(
                lua,
                format!("invalid value for option `{name}`: {}", reason(&error)),
            )
        })?;
        patches.push(patch);
    }
    let mut loading = loading(lua, "gband.set")?;
    for patch in patches {
        loading.options.merge(patch);
    }
    Ok(())
}

fn reason(error: &mlua::Error) -> String {
    match error {
        mlua::Error::DeserializeError(message) => message.clone(),
        other => other.to_string(),
    }
}

pub(crate) fn parse_keys(text: &str) -> Result<Keys, String> {
    let words: Vec<&str> = text.split(' ').collect();
    let key = |name: &str| parse_key(name).map_err(|error| error.to_string());
    match words.as_slice() {
        [name] => key(name).map(Keys::Direct),
        ["prefix", "prefix"] => Ok(Keys::Prefixed(Chord::Prefix)),
        ["prefix", name] => key(name).map(|key| Keys::Prefixed(Chord::Key(key))),
        _ => Err(format!("invalid key list `{text}`")),
    }
}

fn keys_argument(lua: &Lua, keys: &Value, function: &str) -> mlua::Result<(String, Keys)> {
    let Value::String(text) = keys else {
        return Err(ConfigError::raise(
            lua,
            format!("{function} expects key names as a string"),
        ));
    };
    let text = text.to_str()?.to_owned();
    let keys = parse_keys(&text).map_err(|message| ConfigError::raise(lua, message))?;
    Ok((text, keys))
}

fn bind(lua: &Lua, (keys, action): (Value, Value)) -> mlua::Result<()> {
    let (text, keys) = keys_argument(lua, &keys, "gband.bind")?;
    let binding = match &action {
        Value::Function(function) => {
            Binding::Function(lua.create_registry_value(function.clone())?)
        }
        Value::UserData(data) => match data.borrow::<LuaAction>() {
            Ok(action) => Binding::Action(action.0),
            Err(_) => return Err(not_an_action(lua, &text)),
        },
        _ => return Err(not_an_action(lua, &text)),
    };
    let location = caller(lua);
    let mut loading = loading(lua, "gband.bind")?;
    loading.bindings.retain(|bound| bound.keys != keys);
    loading.bindings.push(Bound {
        keys,
        binding,
        location,
    });
    Ok(())
}

fn not_an_action(lua: &Lua, text: &str) -> mlua::Error {
    ConfigError::raise(
        lua,
        format!("the binding of `{text}` must be a gband.action value or a function"),
    )
}

fn unbind(lua: &Lua, keys: Value) -> mlua::Result<()> {
    let (_, keys) = keys_argument(lua, &keys, "gband.unbind")?;
    loading(lua, "gband.unbind")?
        .bindings
        .retain(|bound| bound.keys != keys);
    Ok(())
}

fn spawn(lua: &Lua, request: Value) -> mlua::Result<()> {
    let Value::Table(request) = request else {
        return Err(ConfigError::raise(lua, "gband.spawn expects a table"));
    };
    let mut program = None;
    for pair in request.pairs::<Value, Value>() {
        let (name, value) = pair?;
        if !matches!(&name, Value::String(name) if name == "cmd") {
            return Err(ConfigError::raise(
                lua,
                "gband.spawn takes only the field `cmd`",
            ));
        }
        program = command(lua, value)?;
    }
    queue(lua, Dispatch::Spawn(program), "gband.spawn")
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

fn list_of_strings(list: &Table) -> Option<Vec<String>> {
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

pub fn call(lua: &Lua, function: &RegistryKey) -> (Vec<Dispatch>, Option<ConfigError>) {
    lua.set_app_data(Queue(Vec::new()));
    let result = lua
        .registry_value::<Function>(function)
        .and_then(|function| function.call::<()>(()));
    let dispatched = lua
        .remove_app_data::<Queue>()
        .map(|queue| queue.0)
        .unwrap_or_default();
    let error = result.err().map(|error| {
        let source = lua
            .app_data_ref::<Source>()
            .and_then(|source| source.0.clone());
        ConfigError::from_lua(&error, source.as_deref())
    });
    (dispatched, error)
}
