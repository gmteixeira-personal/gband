use std::cell::Cell;

use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

pub const MAX_DEPTH: usize = 32;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    Bytes(Vec<u8>),
    Table(#[serde(deserialize_with = "nested")] Vec<(Key, Value)>),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Key {
    Int(i64),
    Bytes(Vec<u8>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a.to_bits() == b.to_bits(),
            (Value::Bytes(a), Value::Bytes(b)) => a == b,
            (Value::Table(a), Value::Table(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Value {
    pub fn string(text: impl Into<String>) -> Self {
        Value::Bytes(text.into().into_bytes())
    }

    pub fn encoded_len(&self) -> usize {
        1 + match self {
            Value::Nil => 0,
            Value::Bool(_) => 1,
            Value::Int(number) => varint_len(zigzag(*number)),
            Value::Float(_) => 8,
            Value::Bytes(bytes) => bytes_len(bytes),
            Value::Table(entries) => {
                varint_len(entries.len() as u64)
                    + entries
                        .iter()
                        .map(|(key, value)| key.encoded_len() + value.encoded_len())
                        .sum::<usize>()
            }
        }
    }
}

impl Key {
    pub fn string(text: impl Into<String>) -> Self {
        Key::Bytes(text.into().into_bytes())
    }

    pub fn encoded_len(&self) -> usize {
        1 + match self {
            Key::Int(number) => varint_len(zigzag(*number)),
            Key::Bytes(bytes) => bytes_len(bytes),
        }
    }
}

fn zigzag(number: i64) -> u64 {
    ((number << 1) ^ (number >> 63)) as u64
}

pub fn varint_len(mut number: u64) -> usize {
    let mut len = 1;
    while number >= 0x80 {
        number >>= 7;
        len += 1;
    }
    len
}

fn bytes_len(bytes: &[u8]) -> usize {
    varint_len(bytes.len() as u64) + bytes.len()
}

thread_local! {
    static DEPTH: Cell<usize> = const { Cell::new(0) };
}

struct Level;

impl Level {
    fn enter<E: de::Error>() -> Result<Self, E> {
        let depth = DEPTH.with(|depth| {
            depth.set(depth.get() + 1);
            depth.get()
        });
        let level = Level;
        if depth > MAX_DEPTH {
            return Err(E::custom(format!(
                "a value is nested more than {MAX_DEPTH} tables deep"
            )));
        }
        Ok(level)
    }
}

impl Drop for Level {
    fn drop(&mut self) {
        DEPTH.with(|depth| depth.set(depth.get() - 1));
    }
}

fn nested<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<(Key, Value)>, D::Error> {
    let _level = Level::enter::<D::Error>()?;
    Vec::deserialize(deserializer)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Value {
        Value::Table(vec![
            (Key::Int(1), Value::Int(-300)),
            (Key::string("f"), Value::Float(0.5)),
            (Key::string("b"), Value::Bytes(vec![0, 255])),
            (
                Key::Int(i64::MIN),
                Value::Table(vec![
                    (Key::Int(1), Value::Bool(true)),
                    (Key::Int(2), Value::Nil),
                ]),
            ),
        ])
    }

    #[test]
    fn encoded_len_matches_postcard() {
        for value in [
            Value::Nil,
            Value::Int(i64::MAX),
            Value::Bytes(vec![7; 300]),
            sample(),
        ] {
            assert_eq!(
                value.encoded_len(),
                postcard::to_allocvec(&value).unwrap().len(),
                "{value:?}"
            );
        }
    }

    #[test]
    fn float_equality_is_bitwise() {
        assert_eq!(Value::Float(f64::NAN), Value::Float(f64::NAN));
        assert_ne!(Value::Float(0.0), Value::Float(-0.0));
        assert_ne!(Value::Int(1), Value::Float(1.0));
    }
}
