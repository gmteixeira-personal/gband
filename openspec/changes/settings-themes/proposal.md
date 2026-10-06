## Why

gband ships one colorscheme, `default`, which colors only its own bars and leaves every program in a window in the host terminal's colors. A user who wants a known theme, such as gruvbox or catppuccin, has to write a colorscheme file by hand, and still cannot change the colors programs draw with. Choosing a theme, turning the sidebar off, and picking modal or direct keys are all first-day preferences, and each should be one window away rather than a Lua edit. key-style's chooser covers only the key style; this change replaces it with one settings window that covers all three.

## What Changes

- **Bundled themes**: gband bundles these colorschemes, each a theme: `terminal`, `catppuccin-latte`, `catppuccin-frappe`, `catppuccin-macchiato`, `catppuccin-mocha`, `tokyo-night`, `dracula`, `nord`, `gruvbox`, `one-dark`, `solarized`, `kanagawa`, `rose-pine` and `vesper`. `catppuccin` loads `catppuccin-mocha`.
  - Every theme sets every built-in group: the bar, sidebar, key list, plugin window, prompt, settings, window border and error banner groups.
  - Every theme except `terminal` also sets a terminal palette from the theme's published terminal colors.
  - `terminal` sets no palette and uses only the 16 ANSI colors, so gband and its programs follow the host terminal's own palette.
- **Terminal palette**: `gband.palette.set(spec)` and `gband.palette.get()` set and read a palette of the default foreground, the default background and the 16 ANSI colors, each a hex color. When the client draws its terminal, a cell in the default color, or in ANSI color 0 to 15, is drawn in the palette's color for it. This covers program output in windows, borders, bars, plugin windows and the banner. Other colors are drawn unchanged. Switching colorschemes clears the palette before the new file runs, and restores it when the switch fails.
- **New groups**:
  - `WindowBorder` and `WindowBorderFocused` draw the borders of tiled and floating windows. Their defaults are today's dim and bold.
  - `ErrorBanner` draws the error banner. Its default is today's red and reverse.
  - `SettingsLabel` draws the labels of the settings window.
- **Default theme**: when loading starts, gband loads the saved theme, or `gruvbox` when none is saved or the saved one fails. This applies before `user/init.lua` runs, so a user's own configuration still gets the saved theme unless it calls `gband.colorscheme` itself.
- **BREAKING**: the bundled `default` colorscheme is removed. `gband.colorscheme("default")` fails as a missing colorscheme unless a runtimepath entry holds `colors/default.lua`.
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
- `colorschemes`: the bundled themes replace `default`. The saved theme, or `gruvbox`, is loaded when loading starts. The terminal palette is added, and switching colorschemes clears and restores it.
- `highlights`: the client groups `WindowBorder`, `WindowBorderFocused` and `ErrorBanner` style what the client draws itself.
- `key-style`: the chooser and its first-start offer are removed. Saving the key style is described through the settings window.
- `key-list`: a binding run from the list that opens a focused floating plugin window closes the list.
- `client-attach`: the default table gains `s`, which opens the settings window.
- `configuration`: the saved theme or `gruvbox` replaces `default` before the init file. The default configuration's sidebar follows the saved setting, and the offer opens the settings window.
- `plugins`: `settings` and `palette` join the client-only fields of the side guard.
- `plugin-testing`: a case saves the `terminal` theme by default, and `g.start` takes `theme`.

## Impact

- Lua runtime: new `gband/settings.lua` and `gband/palette.lua`; `gband/colors/` gains the themes, and `default.lua` is removed; `colorscheme.lua` loads the start theme and clears and restores the palette; `keystyle.lua` drops `choose`; both presets bind `s`; `keylist.lua` closes on a focused new window; `defaults.lua` reads the saved sidebar and opens the settings window on first start.
- Lua host (Rust): the palette in the client state, the theme list from runtimepath `colors/` directories, a value kept across one reload so the settings window reopens, `bundled.rs` entries, and `sides.rs`.
- Client: a palette pass over the drawn frame, the border and banner groups in place of fixed styles, and `color.rs` for mapped colors.
- Harness: the `theme` option of `g.start`.
- Tests and screenshots: new settings and palette tests. Screen references that recorded the `default` colorscheme's colors are regenerated under the `terminal` theme.
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
- crates/lua/src/ui.rs
- crates/lua/src/runtime.rs
- crates/lua/src/lib.rs
- crates/lua/src/bundled.rs
- crates/lua/src/sides.rs
- crates/lua/src/defaults.lua
- crates/lua/src/runtime/gband/palette.lua
- crates/lua/src/runtime/gband/settings.lua
- crates/lua/src/runtime/gband/theme.lua
- crates/lua/src/runtime/gband/theme/
- crates/lua/src/runtime/gband/colors/
- crates/lua/src/runtime/gband/colorscheme.lua
- crates/lua/src/runtime/gband/keystyle.lua
- crates/lua/src/runtime/gband/keystyle/
- crates/lua/src/runtime/gband/keylist.lua
- crates/lua/tests/
- crates/client/src/render.rs
- crates/client/src/color.rs
- crates/client/src/lib.rs
- crates/client/src/bindings.rs
- crates/client/tests/
- crates/harness/src/case.rs
- crates/harness/src/runner.rs
- tests/common/mod.rs
- tests/settings.rs
- tests/keystyle.rs
- tests/config.rs
- tests/plugin_testing.rs
- tests/lua_specs.rs
- tests/lua/settings_spec.lua
- tests/lua/keystyle_spec.lua
- tests/lua/screenshots/
- examples/plugins/window/tests/screenshots/
- examples/plugins/hello/tests/screenshots/
- examples/plugins/agent-status/tests/screenshots/
- README.md
- docs/plugins.md
- docs/testing.md
