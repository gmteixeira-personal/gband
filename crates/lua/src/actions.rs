use std::collections::BTreeMap;

use gband_core::action::{Action, ClientAction, SessionCommand};
use gband_core::layout::{Direction, Step};
use gband_core::view::ViewAction;
use mlua::{Lua, MetaMethod, Table, UserData, UserDataMethods, Value};

use crate::api::{self, Dispatch, require_loading};
use crate::callbacks::{self, CallbackId};
use crate::error::ConfigError;
use crate::owner;

pub struct BuiltinAction {
    pub name: &'static str,
    pub action: Action,
    pub desc: &'static str,
}

const fn builtin(name: &'static str, action: Action, desc: &'static str) -> BuiltinAction {
    BuiltinAction { name, action, desc }
}

pub const ACTIONS: [BuiltinAction; 19] = [
    builtin(
        "focus_column_left",
        Action::View(ViewAction::FocusLeft),
        "focus the column to the left",
    ),
    builtin(
        "focus_column_right",
        Action::View(ViewAction::FocusRight),
        "focus the column to the right",
    ),
    builtin(
        "focus_pane_down",
        Action::View(ViewAction::FocusDown),
        "focus the pane below",
    ),
    builtin(
        "focus_pane_up",
        Action::View(ViewAction::FocusUp),
        "focus the pane above",
    ),
    builtin(
        "focus_band_down",
        Action::View(ViewAction::BandDown),
        "view the band below",
    ),
    builtin(
        "focus_band_up",
        Action::View(ViewAction::BandUp),
        "view the band above",
    ),
    builtin(
        "open_pane",
        Action::Session(SessionCommand::OpenPane),
        "open a pane running the user's shell",
    ),
    builtin(
        "close_pane",
        Action::Session(SessionCommand::ClosePane),
        "close the pane",
    ),
    builtin(
        "consume_or_expel_left",
        Action::Session(SessionCommand::ConsumeOrExpel(Direction::Left)),
        "consume or expel the pane to the left",
    ),
    builtin(
        "consume_or_expel_right",
        Action::Session(SessionCommand::ConsumeOrExpel(Direction::Right)),
        "consume or expel the pane to the right",
    ),
    builtin(
        "cycle_column_width",
        Action::Session(SessionCommand::CycleWidth),
        "cycle the width of the pane's column",
    ),
    builtin(
        "toggle_full_width",
        Action::Session(SessionCommand::ToggleFullWidth),
        "toggle full width of the pane's column",
    ),
    builtin(
        "grow_column_width",
        Action::Session(SessionCommand::StepWidth(Step::Grow)),
        "grow the width of the pane's column",
    ),
    builtin(
        "shrink_column_width",
        Action::Session(SessionCommand::StepWidth(Step::Shrink)),
        "shrink the width of the pane's column",
    ),
    builtin(
        "grow_pane_height",
        Action::Session(SessionCommand::StepHeight(Step::Grow)),
        "grow the height of the pane",
    ),
    builtin(
        "shrink_pane_height",
        Action::Session(SessionCommand::StepHeight(Step::Shrink)),
        "shrink the height of the pane",
    ),
    builtin(
        "reset_pane_height",
        Action::Session(SessionCommand::ResetHeight),
        "reset the height of the pane",
    ),
    builtin("detach", Action::Client(ClientAction::Detach), "detach"),
    builtin(
        "send_prefix",
        Action::Client(ClientAction::SendPrefix),
        "send the prefix key to the focused pane",
    ),
];

const RESERVED: [&str; 2] = ["register", "list"];

