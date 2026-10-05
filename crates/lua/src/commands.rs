use std::collections::BTreeMap;

use gband_protocol::Value as Data;
use mlua::{Lua, Table, Value};

use crate::Side;
use crate::api::{list_of_strings, require_loading};
use crate::callbacks::{self, CallbackId, Ran};
use crate::error::ConfigError;
use crate::guard;
use crate::owner;
use crate::runtime::side;
use crate::server::Caller;
use crate::value;

struct Command {
    callback: CallbackId,
    desc: Option<String>,
    args: Vec<String>,
}

#[derive(Default)]
struct Commands(BTreeMap<String, Command>);

fn commands(lua: &Lua) -> mlua::AppDataRefMut<'_, Commands> {
    lua.app_data_mut::<Commands>()
        .expect("commands are installed with the runtime")
}

pub(crate) fn install(lua: &Lua, gband: &Table) -> mlua::Result<()> {
    lua.set_app_data(Commands::default());
    let cmd = lua.create_table()?;
    cmd.set("register", lua.create_function(register)?)?;
    cmd.set("run", lua.create_function(run)?)?;
    cmd.set("list", lua.create_function(list)?)?;
    gband.set("cmd", cmd)
}

fn register(lua: &Lua, (name, function, opts): (Value, Value, Value)) -> mlua::Result<String> {
    let name = match &name {
        Value::String(name) if !name.as_bytes().is_empty() => name.to_str()?.to_owned(),
        _ => {
            return Err(ConfigError::raise(
                lua,
                "gband.cmd.register expects a name as a non-empty string",
            ));
        }
    };
    let Value::Function(function) = function else {
        return Err(ConfigError::raise(
            lua,
            format!("the command `{name}` must be a function"),
        ));
    };
    let (desc, args) = options(lua, &name, &opts)?;
    let full = owner::full_name(lua, &name)?;
    if commands(lua).0.contains_key(&full) {
        return Err(ConfigError::raise(
            lua,
            format!("the command `{full}` is already registered"),
        ));
    }
    require_loading(lua, "gband.cmd.register")?;
    let callback = callbacks::register(lua, function)?;
    commands(lua).0.insert(
        full.clone(),
        Command {
            callback,
            desc,
            args,
        },
    );
    Ok(full)
}

fn options(lua: &Lua, name: &str, opts: &Value) -> mlua::Result<(Option<String>, Vec<String>)> {
    let opts = match opts {
        Value::Nil => return Ok((None, Vec::new())),
        Value::Table(opts) => opts,
        _ => {
            return Err(ConfigError::raise(
                lua,
                format!("the options of the command `{name}` must be a table"),
            ));
        }
    };
    let desc = match opts.get::<Value>("desc")? {
        Value::Nil => None,
        Value::String(desc) => Some(desc.to_str()?.to_owned()),
        _ => {
            return Err(ConfigError::raise(
                lua,
                format!("the `desc` of the command `{name}` must be a string"),
            ));
        }
    };
    let args = match opts.get::<Value>("args")? {
        Value::Nil => Vec::new(),
        Value::Table(args) => list_of_strings(&args).ok_or_else(|| {
            ConfigError::raise(
                lua,
                format!("the `args` of the command `{name}` must be a list of strings"),
            )
        })?,
        _ => {
            return Err(ConfigError::raise(
                lua,
                format!("the `args` of the command `{name}` must be a list of strings"),
            ));
        }
    };
    Ok((desc, args))
}

fn run(lua: &Lua, (name, args): (Value, Value)) -> mlua::Result<bool> {
    let name = match &name {
        Value::String(name) => name.to_str()?.to_owned(),
        _ => {
            return Err(ConfigError::raise(
                lua,
                "gband.cmd.run expects a command name as a string",
            ));
        }
    };
    let callback = commands(lua).0.get(&name).map(|command| command.callback);
    let Some(callback) = callback else {
        return Err(ConfigError::raise(lua, format!("unknown command `{name}`")));
    };
    let args = match args {
        Value::Nil => lua.create_table()?,
        Value::Table(args) => args,
        _ => {
            return Err(ConfigError::raise(
                lua,
                format!("the arguments of the command `{name}` must be a table"),
            ));
        }
    };
    match side(lua) {
        Side::Client => Ok(matches!(
            callbacks::run::<()>(lua, callback, args)?,
            Ran::Returned(())
        )),
        Side::Server => Ok(call(lua, &name, callback, args, None)?.is_ok()),
        Side::Test => Ok(false),
    }
}

fn context(lua: &Lua, caller: Option<Caller>) -> mlua::Result<Table> {
    let context = lua.create_table()?;
    let Some(caller) = caller else {
        context.set("focus", lua.create_function(|_, _: Value| Ok(()))?)?;
        return Ok(context);
    };
    context.set("session", caller.session)?;
    context.set("client", caller.client)?;
    let focus = caller.focus;
    context.set(
        "focus",
        lua.create_function(move |lua, pane: Value| {
            let pane = match pane {
                Value::Integer(pane) => u32::try_from(pane).ok(),
                _ => None,
            }
            .ok_or_else(|| ConfigError::raise(lua, "ctx.focus expects a pane number"))?;
            focus(gband_core::layout::PaneId(pane));
            Ok(())
        })?,
    )?;
    Ok(context)
}

fn call(
    lua: &Lua,
    name: &str,
    callback: CallbackId,
    args: Table,
    caller: Option<Caller>,
) -> mlua::Result<Result<Data, String>> {
    let context = context(lua, caller)?;
    Ok(
        match callbacks::run::<Value>(lua, callback, (args, context))? {
            Ran::Returned(result) => match value::from_lua(&result, "result") {
                Ok(result) => Ok(result),
                Err(reason) => {
                    let owner = callbacks::owner(lua, callback);
                    let message = format!("the command `{name}` returned {reason}");
                    guard::push(
                        lua,
                        ConfigError {
                            plugin: owner,
                            location: None,
                            message: message.clone(),
                        },
                    );
                    Err(message)
                }
            },
            Ran::Failed(error) => Err(error.to_string()),
            Ran::Disabled => Err(format!("the command `{name}` is disabled")),
        },
    )
}

pub(crate) fn invoke(
    lua: &Lua,
    name: &str,
    args: &Data,
    caller: Option<Caller>,
) -> mlua::Result<Result<Data, String>> {
    let callback = commands(lua).0.get(name).map(|command| command.callback);
    let Some(callback) = callback else {
        return Ok(Err(format!("unknown command `{name}`")));
    };
    let args = match value::into_lua(lua, args)? {
        Value::Nil => lua.create_table()?,
        Value::Table(args) => args,
        _ => return Ok(Err(format!("the arguments of `{name}` must be a table"))),
    };
    call(lua, name, callback, args, caller)
}

fn list(lua: &Lua, (): ()) -> mlua::Result<Table> {
    let list = lua.create_table()?;
    let commands = commands(lua);
    for (name, command) in &commands.0 {
        let entry = lua.create_table()?;
        entry.set("name", name.as_str())?;
        entry.set("desc", command.desc.as_deref())?;
        entry.set(
            "args",
            lua.create_sequence_from(command.args.iter().cloned())?,
        )?;
        list.push(entry)?;
    }
    Ok(list)
}
