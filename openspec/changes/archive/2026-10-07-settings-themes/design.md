## Context

See proposal.md for the motivation. This change starts after key-style and band-sidebar are archived. band-sidebar itself waits on sidebars-borders-steps, clear-errors, lua-prompt, key-style and mouse. The state it builds on:

- `crates/lua/src/runtime/gband/colorscheme.lua` defines `gband.colorscheme`. It finds `colors/<name>.lua` on `gband.runtimepath`, then falls back to `bundled::colorscheme`, which serves `colors/default.lua` from `crates/lua/src/bundled.rs`. A switch snapshots the explicit settings, clears them, runs the file under `host.call`, and restores them on failure. The runtime loads `default` before the init file.
- After band-sidebar, `colors/default.lua` sets `Bar`, the four `Sidebar*` groups, `KeyListKey` and `KeyListMuted`. `PluginWindow*` defaults come from `win.lua`, and `PromptCursor` from `prompt.lua`.
- `crates/client/src/render.rs` draws tile and floating window borders with the constants `FOCUSED_BORDER` (bold) and `UNFOCUSED_BORDER` (dim), and the error banner with `BANNER` (red, reversed). `draw_tile` takes the characters from the client's `tile_border` or `floating_border`, whether the window is focused or not, and `draw_outline` draws a lifted tile's drop place with `tile_border`'s characters and `FOCUSED_BORDER`.
- `crates/lua/src/options.rs` holds `tile_border` and `floating_border` as `Border` values whose characters default to `Border::default()`'s `plain`. A floating plugin window's border table also starts from `Border::default()`. `defaults.lua` sets the four border options to their declared defaults.
- `crates/harness/src/terminal.rs`'s `tiles` finds tiles on a screen by their `┌`, `┐` and `└` corners and reads focus from the bold corner. `tests/mouse.rs` uses it. Window content is drawn by `tui_term`'s `PseudoTerminal` from the client's own grid of each window. So every program color the user sees passes through the client's `ratatui` buffer.
- `crates/client/src/color.rs` turns a `gband_lua::Color` into a `ratatui` color, downgrading hex to the nearest index from 16 to 255 when `COLORTERM` lacks truecolor.
- After key-style, `crates/lua/src/runtime/gband/keystyle.lua` holds `use`, `saved` and `choose`. `saved()` reads `user/keystyle.lua` with `loadfile` in text mode and an empty environment. Saving writes a temporary file whose name does not end in `.lua`, then renames it, so `crates/lua/src/watch.rs` sees one change to a `.lua` file and reloads once after its 100 ms debounce. The presets `keystyle/modal.lua` and `keystyle/direct.lua` make every default binding.
- A reload builds a new `Client` from the new configuration (`crates/client/src/lib.rs`), carrying over only `awaiting` and `pending`. `Display`, which owns the plugin windows and the banner, survives, but `close_all` closes every plugin window first. `gband.win.open` is refused while the configuration loads.
- `gband test` cases (`crates/harness/src/case.rs`, `runner.rs`) and end-to-end tests (`tests/common/mod.rs`) save the modal key style before they start. Screen references in `tests/lua/screenshots/` and `examples/plugins/*/tests/screenshots/` record every cell's colors, and today they record `default`'s hex colors.

## Goals / Non-Goals

**Goals:**
- Themes are ordinary colorschemes. A user can shadow one, copy one, or add one under `user/colors/`, and it shows in the theme list with no registration step.
- The look gband has with no theme is itself a choice in the theme list, `default`, and it is what a first start shows.
- The focused window's border can differ from the others in characters as well as style, and both are set from Lua.
- One palette mechanism recolors everything the client draws, so a theme looks the same in gband's own parts and in program output.
- Each saved setting is a small Lua file read with an empty environment, like `user/keystyle.lua`. A broken file degrades to the default, never to a configuration error.
- The settings window survives the reload that its own save causes, so changing several settings in a row needs no reopening.

**Non-Goals:**
- Live sidebar or key style changes without a reload.
- Answering OSC 4, 10 and 11 queries from the palette. The server does not know which client's palette a program would want.
- Mapping 256-color and 24-bit cells.

