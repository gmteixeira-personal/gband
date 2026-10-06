use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

use gband_core::action::Steps;
use gband_core::input::{Key, KeyCode, Modifiers};
use gband_core::layout::{LayoutOptions, Proportion};
use gband_core::view::CenterFocusedColumn;
use mlua::{IntoLua, Lua, LuaSerdeExt, Table, Value};
use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize};

use crate::Side;
use crate::api::require_loading;
use crate::border::{Border, BorderChars, Sides};
use crate::error::{ConfigError, caller};
use crate::guard;
use crate::keys::{key_name, parse_key};
use crate::owner;

const MAX_DENOMINATOR: u64 = 100;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotifyStyle {
    #[default]
    Osc9,
    Osc777,
    Bell,
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    pub prefix: Key,
    pub layout: LayoutOptions,
    pub center_focused_column: CenterFocusedColumn,
    pub loop_bands: bool,
    pub notify_style: NotifyStyle,
    pub tile_border: Border,
    pub floating_border: Border,
    pub steps: Steps,
    pub mouse_mod: Modifiers,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            prefix: Key::new(KeyCode::Char(' '), Modifiers::CTRL),
            layout: LayoutOptions::default(),
            center_focused_column: CenterFocusedColumn::default(),
            loop_bands: true,
            notify_style: NotifyStyle::default(),
            tile_border: Border::default(),
            floating_border: Border::default(),
            steps: Steps::default(),
            mouse_mod: Modifiers::ALT,
        }
    }
}

impl Options {
    pub fn merge(&mut self, patch: OptionsPatch) {
        if let Some(KeySpec(prefix)) = patch.prefix {
            self.prefix = prefix;
        }
        if let Some(Width(width)) = patch.default_column_width {
            self.layout.default_width = width;
        }
        if let Some(Presets(presets)) = patch.width_presets {
            let mut presets: Vec<Proportion> =
                presets.into_iter().map(|Width(width)| width).collect();
            presets.sort_by(|a, b| {
                (u64::from(a.num) * u64::from(b.den)).cmp(&(u64::from(b.num) * u64::from(a.den)))
            });
            presets.dedup();
            self.layout.presets = presets;
        }
        if let Some(policy) = patch.center_focused_column {
            self.center_focused_column = policy;
        }
        if let Some(loop_bands) = patch.loop_bands {
            self.loop_bands = loop_bands;
        }
        if let Some(style) = patch.notify_style {
            self.notify_style = style;
        }
        if let Some(sides) = patch.tile_border_sides {
            self.tile_border.sides = sides;
        }
        if let Some(chars) = patch.tile_border_chars {
            self.tile_border.chars = chars;
        }
        if let Some(sides) = patch.floating_border_sides {
            self.floating_border.sides = sides;
        }
        if let Some(chars) = patch.floating_border_chars {
            self.floating_border.chars = chars;
        }
        if let Some(WidthStep(step)) = patch.width_step {
            self.steps.width = step;
        }
        if let Some(HeightStep(step)) = patch.height_step {
            self.steps.height = step;
        }
        if let Some(MouseMod(modifiers)) = patch.mouse_mod {
            self.mouse_mod = modifiers;
        }
    }
}

impl Options {
    fn reset(&mut self, name: &str) {
        let defaults = Options::default();
        match name {
            "prefix" => self.prefix = defaults.prefix,
            "default_column_width" => self.layout.default_width = defaults.layout.default_width,
            "width_presets" => self.layout.presets = defaults.layout.presets,
            "center_focused_column" => self.center_focused_column = defaults.center_focused_column,
            "loop_bands" => self.loop_bands = defaults.loop_bands,
            "notify_style" => self.notify_style = defaults.notify_style,
            "tile_border_sides" => self.tile_border.sides = defaults.tile_border.sides,
            "tile_border_chars" => self.tile_border.chars = defaults.tile_border.chars,
            "floating_border_sides" => self.floating_border.sides = defaults.floating_border.sides,
            "floating_border_chars" => self.floating_border.chars = defaults.floating_border.chars,
            "width_step" => self.steps.width = defaults.steps.width,
            "height_step" => self.steps.height = defaults.steps.height,
            "mouse_mod" => self.mouse_mod = defaults.mouse_mod,
            _ => {}
        }
    }

