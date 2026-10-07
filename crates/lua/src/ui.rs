use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gband_core::geometry::Size;
use gband_core::layout::{Layout, WindowId};
use gband_protocol::Value as Data;

use mlua::{Function, Lua, MultiValue, RegistryKey, Table, Value};
use unicode_width::UnicodeWidthChar;

use crate::check;
use crate::error::ConfigError;
use crate::events;
use crate::guard;
use crate::owner;
use crate::runtime::is_loading;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BandState {
    pub number: u32,
    pub index: u32,
    pub count: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ViewState {
    pub table: String,
    pub band: BandState,
    pub window: Option<u32>,
    pub width: u16,
    pub height: u16,
    pub error: Option<String>,
    pub errors: Vec<String>,
    pub layout: Arc<Layout>,
    pub area: Size,
    pub ribbon: Size,
    pub states: Arc<WindowStates>,
    pub names: Arc<WindowNames>,
}

pub type WindowStates = BTreeMap<WindowId, BTreeMap<String, Data>>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WindowName {
    pub shown: String,
    pub manual: Option<String>,
}

pub type WindowNames = BTreeMap<WindowId, WindowName>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Rgb(u8, u8, u8),
    Index(u8),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub reverse: bool,
    pub dim: bool,
}

pub type Rgb = (u8, u8, u8);

