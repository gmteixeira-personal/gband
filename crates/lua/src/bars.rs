use mlua::{IntoLuaMulti, Lua, MultiValue, RegistryKey, Table, Value};

use crate::check;
use crate::plugin_windows::{Run, read_runs};
use crate::ui::{self, Style, named};

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
struct ErrorMarker(bool);

pub(crate) fn install(lua: &Lua, core: &Table) -> mlua::Result<()> {
    lua.set_app_data(Presented::default());
    lua.set_app_data(Hooks::default());
    lua.set_app_data(ErrorMarker::default());
    core.set("place_bars", lua.create_function(place_bars)?)?;
    core.set(
        "present_bars",
        lua.create_function(|lua, list: Value| {
            let what = named("present_bars");
            let list = check::table(lua, &what, &list, "a list of bars")?;
            let bars = check::checked(lua, &what, read_list(&list, "bar", read_bar))?;
            presented(lua).0 = Some(bars);
            Ok(())
        })?,
    )?;
    core.set(
        "error_marker",
        lua.create_function(|lua, shown: Value| {
            let shown = check::boolean(lua, &named("error_marker"), &shown, "a boolean")?;
            lua.app_data_mut::<ErrorMarker>()
                .expect("the error marker is installed with the runtime")
                .0 = shown;
            Ok(())
        })?,
    )
}

pub(crate) fn provide(lua: &Lua, implementation: Table) -> mlua::Result<()> {
    let key = lua.create_registry_value(implementation)?;
    lua.app_data_mut::<Hooks>()
        .expect("the bar hooks are installed with the runtime")
        .0 = Some(key);
    Ok(())
}

fn presented(lua: &Lua) -> mlua::AppDataRefMut<'_, Presented> {
    lua.app_data_mut::<Presented>()
        .expect("the bars are installed with the runtime")
}

fn read_list<T>(
    list: &Table,
    entry: &str,
    read: impl Fn(&Table) -> Result<T, String>,
) -> Result<Vec<T>, String> {
    list.sequence_values::<Value>()
        .enumerate()
        .map(|(index, value)| {
            let at = |reason: String| format!("{entry} {}: {reason}", index + 1);
            match value.map_err(|error| error.to_string())? {
                Value::Table(table) => read(&table).map_err(at),
                other => Err(at(format!("expected a table, found {}", other.type_name()))),
            }
        })
        .collect()
}

fn read_slot(table: &Table) -> Result<Slot, String> {
    let side = match check::text_field(table, "side", "`left` or `right`")?.as_str() {
        "left" => BarSide::Left,
        "right" => BarSide::Right,
        other => {
            return Err(format!(
                "the field `side` must be `left` or `right`, found `{other}`"
            ));
        }
    };
    Ok(Slot {
        side,
        size: check::whole_field(table, "size", "an integer from 0 to 65535")?,
        order: check::number_field(table, "order", "a number")?,
        seq: check::whole_field(table, "seq", "a non-negative integer")?,
    })
}

fn read_bar(table: &Table) -> Result<Bar, String> {
    Ok(Bar {
        id: check::text_field(table, "id", "a string")?,
        slot: read_slot(table)?,
        base: ui::style_field(table, "base")?,
        lines: read_runs(table)?,
    })
}

fn place_bars(lua: &Lua, (list, cols): (Value, Value)) -> mlua::Result<MultiValue> {
    let what = named("place_bars");
    let list = check::table(lua, &what, &list, "a list of slots")?;
    let cols: u16 = check::integer(lua, &what, &cols, "a column count from 0 to 65535")?;
    let slots = check::checked(lua, &what, read_list(&list, "slot", read_slot))?;
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

pub(crate) fn error_marker_shown(lua: &Lua) -> bool {
    lua.app_data_ref::<ErrorMarker>()
        .is_some_and(|shown| shown.0)
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
