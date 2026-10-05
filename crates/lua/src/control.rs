use gband_core::action::{Action, ClientAction, SessionCommand};
use gband_core::input::{Key, KeyCode};
use gband_core::layout::{BandId, PaneHeight, PaneId, Proportion, SessionAction, Weight};
use gband_core::view::ViewAction;
use mlua::{Lua, Table, Value};

use crate::api::{self, Dispatch, PaneInput};
use crate::error::ConfigError;
use crate::keys::parse_key;
use crate::options::{self, Width};
use crate::runtime::is_loading;
use crate::ui::{self, is_control};
use crate::windows;

pub(crate) fn install(lua: &Lua, gband: &Table) -> mlua::Result<()> {
    gband.set("layout", lua.create_function(layout)?)?;
    gband.set("view", lua.create_function(view)?)?;
    let pane = lua.create_table()?;
    pane.set("focus", lua.create_function(focus)?)?;
    pane.set("set_width", lua.create_function(set_width)?)?;
    pane.set("set_height", lua.create_function(set_height)?)?;
    pane.set("send_keys", lua.create_function(send_keys)?)?;
    pane.set("send_text", lua.create_function(send_text)?)?;
    pane.set("paste", lua.create_function(paste)?)?;
    gband.set("pane", pane)?;
    let band = lua.create_table()?;
    band.set("view", lua.create_function(view_band)?)?;
    gband.set("band", band)
}

pub(crate) fn field_name(value: &Value) -> String {
    match value {
        Value::String(name) => name.to_string_lossy(),
        Value::Integer(number) => number.to_string(),
        Value::Number(number) => number.to_string(),
        other => other.type_name().to_owned(),
    }
}

fn whole(value: &Value) -> Option<i64> {
    match *value {
        Value::Integer(number) => Some(number),
        Value::Number(number) if number.fract() == 0.0 && number.abs() < 1e15 => {
            Some(number as i64)
        }
        _ => None,
    }
}

fn real(value: &Value) -> Option<f64> {
    match *value {
        Value::Integer(number) => Some(number as f64),
        Value::Number(number) => Some(number),
        _ => None,
    }
}

fn identifier(value: &Value) -> Option<u32> {
    whole(value).and_then(|number| u32::try_from(number).ok())
}

pub(crate) fn pane(lua: &Lua, value: &Value) -> Result<PaneId, String> {
    let Some(number) = identifier(value) else {
        return Err(format!(
            "expected a pane number, found {}",
            field_name(value)
        ));
    };
    let pane = PaneId(number);
    if !ui::current_state(lua).layout.contains(pane) {
        return Err(format!("no pane {number} is in the layout"));
    }
    Ok(pane)
}

fn band(lua: &Lua, value: &Value) -> Result<BandId, String> {
    let Some(number) = identifier(value) else {
        return Err(format!(
            "expected a band number, found {}",
            field_name(value)
        ));
    };
    let band = BandId(number);
    if ui::current_state(lua).layout.band(band).is_none() {
        return Err(format!("no band {number} is in the layout"));
    }
    Ok(band)
}

pub(crate) type OpenTarget = Option<(BandId, Option<PaneId>)>;

pub(crate) fn open_target(
    lua: &Lua,
    band_value: &Value,
    after: &Value,
) -> Result<OpenTarget, String> {
    let after = match after {
        Value::Nil => None,
        value => Some(pane(lua, value)?),
    };
    let layout = ui::current_state(lua).layout;
    let holding = |pane: PaneId| {
        let location = layout.locate(pane).expect("checked to be in the layout");
        layout.bands()[location.band].id
    };
    match (band_value, after) {
        (Value::Nil, None) => Ok(None),
        (Value::Nil, Some(after)) => Ok(Some((holding(after), Some(after)))),
        (value, after) => {
            let band = band(lua, value)?;
            if let Some(after) = after
                && holding(after) != band
            {
                return Err(format!("pane {after} is not in band {band}"));
            }
            Ok(Some((band, after)))
        }
    }
}

fn fields(target: &Table, allowed: &[&str], action: &str) -> Result<(), String> {
    for pair in target.pairs::<Value, Value>() {
        let (name, _) = pair.map_err(|error| error.to_string())?;
        let known =
            matches!(&name, Value::String(text) if allowed.iter().any(|field| *text == *field));
        if !known {
            return Err(format!(
                "the target of `{action}` takes no field `{}`",
                field_name(&name)
            ));
        }
    }
    Ok(())
}

fn pane_target(lua: &Lua, target: &Table, action: &str) -> Result<PaneId, String> {
    fields(target, &["pane"], action)?;
    let value: Value = target.get("pane").map_err(|error| error.to_string())?;
    if value.is_nil() {
        return Err(format!("the target of `{action}` must name a `pane`"));
    }
    pane(lua, &value)
}

