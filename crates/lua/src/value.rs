use std::collections::HashSet;
use std::ffi::c_void;

use gband_protocol::{Key, MAX_DEPTH, Value};
use mlua::{Lua, Table};

pub const MAX_ENCODED: usize = 1024 * 1024;

pub(crate) fn from_lua(value: &mlua::Value, root: &str) -> Result<Value, String> {
    let mut walk = Walk {
        path: vec![root.to_owned()],
        open: HashSet::new(),
    };
    let converted = walk.value(value)?;
    if converted.encoded_len() > MAX_ENCODED {
        return Err(format!("`{root}` is larger than 1 MiB when encoded"));
    }
    Ok(converted)
}

struct Walk {
    path: Vec<String>,
    open: HashSet<*const c_void>,
}

impl Walk {
    fn here(&self) -> String {
        self.path.concat()
    }

    fn value(&mut self, value: &mlua::Value) -> Result<Value, String> {
        Ok(match value {
            mlua::Value::Nil => Value::Nil,
            mlua::Value::Boolean(flag) => Value::Bool(*flag),
            mlua::Value::Integer(number) => Value::Int(*number),
            mlua::Value::Number(number) => Value::Float(*number),
            mlua::Value::String(text) => Value::Bytes(text.as_bytes().to_vec()),
            mlua::Value::Table(table) => self.table(table)?,
            other => {
                return Err(format!(
                    "`{}` is a {}, which is not plain data",
                    self.here(),
                    other.type_name()
                ));
            }
        })
    }

    fn table(&mut self, table: &Table) -> Result<Value, String> {
        let pointer = table.to_pointer();
        if !self.open.insert(pointer) {
            return Err(format!("`{}` holds a table that contains it", self.here()));
        }
        if self.open.len() > MAX_DEPTH {
            return Err(format!(
                "`{}` is nested more than {MAX_DEPTH} tables deep",
                self.here()
            ));
        }
        let mut entries = Vec::new();
        for pair in table.pairs::<mlua::Value, mlua::Value>() {
            let (key, value) = pair.map_err(|error| error.to_string())?;
            let (key, segment) = match &key {
                mlua::Value::Integer(number) => (Key::Int(*number), format!("[{number}]")),
                mlua::Value::String(text) => {
                    let bytes = text.as_bytes().to_vec();
                    let segment = segment(&text.to_string_lossy());
                    (Key::Bytes(bytes), segment)
                }
                other => {
                    return Err(format!(
                        "`{}` has a {} key; keys must be strings or integers",
                        self.here(),
                        other.type_name()
                    ));
                }
            };
            self.path.push(segment);
            let value = self.value(&value)?;
            self.path.pop();
            entries.push((key, value));
        }
        self.open.remove(&pointer);
        Ok(Value::Table(entries))
    }
}

fn segment(name: &str) -> String {
    let identifier = name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if identifier {
        format!(".{name}")
    } else {
        format!("[{name:?}]")
    }
}

