use std::sync::OnceLock;
use std::sync::atomic::{AtomicI64, Ordering};

use mlua::{Function, Lua, MultiValue, Table, Value};

const UNSET: i64 = i64::MIN;

static FROZEN: OnceLock<AtomicI64> = OnceLock::new();

pub fn freeze(instant: Option<i64>) {
    FROZEN
        .get_or_init(|| AtomicI64::new(UNSET))
        .store(instant.unwrap_or(UNSET), Ordering::Release);
}

fn instant() -> Option<i64> {
    let instant = FROZEN.get()?.load(Ordering::Acquire);
    (instant != UNSET).then_some(instant)
}

pub(crate) fn install(lua: &Lua) -> mlua::Result<()> {
    if FROZEN.get().is_none() {
        return Ok(());
    }
    let os: Table = lua.globals().get("os")?;
    let time: Function = os.get("time")?;
    let date: Function = os.get("date")?;
    os.set(
        "time",
        lua.create_function(move |_, args: MultiValue| match (instant(), args.front()) {
            (Some(instant), None | Some(Value::Nil)) => {
                Ok(MultiValue::from_iter([Value::Integer(instant)]))
            }
            _ => time.call::<MultiValue>(args),
        })?,
    )?;
    os.set(
        "date",
        lua.create_function(move |_, args: MultiValue| {
            let mut args: Vec<Value> = args.into_iter().collect();
            if let Some(instant) = instant()
                && matches!(args.get(1), None | Some(Value::Nil))
            {
                args.resize(2, Value::Nil);
                args[1] = Value::Integer(instant);
            }
            date.call::<MultiValue>(MultiValue::from_iter(args))
        })?,
    )
}
