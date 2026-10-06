use std::collections::BTreeMap;

use gband_core::layout::{BandId, WindowId};
use gband_protocol::Value as Data;
use mlua::{Lua, Table, Value};

use crate::Side;
use crate::api;
use crate::callbacks::{self, CallbackId};
use crate::error::ConfigError;
use crate::runtime::side;
use crate::{server, ui, value};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Attached {
        session: String,
    },
    FocusChanged {
        window: Option<WindowId>,
        previous: Option<WindowId>,
    },
    BandChanged {
        band: BandId,
        previous: BandId,
    },
    WindowOpened {
        window: WindowId,
        band: BandId,
    },
    WindowClosed {
        window: WindowId,
        band: BandId,
    },
    LayoutChanged,
    TerminalResized {
        cols: u16,
        rows: u16,
    },
    ConfigReloaded,
    KeyTableChanged {
        table: String,
        previous: String,
    },
    HighlightChanged {
        group: String,
    },
    ColorschemeChanged {
        name: String,
        previous: String,
    },
    ServerEvent {
        name: String,
        data: Data,
        queued: bool,
        time: u64,
    },
    WindowStateChanged {
        window: WindowId,
        key: String,
        value: Option<Data>,
        previous: Option<Data>,
    },
}

const USER: &str = "User";
const SERVER_EVENT: &str = "ServerEvent";

pub(crate) const NAMES: [&str; 13] = [
    "Attached",
    "FocusChanged",
    "BandChanged",
    "WindowOpened",
    "WindowClosed",
    "LayoutChanged",
    "TerminalResized",
    "ConfigReloaded",
    "KeyTableChanged",
    "HighlightChanged",
    "ColorschemeChanged",
    "ServerEvent",
    "WindowStateChanged",
];

fn names(side: Side) -> &'static [&'static str] {
    match side {
        Side::Client => &NAMES,
        Side::Server => &server::NAMES,
        Side::Test => &[],
    }
}

impl Event {
    pub fn name(&self) -> &'static str {
        match self {
            Event::Attached { .. } => "Attached",
            Event::FocusChanged { .. } => "FocusChanged",
            Event::BandChanged { .. } => "BandChanged",
            Event::WindowOpened { .. } => "WindowOpened",
            Event::WindowClosed { .. } => "WindowClosed",
            Event::LayoutChanged => "LayoutChanged",
            Event::TerminalResized { .. } => "TerminalResized",
            Event::ConfigReloaded => "ConfigReloaded",
            Event::KeyTableChanged { .. } => "KeyTableChanged",
            Event::HighlightChanged { .. } => "HighlightChanged",
            Event::ColorschemeChanged { .. } => "ColorschemeChanged",
            Event::ServerEvent { .. } => SERVER_EVENT,
            Event::WindowStateChanged { .. } => "WindowStateChanged",
        }
    }

    fn pattern(&self) -> Option<&str> {
        match self {
            Event::ServerEvent { name, .. } => Some(name),
            _ => None,
        }
    }

    fn payload(&self, lua: &Lua) -> mlua::Result<Table> {
        let payload = lua.create_table()?;
        match self {
            Event::Attached { session } => payload.set("session", session.as_str())?,
            Event::FocusChanged { window, previous } => {
                payload.set("window", window.map(|window| window.0))?;
                payload.set("previous", previous.map(|window| window.0))?;
            }
            Event::BandChanged { band, previous } => {
                payload.set("band", band.0)?;
                payload.set("previous", previous.0)?;
            }
            Event::WindowOpened { window, band } | Event::WindowClosed { window, band } => {
                payload.set("window", window.0)?;
                payload.set("band", band.0)?;
            }
            Event::TerminalResized { cols, rows } => {
                payload.set("cols", *cols)?;
                payload.set("rows", *rows)?;
            }
            Event::LayoutChanged | Event::ConfigReloaded => {}
            Event::KeyTableChanged { table, previous } => {
                payload.set("table", table.as_str())?;
                payload.set("previous", previous.as_str())?;
            }
            Event::HighlightChanged { group } => payload.set("group", group.as_str())?,
            Event::ColorschemeChanged { name, previous } => {
                payload.set("name", name.as_str())?;
                payload.set("previous", previous.as_str())?;
            }
            Event::ServerEvent {
                name,
                data,
                queued,
                time,
            } => {
                payload.set("name", name.as_str())?;
                payload.set("data", value::into_lua(lua, data)?)?;
                payload.set("queued", *queued)?;
                payload.set("time", *time)?;
            }
            Event::WindowStateChanged {
                window,
                key,
                value,
                previous,
            } => {
                let data = |value: &Option<Data>| match value {
                    Some(value) => value::into_lua(lua, value),
                    None => Ok(Value::Nil),
                };
                payload.set("window", window.0)?;
                payload.set("key", key.as_str())?;
                payload.set("value", data(value)?)?;
                payload.set("previous", data(previous)?)?;
            }
        }
        Ok(payload)
    }
}