pub const PALETTE_COLORS: [&str; 16] = [
    "black",
    "red",
    "green",
    "yellow",
    "blue",
    "magenta",
    "cyan",
    "white",
    "bright_black",
    "bright_red",
    "bright_green",
    "bright_yellow",
    "bright_blue",
    "bright_magenta",
    "bright_cyan",
    "bright_white",
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Palette {
    pub fg: Option<Rgb>,
    pub bg: Option<Rgb>,
    pub colors: [Option<Rgb>; 16],
}

impl Palette {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    fn fields(&mut self) -> impl Iterator<Item = (&'static str, &mut Option<Rgb>)> {
        [("fg", &mut self.fg), ("bg", &mut self.bg)]
            .into_iter()
            .chain(PALETTE_COLORS.into_iter().zip(self.colors.iter_mut()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientStyles {
    pub border: Style,
    pub border_focused: Style,
    pub banner: Style,
}

impl Default for ClientStyles {
    fn default() -> Self {
        Self {
            border: Style {
                dim: true,
                ..Style::default()
            },
            border_focused: Style {
                fg: Some(Color::Rgb(0xb1, 0xb9, 0xf9)),
                bold: true,
                ..Style::default()
            },
            banner: Style {
                fg: Some(Color::Index(1)),
                reverse: true,
                ..Style::default()
            },
        }
    }
}

#[derive(Default)]
struct Look {
    palette: Palette,
    palette_dirty: bool,
    styles: Option<RegistryKey>,
    presented: Option<ClientStyles>,
}

#[derive(Default)]
struct Settings {
    reopen: Option<u32>,
    hooks: Option<RegistryKey>,
}

#[derive(Default)]
struct State {
    view: ViewState,
    cleared: bool,
}

#[derive(Default)]
struct Hooks {
    after_event: Vec<RegistryKey>,
    on_state: Vec<RegistryKey>,
}

struct Timer {
    period: Duration,
    next: Instant,
    function: RegistryKey,
}

#[derive(Default)]
struct Timers {
    next_id: i64,
    entries: BTreeMap<i64, Timer>,
}

pub fn is_control(c: char) -> bool {
    matches!(c, '\u{0}'..='\u{1f}' | '\u{7f}'..='\u{9f}')
}

pub fn strip(text: &str) -> String {
    text.chars().filter(|&c| !is_control(c)).collect()
}

fn char_width(c: char) -> usize {
    c.width().unwrap_or(0)
}

pub fn width(text: &str) -> usize {
    text.chars()
        .filter(|&c| !is_control(c))
        .map(char_width)
        .sum()
}

pub fn truncate(text: &str, limit: usize) -> String {
    let text = strip(text);
    if width(&text) <= limit {
        return text;
    }
    if limit == 0 {
        return String::new();
    }
    let mut kept = String::new();
    let mut used = 0;
    for c in text.chars() {
        let cells = char_width(c);
        if used + cells > limit - 1 {
            break;
        }
        used += cells;
        kept.push(c);
    }
    kept.push('…');
    kept
}

fn text_argument(lua: &Lua, value: &Value, function: &str) -> mlua::Result<String> {
    match value {
        Value::String(text) => Ok(text.to_string_lossy()),
        _ => Err(ConfigError::raise(
            lua,
            format!("{function} expects a string"),
        )),
    }
}

pub(crate) fn install(lua: &Lua, gband: &Table) -> mlua::Result<Table> {
    lua.set_app_data(State::default());
    lua.set_app_data(Hooks::default());
    lua.set_app_data(Timers::default());
    lua.set_app_data(Settings::default());
    lua.set_app_data(Look {
        palette_dirty: true,
        ..Look::default()
    });
    let ui = lua.create_table()?;
    ui.set(
        "width",
        lua.create_function(|lua, text: Value| {
            Ok(width(&text_argument(lua, &text, "gband.ui.width")?))
        })?,
    )?;
    ui.set(
        "truncate",
        lua.create_function(|lua, (text, limit): (Value, Value)| {
            let text = text_argument(lua, &text, "gband.ui.truncate")?;
            let limit = match limit {
                Value::Integer(limit) if limit >= 0 => limit as usize,
                Value::Number(limit) if limit >= 0.0 && limit.fract() == 0.0 => limit as usize,
                _ => {
                    return Err(ConfigError::raise(
                        lua,
                        "gband.ui.truncate expects a width as a non-negative integer",
                    ));
                }
            };
            Ok(truncate(&text, limit))
        })?,
    )?;
    gband.set("ui", ui)?;
    let core = core(lua)?;
    gband.set("core", core.clone())?;
    Ok(core)
}

const CORE: &str = "gband.core";

pub(crate) fn named(name: &str) -> String {
    format!("{CORE}.{name}")
}

fn core(lua: &Lua) -> mlua::Result<Table> {
    let core = lua.create_table()?;
    core.set(
        "owner",
        lua.create_function(|lua, ()| Ok(owner::current(lua)))?,
    )?;
    core.set(
        "loading",
        lua.create_function(|lua, ()| Ok(is_loading(lua)))?,
    )?;
    core.set(
        "failed",
        lua.create_function(|lua, plugin: Value| {
            let plugin =
                check::optional_text(lua, &named("failed"), &plugin, "a plugin name as a string")?;
            Ok(owner::is_failed(lua, plugin.as_deref()))
        })?,
    )?;
    core.set("call", lua.create_function(call)?)?;
    core.set("report", lua.create_function(report)?)?;
    core.set("emit", lua.create_function(emit)?)?;
    core.set(
        "events",
        lua.create_sequence_from(events::NAMES.iter().copied())?,
    )?;
    core.set("state", lua.create_function(state)?)?;
    core.set("timer", lua.create_function(timer)?)?;
    core.set(
        "cancel",
        lua.create_function(|lua, id: Value| {
            let id: i64 = check::integer(lua, &named("cancel"), &id, "a timer id as an integer")?;
            if let Some(timer) = timers(lua).entries.remove(&id) {
                lua.remove_registry_value(timer.function)?;
            }
            Ok(())
        })?,
    )?;
    core.set(
        "after_event",
        lua.create_function(|lua, function: Value| {
            let function = check::function(lua, &named("after_event"), &function, "a function")?;
            let key = lua.create_registry_value(function)?;
            hooks(lua).after_event.push(key);
            Ok(())
        })?,
    )?;
    core.set(
        "on_state",
        lua.create_function(|lua, function: Value| {
            let function = check::function(lua, &named("on_state"), &function, "a function")?;
            let key = lua.create_registry_value(function)?;
            hooks(lua).on_state.push(key);
            Ok(())
        })?,
    )?;
    core.set(
        "bundled",
        lua.create_function(|lua, name: Value| {
            let name = check::text(
                lua,
                &named("bundled"),
                &name,
                "a colorscheme name as a string",
            )?;
            crate::bundled::colorscheme(lua, &name)
        })?,
    )?;
    core.set(
        "load",
        lua.create_function(|lua, path: Value| {
            let path = check::text(lua, &named("load"), &path, "a file path as a string")?;
            crate::runtime::load_file(lua, &PathBuf::from(path))
        })?,
    )?;
    core.set(
        "warn",
        lua.create_function(|lua, text: Value| {
            let text = check::text(lua, &named("warn"), &text, "the text as a string")?;
            tracing::warn!("{text}");
            Ok(())
        })?,
    )?;
    let palette = lua.create_table()?;
    palette.set(
        "set",
        lua.create_function(|lua, spec: Value| {
            let what = named("palette.set");
            let spec = check::table(lua, &what, &spec, "a table of colors")?;
            let palette = check::checked(lua, &what, read_palette(&spec))?;
            set_palette(lua, palette)
        })?,
    )?;
    palette.set(
        "get",
        lua.create_function(|lua, ()| palette_table(lua, &look(lua).palette))?,
    )?;
    core.set("palette", palette)?;
    core.set(
        "reopen_settings",
        lua.create_function(|lua, line: Value| {
            settings(lua).reopen = check::optional_integer(
                lua,
                &named("reopen_settings"),
                &line,
                "a line number as a non-negative integer or nil",
            )?;
            Ok(())
        })?,
    )?;
    core.set("provide", lua.create_function(provide)?)?;
    Ok(core)
}

fn provide(lua: &Lua, (kind, implementation): (Value, Value)) -> mlua::Result<()> {
    let what = named("provide");
    let kind = check::text(lua, &what, &kind, "a provider kind as a string")?;
    let table = |description: &str| check::table(lua, &what, &implementation, description);
    match kind.as_str() {
        "windows" => crate::plugin_windows::provide(
            lua,
            table("the plugin window implementation as a table")?,
        ),
        "bars" => crate::bars::provide(lua, table("the bar implementation as a table")?),
        "settings" => {
            let implementation = table("the settings implementation as a table")?;
            settings(lua).hooks = Some(lua.create_registry_value(implementation)?);
            Ok(())
        }
        "styles" => {
            let function = check::function(lua, &what, &implementation, "the styles function")?;
            look(lua).styles = Some(lua.create_registry_value(function)?);
            Ok(())
        }
        other => Err(check::fail(
            lua,
            &what,
            format!(
                "unknown provider kind `{other}`; the kinds are `windows`, `bars`, `settings` and `styles`"
            ),
        )),
    }
}

fn settings(lua: &Lua) -> mlua::AppDataRefMut<'_, Settings> {
    lua.app_data_mut::<Settings>()
        .expect("the settings are installed with the runtime")
}

pub(crate) fn take_settings_reopen(lua: &Lua) -> Option<u32> {
    settings(lua).reopen.take()
}

pub(crate) fn open_settings(lua: &Lua, line: u32) -> mlua::Result<()> {
    let hooks = match &settings(lua).hooks {
        Some(key) => lua.registry_value::<Table>(key)?,
        None => return Ok(()),
    };
    match hooks.get::<Option<Function>>("open")? {
        Some(open) => open.call(line),
        None => Ok(()),
    }
}

fn look(lua: &Lua) -> mlua::AppDataRefMut<'_, Look> {
    lua.app_data_mut::<Look>()
        .expect("the look is installed with the runtime")
}

fn hex(text: &str) -> Option<Rgb> {
    let hex = text.strip_prefix('#').filter(|hex| hex.len() == 6)?;
    let channel = |at: usize| u8::from_str_radix(hex.get(at..at + 2)?, 16).ok();
    Some((channel(0)?, channel(2)?, channel(4)?))
}

fn read_palette(spec: &Table) -> Result<Palette, String> {
    let mut palette = Palette::default();
    for (name, slot) in palette.fields() {
        if let Some(text) = check::optional_text_field(spec, name, "a color as `#rrggbb`")? {
            *slot = Some(
                hex(&text).ok_or_else(|| format!("invalid palette color for `{name}`: {text}"))?,
            );
        }
    }
    Ok(palette)
}

fn palette_table(lua: &Lua, palette: &Palette) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    let mut palette = *palette;
    for (name, slot) in palette.fields() {
        if let Some((r, g, b)) = *slot {
            table.set(name, format!("#{r:02x}{g:02x}{b:02x}"))?;
        }
    }
    Ok(table)
}

fn set_palette(lua: &Lua, palette: Palette) -> mlua::Result<()> {
    let mut look = look(lua);
    if look.palette != palette {
        look.palette = palette;
        look.palette_dirty = true;
    }
    Ok(())
}

pub(crate) fn take_palette(lua: &Lua) -> Option<Palette> {
    let mut look = look(lua);
    std::mem::take(&mut look.palette_dirty).then_some(look.palette)
}

pub(crate) fn take_client_styles(lua: &Lua) -> mlua::Result<Option<ClientStyles>> {
    let function = match &look(lua).styles {
        Some(key) => lua.registry_value::<Function>(key)?,
        None => return Ok(None),
    };
    let table: Table = function.call(())?;
    let read = |name: &str| style_field(&table, name).map_err(mlua::Error::runtime);
    let styles = ClientStyles {
        border: read("border")?,
        border_focused: read("border_focused")?,
        banner: read("banner")?,
    };
    let mut look = look(lua);
    if look.presented == Some(styles) {
        return Ok(None);
    }
    look.presented = Some(styles);
    Ok(Some(styles))
}

fn hooks(lua: &Lua) -> mlua::AppDataRefMut<'_, Hooks> {
    lua.app_data_mut::<Hooks>()
        .expect("the hooks are installed with the runtime")
}

fn timers(lua: &Lua) -> mlua::AppDataRefMut<'_, Timers> {
    lua.app_data_mut::<Timers>()
        .expect("the timers are installed with the runtime")
}

