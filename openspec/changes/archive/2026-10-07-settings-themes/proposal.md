## Why

gband ships one colorscheme, `default`, which colors only its own bars and leaves every program in a window in the host terminal's colors. A user who wants a known theme, such as gruvbox or catppuccin, has to write a colorscheme file by hand, and still cannot change the colors programs draw with. Choosing a theme, turning the sidebar off, and picking modal or direct keys are all first-day preferences, and each should be one window away rather than a Lua edit. key-style's chooser covers only the key style; this change replaces it with one settings window that covers all three.

The client also draws window borders with fixed styles, bold for the focused window and dim for the others, so neither a theme nor a user can color them. Every border uses the same characters, so the focused window cannot be drawn with heavier ones either.

## What Changes

- **Bundled themes**: gband bundles these colorschemes, each a theme: `default`, `terminal`, `catppuccin-latte`, `catppuccin-frappe`, `catppuccin-macchiato`, `catppuccin-mocha`, `tokyo-night`, `dracula`, `nord`, `gruvbox`, `one-dark`, `solarized`, `kanagawa`, `rose-pine` and `vesper`. `catppuccin` loads `catppuccin-mocha`.
  - `default` sets no group and no palette. Loading it removes every explicit setting, as any colorscheme switch does, so gband draws with the groups' built-in defaults and the host terminal's palette.
  - Every other theme sets every built-in group: the bar, sidebar, key list, plugin window, prompt, settings, window border and error banner groups.
  - Every theme except `default` and `terminal` also sets a terminal palette from the theme's published terminal colors.
  - `terminal` sets no palette and uses only the 16 ANSI colors, so gband and its programs follow the host terminal's own palette.
- **Terminal palette**: `gband.palette.set(spec)` and `gband.palette.get()` set and read a palette of the default foreground, the default background and the 16 ANSI colors, each a hex color. When the client draws its terminal, a cell in the default color, or in ANSI color 0 to 15, is drawn in the palette's color for it. This covers program output in windows, borders, bars, plugin windows and the banner. Other colors are drawn unchanged. Switching colorschemes clears the palette before the new file runs, and restores it when the switch fails.
- **New groups**:
  - `WindowBorder` and `WindowBorderFocused` draw the borders of tiled and floating windows. `WindowBorder`'s default is today's dim. `WindowBorderFocused`'s default is `{ fg = "#b1b9f9", bold = true }`, a light lavender blue. Themes set their own.
  - `ErrorBanner` draws the error banner. Its default is today's red and reverse.
  - `SettingsLabel` draws the labels of the settings window.
- **Focused border characters**: two new client options, `focused_tile_border_chars` and `focused_floating_border_chars`, give the character set of the focused tiled window's and the focused floating window's border. They take the same values as `tile_border_chars`. The lifted tile's drop outline uses `focused_tile_border_chars`. The sides stay shared with the window's other border options.
- **BREAKING (borders)**: the declared defaults of `tile_border_chars`, `floating_border_chars`, `focused_tile_border_chars` and `focused_floating_border_chars` become `"rounded"`, and the default configuration sets all four to `"rounded"`. The focused border therefore uses the same characters as the others by default. A floating plugin window's border table still defaults to `plain`.
- **Default theme**: when loading starts, gband loads the saved theme, or `default` when none is saved or the saved one fails. This applies before `user/init.lua` runs, so a user's own configuration still gets the saved theme unless it calls `gband.colorscheme` itself.
- **BREAKING**: the bundled `default` colorscheme sets nothing. The hex colors it gave the sidebar and key list groups are dropped, so those groups draw with their plugins' defaults.
- **Settings window**: `gband.settings.open()` opens a focused floating plugin window titled `settings` with three rows:
  - `theme` shows the active colorscheme.
  - `sidebar` shows `on` or `off`.
  - `keys` shows `modal` or `direct`.
  - j, k and the arrow keys move between rows. h, l, Left and Right change the value of the row. Enter on `sidebar` or `keys` toggles the value. Enter on `theme` opens the theme list. Escape and `q` close the window.
  - Every change is saved at once. The configuration then reloads, and the settings window opens again on the same row.
- **Theme list**: a floating plugin window titled `theme`, with one line per theme that `gband.settings.themes()` returns. The bundled themes come first, in the order above, then every other colorscheme on the runtimepath. Moving the cursor line previews the theme under it. Enter saves it. Escape and `q` restore the theme that was active when the list opened.
- **Saved settings**: each setting is saved in its own file.
  - The theme is saved in `user/theme.lua`, as `return "<name>"`, and read by `gband.settings.theme()`.
  - The sidebar is saved in `user/sidebar.lua`, as `return true` or `return false`, and read by `gband.settings.sidebar()`.
  - The key style is saved in key-style's `user/keystyle.lua`.
  - None of these files is ever a configuration file, and nothing writes `user/init.lua`.
- **Default configuration**: sets up `gband.sidebar` unless the saved sidebar setting is `false`. On `Attached`, it opens the settings window when a configuration directory exists and no theme, sidebar or key style is saved.
- **Key binding**: both key style presets bind `prefix s` to `gband.settings.open`, described `settings`, right after `:`. With the modal style, this is `s` in navigation mode. Opening the window returns to interactive mode.
- **Key list**: the key list shows the new binding. When a line's binding opens a focused floating plugin window, from Enter or from the line's key, the key list closes. So Enter on `s` closes the key list and opens the settings window.
- **BREAKING (key-style)**: `gband.keystyle.choose()`, the key style chooser and its first-start offer are removed. The settings window's `keys` row and its own first-start offer replace them.
- **Tests**: every `gband test` case saves the `terminal` theme unless `g.start` says otherwise, so screens do not depend on the default theme. `g.start` takes `theme`, which is a colorscheme name or `false`. End-to-end tests save it too.