    fn get(&self, lua: &Lua, name: &str) -> mlua::Result<Value> {
        let width = |width: Proportion| f64::from(width.num) / f64::from(width.den);
        match name {
            "prefix" => key_name(self.prefix).into_lua(lua),
            "default_column_width" => width(self.layout.default_width).into_lua(lua),
            "width_presets" => lua
                .create_sequence_from(self.layout.presets.iter().map(|&preset| width(preset)))?
                .into_lua(lua),
            "center_focused_column" => lua.to_value(&self.center_focused_column),
            "loop_bands" => self.loop_bands.into_lua(lua),
            "notify_style" => lua.to_value(&self.notify_style),
            "tile_border_sides" => self.tile_border.sides.to_lua(lua),
            "tile_border_chars" => self.tile_border.chars.to_lua(lua),
            "floating_border_sides" => self.floating_border.sides.to_lua(lua),
            "floating_border_chars" => self.floating_border.chars.to_lua(lua),
            "width_step" => width(self.steps.width).into_lua(lua),
            "height_step" => width(self.steps.height).into_lua(lua),
            "mouse_mod" => modifier_names(self.mouse_mod).into_lua(lua),
            _ => Ok(Value::Nil),
        }
    }
}

const CLIENT_NAMES: [&str; 11] = [
    "prefix",
    "center_focused_column",
    "loop_bands",
    "notify_style",
    "tile_border_sides",
    "tile_border_chars",
    "floating_border_sides",
    "floating_border_chars",
    "width_step",
    "height_step",
    "mouse_mod",
];

const SERVER_NAMES: [&str; 2] = ["default_column_width", "width_presets"];

pub fn names(side: Side) -> &'static [&'static str] {
    match side {
        Side::Client => &CLIENT_NAMES,
        Side::Server => &SERVER_NAMES,
        Side::Test => &[],
    }
}

fn is_builtin(name: &str) -> bool {
    CLIENT_NAMES.contains(&name) || SERVER_NAMES.contains(&name)
}

fn own(lua: &Lua, name: &str) -> bool {
    names(crate::runtime::side(lua)).contains(&name)
}

fn foreign(lua: &Lua, name: &str) -> Option<String> {
    let side = crate::runtime::side(lua);
    names(side.other()).contains(&name).then(|| {
        let other = side.other();
        format!(
            "the option `{name}` belongs to the {}; set it in user/{}",
            other.name(),
            other.init_name()
        )
    })
}

pub(crate) fn check_name(lua: &Lua, name: &str) -> Result<(), String> {
    if own(lua, name) {
        return Ok(());
    }
    Err(foreign(lua, name).unwrap_or_else(|| format!("unknown option `{name}`")))
}

const BUILTIN: [(&str, &str, &str); 13] = [
    ("prefix", "string", "the key that starts a key sequence"),
    (
        "default_column_width",
        "number",
        "the width of a new window's column, as a fraction of the screen",
    ),
    (
        "width_presets",
        "list",
        "the widths that cycling a column's width steps through",
    ),
    (
        "notify_style",
        "string",
        "how gband.notify reaches the terminal: osc9, osc777, bell or none",
    ),
    (
        "center_focused_column",
        "string",
        "when the view centres the focused column: never, always or on-overflow",
    ),
    (
        "loop_bands",
        "boolean",
        "whether focus and the strip go round from a band's last column to its first",
    ),
    (
        "tile_border_sides",
        "list",
        "the sides of a tiled window's border that are drawn",
    ),
    (
        "tile_border_chars",
        "string",
        "the characters of a tiled window's border: plain, rounded, double, thick or a list of 8",
    ),
    (
        "floating_border_sides",
        "list",
        "the sides of a floating window's border that are drawn",
    ),
    (
        "floating_border_chars",
        "string",
        "the characters of a floating window's border: plain, rounded, double, thick or a list of 8",
    ),
    (
        "width_step",
        "number",
        "how much growing or shrinking changes a column's width, as a fraction of the screen",
    ),
    (
        "height_step",
        "number",
        "how much growing or shrinking changes a window's height, as a fraction of the screen",
    ),
    (
        "mouse_mod",
        "string",
        "modifiers that mod stands for in a mouse name",
    ),
];