fn call(lua: &Lua, mut args: MultiValue) -> mlua::Result<MultiValue> {
    let what = named("call");
    let mut next = || args.pop_front().unwrap_or(Value::Nil);
    let (owner, label, function) = (next(), next(), next());
    let owner = check::optional_text(lua, &what, &owner, "an owner as a plugin name or nil")?;
    let label = check::optional_text(lua, &what, &label, "a label as a string or nil")?;
    let function = check::function(lua, &what, &function, "a function")?;
    let labelled = label.is_some();
    match guard::isolated(lua, owner, label, || function.call::<MultiValue>(args))? {
        Ok(mut values) => {
            values.push_front(Value::Boolean(true));
            Ok(values)
        }
        Err(failure) => {
            let message = failure.error.to_string();
            guard::report_isolated(lua, failure, labelled);
            Ok(MultiValue::from_iter([
                Value::Boolean(false),
                Value::String(lua.create_string(message)?),
            ]))
        }
    }
}

fn report(lua: &Lua, (label, message, level): (Value, Value, Value)) -> mlua::Result<()> {
    let what = named("report");
    let plugin = check::optional_text(lua, &what, &label, "a label as a string or nil")?;
    let message = check::text(lua, &what, &message, "a message as a string")?;
    let level: Option<usize> = check::optional_integer(
        lua,
        &what,
        &level,
        "a stack level as a non-negative integer or nil",
    )?;
    let location = level.and_then(|level| {
        lua.inspect_stack(level, |debug| {
            let source = debug.source().source?;
            let path = source.strip_prefix('@')?;
            let line = u32::try_from(debug.current_line()?).ok()?;
            Some((PathBuf::from(path), line))
        })
        .flatten()
    });
    guard::push(
        lua,
        ConfigError {
            plugin,
            location,
            message,
        },
    );
    Ok(())
}

