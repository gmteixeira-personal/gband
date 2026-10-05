use std::collections::BTreeMap;
use std::sync::Arc;

use gband_core::geometry::{Size, placed};
use gband_core::layout::{BandId, Layout, PaneId};
use gband_protocol::{Key, Value as Data};
use mlua::{Function, Lua, MultiValue, Table, Value};

use crate::api;
use crate::error::ConfigError;
use crate::value;

pub const MAX_STATE: usize = 64 * 1024;

pub fn state_size(state: &BTreeMap<String, Data>) -> usize {
    state
        .iter()
        .map(|(key, value)| Key::string(key.as_str()).encoded_len() + value.encoded_len())
        .sum()
}

pub struct SessionView {
    pub layout: Layout,
    pub area: Size,
    pub clients: Vec<u64>,
}

pub trait Host: Send + Sync {
    fn sessions(&self) -> Vec<String>;
    fn session(&self, name: &str) -> Option<SessionView>;
    fn pane_state(&self, session: &str, pane: PaneId) -> Option<BTreeMap<String, Data>>;
    fn set_pane_state(&self, session: &str, pane: PaneId, key: &str, value: Option<Data>);
    fn emit(&self, name: String, data: Data, session: Option<String>);
}

pub struct Caller {
    pub session: String,
    pub client: u64,
    pub focus: Arc<dyn Fn(PaneId) + Send + Sync>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    SessionCreated {
        session: String,
    },
    SessionEnded {
        session: String,
    },
    PaneOpened {
        session: String,
        pane: PaneId,
        band: BandId,
    },
    PaneClosed {
        session: String,
        pane: PaneId,
        band: BandId,
    },
    PaneExited {
        session: String,
        pane: PaneId,
        code: Option<u32>,
        signal: Option<i32>,
    },
    PaneOutput {
        session: String,
        pane: PaneId,
        data: Vec<u8>,
    },
    PaneInput {
        session: String,
        pane: PaneId,
        client: u64,
    },
    ClientAttached {
        session: String,
        client: u64,
    },
    ClientDetached {
        session: String,
        client: u64,
    },
    ConfigReloaded,
}

pub(crate) const NAMES: [&str; 10] = [
    "SessionCreated",
    "SessionEnded",
    "PaneOpened",
    "PaneClosed",
    "PaneExited",
    "PaneOutput",
    "PaneInput",
    "ClientAttached",
    "ClientDetached",
    "ConfigReloaded",
];

impl Event {
    pub fn name(&self) -> &'static str {
        match self {
            Event::SessionCreated { .. } => "SessionCreated",
            Event::SessionEnded { .. } => "SessionEnded",
            Event::PaneOpened { .. } => "PaneOpened",
            Event::PaneClosed { .. } => "PaneClosed",
            Event::PaneExited { .. } => "PaneExited",
            Event::PaneOutput { .. } => "PaneOutput",
            Event::PaneInput { .. } => "PaneInput",
            Event::ClientAttached { .. } => "ClientAttached",
            Event::ClientDetached { .. } => "ClientDetached",
            Event::ConfigReloaded => "ConfigReloaded",
        }
    }

    pub(crate) fn payload(&self, lua: &Lua) -> mlua::Result<Table> {
        let payload = lua.create_table()?;
        match self {
            Event::SessionCreated { session } | Event::SessionEnded { session } => {
                payload.set("session", session.as_str())?;
            }
            Event::PaneOpened {
                session,
                pane,
                band,
            }
            | Event::PaneClosed {
                session,
                pane,
                band,
            } => {
                payload.set("session", session.as_str())?;
                payload.set("pane", pane.0)?;
                payload.set("band", band.0)?;
            }
            Event::PaneExited {
                session,
                pane,
                code,
                signal,
            } => {
                payload.set("session", session.as_str())?;
                payload.set("pane", pane.0)?;
                payload.set("code", *code)?;
                payload.set("signal", *signal)?;
            }
            Event::PaneOutput {
                session,
                pane,
                data,
            } => {
                payload.set("session", session.as_str())?;
                payload.set("pane", pane.0)?;
                payload.set("data", lua.create_string(data)?)?;
            }
            Event::PaneInput {
                session,
                pane,
                client,
            } => {
                payload.set("session", session.as_str())?;
                payload.set("pane", pane.0)?;
                payload.set("client", *client)?;
            }
            Event::ClientAttached { session, client }
            | Event::ClientDetached { session, client } => {
                payload.set("session", session.as_str())?;
                payload.set("client", *client)?;
            }
            Event::ConfigReloaded => {}
        }
        Ok(payload)
    }
}

