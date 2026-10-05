use std::collections::BTreeMap;

use mlua::{Lua, Table, Value};

use crate::api::{list_of_strings, require_loading};
use crate::callbacks::{self, CallbackId, Ran};
use crate::error::ConfigError;
use crate::owner;

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
    Ok(matches!(
        callbacks::run::<()>(lua, callback, args)?,
        Ran::Returned(())
    ))
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