## Decisions

### Themes are bundled colorschemes built from one helper
Each theme is `crates/lua/src/runtime/gband/colors/<name>.lua`, listed in `bundled.rs` beside the others. Each theme file other than `default.lua` holds only its colors: the 18 palette entries and a few semantic UI colors (surface, overlay, muted, accent, error). It hands them to a bundled helper module, `gband.theme`, in `crates/lua/src/runtime/gband/theme.lua`, which calls `gband.palette.set` and sets every theme group from those slots. So every theme sets the same groups, and adding a group touches one file. The four catppuccin flavors read their colors from one bundled module, `gband.theme.catppuccin`, keyed by flavor. `catppuccin.lua` and `catppuccin-mocha.lua` both pass the mocha entry to the helper, so the alias cannot drift from the flavor.

`terminal.lua` does not use the palette part of the helper. Its groups use names and indexes 0 to 15 only. For example, `Bar` is `{ bg = "black" }`, `SidebarBandActive` is `{ fg = "bright_white", bold = true }`, and `WindowBorderFocused` is `{ fg = "blue", bold = true }`.

Palette values come from each theme's own terminal port: gruvbox's `gruvbox-contrib` dark palette, catppuccin's `catppuccin/palette` ANSI mapping for each flavor, tokyo-night's night terminal colors, Dracula's spec, Nord's `nord0` to `nord15` terminal mapping, one-dark's `onedark.vim` terminal colors, Solarized dark, kanagawa wave, rose-pine main, and vesper's terminal colors. Theme files hold no comments, per CLAUDE.md. The sources are listed here and in the palette test.

Alternatives considered:
- *A theme as a data table that Rust applies.* It would bypass `gband.colorscheme`, its atomicity, its events and its runtimepath lookup, and users could not write themes the same way.

### `default` is an empty colorscheme
`colors/default.lua` stays bundled and becomes an empty file, listed first in `bundled.rs`. `gband.colorscheme` already removes every explicit setting and empties the palette before it runs a file, so loading an empty file leaves exactly the default layer: the client's `WindowBorder`, `WindowBorderFocused` and `ErrorBanner`, and each plugin's own defaults. No code path knows the name, so a user's `colors/default.lua` shadows it as it shadows any theme, and the theme list, the saved theme and the preview treat it as any other entry.

Alternatives considered:
- *A special value that loads no colorscheme.* `gband.colorscheme()` would have no name to return, and the theme list, the saved file and the preview would each need a case for it.
- *Keep `default`'s hex colors for the sidebar and key list.* They would then be a theme like the others, and no choice would show the groups' own defaults.

### The palette lives in the client state and is applied after drawing
`crates/lua/src/runtime/gband/palette.lua` validates the spec and calls `host.palette.set` with the parsed colors. The Lua host keeps a `Palette` (`fg`, `bg`, `[Option<Rgb>; 16]`) next to the highlight groups, and exposes it to the client the way it exposes resolved groups. `colorscheme.lua` snapshots and clears the palette with the explicit settings, and restores it on failure. A palette change marks the frame dirty, as a group change does.

`crates/client/src/render.rs` gets one pass, `apply_palette(buffer, palette, support)`, run over the whole buffer at the end of `render`, after bars, plugin windows and the banner. For each cell it maps `Color::Reset` in `fg` to the palette's `fg` and in `bg` to its `bg`, and `Color::Indexed(0..=15)` and the named `ratatui` colors `Black` to `White` and `DarkGray` to `Gray` to the matching entry. Each mapped color goes through `ColorSupport::color` as a hex color, so a 256-color client gets the nearest index. The pass does nothing when the palette is empty, so `terminal` costs nothing.

One late pass covers every source of color the same way, including `PseudoTerminal` output that gband never sees as a `gband_lua::Style`. Mapping at each draw site instead would miss one sooner or later.

Alternatives considered:
- *Send the palette to the server and remap in the emulator.* Every client of a session would share one palette, and the server would hold UI state it does not need.
- *Remap in `color.rs` only.* Program output does not pass through `color.rs`.