struct Handler {
    id: u64,
    event: String,
    callback: CallbackId,
    group: Option<i64>,
    once: bool,
    pattern: Option<String>,
}

#[derive(Default)]
struct Events {
    handlers: Vec<Handler>,
    groups: BTreeMap<String, i64>,
    next: u64,
}

fn events(lua: &Lua) -> mlua::AppDataRefMut<'_, Events> {
    lua.app_data_mut::<Events>()
        .expect("events are installed with the runtime")
}

pub(crate) fn install(lua: &Lua, gband: &Table, side: Side) -> mlua::Result<()> {
    lua.set_app_data(Events::default());
    gband.set("on", lua.create_function(on)?)?;
    gband.set("augroup", lua.create_function(augroup)?)?;
    match side {
        Side::Client => gband.set("emit", lua.create_function(emit)?)?,
        Side::Server => gband.set("emit", lua.create_function(server::emit)?)?,
        Side::Test => {}
    }
    Ok(())
}

fn known(lua: &Lua, event: &str) -> Result<(), String> {
    let side = side(lua);
    if names(side).contains(&event) || (side == Side::Client && event == USER) {
        return Ok(());
    }
    if names(side.other()).contains(&event) || event == USER {
        return Err(format!(
            "`{event}` is a {} event; this is the {}",
            side.other().name(),
            side.name()
        ));
    }
    Err(format!("unknown event `{event}`"))
}

fn on(lua: &Lua, (event, function, opts): (Value, Value, Value)) -> mlua::Result<()> {
    let event = match &event {
        Value::String(event) => event.to_str()?.to_owned(),
        _ => {
            return Err(ConfigError::raise(
                lua,
                "gband.on expects an event name as a string",
            ));
        }
    };
    known(lua, &event).map_err(|message| ConfigError::raise(lua, message))?;
    let Value::Function(function) = function else {
        return Err(ConfigError::raise(
            lua,
            format!("the handler of `{event}` must be a function"),
        ));
    };
    let (group, once, pattern) = match opts {
        Value::Nil => (None, false, None),
        Value::Table(opts) => (
            group(lua, opts.get("group")?)?,
            match opts.get::<Value>("once")? {
                Value::Nil => false,
                Value::Boolean(once) => once,
                _ => return Err(ConfigError::raise(lua, "`once` must be a boolean")),
            },
            match opts.get::<Value>("pattern")? {
                Value::Nil => None,
                Value::String(pattern) if event == USER || event == SERVER_EVENT => {
                    Some(pattern.to_str()?.to_owned())
                }
                Value::String(_) => {
                    return Err(ConfigError::raise(
                        lua,
                        format!(
                            "`pattern` applies only to `User` and `ServerEvent` events, not `{event}`"
                        ),
                    ));
                }
                _ => return Err(ConfigError::raise(lua, "`pattern` must be a string")),
            },
        ),
        _ => {
            return Err(ConfigError::raise(
                lua,
                "gband.on expects its options as a table",
            ));
        }
    };
    let callback = callbacks::register(lua, function)?;
    let mut events = events(lua);
    let id = events.next;
    events.next += 1;
    events.handlers.push(Handler {
        id,
        event,
        callback,
        group,
        once,
        pattern,
    });
    Ok(())
}

