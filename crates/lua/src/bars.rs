use mlua::{IntoLuaMulti, Lua, MultiValue, RegistryKey, Table, Value};

use crate::plugin_windows::{Run, read_runs};
use crate::ui::{self, Style};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarSide {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    pub side: BarSide,
    pub size: u16,
    pub order: f64,
    pub seq: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Columns {
    pub col: u16,
    pub width: u16,
}

pub fn place(slots: &[Slot], cols: u16) -> (Vec<Option<Columns>>, Columns) {
    let mut indices: Vec<usize> = (0..slots.len()).collect();
    indices.sort_by(|&a, &b| {
        slots[a]
            .order
            .total_cmp(&slots[b].order)
            .then(slots[a].seq.cmp(&slots[b].seq))
    });
    let mut placed = vec![None; slots.len()];
    let (mut left, mut right) = (0, cols);
    for index in indices {
        let slot = slots[index];
        if slot.size >= right - left {
            continue;
        }
        placed[index] = Some(match slot.side {
            BarSide::Left => {
                left += slot.size;
                Columns {
                    col: left - slot.size,
                    width: slot.size,
                }
            }
            BarSide::Right => {
                right -= slot.size;
                Columns {
                    col: right,
                    width: slot.size,
                }
            }
        });
    }
    (
        placed,
        Columns {
            col: left,
            width: right - left,
        },
    )
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bar {
    pub id: String,
    pub slot: Slot,
    pub base: Style,
    pub lines: Vec<Vec<Run>>,
}

#[derive(Default)]
struct Presented(Option<Vec<Bar>>);

#[derive(Default)]
struct Hooks(Option<RegistryKey>);

#[derive(Default)]
struct ErrorItem(bool);

pub(crate) fn install(lua: &Lua, host: &Table) -> mlua::Result<()> {
    lua.set_app_data(Presented::default());
    lua.set_app_data(Hooks::default());
    lua.set_app_data(ErrorItem::default());
    host.set("place_bars", lua.create_function(place_bars)?)?;
    host.set(
        "present_bars",
        lua.create_function(|lua, list: Table| {
            let bars = list
                .sequence_values::<Table>()
                .map(|bar| read_bar(&bar?))
                .collect::<mlua::Result<Vec<_>>>()?;
            presented(lua).0 = Some(bars);
            Ok(())
        })?,
    )?;
    host.set(
        "bar_hooks",
        lua.create_function(|lua, table: Table| {
            let key = lua.create_registry_value(table)?;
            lua.app_data_mut::<Hooks>()
                .expect("the bar hooks are installed with the runtime")
                .0 = Some(key);
            Ok(())
        })?,
    )?;
    host.set(
        "error_item",
        lua.create_function(|lua, shown: bool| {
            lua.app_data_mut::<ErrorItem>()
                .expect("the error item is installed with the runtime")
                .0 = shown;
            Ok(())
        })?,
    )
}

fn presented(lua: &Lua) -> mlua::AppDataRefMut<'_, Presented> {
    lua.app_data_mut::<Presented>()
        .expect("the bars are installed with the runtime")
}

fn side(name: &str) -> mlua::Result<BarSide> {
    match name {
        "left" => Ok(BarSide::Left),
        "right" => Ok(BarSide::Right),
        other => Err(mlua::Error::runtime(format!("unknown bar side `{other}`"))),
    }
}

fn read_slot(table: &Table) -> mlua::Result<Slot> {
    Ok(Slot {
        side: side(&table.get::<String>("side")?)?,
        size: table.get("size")?,
        order: table.get("order")?,
        seq: table.get("seq")?,
    })
}

fn read_bar(table: &Table) -> mlua::Result<Bar> {
    Ok(Bar {
        id: table.get("id")?,
        slot: read_slot(table)?,
        base: ui::style(&table.get("base")?)?,
        lines: read_runs(&table.get("lines")?)?,
    })
}

fn place_bars(lua: &Lua, (list, cols): (Table, u16)) -> mlua::Result<MultiValue> {
    let slots = list
        .sequence_values::<Table>()
        .map(|slot| read_slot(&slot?))
        .collect::<mlua::Result<Vec<_>>>()?;
    let (placed, ribbon) = place(&slots, cols);
    let columns = |columns: Columns| -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("col", columns.col)?;
        table.set("width", columns.width)?;
        Ok(table)
    };
    let shown = lua.create_table()?;
    for columns_of in placed {
        match columns_of {
            Some(placed) => shown.push(columns(placed)?)?,
            None => shown.push(false)?,
        }
    }
    (shown, columns(ribbon)?).into_lua_multi(lua)
}

pub(crate) fn take(lua: &Lua) -> Option<Vec<Bar>> {
    presented(lua).0.take()
}

pub(crate) fn error_item_shown(lua: &Lua) -> bool {
    lua.app_data_ref::<ErrorItem>().is_some_and(|shown| shown.0)
}

pub(crate) fn flush(lua: &Lua) -> mlua::Result<()> {
    let hooks = match &lua
        .app_data_ref::<Hooks>()
        .expect("the bar hooks are installed with the runtime")
        .0
    {
        Some(key) => lua.registry_value::<Table>(key)?,
        None => return Ok(()),
    };
    match hooks.get::<Value>("flush")? {
        Value::Function(flush) => flush.call::<()>(()),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(side: BarSide, size: u16, order: f64, seq: u64) -> Slot {
        Slot {
            side,
            size,
            order,
            seq,
        }
    }

    fn columns(col: u16, width: u16) -> Columns {
        Columns { col, width }
    }

    #[test]
    fn left_bar() {
        let (placed, ribbon) = place(&[slot(BarSide::Left, 20, 0.0, 1)], 80);
        assert_eq!(placed, [Some(columns(0, 20))]);
        assert_eq!(ribbon, columns(20, 60));
    }

    #[test]
    fn two_bars_on_one_side() {
        let (placed, ribbon) = place(
            &[
                slot(BarSide::Left, 10, 0.0, 1),
                slot(BarSide::Left, 5, -1.0, 2),
            ],
            80,
        );
        assert_eq!(placed, [Some(columns(5, 10)), Some(columns(0, 5))]);
        assert_eq!(ribbon.col, 15);
    }

    #[test]
    fn equal_orders_follow_the_order_of_adding() {
        let (placed, _) = place(
            &[
                slot(BarSide::Right, 3, 0.0, 2),
                slot(BarSide::Right, 4, 0.0, 1),
            ],
            80,
        );
        assert_eq!(placed, [Some(columns(73, 3)), Some(columns(76, 4))]);
    }

    #[test]
    fn bars_on_both_sides() {
        let (placed, ribbon) = place(
            &[
                slot(BarSide::Left, 20, 0.0, 1),
                slot(BarSide::Right, 10, 0.0, 2),
            ],
            80,
        );
        assert_eq!(placed, [Some(columns(0, 20)), Some(columns(70, 10))]);
        assert_eq!(ribbon, columns(20, 50));
    }

    #[test]
    fn bar_too_wide() {
        let (placed, ribbon) = place(&[slot(BarSide::Left, 10, 0.0, 1)], 10);
        assert_eq!(placed, [None]);
        assert_eq!(ribbon, columns(0, 10));
    }

    #[test]
    fn a_hidden_bar_leaves_room_for_the_next() {
        let (placed, ribbon) = place(
            &[
                slot(BarSide::Left, 30, 0.0, 1),
                slot(BarSide::Right, 5, 1.0, 2),
            ],
            20,
        );
        assert_eq!(placed, [None, Some(columns(15, 5))]);
        assert_eq!(ribbon, columns(0, 15));
    }
}