### Borders and the banner use groups
`WindowBorder`, `WindowBorderFocused` and `ErrorBanner` get their defaults in the client's built-in groups, installed before the init file, beside `SettingsLabel`. `render.rs` resolves them through the `Ribbon` it already receives, replacing the three constants. `FOCUSED_BORDER` and friends go away. `WindowBorder` and `ErrorBanner` keep today's dim and red reverse. `WindowBorderFocused` becomes `{ fg = "#b1b9f9", bold = true }`, so the focused border reads as a light lavender blue under `default`, and stays bold so a terminal without color, and the harness's focus check, still tell it apart. `draw_outline` takes `WindowBorderFocused` too.

### Focused border characters are options
`Options` in `crates/lua/src/options.rs` gains `focused_tile_border_chars` and `focused_floating_border_chars`, each a `BorderChars`, beside `tile_border` and `floating_border`, with the same patch, read, reset and validation paths as `tile_border_chars`, and an entry each in the options table that `gband.opt.list()` reads. The client copies them with the other border options in `crates/client/src/lib.rs` and hands them to `render.rs` in the `Ribbon`. `draw_tile` already receives `focused`, so it builds the drawn `Border` from the window's sides and the focused or unfocused characters. `draw_outline` draws with all four sides and the focused tile characters.

The declared defaults of all four character options become `rounded`, set in `Options::default`. `Border::default()` keeps `plain`, because a floating plugin window's border table starts from it, and its default is not changing. `defaults.lua` sets the four options to `"rounded"`, as it sets every option to its declared default, so the default configuration still reproduces the declared defaults.

Alternatives considered:
- *A focused option that follows the unfocused one when unset.* `gband.opt` would have to read back a nil, and the default configuration could not set every option to its declared default.
- *Focused sides as well.* Sides decide where a border is blank, and a border that grows sides when focused would shift the drawn frame on every focus change.

### The harness finds tiles by any named corner set
`tiles` in `crates/harness/src/terminal.rs` recognises the top-left, top-right and bottom-left corners of `plain`, `rounded`, `double` and `thick`, matching the three corners of one set, and keeps reading focus from bold. So `tests/mouse.rs` keeps working with rounded tiles and with a focused tile drawn in another set.

### The start theme loads before the init file
The runtime's step that loaded `default` now calls a Lua entry point in `colorscheme.lua`: load `gband.settings.theme()` when it is not nil, and `default` when it is nil or its load fails. The failure is reported through `host.report`, as any colorscheme failure. `gband.settings` must therefore be installed before that step, so `settings.lua` comes before `colorscheme.lua`'s start call in `bundled::API`.

This is the one setting a user's own `init.lua` gets without calling anything, because the theme is not tied to the default configuration. The sidebar and key style stay with the default configuration, because they decide which plugins and bindings exist, and a user's own file owns those.

### `gband.settings` is a client API file
`crates/lua/src/runtime/gband/settings.lua` joins `bundled::API`, like `keystyle.lua`. It holds `open`, `theme`, `sidebar` and `themes`. `settings` and `palette` join `CLIENT_ONLY` in `crates/lua/src/sides.rs`.

`theme()` and `sidebar()` share one local reader with `keystyle.saved()`'s rules: `loadfile(path, "t", {})` under `pcall`, then a type check. The writer is key-style's save routine, moved into a local shared by the three files: write `<file>.tmp`, rename it over the target, and raise an error naming `user/<file>` on failure.

`themes()` needs a directory listing, which plain Lua cannot do. The host gets `host.colorschemes()`, which lists `*.lua` files under each runtimepath entry's `colors/` with `std::fs::read_dir`, ignoring unreadable directories. Lua then filters valid names, drops the bundled ones, sorts and de-duplicates.

### The settings window and theme list are plain plugin windows
`open()` enters `root` and opens the window with `gband.win.open`, keeping its id in a local so a second call focuses it. The window's `keys` table binds `enter`, `h`, `l`, `left` and `right` and dispatches on the cursor line, read with `gband.win.info`. Every other key falls to the plugin-windows defaults, so j, k, arrows, Escape and `q` need nothing.