pub(crate) fn targeted(
    lua: &Lua,
    name: &str,
    action: Action,
    target: &Value,
) -> Result<Dispatch, String> {
    let takes_target = matches!(
        action,
        Action::Session(_) | Action::Client(ClientAction::SendPrefix)
    );
    if !takes_target {
        return Err(format!("`{name}` takes no target"));
    }
    let Value::Table(target) = target else {
        return Err(format!(
            "the target of `{name}` must be a table, found {}",
            target.type_name()
        ));
    };
    match action {
        Action::Session(SessionCommand::OpenPane) => {
            fields(target, &["band", "after"], name)?;
            let band: Value = target.get("band").map_err(|error| error.to_string())?;
            let after: Value = target.get("after").map_err(|error| error.to_string())?;
            let (band, after) = open_target(lua, &band, &after)?
                .ok_or_else(|| format!("the target of `{name}` must name `band` or `after`"))?;
            Ok(Dispatch::Session(SessionAction::open(band, after, None)))
        }
        Action::Session(command) => {
            let pane = pane_target(lua, target, name)?;
            let action = command
                .on_pane(pane)
                .expect("every command but open pane names a pane");
            Ok(Dispatch::Session(action))
        }
        _ => {
            let pane = pane_target(lua, target, name)?;
            Ok(Dispatch::Input {
                pane,
                input: PaneInput::Key(options::current(lua).prefix),
            })
        }
    }
}

fn dispatching(lua: &Lua, what: &str) -> mlua::Result<()> {
    if api::in_callback(lua) {
        Ok(())
    } else {
        Err(api::outside_callback(lua, what))
    }
}

fn loaded(lua: &Lua, what: &str) -> mlua::Result<()> {
    if is_loading(lua) {
        Err(ConfigError::raise(
            lua,
            format!("{what} cannot be called while the configuration loads"),
        ))
    } else {
        Ok(())
    }
}

fn fraction(proportion: Proportion) -> f64 {
    f64::from(proportion.num) / f64::from(proportion.den)
}

fn layout(lua: &Lua, (): ()) -> mlua::Result<Table> {
    loaded(lua, "gband.layout")?;
    let state = ui::current_state(lua);
    let table = lua.create_table()?;
    table.set("cols", state.area.cols)?;
    table.set("rows", state.area.rows)?;
    let bands = lua.create_table()?;
    for band in state.layout.bands() {
        let entry = lua.create_table()?;
        entry.set("id", band.id.0)?;
        let columns = lua.create_table()?;
        for column in &band.columns {
            let described = lua.create_table()?;
            described.set("width", fraction(column.width))?;
            described.set("full_width", column.full_width)?;
            let panes = lua.create_table()?;
            for (&pane, height) in column.panes.iter().zip(&column.heights) {
                let item = lua.create_table()?;
                item.set("id", pane.0)?;
                match *height {
                    PaneHeight::Fixed(rows) => item.set("rows", rows)?,
                    PaneHeight::Auto(weight) => {
                        item.set("weight", f64::from(weight.num()) / f64::from(weight.den()))?
                    }
                }
                item.set("window", windows::pane_window(lua, pane)?)?;
                panes.push(item)?;
            }
            described.set("panes", panes)?;
            columns.push(described)?;
        }
        entry.set("columns", columns)?;
        bands.push(entry)?;
    }
    table.set("bands", bands)?;
    Ok(table)
}

fn view(lua: &Lua, (): ()) -> mlua::Result<Table> {
    loaded(lua, "gband.view")?;
    let state = ui::current_state(lua);
    let table = lua.create_table()?;
    table.set("band", state.band.number)?;
    table.set("pane", state.pane)?;
    table.set("window", windows::focused(lua)?)?;
    table.set("table", state.table)?;
    table.set("cols", state.ribbon.cols)?;
    table.set("rows", state.ribbon.rows)?;
    Ok(table)
}

fn queue(lua: &Lua, entries: Vec<Dispatch>, what: &str) -> mlua::Result<()> {
    for entry in entries {
        api::queue(lua, entry, what)?;
    }
    Ok(())
}

fn checked<T>(lua: &Lua, what: &str, result: Result<T, String>) -> mlua::Result<T> {
    result.map_err(|message| ConfigError::raise(lua, format!("{what}: {message}")))
}

fn focus(lua: &Lua, target: Value) -> mlua::Result<()> {
    let what = "gband.pane.focus";
    dispatching(lua, what)?;
    let pane = checked(lua, what, pane(lua, &target))?;
    let action = Action::View(ViewAction::FocusPane(pane));
    api::queue(lua, Dispatch::Action(action), what)
}

fn view_band(lua: &Lua, target: Value) -> mlua::Result<()> {
    let what = "gband.band.view";
    dispatching(lua, what)?;
    let band = checked(lua, what, band(lua, &target))?;
    let action = Action::View(ViewAction::ViewBand(band));
    api::queue(lua, Dispatch::Action(action), what)
}

pub(crate) fn width(value: &Value) -> Result<Proportion, String> {
    let Some(number) = real(value) else {
        return Err(format!(
            "expected a width as a number, found {}",
            field_name(value)
        ));
    };
    Width::from_number(number).map(|Width(width)| width)
}

