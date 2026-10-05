use mlua::{FromLuaMulti, Function, IntoLuaMulti, Lua, RegistryKey};

use crate::error::ConfigError;
use crate::guard;
use crate::owner;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CallbackId(usize);

struct Entry {
    function: RegistryKey,
    owner: Option<String>,
}

#[derive(Default)]
pub(crate) struct Callbacks(Vec<Entry>);

pub(crate) enum Ran<R> {
    Returned(R),
    Failed(ConfigError),
    Disabled,
}

pub(crate) fn register(lua: &Lua, function: Function) -> mlua::Result<CallbackId> {
    let function = lua.create_registry_value(function)?;
    let owner = owner::current(lua);
    let mut callbacks = lua
        .app_data_mut::<Callbacks>()
        .expect("callbacks are installed with the runtime");
    callbacks.0.push(Entry { function, owner });
    Ok(CallbackId(callbacks.0.len() - 1))
}

pub(crate) fn run<R: FromLuaMulti>(
    lua: &Lua,
    id: CallbackId,
    args: impl IntoLuaMulti,
) -> mlua::Result<Ran<R>> {
    let (function, owner) = {
        let callbacks = lua
            .app_data_ref::<Callbacks>()
            .expect("callbacks are installed with the runtime");
        let entry = &callbacks.0[id.0];
        (
            lua.registry_value::<Function>(&entry.function)?,
            entry.owner.clone(),
        )
    };
    if owner::is_failed(lua, owner.as_deref()) {
        return Ok(Ran::Disabled);
    }
    match guard::run(lua, owner, || function.call::<R>(args))? {
        Ok(value) => Ok(Ran::Returned(value)),
        Err(failure) => {
            let error = failure.error.clone();
            guard::report(lua, failure);
            Ok(Ran::Failed(error))
        }
    }
}

pub(crate) fn owner(lua: &Lua, id: CallbackId) -> Option<String> {
    lua.app_data_ref::<Callbacks>()
        .expect("callbacks are installed with the runtime")
        .0[id.0]
        .owner
        .clone()
}