fn emit(lua: &Lua, (name, payload): (Value, Value)) -> mlua::Result<()> {
    let what = named("emit");
    let name = check::text(lua, &what, &name, "an event name as a string")?;
    if !events::NAMES.contains(&name.as_str()) {
        return Err(check::fail(
            lua,
            &what,
            format!("`{name}` is not a built-in event"),
        ));
    }
    let payload = check::table(lua, &what, &payload, "a payload table")?;
    events::deliver_table(lua, &name, &payload)?;
    after_event(lua, Some(&name))
}

pub(crate) fn after_event(lua: &Lua, name: Option<&str>) -> mlua::Result<()> {
    let hooks: Vec<Function> = hooks(lua)
        .after_event
        .iter()
        .map(|key| lua.registry_value::<Function>(key))
        .collect::<mlua::Result<_>>()?;
    for hook in hooks {
        hook.call::<()>(name)?;
    }
    Ok(())
}

pub(crate) fn current_state(lua: &Lua) -> ViewState {
    lua.app_data_ref::<State>()
        .expect("the state is installed with the runtime")
        .view
        .clone()
}

pub(crate) fn clear_errors(lua: &Lua) {
    let mut stored = lua
        .app_data_mut::<State>()
        .expect("the state is installed with the runtime");
    stored.view.error = None;
    stored.view.errors.clear();
    stored.cleared = true;
}

