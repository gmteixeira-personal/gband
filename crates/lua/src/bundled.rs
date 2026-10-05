use mlua::{Function, IntoLuaMulti, Lua, MultiValue, Table};

pub(crate) const API: [(&str, &str); 3] = [
    ("hl.lua", include_str!("runtime/gband/hl.lua")),
    (
        "colorscheme.lua",
        include_str!("runtime/gband/colorscheme.lua"),
    ),
    (
        "statusline.lua",
        include_str!("runtime/gband/statusline.lua"),
    ),
];

const MODULES: [(&str, &str); 4] = [
    (
        "statusline/band.lua",
        include_str!("runtime/gband/statusline/band.lua"),
    ),
    (
        "statusline/mode.lua",
        include_str!("runtime/gband/statusline/mode.lua"),
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

fn search(lua: &Lua, name: String) -> mlua::Result<MultiValue> {
    let Some(relative) = name
        .strip_prefix(ROOT)
        .and_then(|rest| rest.strip_prefix('.'))
    else {
        return format!("\n\tno bundled module '{name}'").into_lua_multi(lua);
    };
    let relative = relative.replace('.', "/");
    for candidate in [format!("{relative}.lua"), format!("{relative}/init.lua")] {
        if let Some((file, source)) = MODULES.iter().find(|(file, _)| *file == candidate) {
            let loader = chunk(lua, file, source)?;
            return (loader, format!("{ROOT}/{file}")).into_lua_multi(lua);
        }
    }
    format!("\n\tno bundled module '{name}'").into_lua_multi(lua)
}

pub(crate) fn install_searcher(lua: &Lua, position: i64) -> mlua::Result<()> {
    let searchers: Table = lua.globals().get::<Table>("package")?.get("searchers")?;
    let insert: Function = lua.globals().get::<Table>("table")?.get("insert")?;
    insert.call::<()>((searchers, position, lua.create_function(search)?))
}