#[derive(Clone, Debug, PartialEq)]
pub enum OptValue {
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(String),
}

impl IntoLua for OptValue {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        match self {
            OptValue::Boolean(value) => value.into_lua(lua),
            OptValue::Integer(value) => value.into_lua(lua),
            OptValue::Number(value) => value.into_lua(lua),
            OptValue::String(value) => value.into_lua(lua),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum OptType {
    Boolean,
    Integer,
    Number,
    String,
}

impl OptType {
    fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "boolean" => OptType::Boolean,
            "integer" => OptType::Integer,
            "number" => OptType::Number,
            "string" => OptType::String,
            _ => return None,
        })
    }

    fn name(self) -> &'static str {
        match self {
            OptType::Boolean => "boolean",
            OptType::Integer => "integer",
            OptType::Number => "number",
            OptType::String => "string",
        }
    }

    fn accept(self, raw: &Raw) -> Option<OptValue> {
        Some(match (self, raw) {
            (OptType::Boolean, Raw::Boolean(value)) => OptValue::Boolean(*value),
            (OptType::Integer, Raw::Integer(value)) => OptValue::Integer(*value),
            (OptType::Integer, Raw::Number(value))
                if value.fract() == 0.0 && value.abs() < 9.007_199_254_740_992e15 =>
            {
                OptValue::Integer(*value as i64)
            }
            (OptType::Number, Raw::Integer(value)) => OptValue::Number(*value as f64),
            (OptType::Number, Raw::Number(value)) => OptValue::Number(*value),
            (OptType::String, Raw::String(value)) => OptValue::String(value.clone()),
            _ => return None,
        })
    }
}

#[derive(Clone, Debug)]
enum Raw {
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(String),
    Other(&'static str),
}

impl Raw {
    fn from_value(value: &Value) -> mlua::Result<Self> {
        Ok(match value {
            Value::Boolean(value) => Raw::Boolean(*value),
            Value::Integer(value) => Raw::Integer(*value),
            Value::Number(value) => Raw::Number(*value),
            Value::String(value) => Raw::String(value.to_str()?.to_owned()),
            other => Raw::Other(other.type_name()),
        })
    }

    fn type_name(&self) -> &'static str {
        match self {
            Raw::Boolean(_) => "boolean",
            Raw::Integer(_) | Raw::Number(_) => "number",
            Raw::String(_) => "string",
            Raw::Other(name) => name,
        }
    }
}

struct Declared {
    kind: OptType,
    values: Option<Vec<OptValue>>,
    default: OptValue,
    value: OptValue,
    desc: Option<String>,
}

impl Declared {
    fn validate(&self, raw: &Raw) -> Result<OptValue, String> {
        let value = self.kind.accept(raw).ok_or_else(|| {
            format!(
                "expected a value of type {}, found {}",
                self.kind.name(),
                raw.type_name()
            )
        })?;
        match &self.values {
            Some(values) if !values.contains(&value) => {
                Err("the value is not one of the allowed values".to_owned())
            }
            _ => Ok(value),
        }
    }
}

type Location = Option<(PathBuf, u32)>;

struct Pending {
    raw: Raw,
    location: Location,
    owner: Option<String>,
}