fn state(lua: &Lua, (): ()) -> mlua::Result<Table> {
    let state = current_state(lua);
    let table = lua.create_table()?;
    table.set("table", state.table)?;
    let band = lua.create_table()?;
    band.set("number", state.band.number)?;
    band.set("index", state.band.index)?;
    band.set("count", state.band.count)?;
    table.set("band", band)?;
    table.set("window", state.window)?;
    table.set("width", state.width)?;
    table.set("height", state.height)?;
    table.set("error", state.error)?;
    let ribbon = lua.create_table()?;
    ribbon.set("cols", state.ribbon.cols)?;
    ribbon.set("rows", state.ribbon.rows)?;
    table.set("ribbon", ribbon)?;
    Ok(table)
}

fn drawn_differs(old: &ViewState, new: &ViewState) -> bool {
    old.table != new.table
        || old.band.index != new.band.index
        || old.band.count != new.band.count
        || old.error != new.error
        || old.errors != new.errors
}

pub(crate) fn set_state(lua: &Lua, state: ViewState) -> mlua::Result<()> {
    let changed = {
        let mut stored = lua
            .app_data_mut::<State>()
            .expect("the state is installed with the runtime");
        let changed = std::mem::take(&mut stored.cleared) || drawn_differs(&stored.view, &state);
        let resized = stored.view.ribbon != state.ribbon;
        stored.view = state;
        (changed, resized)
    };
    let (changed, resized) = changed;
    if resized {
        crate::plugin_windows::ribbon_resized(lua)?;
    }
    if !changed {
        return Ok(());
    }
    let functions: Vec<Function> = hooks(lua)
        .on_state
        .iter()
        .map(|key| lua.registry_value::<Function>(key))
        .collect::<mlua::Result<_>>()?;
    for function in functions {
        function.call::<()>(())?;
    }
    Ok(())
}

fn color(table: &Table, name: &str) -> Result<Option<Color>, String> {
    let invalid = || format!("the field `{name}` must be `#rrggbb` or a color index");
    Ok(
        match table
            .get::<Value>(name)
            .map_err(|error| error.to_string())?
        {
            Value::Nil => None,
            Value::Integer(index) => {
                Some(Color::Index(u8::try_from(index).map_err(|_| invalid())?))
            }
            Value::String(text) => {
                let (r, g, b) = hex(&text.to_string_lossy()).ok_or_else(invalid)?;
                Some(Color::Rgb(r, g, b))
            }
            _ => return Err(invalid()),
        },
    )
}

pub(crate) fn style(table: &Table) -> Result<Style, String> {
    let flag = |name: &str| -> Result<bool, String> {
        Ok(check::optional_boolean_field(table, name, "a boolean")?.unwrap_or(false))
    };
    Ok(Style {
        fg: color(table, "fg")?,
        bg: color(table, "bg")?,
        bold: flag("bold")?,
        italic: flag("italic")?,
        underline: flag("underline")?,
        reverse: flag("reverse")?,
        dim: flag("dim")?,
    })
}

pub(crate) fn style_field(table: &Table, name: &str) -> Result<Style, String> {
    let style_table = check::table_field(table, name, "a style table")?;
    style(&style_table).map_err(|reason| format!("the style `{name}`: {reason}"))
}

