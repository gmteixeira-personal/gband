use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use gband_core::layout::WindowId;
use gband_protocol::Value as Data;
use mlua::{Function, Lua, RegistryKey, Table, Value};

use crate::api::{self, Dispatch};
use crate::error::ConfigError;
use crate::guard;
use crate::options::{self, NotifyStyle};
use crate::owner;
use crate::ui::{self, is_control};
use crate::value;

const MAX_NOTIFICATION: usize = 1024;
const MAX_CLIPBOARD: usize = 1024 * 1024;

struct Call {
    callback: Option<RegistryKey>,
    owner: Option<String>,
}

#[derive(Default)]
struct Pending(HashMap<u64, Call>);

pub(crate) fn install(lua: &Lua, gband: &Table) -> mlua::Result<()> {
    lua.set_app_data(Pending::default());
    gband.set("rpc", lua.create_function(rpc)?)?;
    gband.set("window_state", lua.create_function(window_state)?)?;
    gband.set("notify", lua.create_function(notify)?)?;
    gband.set("bell", lua.create_function(bell)?)?;
    gband.set("clipboard", lua.create_function(clipboard)?)?;
    gband.set("open", lua.create_function(open)?)?;
    Ok(())
}

fn require_callback(lua: &Lua, what: &str) -> mlua::Result<()> {
    if api::in_callback(lua) {
        Ok(())
    } else {
        Err(api::outside_callback(lua, what))
    }
}

fn rpc(lua: &Lua, (name, args, callback): (Value, Value, Value)) -> mlua::Result<()> {
    let name = match &name {
        Value::String(name) if !name.as_bytes().is_empty() => name.to_str()?.to_owned(),
        _ => {
            return Err(ConfigError::raise(
                lua,
                "gband.rpc expects a command name as a non-empty string",
            ));
        }
    };
    let args = match &args {
        Value::Nil | Value::Table(_) => {
            value::from_lua(&args, "args").map_err(|message| ConfigError::raise(lua, message))?
        }
        other => {
            return Err(ConfigError::raise(
                lua,
                format!(
                    "the arguments of gband.rpc must be a table, found {}",
                    other.type_name()
                ),
            ));
        }
    };
    let callback = match callback {
        Value::Nil => None,
        Value::Function(function) => Some(lua.create_registry_value(function)?),
        other => {
            return Err(ConfigError::raise(
                lua,
                format!(
                    "the callback of gband.rpc must be a function, found {}",
                    other.type_name()
                ),
            ));
        }
    };
    require_callback(lua, "gband.rpc")?;
    static CALLS: AtomicU64 = AtomicU64::new(1);
    let call = CALLS.fetch_add(1, Ordering::Relaxed);
    lua.app_data_mut::<Pending>()
        .expect("pending calls are installed with the runtime")
        .0
        .insert(
            call,
            Call {
                callback,
                owner: owner::current(lua),
            },
        );
    api::queue(lua, Dispatch::Call { call, name, args }, "gband.rpc")
}

pub(crate) fn answer(lua: &Lua, call: u64, result: Result<Data, String>) -> mlua::Result<()> {
    let Some(Call { callback, owner }) = lua
        .app_data_mut::<Pending>()
        .and_then(|mut pending| pending.0.remove(&call))
    else {
        return Ok(());
    };
    let Some(callback) = callback else {
        return Ok(());
    };
    let function: Function = lua.registry_value(&callback)?;
    lua.remove_registry_value(callback)?;
    if owner::is_failed(lua, owner.as_deref()) {
        return Ok(());
    }
    let ran = guard::run(lua, owner, || match &result {
        Ok(value) => function.call::<()>((true, value::into_lua(lua, value)?)),
        Err(message) => function.call::<()>((false, message.as_str())),
    })?;
    if let Err(failure) = ran {
        guard::report(lua, failure);
    }
    Ok(())
}

fn window_state(lua: &Lua, window: Value) -> mlua::Result<Option<Table>> {
    let number = match window {
        Value::Integer(number) => u32::try_from(number).ok(),
        _ => None,
    }
    .ok_or_else(|| ConfigError::raise(lua, "gband.window_state expects a window number"))?;
    let state = ui::current_state(lua);
    let window = WindowId(number);
    if !state.layout.contains(window) {
        return Ok(None);
    }
    let table = lua.create_table()?;
    if let Some(entries) = state.states.get(&window) {
        for (key, value) in entries {
            table.set(key.as_str(), value::into_lua(lua, value)?)?;
        }
    }
    Ok(Some(table))
}

fn string(lua: &Lua, value: &Value, what: &str) -> mlua::Result<String> {
    match value {
        Value::String(text) => Ok(text.to_string_lossy()),
        other => Err(ConfigError::raise(
            lua,
            format!("{what} must be a string, found {}", other.type_name()),
        )),
    }
}

fn cut(text: String, limit: usize) -> String {
    if text.len() <= limit {
        return text;
    }
    let mut end = limit;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_owned()
}

