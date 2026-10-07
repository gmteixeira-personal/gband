use gband_core::layout::WindowId;
use mlua::{Function, Lua, RegistryKey, Table, Value};

use crate::check;
use crate::error::ConfigError;
use crate::guard;
use crate::ui::{self, Color, Style};

const LABEL: &str = "decorations";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecorationInfo {
    pub window: WindowId,
    pub floating: bool,
    pub focused: bool,
    pub width: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StylePatch {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub reverse: Option<bool>,
    pub dim: Option<bool>,
}

impl StylePatch {
    pub fn over(&self, base: Style) -> Style {
        Style {
            fg: self.fg.or(base.fg),
            bg: self.bg.or(base.bg),
            bold: self.bold.unwrap_or(base.bold),
            italic: self.italic.unwrap_or(base.italic),
            underline: self.underline.unwrap_or(base.underline),
            reverse: self.reverse.unwrap_or(base.reverse),
            dim: self.dim.unwrap_or(base.dim),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DecorationSpan {
    pub text: String,
    pub style: StylePatch,
}

impl DecorationSpan {
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: StylePatch::default(),
        }
    }

    pub fn width(&self) -> usize {
        ui::width(&self.text)
    }
}

#[derive(Default)]
struct Provider {
    function: Option<RegistryKey>,
    stopped: bool,
}

pub(crate) fn install(lua: &Lua) {
    lua.set_app_data(Provider::default());
}

pub(crate) fn provide(lua: &Lua, function: Function) -> mlua::Result<()> {
    let key = lua.create_registry_value(function)?;
    let mut provider = lua
        .app_data_mut::<Provider>()
        .expect("the decorations provider is installed with the runtime");
    provider.function = Some(key);
    provider.stopped = false;
    Ok(())
}

fn registered(lua: &Lua) -> mlua::Result<Option<Function>> {
    let Some(provider) = lua.app_data_ref::<Provider>() else {
        return Ok(None);
    };
    match &provider.function {
        Some(key) if !provider.stopped => lua.registry_value(key).map(Some),
        _ => Ok(None),
    }
}

fn stop(lua: &Lua) {
    if let Some(mut provider) = lua.app_data_mut::<Provider>() {
        provider.stopped = true;
    }
}

fn labelled(mut error: ConfigError) -> ConfigError {
    error.plugin = Some(LABEL.to_owned());
    error
}

pub(crate) fn call(lua: &Lua, info: &DecorationInfo) -> Result<Vec<DecorationSpan>, ConfigError> {
    let result = run(lua, info);
    if result.is_err() {
        stop(lua);
    }
    result
}

fn run(lua: &Lua, info: &DecorationInfo) -> Result<Vec<DecorationSpan>, ConfigError> {
    let internal =
        |error: mlua::Error| labelled(ConfigError::from_lua(&error, &guard::sources(lua)));
    let Some(function) = registered(lua).map_err(internal)? else {
        return Ok(Vec::new());
    };
    let returned = guard::isolated(lua, None, Some(LABEL.to_owned()), || {
        function.call::<Value>(info_table(lua, info)?)
    })
    .map_err(internal)?
    .map_err(|failure| failure.error)?;
    read_spans(&returned).map_err(|message| labelled(ConfigError::new(message)))
}

fn info_table(lua: &Lua, info: &DecorationInfo) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("window", info.window.0)?;
    table.set("floating", info.floating)?;
    table.set("focused", info.focused)?;
    table.set("width", info.width)?;
    Ok(table)
}

fn read_spans(value: &Value) -> Result<Vec<DecorationSpan>, String> {
    let list = match value {
        Value::Nil => return Ok(Vec::new()),
        Value::Table(list) => list,
        other => {
            return Err(format!(
                "expected nil or a list of spans, found {}",
                other.type_name()
            ));
        }
    };
    (1..=list.raw_len())
        .map(|place| {
            let entry: Value = list.raw_get(place).map_err(|error| error.to_string())?;
            read_span(&entry).map_err(|reason| format!("span {place}: {reason}"))
        })
        .collect()
}

fn read_span(entry: &Value) -> Result<DecorationSpan, String> {
    let expected = || {
        format!(
            "expected a string or a table with a string text, found {}",
            entry.type_name()
        )
    };
    match entry {
        Value::String(text) => Ok(DecorationSpan::plain(ui::strip(&text.to_string_lossy()))),
        Value::Table(span) => {
            let Value::String(text) = span
                .get::<Value>("text")
                .map_err(|error| error.to_string())?
            else {
                return Err(expected());
            };
            let style = match span
                .get::<Value>("style")
                .map_err(|error| error.to_string())?
            {
                Value::Nil => StylePatch::default(),
                Value::Table(style) => {
                    read_patch(&style).map_err(|reason| format!("the style: {reason}"))?
                }
                other => {
                    return Err(format!(
                        "the field `style` must be a style table, found {}",
                        other.type_name()
                    ));
                }
            };
            Ok(DecorationSpan {
                text: ui::strip(&text.to_string_lossy()),
                style,
            })
        }
        _ => Err(expected()),
    }
}

fn read_patch(style: &Table) -> Result<StylePatch, String> {
    let flag = |name: &str| check::optional_boolean_field(style, name, "a boolean");
    Ok(StylePatch {
        fg: ui::color(style, "fg")?,
        bg: ui::color(style, "bg")?,
        bold: flag("bold")?,
        italic: flag("italic")?,
        underline: flag("underline")?,
        reverse: flag("reverse")?,
        dim: flag("dim")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spans(source: &str) -> Result<Vec<DecorationSpan>, String> {
        let lua = Lua::new();
        read_spans(&lua.load(source).eval::<Value>().unwrap())
    }

    #[test]
    fn nil_and_an_empty_list_give_no_spans() {
        assert_eq!(spans("return nil"), Ok(Vec::new()));
        assert_eq!(spans("return {}"), Ok(Vec::new()));
    }

    #[test]
    fn strings_are_plain_spans() {
        assert_eq!(
            spans("return { '[_]', '[X]' }"),
            Ok(vec![
                DecorationSpan::plain("[_]"),
                DecorationSpan::plain("[X]")
            ])
        );
    }

    #[test]
    fn tables_carry_a_style_patch() {
        let read = spans("return { { text = '[X]', style = { fg = 1, bold = false } } }").unwrap();
        assert_eq!(
            read,
            vec![DecorationSpan {
                text: "[X]".to_owned(),
                style: StylePatch {
                    fg: Some(Color::Index(1)),
                    bold: Some(false),
                    ..StylePatch::default()
                },
            }]
        );
    }

    #[test]
    fn a_table_without_a_style_is_plain() {
        assert_eq!(
            spans("return { { text = 'x' } }"),
            Ok(vec![DecorationSpan::plain("x")])
        );
    }

    #[test]
    fn control_characters_are_removed() {
        assert_eq!(
            spans("return { 'a\\tb', { text = 'c\\27d' } }"),
            Ok(vec![
                DecorationSpan::plain("ab"),
                DecorationSpan::plain("cd")
            ])
        );
    }

    #[test]
    fn a_wrong_entry_is_named_by_its_place() {
        assert_eq!(
            spans("return { '[_]', 7 }"),
            Err(
                "span 2: expected a string or a table with a string text, found integer".to_owned()
            )
        );
        assert_eq!(
            spans("return { { text = 3 } }"),
            Err("span 1: expected a string or a table with a string text, found table".to_owned())
        );
    }

    #[test]
    fn a_wrong_style_is_named() {
        assert_eq!(
            spans("return { 'a', { text = 'b', style = 'red' } }"),
            Err("span 2: the field `style` must be a style table, found string".to_owned())
        );
        assert_eq!(
            spans("return { { text = 'b', style = { bold = 1 } } }"),
            Err("span 1: the style: the field `bold` must be a boolean, found integer".to_owned())
        );
    }

    #[test]
    fn a_value_other_than_a_list_is_wrong() {
        assert_eq!(
            spans("return 5"),
            Err("expected nil or a list of spans, found integer".to_owned())
        );
    }

    #[test]
    fn a_patch_keeps_absent_fields_of_the_base() {
        let base = Style {
            fg: Some(Color::Rgb(0xb1, 0xb9, 0xf9)),
            bold: true,
            dim: true,
            ..Style::default()
        };
        let patch = StylePatch {
            fg: Some(Color::Index(1)),
            dim: Some(false),
            ..StylePatch::default()
        };
        assert_eq!(
            patch.over(base),
            Style {
                fg: Some(Color::Index(1)),
                bold: true,
                ..Style::default()
            }
        );
        assert_eq!(StylePatch::default().over(base), base);
    }
}
