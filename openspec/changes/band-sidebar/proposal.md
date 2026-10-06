## Why

The status line takes 20 columns of every terminal for a band number, a mode name, key hints and a window position. The user needs much less: the current mode and which bands exist, with the viewed one marked. One column can show that and give the other 19 back to the ribbon. Key hints are already covered by the key list, and errors by the error list.

## What Changes

- **Sidebar**: a new bundled plugin, `gband.sidebar`, adds a bar one column wide through `gband.bar`, on the left by default.
  - Row 0 shows the mode letter. It is `I` while the `root` key table is active, which is interactive mode. Otherwise it is the first character of the active table's label, uppercased, so navigation mode shows `N`.
  - Row 1 is blank.
  - From row 2 down, one row per band of the layout, in layout order, including the empty last band. Bands 1 to 9 show their digit. Bands 10 to 35 show `a` to `z`. A new session therefore shows `1` and `2`.
  - The viewed band's row is drawn in `SidebarBandActive`, bright and bold. Every other band's row is drawn in `SidebarBand`, dim.
  - The bottom row shows a red `!` in `SidebarError` while the client reports an error. It replaces the status line's `error` item. Without a sidebar, errors still show as the banner on the ribbon's bottom row.
  - Setup options: `side`, `"left"` or `"right"`, and `order`. The width is always one column.
  - The sidebar redraws when the viewed band, the bands, the active key table or the error list change.
  - A left click on a band's label views that band, as `gband.band.view` does. Other clicks on the sidebar do nothing.
  - The sidebar draws on the terminal's default background. The `default` colorscheme no longer sets `Bar`, so every bar keeps the terminal's colours until a colorscheme sets `Bar`.
- **Default configuration**: sets up `gband.errors`, then `gband.sidebar`. On an 80×24 terminal the ribbon area becomes 79×24, from column 1.
- **BREAKING**: the status line is removed. This covers the `gband.statusline` plugin, the segment plugins `gband.statusline.band`, `.mode`, `.position` and `.clock`, the `gband.ui.statusline` component API, the status line's error item, and the groups `StatusLine`, `StatusLineSegment`, `StatusLineSeparator`, `StatusLineMuted`, `StatusLineAccent` and `StatusLineError`. A user file that sets up one of these plugins gets the configuration error of an unknown module. A user file that calls `gband.ui.statusline` gets a Lua error. The removed options `statusline_position`, `statusline_height` and `statusline_separator` lose their migration message, which pointed at `gband.statusline`, and are reported as unknown options.
- **BREAKING**: the key hints segment, `gband.statusline.hints`, is removed, with its `labels` option and the groups `KeyHintKey` and `KeyHintLabel`. The key list stays the way to read the bindings of navigation mode.
- **Kept**: `gband.bar` stays as `sidebars-borders-steps` defines it, so plugins can still add wider bars on either side. `gband.ui.width` and `gband.ui.truncate` stay, and move to the plugin-windows capability. The key form that the key list, the key style chooser and `gband.keyform` use moves to the key-list capability.
- **Key list groups**: `KeyListKey` and `KeyListMuted` get their own defaults in place of their links to the removed status line groups.
- **Examples**: the `window` and `agent-status` example plugins draw through their own `gband.bar` bars in place of status line components. The `dusk` colorscheme sets the sidebar groups.

Out of scope:
- A friendlier error for configurations that still use the status line. The error of an unknown module or a nil field names the line, which is enough while gband is in early development.
- Bands past the 35th, and bands that do not fit the terminal's height. They are not drawn.

## Capabilities

### New Capabilities
- `sidebar`: the bundled `gband.sidebar` plugin. Covers its bar, the mode letter, the band rows and their labels, the viewed band's emphasis, the error marker, clicking a band, setup options, redraw triggers and its highlight groups.

### Modified Capabilities
- `status-line`: every requirement is removed, and the capability is retired.
- `key-hints`: every requirement is removed, and the capability is retired. "Key form" moves to `key-list`.
- `key-list`: gains "Key form". The plugin gives `KeyListKey` and `KeyListMuted` their own defaults.
- `plugin-windows`: gains "Display width helpers". Defines control characters itself in place of naming the status-line capability.
- `configuration`: the default configuration sets up `gband.sidebar` in place of the status line and its segments. Errors show as the sidebar's error marker, or as the banner when no sidebar is drawn. The configuration file's built-in groups no longer name the status line.
- `client-attach`: ribbon area, input size and presentation scenarios follow the 1-column sidebar in place of the 20-column status line. The banner shows when no error marker is drawn. Key binding scenarios read the mode from the sidebar.
- `plugins`: the bundled modules list `gband.sidebar` in place of the segment modules.
- `plugin-testing`: the case environment sets up the sidebar on the right in place of the status line. The observation and reload scenarios read the sidebar or a test bar.
- `test-channel`: the settle and frozen time scenarios draw through a bar in place of status line segments. Frozen time no longer names `redraw_interval`.
- `colorschemes`: the default colorscheme sets the sidebar groups. Its scenarios use sidebar groups.
- `highlights`: the highlight change scenario uses a sidebar group.
- `lua-control`: the removed pane names no longer mention status line components or the hints segment's `labels`.
- `borders`: the scenario without side borders sets up no sidebar.
- `error-list`: the floating window size and the clear scenario follow the sidebar.
- `key-style`: presets leave the sidebar to the default configuration. The direct style, chooser and first start scenarios read the sidebar's mode letter or the 79-column ribbon.
- `lua-prompt`: the prompt's size follows the 79-column ribbon, entering a mode is read from the sidebar, and the prompt hint scenario is removed.
- Renamed requirements: OpenSpec keeps every scenario name of a modified requirement, so the eight requirements whose scenarios named the status line, in client-attach, configuration, key-list, key-style and plugin-testing, are removed and added back under new names, as design.md lists. settings-themes, window-names and alt-mouse are updated to target the new names.

