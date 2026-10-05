## Why

The client shows nothing but the ribbon: there is no place to see which band is viewed, where focus sits on the strip, or that a key table is active, and plugin errors take over the ribbon's bottom row. The plugin foundation gives Lua an owner model, events and error isolation. This change builds the first UI extension point on it: a status line that is only a container of components, written in Lua, where every built-in segment is a bundled plugin using the same public API as a third-party one.

## What Changes

- Highlight groups: `gband.hl.set(name, spec)`, `gband.hl.default(name, spec)` and `gband.hl.get(name, opts)`. A spec holds `fg`, `bg`, `bold`, `italic`, `underline`, `reverse`, `dim` and `link`. Colors are `"#rrggbb"`, an index from 0 to 255, or one of the 16 ANSI names. A linked group inherits from its target, and the fields it sets itself win. Link cycles are refused when they are made and broken when they are resolved. Defaults are a separate layer under explicit settings, so a plugin's default never overrides a theme or a user setting, whatever the load order. Group names are global, not namespaced, so themes can style any plugin's groups.
- Truecolor detection in the client from its own `COLORTERM`. Without truecolor, hex colors are drawn as the nearest index from 16 to 255. The client detects no terminal capabilities today, so this detection is new.
- Built-in groups `StatusLine`, `StatusLineSegment`, `StatusLineSeparator`, `StatusLineMuted`, `StatusLineAccent` and `StatusLineError`, whose defaults belong to the status line API.
- Colorschemes: `colors/<name>.lua` in any runtimepath entry, then among the colorschemes bundled with gband. `gband.colorscheme(name)` clears the explicit settings, runs the file protected and atomically, and emits `ColorschemeChanged` after loading. A bundled `default` colorscheme is active when loading starts. A failing colorscheme is reported like a plugin error and leaves the previous one active.
- New built-in events `HighlightChanged` and `ColorschemeChanged`, named in the foundation's style. Also `LayoutChanged`, because consuming or expelling a pane changes the column count without any foundation event, and the position segment needs to see that.
- Placement options `statusline_position` (`"bottom"`, `"top"` or `"off"`, default `"bottom"`) and `statusline_height` (default 1). The client's **ribbon area** is its terminal less the status line rows. The client reports the ribbon area to the server wherever it reports its terminal size today, and uses it as its viewport. A reload that changes the ribbon area takes the same path as a terminal resize.
- Component API: `gband.ui.statusline.add(spec)`, `remove(id)` and `list()`. A spec holds `id`, `align`, `priority`, `order`, `hl`, `redraw_on`, `redraw_interval` and `render`. `render(ctx)` returns a string or a list of spans `{ text, hl }`, and control characters are removed from the text. `ctx` holds the component's available width, the total width, `side`, the active key table, the viewed band and the focused column. Renders run only on triggers and their results are cached. They never run per frame.
- Layout in left, center and right regions with separators set by the `statusline_separator` option. When the line is too wide, the lowest priority is dropped first, then the last component left is cut with `…`. Widths are display widths. Plugins can use `gband.ui.width` and `gband.ui.truncate`.
- Errors: each render runs protected under its own instruction budget. A failing component is disabled. The status line's error item shows the latest configuration, plugin or render error. It replaces the foundation's bottom-row banner, which remains only while no status line is drawn.
- Component ids follow the foundation's namespacing. A plugin adding one component may omit `id`, and the component then takes the plugin's name.
- Bundled segment plugins: `gband.statusline.band` (the viewed band), `gband.statusline.position` (focused column and count, such as `3/7`), `gband.statusline.mode` (the active key table, hidden in `root`) and `gband.statusline.clock` (off by default, redrawn on an interval). The default configuration sets up `band`, `position` and `mode`. A `user/init.lua` opts in with the same `gband.plugin` calls, and can reorder segments with their `align`, `priority` and `order` options.
- `require` also finds the modules bundled with gband, after the runtimepath and before Lua's own path.
- A sample plugin `examples/plugins/pane` with a segment that redraws on focus changes, its own highlight group, and a colorscheme that overrides that group. The plugin guide documents the component API, highlight groups and the colorscheme format.
- **BREAKING**: with the default placement, the ribbon is one row shorter than the terminal. A `user/init.lua` that sets up no segment still gets an empty status line row. `statusline_position = "off"` restores the full screen.

Out of scope: a key hints plugin, column headers, a band bar, overlays, pickers and a palette, segments that need title, bell, process or agent data, a connection segment (the client has no reconnect state), server-side Lua, and mouse input on the status line.

## Capabilities

### New Capabilities

- `highlights`: highlight groups, the color forms, links and their cycle handling, the defaults layer, `HighlightChanged`, and color downgrade without truecolor.
- `colorschemes`: colorscheme files and lookup, `gband.colorscheme`, the bundled `default` colorscheme, error isolation and `ColorschemeChanged`.
- `status-line`: placement, the component API, render triggers and caching, the context, layout and truncation, the error item, the built-in groups, and the bundled segment plugins.

### Modified Capabilities

- `client-attach`: the ribbon area, which the client reports as its terminal size, uses as its viewport and draws the ribbon in. The status line rows. The banner only while no status line is drawn.
- `configuration`: the gband API installed before the init file includes the status line, its groups and the `default` colorscheme. The default configuration sets up the bundled segments. The options `statusline_position`, `statusline_height` and `statusline_separator`. Errors are shown in the status line.
- `lua-events`: the built-in events `LayoutChanged`, `HighlightChanged` and `ColorschemeChanged`.
- `plugins`: module lookup also finds the modules bundled with gband. A plugin error at setup shows in the status line's error item.

## Impact

- `crates/lua`: new embedded Lua sources for the highlight store, colorscheme loading, the status line container and the bundled segment plugins. New Rust host primitives: an isolated guarded call with its own budget, timers, the client's view state, the presented line, built-in event emission, and display width. A bundled-module searcher, the two placement options, and `Runtime` methods for the line, timers and errors.
- `crates/client`: truecolor detection and color conversion, the ribbon area in `Display`, the handshake and resizes, status line drawing and its row offset, timers in the event loop, and the resize path on a reload that changes placement.
- New `unicode-width` workspace dependency for `crates/lua`. ratatui already depends on it.
- No protocol or server change. The server keeps setting its screen area from the size that clients report.
- New `examples/plugins/pane/` and new sections in `docs/plugins.md`. `README.md` mentions the status line.

## Coordination

### Author
- gmteixeira

### Depends On
- client-plugin-foundation

### Expected Files
- Cargo.toml
- Cargo.lock
- crates/lua/
- crates/client/src/lib.rs
- crates/client/src/render.rs
- crates/client/src/connect.rs
- crates/client/src/color.rs
- crates/client/src/placement.rs
- crates/client/tests/
- tests/statusline.rs
- examples/plugins/pane/
- docs/plugins.md
- README.md
- openspec/changes/status-line/