#[derive(Default)]
pub(crate) struct Store {
    options: Options,
    declared: BTreeMap<String, Declared>,
    pending: BTreeMap<String, Pending>,
}

fn store(lua: &Lua) -> mlua::AppDataRefMut<'_, Store> {
    lua.app_data_mut::<Store>()
        .expect("options are installed with the runtime")
}

pub(crate) fn current(lua: &Lua) -> Options {
    store(lua).options.clone()
}

pub(crate) fn merge(lua: &Lua, patch: OptionsPatch) {
    store(lua).options.merge(patch);
}

pub(crate) fn patch(lua: &Lua, name: &str, value: Value) -> Result<OptionsPatch, String> {
    let single = lua.create_table().map_err(|error| error.to_string())?;
    single.set(name, value).map_err(|error| error.to_string())?;
    lua.from_value(Value::Table(single))
        .map_err(|error| match error {
            mlua::Error::DeserializeError(message) => message,
            other => other.to_string(),
        })
}

pub(crate) fn invalid(name: &str, reason: &str) -> String {
    format!("invalid value for option `{name}`: {reason}")
}

fn report(lua: &Lua, location: Location, owner: Option<String>, message: String) {
    guard::push(
        lua,
        ConfigError {
            plugin: owner,
            location,
            message,
        },
    );
}

pub(crate) fn install(lua: &Lua, gband: &Table) -> mlua::Result<()> {
    lua.set_app_data(Store::default());
    let declare = lua.create_function(declare)?;
    let list = lua.create_function(list)?;
    let opt = lua.create_table()?;
    let meta = lua.create_table()?;
    meta.set(
        "__index",
        lua.create_function(
            move |lua, (_, name): (Value, Value)| -> mlua::Result<Value> {
                let Value::String(name) = name else {
                    return Ok(Value::Nil);
                };
                let name = name.to_str()?.to_owned();
                match name.as_str() {
                    "declare" => return Ok(Value::Function(declare.clone())),
                    "list" => return Ok(Value::Function(list.clone())),
                    _ => {}
                }
                if is_builtin(&name) {
                    if !own(lua, &name) {
                        return Ok(Value::Nil);
                    }
                    return store(lua).options.get(lua, &name);
                }
                let store = store(lua);
                match store.declared.get(&name) {
                    Some(declared) => declared.value.clone().into_lua(lua),
                    None => Ok(Value::Nil),
                }
            },
        )?,
    )?;
    meta.set("__newindex", lua.create_function(assign)?)?;
    opt.set_metatable(Some(meta))?;
    gband.set("opt", opt)
}

fn assign(lua: &Lua, (_, name, value): (Value, Value, Value)) -> mlua::Result<()> {
    let Value::String(name) = name else {
        return Err(ConfigError::raise(lua, "option names must be strings"));
    };
    let name = name.to_str()?.to_owned();
    if name == "declare" || name == "list" {
        return Err(ConfigError::raise(
            lua,
            format!("gband.opt.{name} cannot be assigned"),
        ));
    }
    require_loading(lua, "setting an option")?;
    let location = caller(lua);
    let owner = owner::current(lua);
    if let Some(message) = foreign(lua, &name) {
        report(lua, location, owner, message);
        return Ok(());
    }
    if own(lua, &name) {
        match patch(lua, &name, value) {
            Ok(patch) => merge(lua, patch),
            Err(reason) => {
                store(lua).options.reset(&name);
                report(lua, location, owner, invalid(&name, &reason));
            }
        }
        return Ok(());
    }
    let raw = Raw::from_value(&value)?;
    let mut store = store(lua);
    match store.declared.get_mut(&name) {
        Some(declared) => match declared.validate(&raw) {
            Ok(value) => declared.value = value,
            Err(reason) => {
                declared.value = declared.default.clone();
                drop(store);
                report(lua, location, owner, invalid(&name, &reason));
            }
        },
        None => {
            store.pending.insert(
                name,
                Pending {
                    raw,
                    location,
                    owner,
                },
            );
        }
    }
    Ok(())
}

