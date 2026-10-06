use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use gband_core::action::Action;
use gband_core::layout::{BandId, Proportion, WindowId};
use gband_core::view::ViewAction;
use mlua::{Function, IntoLuaMulti, Lua, MultiValue, RegistryKey, Table, Value};

use crate::api::{self, Dispatch, PluginWindowRequest};
use crate::border::{Border, BorderChars, Sides};
use crate::control;
use crate::keys::{key_name, parse_key};
use crate::ui::{self, Style, strip};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    pub text: String,
    pub style: Style,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FloatingFrame {
    pub row: u16,
    pub col: u16,
    pub width: u16,
    pub height: u16,
    pub border: Option<Border>,
    pub title: Option<String>,
    pub base: Style,
    pub border_style: Style,
    pub title_style: Style,
    pub lines: Vec<Vec<Run>>,
    pub z: u64,
    pub focused: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TiledFrame {
    pub cols: u16,
    pub rows: u16,
    pub base: Style,
    pub lines: Vec<Vec<Run>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Frame {
    Floating(FloatingFrame),
    Tiled(TiledFrame),
}

struct Counter(Arc<AtomicU32>);

#[derive(Default)]
struct Presented(BTreeMap<u32, Option<Frame>>);

#[derive(Default)]
struct Hooks(Option<RegistryKey>);

pub(crate) fn install(lua: &Lua, host: &Table) -> mlua::Result<()> {
    lua.set_app_data(Counter(Arc::new(AtomicU32::new(1))));
    lua.set_app_data(Presented::default());
    lua.set_app_data(Hooks::default());
    host.set(
        "next_window",
        lua.create_function(|lua, ()| Ok(counter(lua).fetch_add(1, Ordering::Relaxed)))?,
    )?;
    host.set(
        "present_window",
        lua.create_function(|lua, (id, frame): (u32, Table)| {
            let frame = read_frame(&frame)?;
            presented(lua).0.insert(id, Some(frame));
            Ok(())
        })?,
    )?;
    host.set(
        "forget_window",
        lua.create_function(|lua, id: u32| {
            presented(lua).0.insert(id, None);
            Ok(())
        })?,
    )?;
    host.set("request", lua.create_function(request)?)?;
    host.set(
        "parse_key",
        lua.create_function(|_, name: String| Ok(parse_key(&name).ok().map(key_name)))?,
    )?;
    host.set(
        "window_hooks",
        lua.create_function(|lua, table: Table| {
            let key = lua.create_registry_value(table)?;
            lua.app_data_mut::<Hooks>()
                .expect("the window hooks are installed with the runtime")
                .0 = Some(key);
            Ok(())
        })?,
    )?;
    host.set(
        "dispatching",
        lua.create_function(|lua, ()| Ok(api::in_callback(lua)))?,
    )?;
    host.set("open_target", lua.create_function(open_target)?)?;
    host.set(
        "border",
        lua.create_function(|lua, table: Table| match border_table(&table) {
            Ok(border) => border_value(lua, &border)?.into_lua_multi(lua),
            Err(message) => (Value::Nil, message).into_lua_multi(lua),
        })?,
    )?;
    host.set(
        "width",
        lua.create_function(|lua, value: Value| match control::width(&value) {
            Ok(width) => (width.num, width.den).into_lua_multi(lua),
            Err(message) => (Value::Nil, message).into_lua_multi(lua),
        })?,
    )?;
    host.set(
        "focus_window",
        lua.create_function(|lua, window: u32| {
            let action = Action::View(ViewAction::FocusWindow(WindowId(window)));
            api::queue(lua, Dispatch::Action(action), "gband.win.focus")
        })?,
    )?;
    Ok(())
}

fn counter(lua: &Lua) -> Arc<AtomicU32> {
    Arc::clone(
        &lua.app_data_ref::<Counter>()
            .expect("the window counter is installed with the runtime")
            .0,
    )
}

pub(crate) fn set_counter(lua: &Lua, shared: Arc<AtomicU32>) {
    lua.set_app_data(Counter(shared));
}

fn presented(lua: &Lua) -> mlua::AppDataRefMut<'_, Presented> {
    lua.app_data_mut::<Presented>()
        .expect("the window frames are installed with the runtime")
}

pub(crate) fn take(lua: &Lua) -> Vec<(u32, Option<Frame>)> {
    std::mem::take(&mut presented(lua).0).into_iter().collect()
}

fn open_target(lua: &Lua, (band, after): (Value, Value)) -> mlua::Result<MultiValue> {
    match control::open_target(lua, &band, &after) {
        Ok(None) => (true, Value::Nil, Value::Nil).into_lua_multi(lua),
        Ok(Some((band, after))) => (true, band.0, after.map(|window| window.0)).into_lua_multi(lua),
        Err(message) => (false, message).into_lua_multi(lua),
    }
}

fn request(lua: &Lua, entry: Table) -> mlua::Result<()> {
    let plugin_window: u32 = entry.get("id")?;
    let request = match entry.get::<String>("op")?.as_str() {
        "open" => {
            let band: Option<u32> = entry.get("band")?;
            let after: Option<u32> = entry.get("after")?;
            let num: Option<u32> = entry.get("num")?;
            let den: Option<u32> = entry.get("den")?;
            PluginWindowRequest::Open {
                plugin_window,
                target: band.map(|band| (BandId(band), after.map(WindowId))),
                width: num.zip(den).map(|(num, den)| Proportion::new(num, den)),
                focus: entry.get("focus")?,
            }
        }
        _ => PluginWindowRequest::Close { plugin_window },
    };
    api::queue(lua, Dispatch::PluginWindow(request), "gband.win")
}

pub(crate) fn read_runs(lines: &Table) -> mlua::Result<Vec<Vec<Run>>> {
    lines
        .sequence_values::<Table>()
        .map(|line| {
            line?
                .sequence_values::<Table>()
                .map(|run| {
                    let run = run?;
                    Ok(Run {
                        text: strip(&run.get::<mlua::LuaString>("text")?.to_string_lossy()),
                        style: ui::style(&run.get("style")?)?,
                    })
                })
                .collect()
        })
        .collect()
}

fn border_table(table: &Table) -> Result<Border, String> {
    let mut border = Border::default();
    for pair in table.pairs::<Value, Value>() {
        let (field, value) = pair.map_err(|error| error.to_string())?;
        let name = match &field {
            Value::String(name) => Some(name.to_string_lossy()),
            _ => None,
        };
        match name.as_deref() {
            Some("sides") => {
                let Value::Table(list) = value else {
                    return Err("`sides` must be a list of side names".to_owned());
                };
                let names = list
                    .sequence_values::<String>()
                    .collect::<mlua::Result<Vec<_>>>()
                    .map_err(|_| "`sides` must be a list of side names".to_owned())?;
                border.sides = Sides::parse(names.iter().map(String::as_str))?;
            }
            Some("chars") => {
                border.chars = match value {
                    Value::String(name) => BorderChars::named(&name.to_string_lossy()),
                    Value::Table(list) => BorderChars::custom(
                        list.sequence_values::<String>()
                            .collect::<mlua::Result<Vec<_>>>()
                            .map_err(|_| "`chars` must be a list of 8 strings".to_owned())?,
                    ),
                    _ => {
                        return Err(
                            "`chars` must be a character set name or a list of 8 strings"
                                .to_owned(),
                        );
                    }
                }
                .map_err(|reason| format!("`chars`: {reason}"))?;
            }
            _ => {
                return Err(format!(
                    "a border table takes no field `{}`",
                    control::field_name(&field)
                ));
            }
        }
    }
    Ok(border)
}

fn border_value(lua: &Lua, border: &Border) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("sides", border.sides.to_lua(lua)?)?;
    table.set("chars", border.chars.to_lua(lua)?)?;
    Ok(table)
}