fn group(lua: &Lua, value: Value) -> mlua::Result<Option<i64>> {
    let events = events(lua);
    let known = match &value {
        Value::Nil => return Ok(None),
        Value::Integer(id) => events
            .groups
            .values()
            .any(|known| known == id)
            .then_some(*id),
        Value::String(name) => events.groups.get(&*name.to_str()?).copied(),
        _ => {
            drop(events);
            return Err(ConfigError::raise(
                lua,
                "`group` must be a group id or name",
            ));
        }
    };
    drop(events);
    known
        .map(Some)
        .ok_or_else(|| ConfigError::raise(lua, "unknown handler group"))
}

fn augroup(lua: &Lua, (name, opts): (Value, Value)) -> mlua::Result<i64> {
    let name = match &name {
        Value::String(name) if !name.as_bytes().is_empty() => name.to_str()?.to_owned(),
        _ => {
            return Err(ConfigError::raise(
                lua,
                "gband.augroup expects a name as a non-empty string",
            ));
        }
    };
    let clear = match opts {
        Value::Nil => true,
        Value::Table(opts) => match opts.get::<Value>("clear")? {
            Value::Nil => true,
            Value::Boolean(clear) => clear,
            _ => return Err(ConfigError::raise(lua, "`clear` must be a boolean")),
        },
        _ => {
            return Err(ConfigError::raise(
                lua,
                "gband.augroup expects its options as a table",
            ));
        }
    };
    let mut events = events(lua);
    let next = i64::try_from(events.groups.len()).unwrap_or(i64::MAX) + 1;
    let id = *events.groups.entry(name).or_insert(next);
    if clear {
        events.handlers.retain(|handler| handler.group != Some(id));
    }
    Ok(id)
}

fn emit(lua: &Lua, (name, data): (Value, Value)) -> mlua::Result<()> {
    let name = match &name {
        Value::String(name) if !name.as_bytes().is_empty() => name.to_str()?.to_owned(),
        _ => {
            return Err(ConfigError::raise(
                lua,
                "gband.emit expects an event name as a non-empty string",
            ));
        }
    };
    if !api::in_callback(lua) {
        return Err(api::outside_callback(lua, "gband.emit"));
    }
    deliver(lua, USER, Some(&name), |lua| {
        let payload = lua.create_table()?;
        payload.set("name", name.as_str())?;
        payload.set("data", data.clone())?;
        Ok(payload)
    })?;
    ui::after_event(lua, Some(USER))
}

pub(crate) fn emit_event(lua: &Lua, event: &Event) -> mlua::Result<()> {
    deliver(lua, event.name(), event.pattern(), |lua| event.payload(lua))
}

pub(crate) fn emit_server(lua: &Lua, event: &server::Event) -> mlua::Result<()> {
    deliver(lua, event.name(), None, |lua| event.payload(lua))
}

pub(crate) fn handles(lua: &Lua, event: &str) -> bool {
    events(lua)
        .handlers
        .iter()
        .any(|handler| handler.event == event)
}

pub(crate) fn deliver_table(lua: &Lua, event: &str, payload: &Table) -> mlua::Result<()> {
    deliver(lua, event, None, |lua| {
        let copy = lua.create_table()?;
        for pair in payload.pairs::<Value, Value>() {
            let (key, value) = pair?;
            copy.set(key, value)?;
        }
        Ok(copy)
    })
}

fn deliver(
    lua: &Lua,
    event: &str,
    name: Option<&str>,
    payload: impl Fn(&Lua) -> mlua::Result<Table>,
) -> mlua::Result<()> {
    let matching: Vec<u64> = events(lua)
        .handlers
        .iter()
        .filter(|handler| {
            handler.event == event
                && handler
                    .pattern
                    .as_deref()
                    .is_none_or(|pattern| Some(pattern) == name)
        })
        .map(|handler| handler.id)
        .collect();
    for id in matching {
        let callback = {
            let mut events = events(lua);
            let Some(index) = events.handlers.iter().position(|handler| handler.id == id) else {
                continue;
            };
            let handler = &events.handlers[index];
            let callback = handler.callback;
            if handler.once {
                events.handlers.remove(index);
            }
            callback
        };
        callbacks::run::<()>(lua, callback, payload(lua)?)?;
    }
    Ok(())
}