pub(crate) enum LuaAction {
    Builtin { name: &'static str, action: Action },
    Registered { name: String, callback: CallbackId },
}

impl UserData for LuaAction {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(MetaMethod::Call, |lua, this, target: Value| match this {
            LuaAction::Builtin { action, .. } if target.is_nil() => {
                api::queue(lua, Dispatch::Action(*action), "an action")
            }
            LuaAction::Builtin { name, action } => {
                if !api::in_callback(lua) {
                    return Err(api::outside_callback(lua, "an action"));
                }
                let entry = crate::control::targeted(lua, name, *action, &target)
                    .map_err(|message| ConfigError::raise(lua, message))?;
                api::queue(lua, entry, "an action")
            }
            LuaAction::Registered { callback, .. } => {
                if !api::in_callback(lua) {
                    return Err(api::outside_callback(lua, "an action"));
                }
                callbacks::run::<()>(lua, *callback, ())?;
                Ok(())
            }
        });
    }
}

#[derive(Default)]
struct Registered(BTreeMap<String, Option<String>>);

pub(crate) fn install(lua: &Lua, gband: &Table) -> mlua::Result<()> {
    lua.set_app_data(Registered::default());
    let actions = lua.create_table()?;
    for BuiltinAction { name, action, .. } in ACTIONS {
        actions.set(name, LuaAction::Builtin { name, action })?;
    }
    actions.set("register", lua.create_function(register)?)?;
    actions.set("list", lua.create_function(list)?)?;
    gband.set("action", actions)
}

fn register(lua: &Lua, (name, function, opts): (Value, Value, Value)) -> mlua::Result<Value> {
    let Value::String(name) = name else {
        return Err(ConfigError::raise(
            lua,
            "gband.action.register expects a name as a non-empty string",
        ));
    };
    let name = name.to_str()?.to_owned();
    if name.is_empty() {
        return Err(ConfigError::raise(
            lua,
            "gband.action.register expects a name as a non-empty string",
        ));
    }
    let Value::Function(function) = function else {
        return Err(ConfigError::raise(
            lua,
            format!("the action `{name}` must be a function"),
        ));
    };
    let desc = description(lua, &opts)?;
    let full = owner::full_name(lua, &name)?;
    let taken = RESERVED.contains(&full.as_str())
        || ACTIONS.iter().any(|action| action.name == full)
        || registered(lua).0.contains_key(&full);
    if taken {
        return Err(ConfigError::raise(
            lua,
            format!("the action name `{full}` is taken"),
        ));
    }
    require_loading(lua, "gband.action.register")?;
    let callback = callbacks::register(lua, function)?;
    registered(lua).0.insert(full.clone(), desc);
    let actions: Table = lua.globals().get::<Table>("gband")?.get("action")?;
    actions.set(
        full.as_str(),
        LuaAction::Registered {
            name: full.clone(),
            callback,
        },
    )?;
    actions.get(full)
}

fn registered(lua: &Lua) -> mlua::AppDataRefMut<'_, Registered> {
    lua.app_data_mut::<Registered>()
        .expect("registered actions are installed with the runtime")
}

fn description(lua: &Lua, opts: &Value) -> mlua::Result<Option<String>> {
    let invalid = || ConfigError::raise(lua, "gband.action.register expects `desc` to be a string");
    match opts {
        Value::Nil => Ok(None),
        Value::Table(opts) => match opts.get::<Value>("desc")? {
            Value::Nil => Ok(None),
            Value::String(desc) => Ok(Some(desc.to_str()?.to_owned())),
            _ => Err(invalid()),
        },
        _ => Err(invalid()),
    }
}

fn list(lua: &Lua, (): ()) -> mlua::Result<Table> {
    let mut entries: Vec<(String, Option<String>)> = ACTIONS
        .iter()
        .map(|action| (action.name.to_owned(), Some(action.desc.to_owned())))
        .collect();
    entries.extend(
        registered(lua)
            .0
            .iter()
            .map(|(name, desc)| (name.clone(), desc.clone())),
    );
    entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let list = lua.create_table()?;
    for (name, desc) in entries {
        let entry = lua.create_table()?;
        entry.set("name", name)?;
        entry.set("desc", desc)?;
        list.push(entry)?;
    }
    Ok(list)
}
