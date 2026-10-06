## Why

The sidebar lists the bands, and a click on a label views that band, but the wheel over the sidebar does nothing. Turning the wheel over the list of bands is the natural way to move through them without aiming at a label or reaching for the keyboard.

## What Changes

- A wheel step down over any cell of a shown sidebar views the band below the viewed band, and a step up views the band above, as `focus_band_down` and `focus_band_up` do. Each step moves one band, so the view follows the wheel band by band.
- The step works in any key table and mode, does not change the active key table, and moves relative to the viewed band, not to the row under the pointer.
- At the first or last band a step past the end changes nothing, because viewing does not loop between bands.
- The sidebar takes no action on a step left or right, or on a step with Ctrl, Alt or Shift held, so modified wheel steps stay free for key bindings. With the default keys, Alt and the wheel over the sidebar view one band through the `mod+wheeldown` and `mod+wheelup` bindings, as the wheel alone does.
- The README's sidebar section describes the wheel.

Out of scope:
- A cooldown that turns a fast flick into one band. Each step moves one band.
- Any change to how the wheel reaches windows, plugin windows or the empty ribbon.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `sidebar`: a new requirement, "Band wheel", lets a wheel step over the sidebar view the band above or below.

## Impact

- `crates/lua/src/runtime/gband/sidebar.lua`: a `MouseScrolled` handler beside the click handler.
- `crates/client/tests/sidebar.rs` and `tests/lua/sidebar_spec.lua`: wheel tests.
- `README.md`: the sidebar section.

## Coordination

### Author
- gmteixeira

### Depends On
- alt-mouse

### Expected Files
- openspec/changes/sidebar-wheel/
- crates/lua/src/runtime/gband/sidebar.lua
- crates/client/tests/sidebar.rs
- tests/lua/sidebar_spec.lua
- README.md