fn declare(lua: &Lua, (name, spec): (Value, Value)) -> mlua::Result<String> {
    let name = match &name {
        Value::String(name) if !name.as_bytes().is_empty() => name.to_str()?.to_owned(),
        _ => {
            return Err(ConfigError::raise(
                lua,
                "gband.opt.declare expects a name as a non-empty string",
            ));
        }
    };
    let full = owner::full_name(lua, &name)?;
    let Value::Table(spec) = spec else {
        return Err(ConfigError::raise(
            lua,
            format!("the declaration of `{full}` must be a table"),
        ));
    };
    let kind = match spec.get::<Value>("type")? {
        Value::String(kind) => OptType::parse(&kind.to_str()?),
        _ => None,
    }
    .ok_or_else(|| {
        ConfigError::raise(
            lua,
            format!("the option `{full}` must have the type boolean, integer, number or string"),
        )
    })?;
    let values = match spec.get::<Value>("values")? {
        Value::Nil => None,
        Value::Table(list) => Some(
            list.sequence_values::<Value>()
                .map(|value| {
                    let raw = Raw::from_value(&value?)?;
                    kind.accept(&raw).ok_or_else(|| {
                        ConfigError::raise(
                            lua,
                            format!(
                                "the allowed values of `{full}` must be of type {}",
                                kind.name()
                            ),
                        )
                    })
                })
                .collect::<mlua::Result<Vec<_>>>()?,
        ),
        _ => {
            return Err(ConfigError::raise(
                lua,
                format!("the allowed values of `{full}` must be a list"),
            ));
        }
    };
    let desc = match spec.get::<Value>("desc")? {
        Value::Nil => None,
        Value::String(desc) => Some(desc.to_str()?.to_owned()),
        _ => {
            return Err(ConfigError::raise(
                lua,
                format!("the description of `{full}` must be a string"),
            ));
        }
    };
    let raw = Raw::from_value(&spec.get::<Value>("default")?)?;
    let mut declared = Declared {
        kind,
        values,
        default: OptValue::Boolean(false),
        value: OptValue::Boolean(false),
        desc,
    };
    declared.default = declared.validate(&raw).map_err(|reason| {
        ConfigError::raise(
            lua,
            format!("invalid default for option `{full}`: {reason}"),
        )
    })?;
    declared.value = declared.default.clone();
    if is_builtin(&full) || store(lua).declared.contains_key(&full) {
        return Err(ConfigError::raise(
            lua,
            format!("the option `{full}` is already declared"),
        ));
    }
    require_loading(lua, "gband.opt.declare")?;
    let mut store = store(lua);
    let mut rejected = None;
    if let Some(pending) = store.pending.remove(&full) {
        match declared.validate(&pending.raw) {
            Ok(value) => declared.value = value,
            Err(reason) => rejected = Some((pending, reason)),
        }
    }
    store.declared.insert(full.clone(), declared);
    drop(store);
    if let Some((pending, reason)) = rejected {
        report(
            lua,
            pending.location,
            pending.owner,
            invalid(&full, &reason),
        );
    }
    Ok(full)
}

