use std::fmt;

use gband_core::input::Key;
use gband_core::layout::{LayoutOptions, Proportion};
use gband_core::view::CenterFocusedColumn;
use serde::Deserialize;
use serde::de::{self, Deserializer, Visitor};

use crate::keys::parse_key;

const MAX_DENOMINATOR: u64 = 100;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    pub prefix: Key,
    pub layout: LayoutOptions,
    pub center_focused_column: CenterFocusedColumn,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PartialOptions {
    pub prefix: Option<Key>,
    pub default_width: Option<Proportion>,
    pub presets: Option<Vec<Proportion>>,
    pub center_focused_column: Option<CenterFocusedColumn>,
}

impl PartialOptions {
    pub fn merge(&mut self, patch: OptionsPatch) {
        if let Some(KeySpec(prefix)) = patch.prefix {
            self.prefix = Some(prefix);
        }
        if let Some(Width(width)) = patch.default_column_width {
            self.default_width = Some(width);
        }
        if let Some(Presets(presets)) = patch.width_presets {
            let mut presets: Vec<Proportion> =
                presets.into_iter().map(|Width(width)| width).collect();
            presets.sort_by(|a, b| {
                (u64::from(a.num) * u64::from(b.den)).cmp(&(u64::from(b.num) * u64::from(a.den)))
            });
            presets.dedup();
            self.presets = Some(presets);
        }
        if let Some(policy) = patch.center_focused_column {
            self.center_focused_column = Some(policy);
        }
    }

    pub fn complete(self) -> Option<Options> {
        Some(Options {
            prefix: self.prefix?,
            layout: LayoutOptions {
                default_width: self.default_width?,
                presets: self.presets?,
            },
            center_focused_column: self.center_focused_column?,
        })
    }
}

pub const NAMES: [&str; 4] = [
    "prefix",
    "default_column_width",
    "width_presets",
    "center_focused_column",
];

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptionsPatch {
    prefix: Option<KeySpec>,
    default_column_width: Option<Width>,
    width_presets: Option<Presets>,
    center_focused_column: Option<CenterFocusedColumn>,
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
    use gband_core::input::{KeyCode, Modifiers};
    use mlua::{Lua, LuaSerdeExt};

    use super::*;

    fn width(source: &str) -> Result<Proportion, String> {
        let lua = Lua::new();
        let value = lua.load(source).eval().unwrap();
        lua.from_value::<Width>(value)
            .map(|Width(width)| width)
            .map_err(|error| error.to_string())
    }

    fn patched(source: &str) -> Result<PartialOptions, String> {
        let lua = Lua::new();
        let value = lua.load(source).eval().unwrap();
        let patch: OptionsPatch = lua.from_value(value).map_err(|error| error.to_string())?;
        let mut options = PartialOptions::default();
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
            options.presets,
            Some(vec![Proportion::new(1, 4), Proportion::new(2, 3)])
        );
    }

    #[test]
    fn patch_changes_only_named_options() {
        let mut options = patched("{ default_column_width = 1/3 }").unwrap();
        assert_eq!(
            options,
            PartialOptions {
                default_width: Some(Proportion::ONE_THIRD),
                ..PartialOptions::default()
            }
        );
        let lua = Lua::new();
        let later = lua.load("{ default_column_width = 2/3 }").eval().unwrap();
        options.merge(lua.from_value(later).unwrap());
        assert_eq!(options.default_width, Some(Proportion::TWO_THIRDS));
        assert_eq!(options.complete(), None);
    }

    #[test]
    fn every_option_is_read() {
        let options = patched(
            "{ prefix = 'ctrl+b', default_column_width = 0.35, width_presets = { 1/2 }, center_focused_column = 'on-overflow' }",
        )
        .unwrap();
        assert_eq!(
            options.complete().unwrap(),
            Options {
                prefix: Key::new(KeyCode::Char('b'), Modifiers::CTRL),
                layout: LayoutOptions {
                    default_width: Proportion::new(7, 20),
                    presets: vec![Proportion::ONE_HALF],
                },
                center_focused_column: CenterFocusedColumn::OnOverflow,
            }
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
        ] {
            assert!(patched(source).is_err(), "{source}");
        }
    }
}