fn set_width(lua: &Lua, (target, value): (Value, Value)) -> mlua::Result<()> {
    let what = "gband.pane.set_width";
    dispatching(lua, what)?;
    let pane = checked(lua, what, pane(lua, &target))?;
    let width = checked(lua, what, width(&value))?;
    api::queue(
        lua,
        Dispatch::Session(SessionAction::SetWidth { pane, width }),
        what,
    )
}

fn height(value: &Value) -> Result<PaneHeight, String> {
    let Value::Table(table) = value else {
        return Err("expected a table holding `rows` or `weight`".to_owned());
    };
    let mut found = None;
    for pair in table.pairs::<Value, Value>() {
        let (name, value) = pair.map_err(|error| error.to_string())?;
        let height = match &name {
            Value::String(text) if text == "rows" => match whole(&value) {
                Some(rows) if rows >= 1 => PaneHeight::Fixed(rows.min(i64::from(u16::MAX)) as u16),
                _ => return Err("`rows` must be an integer of at least 1".to_owned()),
            },
            Value::String(text) if text == "weight" => {
                let weight = width(&value).map_err(|message| format!("`weight`: {message}"))?;
                PaneHeight::Auto(Weight::new(weight.num, weight.den))
            }
            _ => return Err(format!("unknown field `{}`", field_name(&name))),
        };
        if found.replace(height).is_some() {
            return Err("the height must hold exactly one of `rows` and `weight`".to_owned());
        }
    }
    found.ok_or_else(|| "the height must hold `rows` or `weight`".to_owned())
}

fn set_height(lua: &Lua, (target, value): (Value, Value)) -> mlua::Result<()> {
    let what = "gband.pane.set_height";
    dispatching(lua, what)?;
    let pane = checked(lua, what, pane(lua, &target))?;
    let height = checked(lua, what, height(&value))?;
    api::queue(
        lua,
        Dispatch::Session(SessionAction::SetHeight { pane, height }),
        what,
    )
}

fn key_names(value: &Value) -> Result<Vec<Key>, String> {
    let parse = |name: &mlua::LuaString| {
        let name = name.to_string_lossy();
        parse_key(&name).map_err(|error| error.to_string())
    };
    match value {
        Value::String(name) => Ok(vec![parse(name)?]),
        Value::Table(list) => {
            let length = list.raw_len();
            if list.pairs::<Value, Value>().count() != length {
                return Err("expected a key name or a list of key names".to_owned());
            }
            (1..=length)
                .map(|index| match list.raw_get::<Value>(index) {
                    Ok(Value::String(name)) => parse(&name),
                    _ => Err("expected a key name or a list of key names".to_owned()),
                })
                .collect()
        }
        _ => Err("expected a key name or a list of key names".to_owned()),
    }
}

fn send_keys(lua: &Lua, (target, keys): (Value, Value)) -> mlua::Result<()> {
    let what = "gband.pane.send_keys";
    dispatching(lua, what)?;
    let pane = checked(lua, what, pane(lua, &target))?;
    let keys = checked(lua, what, key_names(&keys))?;
    let entries = keys
        .into_iter()
        .map(|key| Dispatch::Input {
            pane,
            input: PaneInput::Key(key),
        })
        .collect();
    queue(lua, entries, what)
}

fn text_keys(text: &str) -> Result<Vec<Key>, String> {
    text.chars()
        .map(|c| match c {
            '\n' | '\r' => Ok(Key::plain(KeyCode::Enter)),
            '\t' => Ok(Key::plain(KeyCode::Tab)),
            c if is_control(c) => Err(format!(
                "the control character U+{:04X} cannot be sent as text",
                u32::from(c)
            )),
            c => Ok(Key::plain(KeyCode::Char(c))),
        })
        .collect()
}

fn text(value: &Value) -> Result<String, String> {
    match value {
        Value::String(text) => Ok(text.to_string_lossy()),
        other => Err(format!("expected a string, found {}", other.type_name())),
    }
}

fn send_text(lua: &Lua, (target, value): (Value, Value)) -> mlua::Result<()> {
    let what = "gband.pane.send_text";
    dispatching(lua, what)?;
    let pane = checked(lua, what, pane(lua, &target))?;
    let text = checked(lua, what, text(&value))?;
    let keys = checked(lua, what, text_keys(&text))?;
    let entries = keys
        .into_iter()
        .map(|key| Dispatch::Input {
            pane,
            input: PaneInput::Key(key),
        })
        .collect();
    queue(lua, entries, what)
}

fn paste(lua: &Lua, (target, value): (Value, Value)) -> mlua::Result<()> {
    let what = "gband.pane.paste";
    dispatching(lua, what)?;
    let pane = checked(lua, what, pane(lua, &target))?;
    let text = checked(lua, what, text(&value))?;
    api::queue(
        lua,
        Dispatch::Input {
            pane,
            input: PaneInput::Paste(text),
        },
        what,
    )
}
