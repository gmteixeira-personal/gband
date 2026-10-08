use mlua::{Function, IntoLuaMulti, Lua, MultiValue, Table};

pub(crate) const KEY_STYLES: [(&str, &str); 3] = [
    ("modal", include_str!("runtime/gband/keystyle/modal.lua")),
    ("direct", include_str!("runtime/gband/keystyle/direct.lua")),
    (
        "floating",
        include_str!("runtime/gband/keystyle/floating.lua"),
    ),
];

pub(crate) const MODULES: [(&str, &str); 19] = [
    ("prelude.lua", include_str!("runtime/gband/prelude.lua")),
    ("hl.lua", include_str!("runtime/gband/hl.lua")),
    ("palette.lua", include_str!("runtime/gband/palette.lua")),
    (
        "colorscheme.lua",
        include_str!("runtime/gband/colorscheme.lua"),
    ),
    ("bar.lua", include_str!("runtime/gband/bar.lua")),
    ("win.lua", include_str!("runtime/gband/win.lua")),
    ("settings.lua", include_str!("runtime/gband/settings.lua")),
    ("keystyle.lua", include_str!("runtime/gband/keystyle.lua")),
    ("theme.lua", include_str!("runtime/gband/theme.lua")),
    (
        "theme/catppuccin.lua",
        include_str!("runtime/gband/theme/catppuccin.lua"),
    ),
    ("errors.lua", include_str!("runtime/gband/errors.lua")),
    ("prompt.lua", include_str!("runtime/gband/prompt.lua")),
    ("keyform.lua", include_str!("runtime/gband/keyform.lua")),
    ("keylist.lua", include_str!("runtime/gband/keylist.lua")),
    ("keystyle/modal.lua", KEY_STYLES[0].1),
    ("keystyle/direct.lua", KEY_STYLES[1].1),
    ("keystyle/floating.lua", KEY_STYLES[2].1),
    ("desktop.lua", include_str!("runtime/gband/desktop.lua")),
    ("sidebar.lua", include_str!("runtime/gband/sidebar.lua")),
];

const COLORS: [(&str, &str); 16] = [
    ("default", include_str!("runtime/gband/colors/default.lua")),
    (
        "terminal",
        include_str!("runtime/gband/colors/terminal.lua"),
    ),
    (
        "catppuccin-latte",
        include_str!("runtime/gband/colors/catppuccin-latte.lua"),
    ),
    (
        "catppuccin-frappe",
        include_str!("runtime/gband/colors/catppuccin-frappe.lua"),
    ),
    (
        "catppuccin-macchiato",
        include_str!("runtime/gband/colors/catppuccin-macchiato.lua"),
    ),
    (
        "catppuccin-mocha",
        include_str!("runtime/gband/colors/catppuccin-mocha.lua"),
    ),
    (
        "tokyo-night",
        include_str!("runtime/gband/colors/tokyo-night.lua"),
    ),
    ("dracula", include_str!("runtime/gband/colors/dracula.lua")),
    ("nord", include_str!("runtime/gband/colors/nord.lua")),
    ("gruvbox", include_str!("runtime/gband/colors/gruvbox.lua")),
    (
        "one-dark",
        include_str!("runtime/gband/colors/one-dark.lua"),
    ),
    (
        "solarized",
        include_str!("runtime/gband/colors/solarized.lua"),
    ),
    (
        "kanagawa",
        include_str!("runtime/gband/colors/kanagawa.lua"),
    ),
    (
        "rose-pine",
        include_str!("runtime/gband/colors/rose-pine.lua"),
    ),
    ("vesper", include_str!("runtime/gband/colors/vesper.lua")),
    (
        "catppuccin",
        include_str!("runtime/gband/colors/catppuccin.lua"),
    ),
];

const ALIASES: [&str; 1] = ["catppuccin"];

pub(crate) fn themes() -> impl Iterator<Item = &'static str> {
    COLORS
        .iter()
        .map(|&(name, _)| name)
        .filter(|name| !ALIASES.contains(name))
}

const ROOT: &str = "gband";

pub fn files() -> impl Iterator<Item = (String, &'static str)> {
    let modules = MODULES
        .iter()
        .map(|(file, source)| (format!("lua/{ROOT}/{file}"), *source));
    let colors = COLORS
        .iter()
        .map(|(name, source)| (format!("colors/{name}.lua"), *source));
    modules.chain(colors)
}

pub fn is_alias(file: &str) -> bool {
    ALIASES
        .iter()
        .any(|alias| file == format!("colors/{alias}.lua"))
}

pub(crate) fn is_key_style(file: &str) -> bool {
    KEY_STYLES
        .iter()
        .any(|(style, _)| file == format!("lua/{ROOT}/keystyle/{style}.lua"))
}

pub(crate) fn chunk(lua: &Lua, path: &str, source: &str) -> mlua::Result<Function> {
    lua.load(source)
        .set_name(format!("@{ROOT}/{path}"))
        .into_function()
}

pub(crate) fn colorscheme(lua: &Lua, name: &str) -> mlua::Result<Option<Function>> {
    COLORS
        .iter()
        .find(|(bundled, _)| *bundled == name)
        .map(|(bundled, source)| chunk(lua, &format!("colors/{bundled}.lua"), source))
        .transpose()
}

fn search(lua: &Lua, name: &str) -> mlua::Result<MultiValue> {
    let Some(relative) = name
        .strip_prefix(ROOT)
        .and_then(|rest| rest.strip_prefix('.'))
    else {
        return format!("\n\tno bundled module '{name}'").into_lua_multi(lua);
    };
    let relative = relative.replace('.', "/");
    for candidate in [format!("{relative}.lua"), format!("{relative}/init.lua")] {
        if let Some((file, source)) = MODULES.iter().find(|(file, _)| *file == candidate) {
            return (chunk(lua, file, source)?, format!("{ROOT}/{file}")).into_lua_multi(lua);
        }
    }
    format!("\n\tno bundled module '{name}'").into_lua_multi(lua)
}

pub(crate) fn install_searcher(lua: &Lua, position: i64) -> mlua::Result<()> {
    let searchers: Table = lua.globals().get::<Table>("package")?.get("searchers")?;
    let insert: Function = lua.globals().get::<Table>("table")?.get("insert")?;
    let searcher = lua.create_function(|lua, name: String| search(lua, &name))?;
    insert.call::<()>((searchers, position, searcher))
}
