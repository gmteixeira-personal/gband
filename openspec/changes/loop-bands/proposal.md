## Why

A band's strip ends at its first and last column, so reaching the far side of a long band means walking back across every column. Making the strip a loop lets focus keep moving in one direction: past the last column comes the first again, drawn right after it, and past the first comes the last, drawn right before it.

## What Changes

- **Focus loops**: with looping on, focusing the column to the right of the last column focuses the first column, and focusing the column to the left of the first focuses the last. A band with one column does not change. This holds on every band, however short its strip.
- **The strip is drawn as a loop**: when a band's strip is long enough, the view draws it as a circle. The first column follows the last on the right, the last precedes the first on the left, and the camera can scroll without end in either direction. The camera of a looping band is held within the strip's width.
- **No window drawn twice**: a strip loops in drawing only when its width less its widest column is at least the terminal's width. A shorter strip is drawn once, as today, but focus still loops on it.
- **Camera follows the loop**: focus moved left or right scrolls the camera in the direction of the move, also across the seam. Other camera moves take the copy of the focused column that needs the smallest move. The camera scroll animates without a jump when the camera is held back within the strip.
- **Floating layer unaffected**: floating windows stay where they are on the terminal. Only the tiled strip loops, and the two layers stay independent.
- **Option**: a new client option, `loop_bands`, a boolean, default `true`. With it `false`, the strip and focus behave as they do today.

Out of scope:
- Session actions at the strip's edge: consume or expel, and moving a column, still stop at the first and last column.
- Looping between bands: viewing the band above the first or below the last still changes nothing.

## Capabilities

### New Capabilities

None.

### Modified Capabilities
- `layout-view`: "Focus across columns" loops, "Camera" gains the looping strip and its copies, "Shown windows" places tiles at the copy inside the terminal.
- `animations`: "Camera scroll" scrolls across the seam without a jump.
- `configuration`: "Options" gains the client option `loop_bands`.

## Impact

- `crates/core/src/view.rs`: the looping rules for focus, the camera and the shown windows, and the option's setter.
- `crates/core/src/geometry.rs`: the strip width and the loop test, if they do not fit in `view.rs`.
- `crates/client/src/lib.rs`: passes `loop_bands` to the view.
- `crates/client/src/animation.rs`: the camera spring follows the unreduced camera.
- `crates/client/src/render.rs`: places each tile at its copy inside the terminal.
- `crates/lua/src/options.rs` and `crates/lua/src/defaults.lua`: the option.
- Tests in `crates/core/tests/view.rs`, `crates/client/tests/actions.rs`, `crates/client/tests/animation.rs`, `crates/client/tests/render.rs` and `crates/lua/tests/options.rs`.
- `README.md` and `docs/plugins.md`: the option tables.
- No protocol change: looping is a client view rule and never crosses the connection.

## Coordination

### Author
- gmteixeira

### Depends On
- floating-windows
- rename-panes-to-windows

### Expected Files
- crates/core/src/view.rs
- crates/core/src/geometry.rs
- crates/core/tests/view.rs
- crates/core/tests/geometry.rs
- crates/client/src/lib.rs
- crates/client/src/animation.rs
- crates/client/src/render.rs
- crates/client/tests/actions.rs
- crates/client/tests/animation.rs
- crates/client/tests/render.rs
- crates/client/tests/snapshots/
- crates/lua/src/options.rs
- crates/lua/src/defaults.lua
- crates/lua/tests/options.rs
- README.md
- docs/plugins.md
- openspec/changes/loop-bands/
