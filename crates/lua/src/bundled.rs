use mlua::{Function, IntoLuaMulti, Lua, MultiValue, Table};

pub(crate) const API: [(&str, &str); 5] = [
    ("hl.lua", include_str!("runtime/gband/hl.lua")),
    (
        "colorscheme.lua",
        include_str!("runtime/gband/colorscheme.lua"),
    ),
    ("bar.lua", include_str!("runtime/gband/bar.lua")),
    (
        "statusline.lua",
        include_str!("runtime/gband/statusline.lua"),
    ),
    ("win.lua", include_str!("runtime/gband/win.lua")),
];

const MODULES: [(&str, &str); 9] = [
    ("errors.lua", include_str!("runtime/gband/errors.lua")),
    ("prompt.lua", include_str!("runtime/gband/prompt.lua")),
    ("keyform.lua", include_str!("runtime/gband/keyform.lua")),
    ("keylist.lua", include_str!("runtime/gband/keylist.lua")),
    (
        "statusline/band.lua",
        include_str!("runtime/gband/statusline/band.lua"),
    ),
    (
        "statusline/mode.lua",
        include_str!("runtime/gband/statusline/mode.lua"),
    ),
    (
        "statusline/hints.lua",
        include_str!("runtime/gband/statusline/hints.lua"),
    ),
    (
        "statusline/position.lua",
        include_str!("runtime/gband/statusline/position.lua"),
    ),
    (
        "statusline/clock.lua",
        include_str!("runtime/gband/statusline/clock.lua"),
    ),
];

const COLORS: [(&str, &str); 1] = [(
    "colors/default.lua",
    include_str!("runtime/gband/colors/default.lua"),
)];

const ROOT: &str = "gband";

pub(crate) fn chunk(lua: &Lua, path: &str, source: &str) -> mlua::Result<Function> {
    lua.load(source)
        .set_name(format!("@{ROOT}/{path}"))
        .into_function()
}

pub(crate) fn colorscheme(lua: &Lua, name: &str) -> mlua::Result<Option<Function>> {
    let path = format!("colors/{name}.lua");
    COLORS
        .iter()
        .find(|(file, _)| *file == path)
        .map(|(file, source)| chunk(lua, file, source))
        .transpose()
}

const BIND: &str = "local chunk, host = ...\nreturn function() return chunk(host) end";

fn search(lua: &Lua, name: &str, host: Option<&Table>) -> mlua::Result<MultiValue> {
    let Some(relative) = name
        .strip_prefix(ROOT)
        .and_then(|rest| rest.strip_prefix('.'))
    else {
        return format!("\n\tno bundled module '{name}'").into_lua_multi(lua);
    };
    let relative = relative.replace('.', "/");
    for candidate in [format!("{relative}.lua"), format!("{relative}/init.lua")] {
        if let Some((file, source)) = MODULES.iter().find(|(file, _)| *file == candidate) {
            let loader = lua
                .load(BIND)
                .set_name("=bundled")
                .call::<Function>((chunk(lua, file, source)?, host))?;
            return (loader, format!("{ROOT}/{file}")).into_lua_multi(lua);
        }
    }
    format!("\n\tno bundled module '{name}'").into_lua_multi(lua)
}

pub(crate) fn install_searcher(lua: &Lua, position: i64, host: Option<Table>) -> mlua::Result<()> {
    let searchers: Table = lua.globals().get::<Table>("package")?.get("searchers")?;
    let insert: Function = lua.globals().get::<Table>("table")?.get("insert")?;
    let searcher =
        lua.create_function(move |lua, name: String| search(lua, &name, host.as_ref()))?;
    insert.call::<()>((searchers, position, searcher))
}