## Impact

- Lua runtime: new `gband/sidebar.lua`; `gband/statusline.lua` and `gband/statusline/` are removed; `defaults.lua` sets up the sidebar; `keylist.lua` drops the status line group defaults; `colors/default.lua` sets the sidebar groups and no longer sets `Bar`.
- Lua host (Rust): the status line component API and its layout in `ui.rs` and `runtime.rs` are removed. The pushed client state drops the `column` and the `windows` list, which only status line components read. It keeps the band number and the focused window, which `gband.win` and the control API read, and the band index and count, the active table and the errors, which the sidebar reads. `options.rs` and `api.rs` lose the list of removed status line options. `gband.ui` keeps `width` and `truncate`. `bundled.rs` lists the new module.
- Client: the banner shows when no error marker is drawn. Nothing else in the client changes, because the sidebar is an ordinary bar.
- Docs and examples: `README.md`, `docs/plugins.md`, `docs/testing.md`, the `gband-test` skill, and the `window`, `agent-status` and `hello` examples.
- Tests: the status line and key hint tests are removed or replaced by sidebar tests. Tests and screenshots that assume the 20-column status line move to the 1-column sidebar.

## Coordination

### Author
- gmteixeira

### Depends On
- sidebars-borders-steps
- clear-errors
- lua-prompt
- key-style
- mouse

### Expected Files
- openspec/changes/band-sidebar/
- .claude/skills/gband-test/SKILL.md
- README.md
- crates/client/src/lib.rs
- crates/client/tests/actions.rs
- crates/client/tests/bars.rs
- crates/client/tests/plugin_windows.rs
- crates/client/tests/render.rs
- crates/client/tests/sidebar.rs
- crates/client/tests/snapshots/render__ribbon_beside_the_sidebar.snap
- crates/client/tests/snapshots/render__tile_taller_than_the_ribbon_area.snap
- crates/client/tests/snapshots/sidebar__default_sidebar_snapshot.snap
- crates/client/tests/snapshots/sidebar__error_marker_snapshot.snap
- crates/client/tests/snapshots/statusline__default_status_line.snap
- crates/client/tests/snapshots/statusline__error_item.snap
- crates/client/tests/snapshots/statusline__prefix_hints_cut.snap
- crates/client/tests/snapshots/statusline__right_placement.snap
- crates/client/tests/snapshots/statusline__short_terminal_drops_and_cuts.snap
- crates/client/tests/statusline.rs
- crates/lua/src/api.rs
- crates/lua/src/bundled.rs
- crates/lua/src/defaults.lua
- crates/lua/src/lib.rs
- crates/lua/src/options.rs
- crates/lua/src/runtime.rs
- crates/lua/src/runtime/gband/colors/default.lua
- crates/lua/src/runtime/gband/keylist.lua
- crates/lua/src/runtime/gband/sidebar.lua
- crates/lua/src/runtime/gband/statusline.lua
- crates/lua/src/runtime/gband/statusline/
- crates/lua/src/ui.rs
- crates/lua/tests/colorschemes.rs
- crates/lua/tests/common/mod.rs
- crates/lua/tests/config.rs
- crates/lua/tests/highlights.rs
- crates/lua/tests/key_hints.rs
- crates/lua/tests/keystyle.rs
- crates/lua/tests/options.rs
- crates/lua/tests/plugin_windows.rs
- crates/lua/tests/plugins.rs
- crates/lua/tests/prompt.rs
- crates/lua/tests/removed.rs
- crates/lua/tests/sidebar.rs
- crates/lua/tests/statusline.rs
- docs/plugins.md
- docs/testing.md
- examples/plugins/agent-status/README.md
- examples/plugins/agent-status/client.lua
- examples/plugins/agent-status/tests/
- examples/plugins/window/README.md
- examples/plugins/window/colors/dusk.lua
- examples/plugins/window/lua/window/init.lua
- examples/plugins/window/tests/
- openspec/changes/alt-mouse/specs/configuration/spec.md
- openspec/changes/settings-themes/specs/configuration/spec.md
- openspec/changes/settings-themes/specs/key-style/spec.md
- openspec/changes/window-names/specs/configuration/spec.md
- openspec/changes/window-names/tasks.md
- tests/attach.rs
- tests/common/mod.rs
- tests/config.rs
- tests/errors.rs
- tests/floating.rs
- tests/keystyle.rs
- tests/lua/errors_spec.lua
- tests/lua/keylist_spec.lua
- tests/lua/keystyle_spec.lua
- tests/lua/loop_bands_spec.lua
- tests/lua/prompt_spec.lua
- tests/lua/screenshots/errors_spec/
- tests/lua/screenshots/keylist_spec/
- tests/lua/screenshots/keystyle_spec/
- tests/lua/screenshots/prompt_spec/
- tests/lua/screenshots/sidebar_spec/
- tests/lua/screenshots/statusline_spec/
- tests/lua/sidebar_spec.lua
- tests/lua/statusline_spec.lua
- tests/lua_specs.rs
- tests/mouse.rs
- tests/navigation.rs
- tests/plugin_runtime.rs
- tests/plugin_testing.rs
- tests/plugin_windows.rs
- tests/prompt.rs
- tests/sidebar.rs
- tests/statusline.rs
