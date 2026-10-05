use std::collections::BTreeSet;

use mlua::Lua;

use crate::error::ConfigError;

#[derive(Default)]
pub(crate) struct Owners {
    stack: Vec<Option<String>>,
    failed: BTreeSet<String>,
}

fn owners(lua: &Lua) -> mlua::AppDataRefMut<'_, Owners> {
    lua.app_data_mut::<Owners>()
        .expect("owners are installed with the runtime")
}

pub(crate) fn current(lua: &Lua) -> Option<String> {
    owners(lua).stack.last().cloned().flatten()
}

pub(crate) fn push(lua: &Lua, owner: Option<String>) {
    owners(lua).stack.push(owner);
}

pub(crate) fn pop(lua: &Lua) {
    owners(lua).stack.pop();
}

pub(crate) fn is_failed(lua: &Lua, owner: Option<&str>) -> bool {
    owner.is_some_and(|plugin| owners(lua).failed.contains(plugin))
}

pub(crate) fn mark_failed(lua: &Lua, plugin: &str) {
    owners(lua).failed.insert(plugin.to_owned());
}

pub(crate) fn full_name(lua: &Lua, name: &str) -> mlua::Result<String> {
    let Some(plugin) = current(lua) else {
        return Ok(name.to_owned());
    };
    if name
        .strip_prefix(&plugin)
        .is_some_and(|rest| rest.starts_with('.'))
    {
        return Ok(name.to_owned());
    }
    if name.contains('.') {
        return Err(ConfigError::raise(
            lua,
            format!("`{name}` is outside the namespace of the plugin `{plugin}`"),
        ));
    }
    Ok(format!("{plugin}.{name}"))
}
