## Why

A later Lua-only change, `floating-key-style`, adds a key style in which every window floats and each floating window's top border is a title bar: the window's name on the left and `[_][□][X]` (minimize, maximize, close) on the right. Today the client draws only the window's shown name on a top border, and Lua has no way to add text there. Small floating plugin windows laid over the border do not work: the stacking order of floating windows is not exposed, and plugin windows draw above every floating window, so a lower window's buttons would show over a higher window.

## What Changes

- **Decorations provider.** `gband.core.provide("decorations", fn)` registers a function. While one is registered, the client calls it for each tiled and floating window whose top border it draws, with a new info table:
  - `window`: the window's number;
  - `floating`: whether the window floats;
  - `focused`: whether its border is drawn in the focused style;
  - `width`: the box's width in cells, border included, as drawn in that frame.
- The function returns nil, or a list of spans. A span is a string, or `{ text = <string>, style = <style table> }`, where `style` is optional and is a style as plugin window frames take it. Lua gets a highlight group's style from `require("gband.hl").drawn(name)`. Control characters are removed, and widths are measured as `gband.ui.width` measures them.
- **Placement.** The client draws the spans right-aligned on the box's top row, so that their last cell is the box's third-last column. The top-right corner and the cell left of it stay border, so a resize corner of two cells stays free. In a 40-wide box, `[_]`, `[□]` and `[X]` take columns 29 to 37.
  - When the spans are wider than the box less 4 cells, the client draws none of them.
  - When the window's border does not draw the top side, or the box's top row is not drawn, the client draws none, as for the title.
  - Decorations are drawn whether window titles are on or off.
  - They follow the box as its border does, also while an animation runs, and are cut at the ribbon area's edge as the border is.
- **Title cut.** While decorations are drawn, the border title is cut to the box's width less 4 cells and less the decorations' width, so at least one border cell separates the two. Without decorations, the title keeps today's cut, the box's width less 2.
- **Style.** A span without `style` is drawn in the window's border style, focused or not. A span's `style` is applied over the border style, field by field.
- **Cheap calls.** The client calls the function at most once per window per frame. It keeps each window's result and draws it again without calling while the info is the same and no other Lua code has run since. A call runs outside a callback, with its own instruction limit.
- **Failures.** An error, the instruction limit, or a wrong return value is reported once, preceded by `decorations: `. The client then draws no decorations and calls the function no more until it is registered again or a load succeeds. This keeps a broken provider from reporting an error at every frame.
- **Hit-testing stays in Lua.** The placement rule is documented. Lua finds the span under a press from the box cell and the box width that the sibling change `mouse-binding-fallthrough` adds to the mouse fields. This change adds no mouse field.
- No provider is registered by default, so nothing gband draws today changes. No protocol, server or default configuration change. `gband.api_version` does not rise, because a new provider kind only adds to the documented API.

Out of scope:
- The title bar, its buttons and what clicking them does. These belong to `floating-key-style`.
- Decorations on the left of the title, on any side other than the top, and on floating plugin windows.
- Several decoration sources at once. One function serves all windows. A module that merges the spans of several plugins can be added later on top of the provider.

## Capabilities

### New Capabilities
- `window-decorations`: the decorations provider's info table, return value and call rules, the placement of the spans on the top border and the narrow-box rule, their style, and how a failing provider is reported.

### Modified Capabilities
- `lua-api`: "Providers" adds the `"decorations"` kind and its absent behaviour.
- `window-names`: "Border title" cuts the title short of the decorations.
- `borders`: "Drawing a border" lets a title or decorations that another capability draws on a border replace the cells they cover.

## Impact

- Lua crate: a new `crates/lua/src/decorations.rs`, declared and exported from `crates/lua/src/lib.rs`, holds the registration, the call with its own instruction limit, the span reader and the failed state. `crates/lua/src/ui.rs` accepts the new kind in `provide`. `crates/lua/src/runtime.rs` exposes the call and a counter of Lua runs for the client's cache.
- Client: `crates/client/src/render.rs` lists the top borders a frame draws, draws the spans and cuts the title. `crates/client/src/lib.rs` keeps the cache, calls the provider before each draw and reports failures.
- Tests: `crates/lua/tests/core.rs`, a new `crates/lua/tests/decorations.rs`, `crates/client/tests/render.rs` with two new snapshots, a new `crates/client/tests/decorations.rs`, and a new end-to-end spec `tests/lua/decorations_spec.lua` with its screenshots, registered in `tests/lua_specs.rs`.
- Docs: `docs/plugins.md`, `README.md`, `docs/internals/00-boundary.md` and `docs/tutorial/05-layout.md`.
- No new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- openspec/changes/window-decorations/
- README.md
- crates/client/src/lib.rs
- crates/client/src/render.rs
- crates/client/tests/decorations.rs
- crates/client/tests/render.rs
- crates/client/tests/snapshots/render__decorations_beside_a_title.snap
- crates/client/tests/snapshots/render__decorations_on_a_floating_window.snap
- crates/lua/src/decorations.rs
- crates/lua/src/lib.rs
- crates/lua/src/runtime.rs
- crates/lua/src/ui.rs
- crates/lua/tests/core.rs
- crates/lua/tests/decorations.rs
- docs/internals/00-boundary.md
- docs/plugins.md
- docs/tutorial/05-layout.md
- tests/lua/decorations_spec.lua
- tests/lua/screenshots/decorations_spec/
- tests/lua_specs.rs