pub fn notification(style: NotifyStyle, text: &str, title: Option<&str>) -> Vec<u8> {
    let text = cut(
        text.chars().filter(|&c| !is_control(c)).collect(),
        MAX_NOTIFICATION,
    );
    let title = title.map(|title| {
        cut(
            title
                .chars()
                .filter(|&c| !is_control(c) && c != ';')
                .collect(),
            MAX_NOTIFICATION,
        )
    });
    match style {
        NotifyStyle::Osc9 => format!("\x1b]9;{text}\x07").into_bytes(),
        NotifyStyle::Osc777 => format!(
            "\x1b]777;notify;{};{text}\x07",
            title.as_deref().unwrap_or("gband")
        )
        .into_bytes(),
        NotifyStyle::Bell => b"\x07".to_vec(),
        NotifyStyle::None => Vec::new(),
    }
}

fn notify(lua: &Lua, (text, opts): (Value, Value)) -> mlua::Result<()> {
    let text = string(lua, &text, "the text of gband.notify")?;
    let title = match opts {
        Value::Nil => None,
        Value::Table(opts) => match opts.get::<Value>("title")? {
            Value::Nil => None,
            title => Some(string(lua, &title, "the title of gband.notify")?),
        },
        other => {
            return Err(ConfigError::raise(
                lua,
                format!(
                    "the options of gband.notify must be a table, found {}",
                    other.type_name()
                ),
            ));
        }
    };
    require_callback(lua, "gband.notify")?;
    let bytes = notification(options::current(lua).notify_style, &text, title.as_deref());
    if bytes.is_empty() {
        return Ok(());
    }
    api::queue(lua, Dispatch::Write(bytes), "gband.notify")
}

fn bell(lua: &Lua, (): ()) -> mlua::Result<()> {
    require_callback(lua, "gband.bell")?;
    api::queue(lua, Dispatch::Write(b"\x07".to_vec()), "gband.bell")
}

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let joined = chunk
            .iter()
            .enumerate()
            .fold(0u32, |joined, (index, &byte)| {
                joined | u32::from(byte) << (16 - 8 * index)
            });
        for index in 0..4 {
            if index <= chunk.len() {
                encoded.push(char::from(
                    BASE64[(joined >> (18 - 6 * index) & 63) as usize],
                ));
            } else {
                encoded.push('=');
            }
        }
    }
    encoded
}

fn clipboard(lua: &Lua, text: Value) -> mlua::Result<()> {
    let Value::String(text) = text else {
        return Err(ConfigError::raise(
            lua,
            format!(
                "gband.clipboard expects a string, found {}",
                text.type_name()
            ),
        ));
    };
    let Some(sequence) = clipboard_sequence(&text.as_bytes()) else {
        return Err(ConfigError::raise(
            lua,
            "gband.clipboard takes at most 1 MiB of text",
        ));
    };
    require_callback(lua, "gband.clipboard")?;
    api::queue(lua, Dispatch::Write(sequence), "gband.clipboard")
}

pub fn clipboard_sequence(text: &[u8]) -> Option<Vec<u8>> {
    (text.len() <= MAX_CLIPBOARD).then(|| format!("\x1b]52;c;{}\x07", base64(text)).into_bytes())
}

const OPENER: &str = if cfg!(target_os = "macos") {
    "open"
} else {
    "xdg-open"
};

fn open(lua: &Lua, target: Value) -> mlua::Result<bool> {
    let target = match &target {
        Value::String(target) if !target.as_bytes().is_empty() => target.to_string_lossy(),
        _ => {
            return Err(ConfigError::raise(
                lua,
                "gband.open expects a URL or a path as a non-empty string",
            ));
        }
    };
    require_callback(lua, "gband.open")?;
    let spawned = Command::new(OPENER)
        .arg(&target)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    match spawned {
        Ok(mut child) => {
            std::thread::spawn(move || child.wait());
            Ok(true)
        }
        Err(error) => {
            tracing::warn!("cannot start {OPENER} to open {target}: {error}");
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_pads() {
        assert_eq!(base64(b"hi"), "aGk=");
        assert_eq!(base64(b"h"), "aA==");
        assert_eq!(base64(b"hey"), "aGV5");
        assert_eq!(base64(b""), "");
        assert_eq!(base64(&[0xff, 0x00, 0x80, 0x01]), "/wCAAQ==");
    }

    #[test]
    fn notification_is_cleaned_and_cut() {
        assert_eq!(
            notification(NotifyStyle::Osc9, "a\x1b]52;c;eA==\x07b", Some("x")),
            b"\x1b]9;a]52;c;eA==b\x07"
        );
        assert_eq!(
            notification(NotifyStyle::Osc777, "done", Some("a;g\nent")),
            b"\x1b]777;notify;agent;done\x07"
        );
        let long = "é".repeat(600);
        let bytes = notification(NotifyStyle::Osc9, &long, None);
        assert_eq!(bytes.len(), 4 + 1024 + 1);
    }
}
