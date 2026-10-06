use std::collections::BTreeMap;
use std::sync::OnceLock;

use gband_core::action::Action;
use mlua::{Function, Lua, MultiValue, Table, Value};

use crate::Side;
use crate::actions::ACTIONS;
use crate::error::{ConfigError, caller};
use crate::removed;

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
    "window",
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
        Side::Test => &[],
    }
}

fn provided(side: Side) -> Vec<String> {
    let lua = Lua::new();
    let installed = crate::runtime::install(&lua, side, None, crate::BUDGET);
    let gband = installed.and_then(|()| lua.globals().get::<Table>("gband"));
    gband
        .map(|gband| {
            gband
                .pairs::<String, Value>()
                .filter_map(Result::ok)
                .map(|(name, _)| name)
                .collect()
        })
        .unwrap_or_default()
}

fn providers() -> &'static BTreeMap<String, Vec<Side>> {
    static PROVIDERS: OnceLock<BTreeMap<String, Vec<Side>>> = OnceLock::new();
    PROVIDERS.get_or_init(|| {
        let mut providers: BTreeMap<String, Vec<Side>> = BTreeMap::new();
        for side in [Side::Client, Side::Server] {
            let only: &[&str] = match side {
                Side::Server => &SERVER_ONLY,
                _ => &CLIENT_ONLY,
            };
            let names = provided(side)
                .into_iter()
                .chain(only.iter().map(|name| (*name).to_owned()));
            for name in names {
                let sides = providers.entry(name).or_default();
                if !sides.contains(&side) {
                    sides.push(side);
                }
            }
        }
        providers
    })
}

pub(crate) fn located(lua: &Lua, message: String) -> mlua::Error {
    match caller(lua) {
        Some((path, line)) => mlua::Error::runtime(format!("{}:{line}: {message}", path.display())),
        None => mlua::Error::runtime(message),
    }
}

pub fn install_test(lua: &Lua, print: impl Fn(&str) + Send + 'static) -> mlua::Result<()> {
    lua.set_app_data(Side::Test);
    let gband = lua.create_table()?;
    gband.set("side", Side::Test.name())?;
    gband.set("api_version", crate::API_VERSION)?;
    let meta = lua.create_table()?;
    meta.set(
        "__index",
        lua.create_function(|lua, (_, key): (Value, Value)| -> mlua::Result<Value> {
            let Value::String(key) = key else {
                return Ok(Value::Nil);
            };
            let key = key.to_string_lossy();
            let Some(sides) = providers().get(key.as_str()) else {
                return Ok(Value::Nil);
            };
            let sides = sides
                .iter()
                .map(|side| side.name())
                .collect::<Vec<_>>()
                .join(" and ");
            Err(located(
                lua,
                format!("`gband.{key}` is a {sides} API; this is the test side"),
            ))
        })?,
    )?;
    gband.set_metatable(Some(meta))?;
    lua.globals().set("gband", gband)?;
    lua.globals().set(
        "print",
        lua.create_function(move |lua, values: MultiValue| {
            let tostring: Function = lua.globals().get("tostring")?;
            let text = values
                .into_iter()
                .map(|value| tostring.call::<String>(value))
                .collect::<mlua::Result<Vec<_>>>()?
                .join("\t");
            print(&text);
            Ok(())
        })?,
    )
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
                    let qualified = match prefix {
                        "gband." => format!("{prefix}{key}"),
                        _ => key.clone(),
                    };
                    if let Some(error) = removed::raise(lua, &qualified) {
                        return Err(error);
                    }
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
    let actions: Table = gband.get("action")?;
    let names = match side {
        Side::Server => ACTIONS
            .iter()
            .filter(|action| is_client_action(action.action))
            .map(|action| action.name)
            .collect(),
        _ => Vec::new(),
    };
    actions.set_metatable(Some(foreign(names, "gband.action.", "action")?))?;
    Ok(())
}
