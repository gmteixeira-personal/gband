use mlua::{Function, Lua, Table, Value};

use crate::control::whole;
use crate::error::ConfigError;

pub(crate) fn fail(lua: &Lua, what: &str, message: impl std::fmt::Display) -> mlua::Error {
    ConfigError::raise(lua, format!("{what}: {message}"))
}

pub(crate) fn checked<T>(lua: &Lua, what: &str, result: Result<T, String>) -> mlua::Result<T> {
    result.map_err(|message| fail(lua, what, message))
}

fn expected<T>(
    lua: &Lua,
    what: &str,
    value: &Value,
    description: &str,
    read: Option<T>,
) -> mlua::Result<T> {
    read.ok_or_else(|| {
        fail(
            lua,
            what,
            format!("expected {description}, found {}", value.type_name()),
        )
    })
}

pub(crate) fn text(
    lua: &Lua,
    what: &str,
    value: &Value,
    description: &str,
) -> mlua::Result<String> {
    let read = match value {
        Value::String(text) => Some(text.to_string_lossy()),
        _ => None,
    };
    expected(lua, what, value, description, read)
}

pub(crate) fn optional_text(
    lua: &Lua,
    what: &str,
    value: &Value,
    description: &str,
) -> mlua::Result<Option<String>> {
    match value {
        Value::Nil => Ok(None),
        value => text(lua, what, value, description).map(Some),
    }
}

pub(crate) fn function(
    lua: &Lua,
    what: &str,
    value: &Value,
    description: &str,
) -> mlua::Result<Function> {
    let read = match value {
        Value::Function(function) => Some(function.clone()),
        _ => None,
    };
    expected(lua, what, value, description, read)
}

pub(crate) fn table(
    lua: &Lua,
    what: &str,
    value: &Value,
    description: &str,
) -> mlua::Result<Table> {
    let read = match value {
        Value::Table(table) => Some(table.clone()),
        _ => None,
    };
    expected(lua, what, value, description, read)
}

pub(crate) fn boolean(
    lua: &Lua,
    what: &str,
    value: &Value,
    description: &str,
) -> mlua::Result<bool> {
    let read = match value {
        Value::Boolean(flag) => Some(*flag),
        _ => None,
    };
    expected(lua, what, value, description, read)
}

pub(crate) fn integer<T: TryFrom<i64>>(
    lua: &Lua,
    what: &str,
    value: &Value,
    description: &str,
) -> mlua::Result<T> {
    let read = whole(value).and_then(|number| T::try_from(number).ok());
    expected(lua, what, value, description, read)
}

pub(crate) fn optional_integer<T: TryFrom<i64>>(
    lua: &Lua,
    what: &str,
    value: &Value,
    description: &str,
) -> mlua::Result<Option<T>> {
    match value {
        Value::Nil => Ok(None),
        value => integer(lua, what, value, description).map(Some),
    }
}

fn read_field<T>(
    table: &Table,
    name: &str,
    description: &str,
    read: impl FnOnce(&Value) -> Option<T>,
) -> Result<T, String> {
    let value: Value = table.get(name).map_err(|error| error.to_string())?;
    read(&value).ok_or_else(|| {
        format!(
            "the field `{name}` must be {description}, found {}",
            value.type_name()
        )
    })
}

pub(crate) fn whole_field<T: TryFrom<i64>>(
    table: &Table,
    name: &str,
    description: &str,
) -> Result<T, String> {
    read_field(table, name, description, |value| {
        whole(value).and_then(|number| T::try_from(number).ok())
    })
}

pub(crate) fn number_field(table: &Table, name: &str, description: &str) -> Result<f64, String> {
    read_field(table, name, description, |value| match *value {
        Value::Integer(number) => Some(number as f64),
        Value::Number(number) => Some(number),
        _ => None,
    })
}

pub(crate) fn text_field(table: &Table, name: &str, description: &str) -> Result<String, String> {
    read_field(table, name, description, |value| match value {
        Value::String(text) => Some(text.to_string_lossy()),
        _ => None,
    })
}

pub(crate) fn optional_text_field(
    table: &Table,
    name: &str,
    description: &str,
) -> Result<Option<String>, String> {
    read_field(table, name, description, |value| match value {
        Value::Nil => Some(None),
        Value::String(text) => Some(Some(text.to_string_lossy())),
        _ => None,
    })
}

pub(crate) fn boolean_field(table: &Table, name: &str, description: &str) -> Result<bool, String> {
    read_field(table, name, description, |value| match value {
        Value::Boolean(flag) => Some(*flag),
        _ => None,
    })
}

pub(crate) fn optional_boolean_field(
    table: &Table,
    name: &str,
    description: &str,
) -> Result<Option<bool>, String> {
    read_field(table, name, description, |value| match value {
        Value::Nil => Some(None),
        Value::Boolean(flag) => Some(Some(*flag)),
        _ => None,
    })
}

pub(crate) fn table_field(table: &Table, name: &str, description: &str) -> Result<Table, String> {
    read_field(table, name, description, |value| match value {
        Value::Table(table) => Some(table.clone()),
        _ => None,
    })
}

pub(crate) fn optional_whole_field<T: TryFrom<i64>>(
    table: &Table,
    name: &str,
    description: &str,
) -> Result<Option<T>, String> {
    match table
        .get::<Value>(name)
        .map_err(|error| error.to_string())?
    {
        Value::Nil => Ok(None),
        _ => whole_field(table, name, description).map(Some),
    }
}
