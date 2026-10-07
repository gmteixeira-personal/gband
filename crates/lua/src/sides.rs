use std::collections::BTreeMap;
use std::sync::OnceLock;

use gband_core::action::Action;
use mlua::{Function, Lua, MultiValue, Table, Value};

use crate::Side;
use crate::actions::ACTIONS;
use crate::error::{ConfigError, caller};
use crate::removed;

pub(crate) const CLIENT_ONLY: [&str; 24] = [
    "bind",
    "unbind",
    "spawn",
    "keymap",
    "keystyle",
    "settings",
    "ui",
    "hl",
    "colorscheme",
    "palette",
    "layout",
    "view",
    "window",
    "band",
    "win",
    "bar",
    "errors",
    "clear_errors",
    "rpc",
    "notify",
    "bell",
    "clipboard",
    "open",
    "core",
];

pub(crate) const SERVER_ONLY: [&str; 2] = ["sessions", "session"];

const UNSET_WITHOUT_LOCATIONS: [&str; 1] = ["config_dir"];

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
                .chain(only.iter().map(|name| (*name).to_owned()))
                .chain(
                    UNSET_WITHOUT_LOCATIONS
                        .iter()
                        .map(|name| (*name).to_owned()),
                );
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

fn client_core() -> &'static Vec<String> {
    static CORE: OnceLock<Vec<String>> = OnceLock::new();
    CORE.get_or_init(|| {
        let lua = Lua::new();
        let installed = crate::runtime::install(&lua, Side::Client, None, crate::BUDGET);
        let core = installed.and_then(|()| {
            lua.globals()
                .get::<Table>("gband")?
                .raw_get::<Table>("core")
        });
        core.map(|core| {
            core.pairs::<String, Value>()
                .filter_map(Result::ok)
                .map(|(name, _)| name)
                .collect()
        })
        .unwrap_or_default()
    })
}

fn test_core(lua: &Lua) -> mlua::Result<Table> {
    let core = lua.create_table()?;
    let meta = lua.create_table()?;
    meta.set(
        "__index",
        lua.create_function(|lua, (_, key): (Value, Value)| -> mlua::Result<Value> {
            let Value::String(key) = key else {
                return Ok(Value::Nil);
            };
            let key = key.to_string_lossy();
            if !client_core().contains(&key) {
                return Ok(Value::Nil);
            }
            Err(located(
                lua,
                format!("`gband.core.{key}` is a client API; this is the test side"),
            ))
        })?,
    )?;
    core.set_metatable(Some(meta))?;
    Ok(core)
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
    gband.set("core", test_core(lua)?)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_side() -> Lua {
        let lua = Lua::new();
        install_test(&lua, |_| {}).unwrap();
        lua
    }

    fn read_error(lua: &Lua, source: &str) -> String {
        lua.load(source)
            .set_name("@spec_test.lua")
            .exec()
            .unwrap_err()
            .to_string()
    }

    #[test]
    fn configuration_directory_in_a_test_file() {
        let error = read_error(&test_side(), "local a = 1\nreturn gband.config_dir");
        assert!(error.contains("spec_test.lua:2:"), "{error}");
        assert!(
            error.contains("`gband.config_dir` is a client and server API"),
            "{error}"
        );
    }

    #[test]
    fn key_style_api_in_the_server() {
        let lua = Lua::new();
        crate::runtime::install(&lua, Side::Server, None, crate::BUDGET).unwrap();
        let error = read_error(&lua, "local a = 1\ngband.keystyle.use()");
        assert!(
            error.contains("`gband.keystyle` is a client API"),
            "{error}"
        );
        assert!(error.contains("this is the server"), "{error}");
        let error = read_error(&test_side(), "return gband.keystyle");
        assert!(
            error.contains("`gband.keystyle` is a client API"),
            "{error}"
        );
    }

    fn server() -> Lua {
        let lua = Lua::new();
        crate::runtime::install(&lua, Side::Server, None, crate::BUDGET).unwrap();
        lua
    }

    #[test]
    fn palette_in_the_server() {
        let error = read_error(&server(), "local a = 1\nlocal b = 2\ngband.palette.get()");
        assert!(error.contains("spec_test.lua:3:"), "{error}");
        assert!(error.contains("`gband.palette` is a client API"), "{error}");
    }

    #[test]
    fn settings_api_in_the_server() {
        let error = read_error(&server(), "local a = 1\ngband.settings.theme()");
        assert!(error.contains("spec_test.lua:2:"), "{error}");
        assert!(
            error.contains("`gband.settings` is a client API"),
            "{error}"
        );
    }
}
