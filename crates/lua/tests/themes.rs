mod common;

use common::*;
use gband_lua::Config;

const THEME_GROUPS: [&str; 16] = [
    "Bar",
    "SidebarMode",
    "SidebarBand",
    "SidebarBandActive",
    "SidebarError",
    "KeyListKey",
    "KeyListMuted",
    "PluginWindow",
    "PluginWindowBorder",
    "PluginWindowTitle",
    "PluginWindowCursorLine",
    "PromptCursor",
    "SettingsLabel",
    "WindowBorder",
    "WindowBorderFocused",
    "ErrorBanner",
];

const THEMES: [&str; 14] = [
    "terminal",
    "catppuccin-latte",
    "catppuccin-frappe",
    "catppuccin-macchiato",
    "catppuccin-mocha",
    "tokyo-night",
    "dracula",
    "nord",
    "gruvbox",
    "one-dark",
    "solarized",
    "kanagawa",
    "rose-pine",
    "vesper",
];

const BASE: [&str; 8] = [
    "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
];

const BASE_COLORS: [(&str, [&str; 8]); 13] = [
    (
        "gruvbox",
        [
            "#282828", "#cc241d", "#98971a", "#d79921", "#458588", "#b16286", "#689d6a", "#a89984",
        ],
    ),
    (
        "catppuccin-latte",
        [
            "#5c5f77", "#d20f39", "#40a02b", "#df8e1d", "#1e66f5", "#ea76cb", "#179299", "#acb0be",
        ],
    ),
    (
        "catppuccin-frappe",
        [
            "#51576d", "#e78284", "#a6d189", "#e5c890", "#8caaee", "#f4b8e4", "#81c8be", "#a5adce",
        ],
    ),
    (
        "catppuccin-macchiato",
        [
            "#494d64", "#ed8796", "#a6da95", "#eed49f", "#8aadf4", "#f5bde6", "#8bd5ca", "#a5adcb",
        ],
    ),
    (
        "catppuccin-mocha",
        [
            "#45475a", "#f38ba8", "#a6e3a1", "#f9e2af", "#89b4fa", "#f5c2e7", "#94e2d5", "#a6adc8",
        ],
    ),
    (
        "tokyo-night",
        [
            "#15161e", "#f7768e", "#9ece6a", "#e0af68", "#7aa2f7", "#bb9af7", "#7dcfff", "#a9b1d6",
        ],
    ),
    (
        "dracula",
        [
            "#21222c", "#ff5555", "#50fa7b", "#f1fa8c", "#bd93f9", "#ff79c6", "#8be9fd", "#f8f8f2",
        ],
    ),
    (
        "nord",
        [
            "#3b4252", "#bf616a", "#a3be8c", "#ebcb8b", "#81a1c1", "#b48ead", "#88c0d0", "#e5e9f0",
        ],
    ),
    (
        "one-dark",
        [
            "#282c34", "#e06c75", "#98c379", "#e5c07b", "#61afef", "#c678dd", "#56b6c2", "#abb2bf",
        ],
    ),
    (
        "solarized",
        [
            "#073642", "#dc322f", "#859900", "#b58900", "#268bd2", "#d33682", "#2aa198", "#eee8d5",
        ],
    ),
    (
        "kanagawa",
        [
            "#16161d", "#c34043", "#76946a", "#c0a36e", "#7e9cd8", "#957fb8", "#6a9589", "#c8c093",
        ],
    ),
    (
        "rose-pine",
        [
            "#26233a", "#eb6f92", "#31748f", "#f6c177", "#9ccfd8", "#c4a7e7", "#ebbcba", "#e0def4",
        ],
    ),
    (
        "vesper",
        [
            "#101010", "#f5a191", "#90b99f", "#e6b99d", "#aca1cf", "#e29eca", "#ea83a5", "#a0a0a0",
        ],
    ),
];