fn list(lua: &Lua, (): ()) -> mlua::Result<Table> {
    let defaults = Options::default();
    let list = lua.create_table()?;
    let store = store(lua);
    let mut entries: Vec<(String, Table)> = Vec::new();
    for (name, kind, desc) in BUILTIN {
        if !own(lua, name) {
            continue;
        }
        let entry = lua.create_table()?;
        entry.set("name", name)?;
        entry.set("type", kind)?;
        entry.set("default", defaults.get(lua, name)?)?;
        entry.set("value", store.options.get(lua, name)?)?;
        entry.set("desc", desc)?;
        entries.push((name.to_owned(), entry));
    }
    for (name, declared) in &store.declared {
        let entry = lua.create_table()?;
        entry.set("name", name.as_str())?;
        entry.set("type", declared.kind.name())?;
        entry.set("default", declared.default.clone())?;
        entry.set("value", declared.value.clone())?;
        entry.set("desc", declared.desc.as_deref())?;
        entries.push((name.clone(), entry));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    for (_, entry) in entries {
        list.push(entry)?;
    }
    Ok(list)
}

pub(crate) fn finish(lua: &Lua) {
    let pending = std::mem::take(&mut store(lua).pending);
    for (name, pending) in pending {
        report(
            lua,
            pending.location,
            pending.owner,
            format!("no option `{name}` is declared"),
        );
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptionsPatch {
    prefix: Option<KeySpec>,
    default_column_width: Option<Width>,
    width_presets: Option<Presets>,
    center_focused_column: Option<CenterFocusedColumn>,
    loop_bands: Option<bool>,
    notify_style: Option<NotifyStyle>,
    tile_border_sides: Option<Sides>,
    tile_border_chars: Option<BorderChars>,
    floating_border_sides: Option<Sides>,
    floating_border_chars: Option<BorderChars>,
    width_step: Option<WidthStep>,
    height_step: Option<HeightStep>,
    mouse_mod: Option<MouseMod>,
}

pub(crate) fn step(value: f64, limit: u32) -> Result<Proportion, String> {
    let read = (value.is_finite() && value > 0.0 && value <= f64::from(limit))
        .then(|| Width::from_number(value).ok())
        .flatten();
    read.map(|Width(step)| step)
        .ok_or_else(|| format!("expected a step greater than 0 and at most {limit}, found {value}"))
}

pub(crate) const WIDTH_STEP_LIMIT: u32 = Proportion::MAX;
pub(crate) const HEIGHT_STEP_LIMIT: u32 = 1;

#[derive(Debug)]
struct WidthStep(Proportion);

impl<'de> Deserialize<'de> for WidthStep {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = deserializer.deserialize_f64(Number)?;
        step(value, WIDTH_STEP_LIMIT)
            .map(WidthStep)
            .map_err(de::Error::custom)
    }
}

#[derive(Debug)]
struct HeightStep(Proportion);

impl<'de> Deserialize<'de> for HeightStep {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = deserializer.deserialize_f64(Number)?;
        step(value, HEIGHT_STEP_LIMIT)
            .map(HeightStep)
            .map_err(de::Error::custom)
    }
}

struct Number;

impl Visitor<'_> for Number {
    type Value = f64;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a number")
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<f64, E> {
        Ok(value)
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<f64, E> {
        Ok(value as f64)
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<f64, E> {
        Ok(value as f64)
    }
}

#[derive(Debug)]
struct KeySpec(Key);

impl<'de> Deserialize<'de> for KeySpec {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        parse_key(&name).map(KeySpec).map_err(de::Error::custom)
    }
}

#[derive(Debug)]
struct MouseMod(Modifiers);

impl<'de> Deserialize<'de> for MouseMod {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let names = String::deserialize(deserializer)?;
        let mut modifiers = Modifiers::NONE;
        for name in names.split('+') {
            match name.to_ascii_lowercase().as_str() {
                "ctrl" => modifiers.ctrl = true,
                "alt" => modifiers.alt = true,
                "shift" => modifiers.shift = true,
                _ => {
                    return Err(de::Error::custom(format!(
                        "expected modifiers among ctrl, alt and shift joined by +, found `{names}`"
                    )));
                }
            }
        }
        Ok(MouseMod(modifiers))
    }
}

fn modifier_names(modifiers: Modifiers) -> String {
    [
        (modifiers.ctrl, "ctrl"),
        (modifiers.alt, "alt"),
        (modifiers.shift, "shift"),
    ]
    .into_iter()
    .filter_map(|(held, name)| held.then_some(name))
    .collect::<Vec<_>>()
    .join("+")
}