The theme list binds `j`, `k`, `up`, `down`, `pageup`, `pagedown`, `home` and `end` itself. Each moves the cursor with `gband.win.set_cursor` and then calls `gband.colorscheme` with the new line's name, which gives the preview. Enter saves and closes. Escape and `q` restore the name kept when the list opened, then close. `on_close` of the list focuses the settings window.

Both windows belong to no plugin, as key-style's chooser did, so a failing callback reports as a configuration error and disables nothing.

### Reopening rides on the client, not on Lua
A reload replaces the Lua state, so Lua cannot remember the line. The host gets `host.settings_reopen(line)`, which stores the line in `Display`, the part of the client that outlives a reload. `settings.lua` calls it right before it saves. When the reload in `crates/client/src/lib.rs` succeeds, the client takes the stored line after `ConfigReloaded` is dispatched and calls a runtime entry point that runs `gband.settings.open` with that line in a callback context. A failed reload clears the line. A save that writes the same content still touches the file, so the watcher still reloads and the line is always consumed.

Alternatives considered:
- *Save without a reload.* The theme could apply live, but the sidebar and the key style cannot, and two kinds of save would behave differently.
- *A marker file.* The client already outlives the reload. A file adds a write and a cleanup for state that lives one reload long.

### Key list closes when a binding opens a focused plugin window
`crates/lua/src/runtime/gband/keylist.lua` compares the focused plugin window before and after running a line. When another floating plugin window is focused afterwards, it closes itself. This gives the requested behaviour for `s` and also for `:`, without the key list knowing about settings.

### Presets bind `s` to the API function
Both presets bind `prefix s` to `gband.settings.open` with `{ desc = "settings" }`, right after `:`. It is a function binding, so the client-attach table gives it the description. `keystyle.choose` is removed, and the `Attached` handler in `defaults.lua` calls `gband.settings.open()` under the new condition.

### Tests run under `terminal`
`g.start` gains `theme`, default `"terminal"`, written to `user/theme.lua` with the key style. `tests/common/mod.rs` does the same. `terminal` sets no palette, so program output in references keeps its own colors, and its groups use small indexes that read well in a reference. The reference files that recorded `default`'s colors are regenerated once with `GBAND_UPDATE_SCREENSHOTS`, and each diff is checked to contain only color lines and the rounded corners of tiles and floating windows. Tests that assert plain tile corners, such as `tests/attach.rs`, `tests/config.rs`, `tests/plugin_testing.rs`, `crates/client/tests/bars.rs` and the example in `docs/testing.md`, change to the rounded ones.

## Risks / Trade-offs

- [Every screen reference changes once] → Regenerate them in one task, review that only style lines and tile and floating window corners differ, and keep the regeneration in its own commit.
- [Users with their own `init.lua` see rounded borders] → The declared defaults change, so a file that never set the options changes too. Setting the four character options to `"plain"` restores the old look, and the README says so.
- [Rounded has no heavy variant] → A user who sets `focused_tile_border_chars` to `thick` gets square heavy corners beside rounded ones. `double` is the alternative, and both are documented.
- [A late palette pass costs a scan of the whole buffer per frame] → The scan is linear in cells and skipped when the palette is empty. 80×24 is under 2,000 cells, and large terminals are still far below the frame budget.
- [Two palette sources disagree: the host terminal's and the theme's] → Under a theme other than `default` and `terminal`, every default and ANSI color is replaced, so the host palette no longer shows. 256-color and 24-bit output keeps its own colors, which is what programs asked for.
- [Programs that pick light or dark from OSC 11 guess wrong] → Out of scope, as terminal-query-replies already decided. `terminal` remains the theme for users who rely on it.
- [A user's `init.lua` that calls `gband.colorscheme` hides the saved theme] → Intended: the explicit call wins. The settings window still shows the active colorscheme, so the user can see that their file decides.
- [The sidebar and keys lines do nothing under a user's own `init.lua` that ignores them] → Documented in the README. `gband.settings.sidebar()` and `gband.keystyle.use()` are the hooks such a file can call.
- [Theme palettes copied by hand drift from upstream] → Each theme's palette is checked in a test against a table of its eight base colors from the cited source.