const DESCRIBE: &str = r#"
local function encode(value)
  if type(value) ~= "table" then
    return tostring(value)
  end
  local keys = {}
  for key in pairs(value) do
    keys[#keys + 1] = tostring(key)
  end
  table.sort(keys)
  local parts = {}
  for _, key in ipairs(keys) do
    parts[#parts + 1] = key .. "=" .. encode(value[key])
  end
  return "{" .. table.concat(parts, ",") .. "}"
end
describe = function(groups)
  local out = { palette = encode(gband.palette.get()) }
  for _, group in ipairs(groups) do
    out[group] = encode(gband.hl.get(group))
  end
  return encode(out)
end
"#;

fn loaded_with(name: &str) -> Config {
    let scratch = Scratch::new(name);
    scratch.write(&format!("{DESCRIBE}\n{JOB}"));
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    config
}

fn switch(config: &Config, theme: &str) {
    clean(&run_job(
        config,
        &format!("assert(gband.colorscheme('{theme}'))"),
    ));
}

fn groups_table() -> String {
    let names: Vec<String> = THEME_GROUPS
        .iter()
        .map(|name| format!("'{name}'"))
        .collect();
    format!("{{ {} }}", names.join(", "))
}

fn describe(config: &Config) -> String {
    eval(config, &format!("return describe({})", groups_table()))
}

fn palette_field(config: &Config, field: &str) -> Option<String> {
    eval(config, &format!("return gband.palette.get().{field}"))
}

#[test]
fn helper_sets_every_theme_group_and_the_palette() {
    let config = loaded_with("helper");
    clean(&run_job(
        &config,
        "require('gband.theme').apply({
          palette = { fg = '#eeeeee', bg = '#111111', red = '#aa0000' },
          ui = { surface = '#222222', selection = '#333333', muted = '#777777', accent = '#0000ff', error = '#ff0000' },
        })",
    ));
    for group in THEME_GROUPS {
        let set: bool = eval(&config, &format!("return gband.hl.get('{group}') ~= nil"));
        assert!(set, "{group}");
    }
    assert_eq!(palette_field(&config, "bg").as_deref(), Some("#111111"));
    assert_eq!(palette_field(&config, "red").as_deref(), Some("#aa0000"));
}

#[test]
fn every_theme_sets_every_theme_group() {
    let config = loaded_with("every-group");
    for theme in THEMES {
        switch(&config, theme);
        for group in THEME_GROUPS {
            let set: bool = eval(&config, &format!("return gband.hl.get('{group}') ~= nil"));
            assert!(set, "{theme} {group}");
        }
    }
}

#[test]
fn every_palette_theme_sets_every_palette_field() {
    let config = loaded_with("every-field");
    for (theme, _) in BASE_COLORS {
        switch(&config, theme);
        let fields: i64 = eval(
            &config,
            "local n = 0 for _ in pairs(gband.palette.get()) do n = n + 1 end return n",
        );
        assert_eq!(fields, 18, "{theme}");
    }
}

#[test]
fn base_colors_follow_the_sources() {
    let config = loaded_with("base-colors");
    for (theme, colors) in BASE_COLORS {
        switch(&config, theme);
        for (field, expected) in BASE.into_iter().zip(colors) {
            assert_eq!(
                palette_field(&config, field).as_deref(),
                Some(expected),
                "{theme} {field}"
            );
        }
    }
}

#[test]
fn gruvbox_palette() {
    let config = loaded_with("gruvbox");
    switch(&config, "gruvbox");
    for (field, expected) in [
        ("bg", "#282828"),
        ("fg", "#ebdbb2"),
        ("red", "#cc241d"),
        ("bright_red", "#fb4934"),
    ] {
        assert_eq!(
            palette_field(&config, field).as_deref(),
            Some(expected),
            "{field}"
        );
    }
}

#[test]
fn focused_and_selected_parts_stand_out() {
    let config = loaded_with("stand-out");
    for theme in THEMES {
        switch(&config, theme);
        for (focused, other) in [
            ("WindowBorderFocused", "WindowBorder"),
            ("SidebarBandActive", "SidebarBand"),
            ("PluginWindowCursorLine", "PluginWindow"),
        ] {
            let differ: bool = eval(
                &config,
                &format!(
                    "return describe({{ '{focused}' }}):gsub('{focused}', 'X') ~= describe({{ '{other}' }}):gsub('{other}', 'X')"
                ),
            );
            assert!(differ, "{theme}: {focused} looks like {other}");
        }
    }
}

#[test]
fn terminal_theme_uses_the_host_palette() {
    let config = loaded_with("terminal");
    switch(&config, "terminal");
    let empty: bool = eval(&config, "return next(gband.palette.get()) == nil");
    assert!(empty);
    for group in THEME_GROUPS {
        let small: bool = eval(
            &config,
            &format!(
                "local style = gband.hl.get('{group}', {{ resolve = true }})
                for _, field in ipairs({{ 'fg', 'bg' }}) do
                  local color = style[field]
                  if type(color) == 'number' and color > 15 then return false end
                  if type(color) == 'string' and color:sub(1, 1) == '#' then return false end
                end
                return true"
            ),
        );
        assert!(small, "{group}");
    }
}

#[test]
fn catppuccin_alias() {
    let config = loaded_with("catppuccin");
    switch(&config, "catppuccin-mocha");
    let mocha = describe(&config);
    switch(&config, "catppuccin");
    assert_eq!(
        eval::<String>(&config, "return gband.colorscheme()"),
        "catppuccin"
    );
    assert_eq!(describe(&config), mocha);
    switch(&config, "catppuccin-latte");
    assert_ne!(describe(&config), mocha);
}
