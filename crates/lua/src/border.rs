use std::fmt;

use mlua::{IntoLua, Lua, Value};
use serde::Deserialize;
use serde::de::{self, Deserializer, MapAccess, SeqAccess, Visitor};

use crate::ui;

const SIDE_NAMES: [&str; 4] = ["top", "right", "bottom", "left"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sides {
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
    pub left: bool,
}

impl Sides {
    pub const ALL: Self = Self {
        top: true,
        right: true,
        bottom: true,
        left: true,
    };
    pub const NONE: Self = Self {
        top: false,
        right: false,
        bottom: false,
        left: false,
    };

    fn flags(self) -> [bool; 4] {
        [self.top, self.right, self.bottom, self.left]
    }

    pub fn names(self) -> Vec<&'static str> {
        SIDE_NAMES
            .iter()
            .zip(self.flags())
            .filter_map(|(&name, drawn)| drawn.then_some(name))
            .collect()
    }

    pub fn parse<'a>(names: impl IntoIterator<Item = &'a str>) -> Result<Self, String> {
        let mut sides = Self::NONE;
        for name in names {
            let flag = match name {
                "top" => &mut sides.top,
                "right" => &mut sides.right,
                "bottom" => &mut sides.bottom,
                "left" => &mut sides.left,
                other => {
                    return Err(format!(
                        "unknown side `{other}`; expected top, right, bottom or left"
                    ));
                }
            };
            *flag = true;
        }
        Ok(sides)
    }

    pub fn to_lua(self, lua: &Lua) -> mlua::Result<Value> {
        lua.create_sequence_from(self.names())?.into_lua(lua)
    }
}

impl Default for Sides {
    fn default() -> Self {
        Self::ALL
    }
}

impl<'de> Deserialize<'de> for Sides {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let names = Vec::<String>::deserialize(deserializer)?;
        Sides::parse(names.iter().map(String::as_str)).map_err(de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CharSet {
    #[default]
    Plain,
    Rounded,
    Double,
    Thick,
}

impl CharSet {
    const ALL: [(&'static str, CharSet); 4] = [
        ("plain", CharSet::Plain),
        ("rounded", CharSet::Rounded),
        ("double", CharSet::Double),
        ("thick", CharSet::Thick),
    ];

    fn name(self) -> &'static str {
        Self::ALL
            .iter()
            .find(|(_, set)| *set == self)
            .map_or("plain", |(name, _)| name)
    }

    fn glyphs(self) -> [&'static str; 8] {
        match self {
            CharSet::Plain => ["┌", "─", "┐", "│", "┘", "─", "└", "│"],
            CharSet::Rounded => ["╭", "─", "╮", "│", "╯", "─", "╰", "│"],
            CharSet::Double => ["╔", "═", "╗", "║", "╝", "═", "╚", "║"],
            CharSet::Thick => ["┏", "━", "┓", "┃", "┛", "━", "┗", "┃"],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BorderChars {
    Named(CharSet),
    Custom(Box<[String; 8]>),
}

impl Default for BorderChars {
    fn default() -> Self {
        BorderChars::Named(CharSet::Plain)
    }
}

impl BorderChars {
    pub fn glyphs(&self) -> [&str; 8] {
        match self {
            BorderChars::Named(set) => set.glyphs(),
            BorderChars::Custom(glyphs) => std::array::from_fn(|index| glyphs[index].as_str()),
        }
    }

    pub fn named(name: &str) -> Result<Self, String> {
        CharSet::ALL
            .iter()
            .find(|(known, _)| *known == name)
            .map(|&(_, set)| BorderChars::Named(set))
            .ok_or_else(|| {
                format!("unknown character set `{name}`; expected plain, rounded, double or thick")
            })
    }

    pub fn custom(glyphs: Vec<String>) -> Result<Self, String> {
        let count = glyphs.len();
        let glyphs: [String; 8] = glyphs
            .try_into()
            .map_err(|_| format!("expected a list of 8 strings, found {count}"))?;
        if let Some(wide) = glyphs.iter().find(|glyph| ui::width(glyph) != 1) {
            return Err(format!(
                "every border character must be one cell wide, found `{wide}`"
            ));
        }
        Ok(BorderChars::Custom(Box::new(glyphs)))
    }

    pub fn to_lua(&self, lua: &Lua) -> mlua::Result<Value> {
        match self {
            BorderChars::Named(set) => set.name().into_lua(lua),
            BorderChars::Custom(glyphs) => lua
                .create_sequence_from(glyphs.iter().map(String::as_str))?
                .into_lua(lua),
        }
    }
}

impl<'de> Deserialize<'de> for BorderChars {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Chars;

        impl<'de> Visitor<'de> for Chars {
            type Value = BorderChars;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a character set name or a list of 8 strings")
            }

            fn visit_str<E: de::Error>(self, name: &str) -> Result<BorderChars, E> {
                BorderChars::named(name).map_err(E::custom)
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<BorderChars, A::Error> {
                let mut glyphs = Vec::new();
                while let Some(glyph) = seq.next_element::<String>()? {
                    glyphs.push(glyph);
                }
                BorderChars::custom(glyphs).map_err(de::Error::custom)
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<BorderChars, A::Error> {
                match map.next_key::<de::IgnoredAny>()? {
                    None => BorderChars::custom(Vec::new()).map_err(de::Error::custom),
                    Some(_) => Err(de::Error::custom("expected a list of 8 strings")),
                }
            }
        }

        deserializer.deserialize_any(Chars)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Border {
    pub sides: Sides,
    pub chars: BorderChars,
}