pub(crate) fn into_lua(lua: &Lua, value: &Value) -> mlua::Result<mlua::Value> {
    Ok(match value {
        Value::Nil => mlua::Value::Nil,
        Value::Bool(flag) => mlua::Value::Boolean(*flag),
        Value::Int(number) => mlua::Value::Integer(*number),
        Value::Float(number) => mlua::Value::Number(*number),
        Value::Bytes(bytes) => mlua::Value::String(lua.create_string(bytes)?),
        Value::Table(entries) => {
            let table = lua.create_table_with_capacity(0, entries.len())?;
            for (key, value) in entries {
                let value = into_lua(lua, value)?;
                match key {
                    Key::Int(number) => table.raw_set(*number, value)?,
                    Key::Bytes(bytes) => table.raw_set(lua.create_string(bytes)?, value)?,
                }
            }
            mlua::Value::Table(table)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn convert(source: &str) -> Result<Value, String> {
        let lua = Lua::new();
        let value: mlua::Value = lua.load(source).eval().unwrap();
        from_lua(&value, "data")
    }

    fn round_trip(source: &str) -> bool {
        let lua = Lua::new();
        let value: mlua::Value = lua.load(source).eval().unwrap();
        let converted = from_lua(&value, "data").unwrap();
        let back = into_lua(&lua, &converted).unwrap();
        lua.globals().set("a", value).unwrap();
        lua.globals().set("b", back).unwrap();
        lua.load(
            r#"
            local function equal(x, y)
              if type(x) ~= type(y) then return false end
              if type(x) ~= "table" then return x == y and math.type(x) == math.type(y) end
              if x == y then return false end
              for k, v in pairs(x) do if not equal(v, y[k]) then return false end end
              for k in pairs(y) do if x[k] == nil then return false end end
              return true
            end
            return equal(a, b)
            "#,
        )
        .eval()
        .unwrap()
    }

    #[test]
    fn nested_data_round_trip() {
        assert!(round_trip(
            r#"{ title = "a", n = 2.5, tags = { "x", "y" }, ok = true, deep = { { 1 } } }"#
        ));
        assert!(round_trip("3"));
        assert!(round_trip("'text'"));
    }

    #[test]
    fn function_inside_a_table() {
        let error = convert("{ cb = { run = function() end } }").unwrap_err();
        assert!(error.contains("`data.cb.run`"), "{error}");
        assert!(error.contains("function"), "{error}");
    }

    #[test]
    fn cycle() {
        let error = convert("local t = {}; t.self = t; return t").unwrap_err();
        assert!(error.contains("`data.self`"), "{error}");
    }

    #[test]
    fn shared_table_is_not_a_cycle() {
        let value = convert("local s = { 1 }; return { a = s, b = s }").unwrap();
        let Value::Table(entries) = value else {
            panic!("a table");
        };
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn binary_string() {
        assert_eq!(convert(r#""\0\255""#), Ok(Value::Bytes(vec![0, 255])));
        assert!(round_trip(r#"{ [1] = "\0\255" }"#));
    }

    #[test]
    fn integer_and_string_keys_are_kept_apart() {
        let value = convert(r#"{ [1] = "int", ["1"] = "text" }"#).unwrap();
        let Value::Table(mut entries) = value else {
            panic!("a table");
        };
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(
            entries,
            [
                (Key::Int(1), Value::string("int")),
                (Key::string("1"), Value::string("text")),
            ]
        );
        assert!(round_trip(r#"{ [1] = "int", ["1"] = "text" }"#));
    }

    #[test]
    fn metatables_are_ignored() {
        let value = convert(
            "return setmetatable({ a = 1 }, { __index = function() return 2 end, __pairs = error })",
        )
        .unwrap();
        assert_eq!(value, Value::Table(vec![(Key::string("a"), Value::Int(1))]));
    }

    #[test]
    fn depth_limit() {
        let nested = |depth: usize| format!("{}{}", "{ ".repeat(depth), " }".repeat(depth));
        assert!(convert(&format!("return {}", nested(32))).is_ok());
        let error = convert(&format!("return {}", nested(33))).unwrap_err();
        assert!(error.contains("32"), "{error}");
    }

    #[test]
    fn size_limit() {
        assert!(convert("return string.rep('x', 1024 * 1024 - 8)").is_ok());
        let error = convert("return { string.rep('x', 1024 * 1024) }").unwrap_err();
        assert!(error.contains("1 MiB"), "{error}");
    }

    #[test]
    fn odd_keys_are_named() {
        let error = convert(r#"{ ["a b"] = { [2] = print } }"#).unwrap_err();
        assert!(error.contains(r#"`data["a b"][2]`"#), "{error}");
        let error = convert("{ [1.5] = 1 }").unwrap_err();
        assert!(error.contains("number key"), "{error}");
    }
}