fn timer(lua: &Lua, (period, function): (Value, Value)) -> mlua::Result<i64> {
    let what = named("timer");
    let period: u64 = check::integer(
        lua,
        &what,
        &period,
        "a period in milliseconds as a non-negative integer",
    )?;
    let function = check::function(lua, &what, &function, "a function")?;
    start_timer(lua, period, function)
}

fn start_timer(lua: &Lua, period: u64, function: Function) -> mlua::Result<i64> {
    let period = Duration::from_millis(period.max(1));
    let function = lua.create_registry_value(function)?;
    let mut timers = timers(lua);
    let id = timers.next_id;
    timers.next_id += 1;
    timers.entries.insert(
        id,
        Timer {
            period,
            next: Instant::now() + period,
            function,
        },
    );
    Ok(id)
}

pub(crate) fn next_timer(lua: &Lua) -> Option<Instant> {
    timers(lua).entries.values().map(|timer| timer.next).min()
}

pub(crate) fn fire_timers(lua: &Lua, now: Instant) -> mlua::Result<()> {
    let mut due: Vec<(Instant, i64)> = timers(lua)
        .entries
        .iter()
        .filter(|(_, timer)| timer.next <= now)
        .map(|(&id, timer)| (timer.next, id))
        .collect();
    due.sort();
    for (_, id) in due {
        let function = {
            let mut timers = timers(lua);
            let Some(timer) = timers.entries.get_mut(&id) else {
                continue;
            };
            while timer.next <= now {
                timer.next += timer.period;
            }
            lua.registry_value::<Function>(&timer.function)?
        };
        match guard::run(lua, None, || function.call::<()>(()))? {
            Ok(()) => {}
            Err(failure) => guard::report(lua, failure),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use gband_core::layout::WindowId;

    use super::*;
    use crate::{Config, Event, LoadOptions};

    fn config(source: &str) -> Config {
        crate::evaluate(
            None,
            crate::Side::Client,
            &LoadOptions::default(),
            Path::new("init.lua"),
            source.as_bytes(),
        )
        .unwrap()
    }

    fn counting_timer(config: &Config, period: u64) {
        let lua = config.runtime.lua();
        let function = lua
            .load("count = (count or 0) + 1")
            .into_function()
            .unwrap();
        start_timer(lua, period, function).unwrap();
    }

    #[test]
    fn timer_fires_once_per_period() {
        let config = config("");
        let start = Instant::now();
        counting_timer(&config, 100);
        for step in 0..=7 {
            let outcome = config
                .runtime
                .fire_timers(start + Duration::from_millis(50 * step));
            assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
        }
        let count: i64 = config.runtime.lua().globals().get("count").unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn missed_periods_are_skipped() {
        let config = config("");
        let start = Instant::now();
        counting_timer(&config, 100);
        config
            .runtime
            .fire_timers(start + Duration::from_millis(350));
        let count: i64 = config.runtime.lua().globals().get("count").unwrap();
        assert_eq!(count, 1);
        let next = config.runtime.next_timer().unwrap();
        assert!(next > start + Duration::from_millis(350));
        assert!(next <= start + Duration::from_millis(500));
    }

    #[test]
    fn cancelled_timer_never_fires() {
        let config = config("");
        let lua = config.runtime.lua();
        let function = lua.load("count = 1").into_function().unwrap();
        let id = start_timer(lua, 100, function).unwrap();
        timers(lua).entries.remove(&id);
        assert_eq!(config.runtime.next_timer(), None);
    }

    #[test]
    fn after_event_runs_after_the_handlers() {
        let config =
            config("log = {}\ngband.on('FocusChanged', function() log[#log + 1] = 'handler' end)");
        let lua = config.runtime.lua();
        let hook = lua
            .load("return function(name) log[#log + 1] = 'after ' .. tostring(name) end")
            .eval::<Function>()
            .unwrap();
        hooks(lua)
            .after_event
            .push(lua.create_registry_value(hook).unwrap());
        let outcome = config.runtime.emit(&Event::FocusChanged {
            window: Some(WindowId(1)),
            previous: None,
        });
        assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
        let log: Vec<String> = lua.globals().get("log").unwrap();
        assert_eq!(log, ["handler", "after FocusChanged"]);
    }

    #[test]
    fn every_after_event_hook_runs() {
        let config = config("log = {}");
        let lua = config.runtime.lua();
        assert!(hooks(lua).after_event.len() >= 2);
        let hook = lua
            .load("return function(name) log[#log + 1] = name end")
            .eval::<Function>()
            .unwrap();
        hooks(lua)
            .after_event
            .push(lua.create_registry_value(hook).unwrap());
        let state = ViewState {
            width: 40,
            height: 24,
            ..ViewState::default()
        };
        assert!(config.runtime.set_state(state).errors.is_empty());
        let outcome = config.runtime.emit(&Event::FocusChanged {
            window: Some(WindowId(1)),
            previous: None,
        });
        assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
        let log: Vec<String> = lua.globals().get("log").unwrap();
        assert_eq!(log, ["FocusChanged"]);
    }

    #[test]
    fn width_counts_cells() {
        assert_eq!(width("a日b"), 4);
        assert_eq!(width("a\u{301}"), 1);
        assert_eq!(width("a\x1b[31mb"), 6);
        assert_eq!(width("\u{85}x\u{7f}"), 1);
    }

    #[test]
    fn truncate_at_a_wide_character() {
        assert_eq!(truncate("日本語", 4), "日…");
        assert_eq!(truncate("日本語", 5), "日本…");
        assert_eq!(truncate("日本語", 6), "日本語");
        assert_eq!(truncate("abcdefghij", 6), "abcde…");
    }

    #[test]
    fn truncate_to_nothing_and_to_one_cell() {
        assert_eq!(truncate("abc", 0), "");
        assert_eq!(truncate("abc", 1), "…");
        assert_eq!(truncate("a", 1), "a");
        assert_eq!(truncate("", 0), "");
    }

    fn gruvbox() -> Palette {
        let mut palette = Palette {
            fg: Some((0xeb, 0xdb, 0xb2)),
            bg: Some((0x28, 0x28, 0x28)),
            ..Palette::default()
        };
        palette.colors[1] = Some((0xcc, 0x24, 0x1d));
        palette
    }

    #[test]
    fn palette_set_marks_it_changed_once() {
        let config = config("");
        let lua = config.runtime.lua();
        take_palette(lua);
        set_palette(lua, gruvbox()).unwrap();
        assert_eq!(take_palette(lua), Some(gruvbox()));
        assert_eq!(take_palette(lua), None);
        set_palette(lua, gruvbox()).unwrap();
        assert_eq!(take_palette(lua), None);
    }

    #[test]
    fn palette_set_replaces_every_field() {
        let config = config("");
        let lua = config.runtime.lua();
        set_palette(lua, gruvbox()).unwrap();
        let only_fg = Palette {
            fg: Some((1, 2, 3)),
            ..Palette::default()
        };
        set_palette(lua, only_fg).unwrap();
        assert_eq!(take_palette(lua), Some(only_fg));
    }

    #[test]
    fn palette_emptied() {
        let config = config("");
        let lua = config.runtime.lua();
        set_palette(lua, gruvbox()).unwrap();
        take_palette(lua);
        set_palette(lua, Palette::default()).unwrap();
        let taken = take_palette(lua).unwrap();
        assert!(taken.is_empty());
    }

    #[test]
    fn palette_restored_from_its_table() {
        let config = config("");
        let lua = config.runtime.lua();
        set_palette(lua, gruvbox()).unwrap();
        let saved = palette_table(lua, &look(lua).palette).unwrap();
        assert_eq!(saved.get::<String>("red").unwrap(), "#cc241d");
        set_palette(lua, Palette::default()).unwrap();
        set_palette(lua, read_palette(&saved).unwrap()).unwrap();
        assert_eq!(look(lua).palette, gruvbox());
    }

    #[test]
    fn controls_are_stripped() {
        assert_eq!(strip("a\x1b[31mb\u{9b}\n"), "a[31mb");
        assert_eq!(truncate("a\tb", 5), "ab");
    }
}