#[derive(Default)]
struct Slot(Option<Arc<dyn Host>>);

pub(crate) fn set_host(lua: &Lua, host: Arc<dyn Host>) {
    lua.set_app_data(Slot(Some(host)));
}

fn host(lua: &Lua) -> Option<Arc<dyn Host>> {
    lua.app_data_ref::<Slot>().and_then(|slot| slot.0.clone())
}

pub(crate) fn install(lua: &Lua, gband: &Table) -> mlua::Result<()> {
    lua.set_app_data(Slot::default());
    gband.set("sessions", lua.create_function(sessions)?)?;
    gband.set("session", lua.create_function(session)?)?;
    gband.set("pane_state", lua.create_function(pane_state)?)?;
    Ok(())
}

fn text(lua: &Lua, value: &Value, what: &str) -> mlua::Result<String> {
    match value {
        Value::String(text) if !text.as_bytes().is_empty() => Ok(text.to_str()?.to_owned()),
        _ => Err(ConfigError::raise(
            lua,
            format!("{what} must be a non-empty string"),
        )),
    }
}

fn number(lua: &Lua, value: &Value, what: &str) -> mlua::Result<u32> {
    match *value {
        Value::Integer(number) => u32::try_from(number).ok(),
        Value::Number(number) if number.fract() == 0.0 && number >= 0.0 => {
            u32::try_from(number as i64).ok()
        }
        _ => None,
    }
    .ok_or_else(|| ConfigError::raise(lua, format!("{what} must be a pane number")))
}

fn sessions(lua: &Lua, (): ()) -> mlua::Result<Table> {
    let mut names = host(lua).map(|host| host.sessions()).unwrap_or_default();
    names.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    lua.create_sequence_from(names)
}

fn session(lua: &Lua, name: Value) -> mlua::Result<Option<Table>> {
    let name = text(lua, &name, "the session name")?;
    let Some(view) = host(lua).and_then(|host| host.session(&name)) else {
        return Ok(None);
    };
    let table = lua.create_table()?;
    table.set("name", name)?;
    let bands = lua.create_table()?;
    for band in view.layout.bands() {
        let entry = lua.create_table()?;
        entry.set("band", band.id.0)?;
        let columns = lua.create_table()?;
        for column in &band.columns {
            let described = lua.create_table()?;
            described.set(
                "width",
                f64::from(column.width.num) / f64::from(column.width.den),
            )?;
            described.set("full_width", column.full_width)?;
            let panes = lua.create_table()?;
            for pane in &column.panes {
                let item = lua.create_table()?;
                item.set("pane", pane.0)?;
                panes.push(item)?;
            }
            described.set("panes", panes)?;
            columns.push(described)?;
        }
        entry.set("columns", columns)?;
        let floating = lua.create_table()?;
        for record in &band.floating {
            let placed = placed(record, view.area);
            let item = lua.create_table()?;
            item.set("pane", record.pane.0)?;
            item.set(
                "width",
                f64::from(record.width.num) / f64::from(record.width.den),
            )?;
            item.set("full_width", record.full_width)?;
            item.set("rows", record.rows)?;
            item.set("col", placed.x)?;
            item.set("row", placed.y)?;
            floating.push(item)?;
        }
        entry.set("floating", floating)?;
        bands.push(entry)?;
    }
    table.set("bands", bands)?;
    let mut clients = view.clients;
    clients.sort_unstable();
    table.set("clients", lua.create_sequence_from(clients)?)?;
    Ok(Some(table))
}