fn read_border(value: Value) -> mlua::Result<Option<Border>> {
    Ok(match value {
        Value::Nil | Value::Boolean(false) => None,
        Value::Boolean(true) => Some(Border::default()),
        Value::Table(table) => Some(border_table(&table).map_err(mlua::Error::runtime)?),
        other => {
            return Err(mlua::Error::runtime(format!(
                "a frame border must be a boolean or a table, found {}",
                other.type_name()
            )));
        }
    })
}

fn read_frame(frame: &Table) -> mlua::Result<Frame> {
    let base = ui::style(&frame.get("base")?)?;
    let lines = read_runs(&frame.get("lines")?)?;
    Ok(match frame.get::<String>("kind")?.as_str() {
        "floating" => Frame::Floating(FloatingFrame {
            row: frame.get("row")?,
            col: frame.get("col")?,
            width: frame.get("width")?,
            height: frame.get("height")?,
            border: read_border(frame.get("border")?)?,
            title: frame
                .get::<Option<mlua::LuaString>>("title")?
                .map(|title| strip(&title.to_string_lossy())),
            base,
            border_style: ui::style(&frame.get("border_style")?)?,
            title_style: ui::style(&frame.get("title_style")?)?,
            lines,
            z: frame.get("z")?,
            focused: frame.get("focused")?,
        }),
        _ => Frame::Tiled(TiledFrame {
            cols: frame.get("cols")?,
            rows: frame.get("rows")?,
            base,
            lines,
        }),
    })
}

fn hook(lua: &Lua, name: &str) -> mlua::Result<Option<Function>> {
    let table = match &lua
        .app_data_ref::<Hooks>()
        .expect("the window hooks are installed with the runtime")
        .0
    {
        Some(key) => lua.registry_value::<Table>(key)?,
        None => return Ok(None),
    };
    table.get(name)
}

pub(crate) fn call<R: mlua::FromLuaMulti + Default>(
    lua: &Lua,
    name: &str,
    args: impl mlua::IntoLuaMulti,
) -> mlua::Result<R> {
    match hook(lua, name)? {
        Some(function) => function.call(args),
        None => Ok(R::default()),
    }
}

pub(crate) fn plugin_window_of(lua: &Lua, window: WindowId) -> mlua::Result<Option<u32>> {
    call(lua, "plugin_window_of", window.0)
}

pub(crate) fn focused(lua: &Lua) -> mlua::Result<Option<u32>> {
    call(lua, "focused", ())
}

pub(crate) fn ribbon_resized(lua: &Lua) -> mlua::Result<()> {
    call(lua, "ribbon_resized", ())
}

pub(crate) fn flush(lua: &Lua) -> mlua::Result<()> {
    call(lua, "flush", ())
}
