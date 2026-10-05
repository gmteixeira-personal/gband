use gband_core::action::Action;
use mlua::{Lua, Table, Value};

use crate::Side;
use crate::actions::ACTIONS;
use crate::error::ConfigError;

pub(crate) const CLIENT_ONLY: [&str; 17] = [
    "bind",
    "unbind",
    "spawn",
    "keymap",
    "ui",
    "hl",
    "colorscheme",
    "layout",
    "view",
    "pane",
    "band",
    "win",
    "rpc",
    "notify",
    "bell",
    "clipboard",
    "open",
];

pub(crate) const SERVER_ONLY: [&str; 2] = ["sessions", "session"];

fn other_only(side: Side) -> &'static [&'static str] {
    match side {
        Side::Client => &SERVER_ONLY,
        Side::Server => &CLIENT_ONLY,
    }
}

pub(crate) fn is_client_action(action: Action) -> bool {
    !matches!(action, Action::Session(_))
}

pub(crate) fn guard(lua: &Lua, gband: &Table, side: Side) -> mlua::Result<()> {
    let foreign = |names: Vec<&'static str>, prefix: &'static str, kind: &'static str| {
        let meta = lua.create_table()?;
        meta.set(
            "__index",
            lua.create_function(
                move |lua, (_, key): (Value, Value)| -> mlua::Result<Value> {
                    let Value::String(key) = key else {
                        return Ok(Value::Nil);
                    };
                    let key = key.to_string_lossy();
                    if !names.contains(&key.as_str()) {
                        return Ok(Value::Nil);
                    }
                    Err(ConfigError::raise(
                        lua,
                        format!(
                            "`{prefix}{key}` is a {} {kind}; this is the {}",
                            side.other().name(),
                            side.name()
                        ),
                    ))
                },
            )?,
        )?;
        Ok::<_, mlua::Error>(meta)
    };
    gband.set_metatable(Some(foreign(other_only(side).to_vec(), "gband.", "API")?))?;
    if side == Side::Server {
        let actions: Table = gband.get("action")?;
        let names = ACTIONS
            .iter()
            .filter(|action| is_client_action(action.action))
            .map(|action| action.name)
            .collect();
        actions.set_metatable(Some(foreign(names, "gband.action.", "action")?))?;
    }
    Ok(())
}