#[derive(Debug)]
struct Presets(Vec<Width>);

impl<'de> Deserialize<'de> for Presets {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let widths = Vec::<Width>::deserialize(deserializer)?;
        if widths.is_empty() {
            return Err(de::Error::custom("expected at least one width"));
        }
        Ok(Presets(widths))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Width(pub Proportion);

impl Width {
    pub fn from_number(value: f64) -> Result<Self, String> {
        if !value.is_finite() || value <= 0.0 || value > f64::from(Proportion::MAX) {
            return Err(format!(
                "expected a width greater than 0 and at most {}, found {value}",
                Proportion::MAX
            ));
        }
        let (num, den) = approximate(value);
        if num == 0 {
            return Err(format!("width {value} is too small"));
        }
        Ok(Width(Proportion::new(num as u32, den as u32)))
    }
}

impl<'de> Deserialize<'de> for Width {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = deserializer.deserialize_f64(Number)?;
        Width::from_number(value).map_err(de::Error::custom)
    }
}

fn approximate(value: f64) -> (u64, u64) {
    let (mut p0, mut q0, mut p1, mut q1) = (0u64, 1u64, 1u64, 0u64);
    let mut rest = value;
    loop {
        let whole = rest.floor();
        let term = whole as u64;
        let q2 = q0 + term * q1;
        if q2 > MAX_DENOMINATOR {
            let step = (MAX_DENOMINATOR - q0) / q1;
            let (p3, q3) = (p0 + step * p1, q0 + step * q1);
            let error = |p: u64, q: u64| (value - p as f64 / q as f64).abs();
            return if error(p3, q3) < error(p1, q1) {
                (p3, q3)
            } else {
                (p1, q1)
            };
        }
        (p0, q0, p1, q1) = (p1, q1, p0 + term * p1, q2);
        let fraction = rest - whole;
        if fraction < 1e-9 {
            return (p1, q1);
        }
        rest = 1.0 / fraction;
    }
}

#[cfg(test)]
mod tests {
    use mlua::{Lua, LuaSerdeExt};

    use super::*;

    fn width(source: &str) -> Result<Proportion, String> {
        let lua = Lua::new();
        let value = lua.load(source).eval().unwrap();
        lua.from_value::<Width>(value)
            .map(|Width(width)| width)
            .map_err(|error| error.to_string())
    }

    fn patched(source: &str) -> Result<Options, String> {
        let lua = Lua::new();
        let value = lua.load(source).eval().unwrap();
        let patch: OptionsPatch = lua.from_value(value).map_err(|error| error.to_string())?;
        let mut options = Options::default();
        options.merge(patch);
        Ok(options)
    }

    #[test]
    fn fractions_read_exactly() {
        assert_eq!(width("1/3"), Ok(Proportion::new(1, 3)));
        assert_eq!(width("2/3"), Ok(Proportion::new(2, 3)));
        assert_eq!(width("1/2"), Ok(Proportion::new(1, 2)));
        assert_eq!(width("1"), Ok(Proportion::new(1, 1)));
        assert_eq!(width("10000"), Ok(Proportion::new(10000, 1)));
        assert_eq!(width("0.333"), Ok(Proportion::new(1, 3)));
        assert_eq!(width("1/97"), Ok(Proportion::new(1, 97)));
    }

    #[test]
    fn decimals_read_as_the_closest_fraction() {
        assert_eq!(width("0.35"), Ok(Proportion::new(7, 20)));
        assert_eq!(width("1.5"), Ok(Proportion::new(3, 2)));
        assert_eq!(width("math.pi"), Ok(Proportion::new(311, 99)));
    }

    #[test]
    fn out_of_range_widths_fail() {
        for source in ["0", "-1/2", "10000.5", "0/0", "1/0", "-1/0", "0.001"] {
            assert!(width(source).is_err(), "{source}");
        }
        assert!(width("'1/2'").is_err());
    }