fn pane_state(lua: &Lua, (session, pane): (Value, Value)) -> mlua::Result<Option<Table>> {
    let session = text(lua, &session, "the session of gband.pane_state")?;
    let pane = PaneId(number(lua, &pane, "the pane of gband.pane_state")?);
    let Some(host) = host(lua) else {
        return Ok(None);
    };
    if host.pane_state(&session, pane).is_none() {
        return Ok(None);
    }
    proxy(lua, host, session, pane).map(Some)
}

fn proxy(lua: &Lua, host: Arc<dyn Host>, session: String, pane: PaneId) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    let meta = lua.create_table()?;
    let (reader, writer, lister) = (Arc::clone(&host), Arc::clone(&host), host);
    let (read_session, write_session, list_session) = (session.clone(), session.clone(), session);
    meta.set(
        "__index",
        lua.create_function(move |lua, (_, key): (Value, Value)| {
            let Value::String(key) = key else {
                return Ok(Value::Nil);
            };
            let key = key.to_str()?.to_owned();
            match reader
                .pane_state(&read_session, pane)
                .and_then(|mut state| state.remove(&key))
            {
                Some(value) => value::into_lua(lua, &value),
                None => Ok(Value::Nil),
            }
        })?,
    )?;
    meta.set(
        "__newindex",
        lua.create_function(move |lua, (_, key, value): (Value, Value, Value)| {
            let key = match &key {
                Value::String(key) if !key.as_bytes().is_empty() => key.to_str()?.to_owned(),
                _ => {
                    return Err(ConfigError::raise(
                        lua,
                        "a pane state key must be a non-empty string",
                    ));
                }
            };
            let value = match value {
                Value::Nil => None,
                value => Some(
                    value::from_lua(&value, &format!("state.{key}"))
                        .map_err(|message| ConfigError::raise(lua, message))?,
                ),
            };
            let Some(mut state) = writer.pane_state(&write_session, pane) else {
                return Ok(());
            };
            let previous = match &value {
                Some(value) => state.insert(key.clone(), value.clone()),
                None => state.remove(&key),
            };
            if previous == value {
                return Ok(());
            }
            if state_size(&state) > MAX_STATE {
                return Err(ConfigError::raise(
                    lua,
                    format!(
                        "setting `{key}` would make the state of pane {pane} larger than 64 KiB"
                    ),
                ));
            }
            writer.set_pane_state(&write_session, pane, &key, value);
            Ok(())
        })?,
    )?;
    meta.set(
        "__pairs",
        lua.create_function(move |lua, _: Value| {
            let snapshot = lua.create_table()?;
            for (key, value) in lister.pane_state(&list_session, pane).unwrap_or_default() {
                snapshot.set(key, value::into_lua(lua, &value)?)?;
            }
            let next: Function = lua.globals().get("next")?;
            Ok(MultiValue::from_iter([
                Value::Function(next),
                Value::Table(snapshot),
                Value::Nil,
            ]))
        })?,
    )?;
    table.set_metatable(Some(meta))?;
    Ok(table)
}

pub(crate) fn emit(lua: &Lua, (name, data, opts): (Value, Value, Value)) -> mlua::Result<()> {
    let name = text(lua, &name, "the event name of gband.emit")?;
    let data =
        value::from_lua(&data, "data").map_err(|message| ConfigError::raise(lua, message))?;
    let session = match opts {
        Value::Nil => None,
        Value::Table(opts) => {
            let mut session = None;
            for pair in opts.pairs::<Value, Value>() {
                let (field, value) = pair?;
                match &field {
                    Value::String(field) if field == "session" => {
                        session = Some(text(lua, &value, "`session`")?);
                    }
                    other => {
                        return Err(ConfigError::raise(
                            lua,
                            format!(
                                "gband.emit takes only the option `session`, found `{}`",
                                crate::control::field_name(other)
                            ),
                        ));
                    }
                }
            }
            session
        }
        _ => {
            return Err(ConfigError::raise(
                lua,
                "gband.emit expects its options as a table",
            ));
        }
    };
    if !api::in_callback(lua) {
        return Err(api::outside_callback(lua, "gband.emit"));
    }
    if let Some(host) = host(lua) {
        host.emit(name, data, session);
    }
    Ok(())
}