Out of scope:
- Answering programs' OSC 4, 10 and 11 color queries with the palette. The server still does not answer them, as terminal-query-replies left it.
- Remapping 256-color indexes 16 to 255 and 24-bit colors, and the cursor color.
- Light and dark variants of themes other than catppuccin.
- Settings other than the theme, the sidebar and the key style, and a settings window on the server.
- Applying a setting without a reload.

## Capabilities

### New Capabilities
- `settings`: `gband.settings.open`, `theme`, `sidebar` and `themes`; the settings window, its rows and keys; the theme list and its preview; the saved theme and sidebar files; reopening after a save; and the offer on the first start.

### Modified Capabilities
- `colorschemes`: the bundled themes are added, and `default` becomes the theme that sets nothing. The saved theme, or `default`, is loaded when loading starts. The terminal palette is added, and switching colorschemes clears and restores it.
- `highlights`: the client groups `WindowBorder`, `WindowBorderFocused` and `ErrorBanner` style what the client draws itself.
- `borders`: the focused tiled and floating window take their own character sets, and the lifted tile's drop outline takes the focused tile's.
- `key-style`: the chooser and its first-start offer are removed. Saving the key style is described through the settings window.
- `key-list`: a binding run from the list that opens a focused floating plugin window closes the list.
- `client-attach`: the default table gains `s`, which opens the settings window.
- `configuration`: the saved theme or `default` is loaded before the init file. The options table gains the two focused border options, and the four border character options default to `"rounded"`. The default configuration's sidebar follows the saved setting, and the offer opens the settings window.
- `plugins`: `settings` and `palette` join the client-only fields of the side guard.
- `plugin-testing`: a case saves the `terminal` theme by default, and `g.start` takes `theme`.

## Impact

- Lua runtime: new `gband/settings.lua` and `gband/palette.lua`; `gband/colors/` gains the themes, and `default.lua` becomes empty; `colorscheme.lua` loads the start theme and clears and restores the palette; `keystyle.lua` drops `choose`; both presets bind `s`; `keylist.lua` closes on a focused new window; `defaults.lua` reads the saved sidebar, sets the four border character options to `"rounded"`, and opens the settings window on first start.
- Lua host (Rust): the palette in the client state, the theme list from runtimepath `colors/` directories, a value kept across one reload so the settings window reopens, `bundled.rs` entries, `sides.rs`, and the two focused border options with the rounded defaults in `options.rs`.
- Client: a palette pass over the drawn frame, the border and banner groups in place of fixed styles, the focused border characters for the focused window and the drop outline, and `color.rs` for mapped colors.
- Harness: the `theme` option of `g.start`, and tile detection that knows the corners of every named character set.
- Tests and screenshots: new settings and palette tests. Screen references that recorded the `default` colorscheme's colors are regenerated under the `terminal` theme, with rounded tile corners. Tests that assert plain tile corners change to rounded ones.
- `README.md`, `docs/plugins.md` and `docs/testing.md`.
- No protocol change and no new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- band-sidebar
- key-style

### Expected Files
- openspec/changes/settings-themes/
- README.md
- crates/client/src/bindings.rs
- crates/client/src/lib.rs
- crates/client/src/render.rs
- crates/client/tests/bars.rs
- crates/client/tests/look.rs
- crates/client/tests/render.rs
- crates/client/tests/sidebar.rs
- crates/client/tests/snapshots/
- crates/harness/src/case.rs
- crates/harness/src/runner.rs
- crates/harness/src/terminal.rs
- crates/lua/src/bundled.rs
- crates/lua/src/defaults.lua
- crates/lua/src/lib.rs
- crates/lua/src/options.rs
- crates/lua/src/runtime.rs
- crates/lua/src/runtime/gband/colors/
- crates/lua/src/runtime/gband/colorscheme.lua
- crates/lua/src/runtime/gband/hl.lua
- crates/lua/src/runtime/gband/keylist.lua
- crates/lua/src/runtime/gband/keystyle.lua
- crates/lua/src/runtime/gband/keystyle/
- crates/lua/src/runtime/gband/palette.lua
- crates/lua/src/runtime/gband/settings.lua
- crates/lua/src/runtime/gband/theme.lua
- crates/lua/src/runtime/gband/theme/
- crates/lua/src/sides.rs
- crates/lua/src/ui.rs
- crates/lua/tests/colorschemes.rs
- crates/lua/tests/config.rs
- crates/lua/tests/keystyle.rs
- crates/lua/tests/options.rs
- crates/lua/tests/palette.rs
- crates/lua/tests/plugins.rs
- crates/lua/tests/settings.rs
- crates/lua/tests/themes.rs
- docs/plugins.md
- docs/testing.md
- examples/plugins/agent-status/tests/screenshots/
- examples/plugins/hello/tests/screenshots/
- examples/plugins/window/tests/screenshots/
- examples/plugins/window/tests/window_spec.lua
- tests/common/mod.rs
- tests/config.rs
- tests/keystyle.rs
- tests/lua/keylist_spec.lua
- tests/lua/keystyle_spec.lua
- tests/lua/prompt_spec.lua
- tests/lua/screenshots/
- tests/lua/settings_spec.lua
- tests/lua/sidebar_spec.lua
- tests/lua_specs.rs
- tests/navigation.rs
- tests/plugin_testing.rs
- tests/plugin_windows.rs
- tests/settings.rs
