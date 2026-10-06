use gband_core::action::{Action, ClientAction, SessionCommand};
use gband_core::geometry::placed;
use gband_core::input::{Key, KeyCode};
use gband_core::layout::{
    BandId, Place, Proportion, SessionAction, Weight, WindowContent, WindowHeight, WindowId,
};
use gband_core::view::ViewAction;
use mlua::{Lua, Table, Value};

use crate::api::{self, Dispatch, WindowInput};
use crate::error::ConfigError;
use crate::keys::parse_key;
use crate::options::{self, Width};
use crate::plugin_windows;
use crate::removed;
use crate::runtime::is_loading;
use crate::ui::{self, is_control};

pub(crate) fn install(lua: &Lua, gband: &Table) -> mlua::Result<()> {
    gband.set("layout", lua.create_function(layout)?)?;
    gband.set("view", lua.create_function(view)?)?;
    let window = lua.create_table()?;
    window.set("focus", lua.create_function(focus)?)?;
    window.set("set_width", lua.create_function(set_width)?)?;
    window.set("set_height", lua.create_function(set_height)?)?;
    window.set("set_position", lua.create_function(set_position)?)?;
    window.set("send_keys", lua.create_function(send_keys)?)?;
    window.set("send_text", lua.create_function(send_text)?)?;
    window.set("paste", lua.create_function(paste)?)?;
    gband.set("window", window)?;
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

pub(crate) fn window(lua: &Lua, value: &Value) -> Result<WindowId, String> {
    let Some(number) = identifier(value) else {
        return Err(format!(
            "expected a window number, found {}",
            field_name(value)
        ));
    };
    let window = WindowId(number);
    if !ui::current_state(lua).layout.contains(window) {
        return Err(format!("no window {number} is in the layout"));
    }
    Ok(window)
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

pub(crate) type OpenTarget = Option<(BandId, Option<WindowId>)>;

pub(crate) fn open_target(
    lua: &Lua,
    band_value: &Value,
    after: &Value,
) -> Result<OpenTarget, String> {
    let after = match after {
        Value::Nil => None,
        value => Some(window(lua, value)?),
    };
    let layout = ui::current_state(lua).layout;
    let holding = match after {
        None => None,
        Some(after) => match layout.place(after) {
            Some(Place::Tiled(location)) => Some(layout.bands()[location.band].id),
            _ => return Err(format!("window {after} is not a tiled window")),
        },
    };
    match (band_value, after.zip(holding)) {
        (Value::Nil, None) => Ok(None),
        (Value::Nil, Some((after, holding))) => Ok(Some((holding, Some(after)))),
        (value, after) => {
            let band = band(lua, value)?;
            if let Some((after, holding)) = after
                && holding != band
            {
                return Err(format!("window {after} is not in band {band}"));
            }
            Ok(Some((band, after.map(|(after, _)| after))))
        }
    }
}

fn get(target: &Table, field: &str) -> Result<Value, String> {
    target.get(field).map_err(|error| error.to_string())
}

fn open_window_target(lua: &Lua, target: &Table, name: &str) -> Result<Dispatch, String> {
    fields(target, &["band", "after", "floating"], name)?;
    let band = get(target, "band")?;
    let after = get(target, "after")?;
    let floating = match get(target, "floating")? {
        Value::Nil => false,
        Value::Boolean(floating) => floating,
        other => {
            return Err(format!(
                "the `floating` of `{name}` must be a boolean, found {}",
                other.type_name()
            ));
        }
    };
    if !floating {
        let (band, after) = open_target(lua, &band, &after)?
            .ok_or_else(|| format!("the target of `{name}` must name `band` or `after`"))?;
        return Ok(Dispatch::Session(SessionAction::open(band, after, None)));
    }
    if !after.is_nil() {
        return Err(format!(
            "the target of `{name}` cannot hold `after` with `floating = true`"
        ));
    }
    let band = match band {
        Value::Nil => BandId(ui::current_state(lua).band.number),
        value => self::band(lua, &value)?,
    };
    Ok(Dispatch::Session(SessionAction::OpenWindow {
        band,
        after: None,
        width: None,
        floating: true,
        focus: true,
        content: WindowContent::Program(None),
    }))
}

fn toggle_floating_target(lua: &Lua, target: &Table, name: &str) -> Result<Dispatch, String> {
    fields(target, &["window", "after"], name)?;
    let value = get(target, "window")?;
    if value.is_nil() {
        return Err(format!("the target of `{name}` must name a `window`"));
    }
    let window = self::window(lua, &value)?;
    let after = match get(target, "after")? {
        Value::Nil => None,
        value => Some(self::window(lua, &value)?),
    };
    let layout = ui::current_state(lua).layout;
    let band_of = |window: WindowId| match layout.place(window) {
        Some(Place::Tiled(location)) => Some(location.band),
        Some(Place::Floating { band, .. }) => Some(band),
        None => None,
    };
    if let Some(after) = after
        && !matches!(layout.place(after), Some(Place::Tiled(location)) if Some(location.band) == band_of(window))
    {
        return Err(format!(
            "window {after} is not a tiled window of the band holding window {window}"
        ));
    }
    Ok(Dispatch::Session(SessionAction::ToggleFloating {
        window,
        after,
    }))
}

fn fields(target: &Table, allowed: &[&str], action: &str) -> Result<(), String> {
    for pair in target.pairs::<Value, Value>() {
        let (name, _) = pair.map_err(|error| error.to_string())?;
        let known =
            matches!(&name, Value::String(text) if allowed.iter().any(|field| *text == *field));
        if !known {
            let name = field_name(&name);
            return Err(removed::message(&name)
                .unwrap_or_else(|| format!("the target of `{action}` takes no field `{name}`")));
        }
    }
    Ok(())
}

fn window_target(lua: &Lua, target: &Table, action: &str) -> Result<WindowId, String> {
    fields(target, &["window"], action)?;
    let value: Value = target.get("window").map_err(|error| error.to_string())?;
    if value.is_nil() {
        return Err(format!("the target of `{action}` must name a `window`"));
    }
    window(lua, &value)
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
        Action::Session(SessionCommand::OpenWindow) => open_window_target(lua, target, name),
        Action::Session(SessionCommand::ToggleFloating) => {
            toggle_floating_target(lua, target, name)
        }
        Action::Session(command) => {
            let window = window_target(lua, target, name)?;
            let action = command
                .on_window(window)
                .expect("every command but open window names a window");
            Ok(Dispatch::Session(action))
        }
        _ => {
            let window = window_target(lua, target, name)?;
            Ok(Dispatch::Input {
                window,
                input: WindowInput::Key(options::current(lua).prefix),
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
            let windows = lua.create_table()?;
            for (&window, height) in column.windows.iter().zip(&column.heights) {
                let item = lua.create_table()?;
                item.set("id", window.0)?;
                match *height {
                    WindowHeight::Fixed(rows) => item.set("rows", rows)?,
                    WindowHeight::Auto(weight) => {
                        item.set("weight", f64::from(weight.num()) / f64::from(weight.den()))?
                    }
                }
                item.set(
                    "plugin_window",
                    plugin_windows::plugin_window_of(lua, window)?,
                )?;
                windows.push(item)?;
            }
            described.set("windows", windows)?;
            columns.push(described)?;
        }
        entry.set("columns", columns)?;
        let floating = lua.create_table()?;
        for record in &band.floating {
            let placed = placed(record, state.area);
            let item = lua.create_table()?;
            item.set("id", record.window.0)?;
            item.set("width", fraction(record.width))?;
            item.set("full_width", record.full_width)?;
            item.set("rows", record.rows)?;
            item.set("col", placed.x)?;
            item.set("row", placed.y)?;
            item.set(
                "plugin_window",
                plugin_windows::plugin_window_of(lua, record.window)?,
            )?;
            floating.push(item)?;
        }
        entry.set("floating", floating)?;
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
    table.set("window", state.window)?;
    let floating = state
        .window
        .is_some_and(|window| state.layout.floating(WindowId(window)).is_some());
    table.set("floating", floating)?;
    table.set("plugin_window", plugin_windows::focused(lua)?)?;
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
    let what = "gband.window.focus";
    dispatching(lua, what)?;
    let window = checked(lua, what, window(lua, &target))?;
    let action = Action::View(ViewAction::FocusWindow(window));
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
    let what = "gband.window.set_width";
    dispatching(lua, what)?;
    let window = checked(lua, what, window(lua, &target))?;
    let width = checked(lua, what, width(&value))?;
    api::queue(
        lua,
        Dispatch::Session(SessionAction::SetWidth { window, width }),
        what,
    )
}

fn height(value: &Value) -> Result<WindowHeight, String> {
    let Value::Table(table) = value else {
        return Err("expected a table holding `rows` or `weight`".to_owned());
    };
    let mut found = None;
    for pair in table.pairs::<Value, Value>() {
        let (name, value) = pair.map_err(|error| error.to_string())?;
        let height = match &name {
            Value::String(text) if text == "rows" => match whole(&value) {
                Some(rows) if rows >= 1 => {
                    WindowHeight::Fixed(rows.min(i64::from(u16::MAX)) as u16)
                }
                _ => return Err("`rows` must be an integer of at least 1".to_owned()),
            },
            Value::String(text) if text == "weight" => {
                let weight = width(&value).map_err(|message| format!("`weight`: {message}"))?;
                WindowHeight::Auto(Weight::new(weight.num, weight.den))
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
    let what = "gband.window.set_height";
    dispatching(lua, what)?;
    let window = checked(lua, what, window(lua, &target))?;
    let height = checked(lua, what, height(&value))?;
    api::queue(
        lua,
        Dispatch::Session(SessionAction::SetHeight { window, height }),
        what,
    )
}

fn position(value: &Value) -> Result<(u16, u16), String> {
    let Value::Table(table) = value else {
        return Err(format!(
            "expected a table holding `col` and `row`, found {}",
            value.type_name()
        ));
    };
    fields(table, &["col", "row"], "the position")
        .map_err(|_| "the position takes only `col` and `row`".to_owned())?;
    let coordinate = |field: &str| -> Result<u16, String> {
        match get(table, field)? {
            Value::Nil => Err(format!("the position must hold `{field}`")),
            value => match whole(&value) {
                Some(cells) if cells >= 0 => Ok(cells.min(i64::from(u16::MAX)) as u16),
                _ => Err(format!("`{field}` must be an integer of at least 0")),
            },
        }
    };
    Ok((coordinate("col")?, coordinate("row")?))
}

fn set_position(lua: &Lua, (target, value): (Value, Value)) -> mlua::Result<()> {
    let what = "gband.window.set_position";
    dispatching(lua, what)?;
    let window = checked(lua, what, window(lua, &target))?;
    if ui::current_state(lua).layout.floating(window).is_none() {
        return Err(ConfigError::raise(
            lua,
            format!("{what}: window {window} is not a floating window"),
        ));
    }
    let (col, row) = checked(lua, what, position(&value))?;
    api::queue(
        lua,
        Dispatch::Session(SessionAction::SetPosition { window, col, row }),
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
    let what = "gband.window.send_keys";
    dispatching(lua, what)?;
    let window = checked(lua, what, window(lua, &target))?;
    let keys = checked(lua, what, key_names(&keys))?;
    let entries = keys
        .into_iter()
        .map(|key| Dispatch::Input {
            window,
            input: WindowInput::Key(key),
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
    let what = "gband.window.send_text";
    dispatching(lua, what)?;
    let window = checked(lua, what, window(lua, &target))?;
    let text = checked(lua, what, text(&value))?;
    let keys = checked(lua, what, text_keys(&text))?;
    let entries = keys
        .into_iter()
        .map(|key| Dispatch::Input {
            window,
            input: WindowInput::Key(key),
        })
        .collect();
    queue(lua, entries, what)
}

fn paste(lua: &Lua, (target, value): (Value, Value)) -> mlua::Result<()> {
    let what = "gband.window.paste";
    dispatching(lua, what)?;
    let window = checked(lua, what, window(lua, &target))?;
    let text = checked(lua, what, text(&value))?;
    api::queue(
        lua,
        Dispatch::Input {
            window,
            input: WindowInput::Paste(text),
        },
        what,
    )
}