    #[test]
    fn presets_are_sorted_and_deduplicated() {
        let options = patched("{ width_presets = { 2/3, 1/4, 2/3 } }").unwrap();
        assert_eq!(
            options.layout.presets,
            [Proportion::new(1, 4), Proportion::new(2, 3)]
        );
    }

    #[test]
    fn patch_changes_only_named_options() {
        let mut options = patched("{ default_column_width = 1/3 }").unwrap();
        let mut expected = Options::default();
        expected.layout.default_width = Proportion::ONE_THIRD;
        assert_eq!(options, expected);
        let lua = Lua::new();
        let later = lua.load("{ default_column_width = 2/3 }").eval().unwrap();
        options.merge(lua.from_value(later).unwrap());
        assert_eq!(options.layout.default_width, Proportion::TWO_THIRDS);
    }

    #[test]
    fn every_option_is_read() {
        let options = patched(
            "{ prefix = 'ctrl+b', default_column_width = 0.35, width_presets = { 1/2 }, center_focused_column = 'on-overflow', loop_bands = false, notify_style = 'osc777' }",
        )
        .unwrap();
        assert_eq!(
            options,
            Options {
                prefix: Key::new(KeyCode::Char('b'), Modifiers::CTRL),
                layout: LayoutOptions {
                    default_width: Proportion::new(7, 20),
                    presets: vec![Proportion::ONE_HALF],
                },
                center_focused_column: CenterFocusedColumn::OnOverflow,
                loop_bands: false,
                notify_style: NotifyStyle::Osc777,
                ..Options::default()
            }
        );
    }

    #[test]
    fn steps_and_borders_are_read() {
        let options = patched(
            "{ width_step = 1/4, height_step = 0.05, tile_border_sides = { 'left', 'top', 'left' }, tile_border_chars = 'rounded', floating_border_sides = {}, floating_border_chars = { '+', '-', '+', '|', '+', '-', '+', '|' } }",
        )
        .unwrap();
        assert_eq!(
            options.steps,
            Steps {
                width: Proportion::new(1, 4),
                height: Proportion::new(1, 20),
            }
        );
        assert_eq!(options.tile_border.sides.names(), ["top", "left"]);
        assert_eq!(
            options.tile_border.chars,
            BorderChars::Named(crate::border::CharSet::Rounded)
        );
        assert_eq!(options.floating_border.sides, Sides::NONE);
        assert_eq!(
            options.floating_border.chars.glyphs(),
            ["+", "-", "+", "|", "+", "-", "+", "|"]
        );
    }

    #[test]
    fn invalid_options_fail() {
        for source in [
            "{ colum_width = 1/2 }",
            "{ width_presets = '1/2' }",
            "{ width_presets = {} }",
            "{ width_presets = { 0 } }",
            "{ default_column_width = 'wide' }",
            "{ center_focused_column = 'sometimes' }",
            "{ prefix = 'ctrl+hyper' }",
            "{ notify_style = 'osc8' }",
            "{ loop_bands = 'yes' }",
            "{ width_step = 0 }",
            "{ width_step = 10001 }",
            "{ height_step = 2 }",
            "{ height_step = 'tenth' }",
            "{ mouse_mod = 'super' }",
            "{ mouse_mod = '' }",
            "{ mouse_mod = 'ctrl+' }",
            "{ mouse_mod = 'mod' }",
            "{ mouse_mod = true }",
            "{ tile_border_sides = { 'middle' } }",
            "{ tile_border_sides = 'top' }",
            "{ tile_border_chars = 'dotted' }",
            "{ tile_border_chars = { '+', '-' } }",
            "{ floating_border_chars = { '日', '-', '+', '|', '+', '-', '+', '|' } }",
        ] {
            assert!(patched(source).is_err(), "{source}");
        }
    }
}
