use std::path::PathBuf;

use mlua::{Lua, Table, Value};

use crate::error::ConfigError;

pub const VARIABLE: &str = "GBAND_PANE";

const NAMES: [(&str, &str); 21] = [
    ("gband.pane", "gband.window"),
    ("gband.pane_state", "gband.window_state"),
    ("open_pane", "open_window"),
    ("close_pane", "close_window"),
    ("focus_pane_down", "focus_window_down"),
    ("focus_pane_up", "focus_window_up"),
    ("move_pane_down", "move_window_down"),
    ("move_pane_up", "move_window_up"),
    ("toggle_pane_floating", "toggle_window_floating"),
    ("grow_pane_height", "grow_window_height"),
    ("shrink_pane_height", "shrink_window_height"),
    ("reset_pane_height", "reset_window_height"),
    ("PaneOpened", "WindowOpened"),
    ("PaneClosed", "WindowClosed"),
    ("PaneExited", "WindowExited"),
    ("PaneOutput", "WindowOutput"),
    ("PaneInput", "WindowInput"),
    ("PaneStateChanged", "WindowStateChanged"),
    ("pane", "window"),
    ("kind = \"pane\"", "kind = \"tiled\""),
    ("kind = \"float\"", "kind = \"floating\""),
];

pub(crate) fn message(name: &str) -> Option<String> {
    NAMES
        .iter()
        .find(|(old, _)| *old == name)
        .map(|(old, new)| format!("`{old}` is now `{new}`"))
}

pub(crate) fn raise(lua: &Lua, name: &str) -> Option<mlua::Error> {
    let message = message(name)?;
    Some(mlua::Error::external(ConfigError {
        plugin: None,
        location: user_caller(lua),
        message,
    }))
}

fn user_caller(lua: &Lua) -> Option<(PathBuf, u32)> {
    (1..)
        .map_while(|level| {
            lua.inspect_stack(level, |debug| {
                let source = debug.source().source?.to_string();
                let line = u32::try_from(debug.current_line()?).ok()?;
                Some((source, line))
            })
        })
        .flatten()
        .find_map(|(source, line)| {
            let path = source.strip_prefix('@')?;
            (!path.starts_with("gband/")).then(|| (PathBuf::from(path), line))
        })
}

pub(crate) fn install(lua: &Lua, core: &Table) -> mlua::Result<()> {
    core.set(
        "removed",
        lua.create_function(|lua, name: Value| {
            let name = crate::check::text(
                lua,
                &crate::ui::named("removed"),
                &name,
                "an API name as a string",
            )?;
            Ok(message(&name))
        })?,
    )
}
