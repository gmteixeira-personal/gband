use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use gband_core::action::Action;
use gband_core::input::{Modifiers, MouseButton, WheelDirection};
use gband_core::layout::{BandId, Proportion, WindowId};
use gband_core::view::ViewAction;
use mlua::{Function, IntoLuaMulti, Lua, MultiValue, RegistryKey, Table, Value};

use crate::api::{self, Dispatch, PluginWindowRequest};
use crate::border::{Border, BorderChars, Sides};
use crate::check;
use crate::control;
use crate::events::{BoxCell, button_name, direction_name};
use crate::keys::{key_name, parse_key};
use crate::ui::{self, Style, named, strip};

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
    pub hover: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginBox {
    pub col: u16,
    pub row: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginMouseKind {
    Press(MouseButton),
    Release(MouseButton),
    Drag(MouseButton),
    Scroll(WheelDirection),
    Move,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginMouse {
    pub kind: PluginMouseKind,
    pub content: Option<(u16, u16)>,
    pub boxed: Option<BoxCell>,
    pub modifiers: Modifiers,
}

impl PluginMouse {
    pub(crate) fn to_lua(self, lua: &Lua) -> mlua::Result<Table> {
        let event = lua.create_table()?;
        let (kind, button) = match self.kind {
            PluginMouseKind::Press(button) => ("press", Some(button)),
            PluginMouseKind::Release(button) => ("release", Some(button)),
            PluginMouseKind::Drag(button) => ("drag", Some(button)),
            PluginMouseKind::Scroll(direction) => {
                event.set("direction", direction_name(direction))?;
                ("scroll", None)
            }
            PluginMouseKind::Move => ("move", None),
        };
        event.set("kind", kind)?;
        event.set("button", button.map(button_name))?;
        event.set("content_col", self.content.map(|(col, _)| col))?;
        event.set("content_row", self.content.map(|(_, row)| row))?;
        BoxCell::fields(self.boxed, &event)?;
        event.set("ctrl", self.modifiers.ctrl)?;
        event.set("alt", self.modifiers.alt)?;
        event.set("shift", self.modifiers.shift)?;
        Ok(event)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TiledFrame {
    pub cols: u16,
    pub rows: u16,
    pub base: Style,
    pub lines: Vec<Vec<Run>>,
    pub hover: bool,
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

pub(crate) fn install(lua: &Lua, core: &Table) -> mlua::Result<()> {
    lua.set_app_data(Counter(Arc::new(AtomicU32::new(1))));
    lua.set_app_data(Presented::default());
    lua.set_app_data(Hooks::default());
    core.set(
        "next_window",
        lua.create_function(|lua, ()| Ok(counter(lua).fetch_add(1, Ordering::Relaxed)))?,
    )?;
    core.set(
        "present_window",
        lua.create_function(|lua, (id, frame): (Value, Value)| {
            let what = named("present_window");
            let id: u32 = check::integer(lua, &what, &id, "a plugin window number")?;
            let frame = check::table(lua, &what, &frame, "a frame table")?;
            let frame = check::checked(lua, &what, read_frame(&frame))?;
            presented(lua).0.insert(id, Some(frame));
            Ok(())
        })?,
    )?;
    core.set(
        "forget_window",
        lua.create_function(|lua, id: Value| {
            let id: u32 =
                check::integer(lua, &named("forget_window"), &id, "a plugin window number")?;
            presented(lua).0.insert(id, None);
            Ok(())
        })?,
    )?;
    core.set("request", lua.create_function(request)?)?;
    core.set(
        "parse_key",
        lua.create_function(|lua, name: Value| {
            let name = check::text(lua, &named("parse_key"), &name, "a key name as a string")?;
            Ok(parse_key(&name).ok().map(key_name))
        })?,
    )?;
    core.set(
        "dispatching",
        lua.create_function(|lua, ()| Ok(api::in_callback(lua)))?,
    )?;
    core.set(
        "focus_window",
        lua.create_function(|lua, window: Value| {
            let what = named("focus_window");
            let window: u32 = check::integer(lua, &what, &window, "a window number")?;
            let action = Action::View(ViewAction::FocusWindow(WindowId(window)));
            api::queue(lua, Dispatch::Action(action), &what)
        })?,
    )?;
    core.set("open_target", lua.create_function(open_target)?)?;
    core.set(
        "border",
        lua.create_function(|lua, spec: Value| {
            let table = check::table(lua, &named("border"), &spec, "a border table")?;
            match border_table(&table) {
                Ok(border) => border_value(lua, &border)?.into_lua_multi(lua),
                Err(message) => (Value::Nil, message).into_lua_multi(lua),
            }
        })?,
    )?;
    core.set(
        "width",
        lua.create_function(|lua, value: Value| match control::width(&value) {
            Ok(width) => (width.num, width.den).into_lua_multi(lua),
            Err(message) => (Value::Nil, message).into_lua_multi(lua),
        })?,
    )?;
    Ok(())
}

pub(crate) fn provide(lua: &Lua, implementation: Table) -> mlua::Result<()> {
    let key = lua.create_registry_value(implementation)?;
    lua.app_data_mut::<Hooks>()
        .expect("the window hooks are installed with the runtime")
        .0 = Some(key);
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

fn request(lua: &Lua, entry: Value) -> mlua::Result<()> {
    let what = named("request");
    let entry = check::table(lua, &what, &entry, "a request entry table")?;
    let request = check::checked(lua, &what, read_request(&entry))?;
    api::queue(lua, Dispatch::PluginWindow(request), "gband.win")
}

const POSITIVE: &str = "a positive integer";

fn read_request(entry: &Table) -> Result<PluginWindowRequest, String> {
    let plugin_window = check::whole_field(entry, "id", "a plugin window number")?;
    match check::text_field(entry, "op", "`open` or `close`")?.as_str() {
        "open" => {
            let band: Option<u32> = check::optional_whole_field(entry, "band", "a band number")?;
            let after: Option<u32> =
                check::optional_whole_field(entry, "after", "a window number")?;
            let num: Option<u32> = check::optional_whole_field(entry, "num", POSITIVE)?;
            let den: Option<u32> = check::optional_whole_field(entry, "den", POSITIVE)?;
            if num == Some(0) || den == Some(0) {
                return Err(format!(
                    "the fields `num` and `den` must each be {POSITIVE}"
                ));
            }
            Ok(PluginWindowRequest::Open {
                plugin_window,
                target: band.map(|band| (BandId(band), after.map(WindowId))),
                width: num.zip(den).map(|(num, den)| Proportion::new(num, den)),
                focus: check::optional_boolean_field(entry, "focus", "a boolean")?.unwrap_or(false),
            })
        }
        "close" => Ok(PluginWindowRequest::Close { plugin_window }),
        other => Err(format!(
            "the field `op` must be `open` or `close`, found `{other}`"
        )),
    }
}

pub(crate) fn read_runs(owner: &Table) -> Result<Vec<Vec<Run>>, String> {
    let lines = check::table_field(owner, "lines", "a list of lines")?;
    lines
        .sequence_values::<Value>()
        .enumerate()
        .map(|(row, line)| {
            let at = |reason: String| format!("line {}: {reason}", row + 1);
            let Value::Table(line) = line.map_err(|error| error.to_string())? else {
                return Err(at("a line must be a list of runs".to_owned()));
            };
            line.sequence_values::<Value>()
                .enumerate()
                .map(|(index, run)| {
                    let at = |reason: String| at(format!("run {}: {reason}", index + 1));
                    let Value::Table(run) = run.map_err(|error| error.to_string())? else {
                        return Err(at("a run must be a table".to_owned()));
                    };
                    Ok(Run {
                        text: strip(&check::text_field(&run, "text", "a string").map_err(at)?),
                        style: ui::style_field(&run, "style").map_err(at)?,
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

fn read_border(value: Value) -> Result<Option<Border>, String> {
    Ok(match value {
        Value::Nil | Value::Boolean(false) => None,
        Value::Boolean(true) => Some(Border::default()),
        Value::Table(table) => {
            Some(border_table(&table).map_err(|reason| format!("the field `border`: {reason}"))?)
        }
        other => {
            return Err(format!(
                "the field `border` must be a boolean or a table, found {}",
                other.type_name()
            ));
        }
    })
}

const CELLS: &str = "an integer from 0 to 65535";

fn read_frame(frame: &Table) -> Result<Frame, String> {
    let base = ui::style_field(frame, "base")?;
    let lines = read_runs(frame)?;
    let hover = check::optional_boolean_field(frame, "hover", "a boolean")?.unwrap_or(false);
    Ok(
        match check::text_field(frame, "kind", "`floating` or `tiled`")?.as_str() {
            "floating" => Frame::Floating(FloatingFrame {
                row: check::whole_field(frame, "row", CELLS)?,
                col: check::whole_field(frame, "col", CELLS)?,
                width: check::whole_field(frame, "width", CELLS)?,
                height: check::whole_field(frame, "height", CELLS)?,
                border: read_border(frame.get("border").map_err(|error| error.to_string())?)?,
                title: check::optional_text_field(frame, "title", "a string")?
                    .map(|title| strip(&title)),
                base,
                border_style: ui::style_field(frame, "border_style")?,
                title_style: ui::style_field(frame, "title_style")?,
                lines,
                z: check::whole_field(frame, "z", "a non-negative integer")?,
                focused: check::boolean_field(frame, "focused", "a boolean")?,
                hover,
            }),
            "tiled" => Frame::Tiled(TiledFrame {
                cols: check::whole_field(frame, "cols", CELLS)?,
                rows: check::whole_field(frame, "rows", CELLS)?,
                base,
                lines,
                hover,
            }),
            other => {
                return Err(format!(
                    "the field `kind` must be `floating` or `tiled`, found `{other}`"
                ));
            }
        },
    )
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
