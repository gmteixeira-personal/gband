## Why

With `loop_bands = false`, focusing right on a band's last column does nothing, so getting back to the first column means stepping left across every column. Some users want two ends without the loop drawing, and still want focus to go round: past the last column the camera scrolls back to the first. One option with three values covers all three behaviours. A second boolean would do nothing while looping is on, which is the default.

## What Changes

- **BREAKING** The client option `loop_bands` is replaced by `band_ends`, which takes `"loop"`, `"bounce"` or `"stop"` and defaults to `"loop"`. A config that still sets `loop_bands` gets a configuration error naming it, the load still succeeds, and `band_ends` stays `"loop"`.
- `"loop"` behaves as `loop_bands = true` does today. Focus goes round, and a long enough strip is drawn as a loop.
- `"bounce"` is new. Right of the last column focuses the first column, and left of the first focuses the last. The strip is drawn once, so the camera scrolls back across the band to the newly focused column, with the scroll animated.
- `"stop"` behaves as `loop_bands = false` does today. Focus stops at either end.
- Band up and down, including bands mode `j`, `k`, Up and Down, still stop at the first and last band whatever `band_ends` holds. The jump-modes spec is corrected to say so, because it currently says they follow `loop_bands`, and the code never looped them.
- The settings window gains a `bands` line as its third line, showing `loop`, `bounce` or `stop`. Its keys step through the three values and save the choice to `user/band_ends.lua`. `keys` moves to the fourth line, `I on new` to the fifth, and `reload` to the sixth with the modal style and the fifth with the others.
- The default configuration sets `band_ends` to the value `gband.settings.band_ends()` reads from `user/band_ends.lua`, and to `"loop"` when nothing valid is saved. A `user/init.lua` keeps whatever `band_ends` it sets.

## Capabilities

### New Capabilities

None.

### Modified Capabilities
- `layout-view`: "Focus across columns" gains `"bounce"`, "Camera" ties the looping strip to `"loop"`, and every scenario that named `loop_bands` now names a `band_ends` value.
- `animations`: "Camera scroll" gains a scenario for the bounce scroll back.
- `configuration`: "Options and their values" replaces `loop_bands` with `band_ends`, and "Defaults use the public API" sets `band_ends` from the saved setting.
- `jump-modes`: "Bands mode" says band steps stop at the ends.
- `mouse`: "Slide the band by dragging" renames the option in its scenario.
- `settings`: "Saved band ends" is added. "Settings window" and "Settings window keys" gain the `bands` line, and the scenarios of "Record the layout entering the floating style", "Restore the layout leaving the floating style" and "Reopen after a save" follow the lines it moves down.
- `floating-key-style`: the "Windows float" scenario that switches the style names the `keys` line's new place.

## Impact

- `crates/lua/src/options.rs`: `band_ends` replaces `loop_bands`, as a three-value enum beside `center_focused_column`.
- `crates/lua/src/defaults.lua` and `docs/internals/11-default-configs.md`: `gband.opt.band_ends = gband.settings.band_ends() or "loop"`.
- `crates/lua/src/runtime/gband/settings.lua`, its copy in `docs/internals/07-settings.md`, and `crates/lua/src/runtime/meta/gband.lua`: `gband.settings.band_ends()` and the `bands` line.
- Settings tests in `crates/lua/tests/settings.rs`, `tests/settings.rs`, `tests/lua/settings_spec.lua` and its screenshots, and the floating style tests that open the `keys` line.
- `crates/core/src/view.rs`: `View::set_band_ends`. `neighbour_column` wraps under loop and bounce, and the strip loops only under loop.
- `crates/client/src/lib.rs`: passes `band_ends` to the view.
- Tests in `crates/core/tests/view.rs`, `crates/client/tests/actions.rs`, `crates/client/tests/animation.rs`, `crates/lua/tests/options.rs`, `tests/lua/loop_bands_spec.lua` and `tests/lua_specs.rs`.
- `README.md`, `docs/plugins.md` and `docs/internals/08-key-styles.md`: the option's and the settings line's documentation.
- No protocol change. `band_ends` is a client view rule.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- README.md
- crates/client/src/lib.rs
- crates/client/tests/actions.rs
- crates/client/tests/animation.rs
- crates/core/src/view.rs
- crates/core/tests/view.rs
- crates/lua/src/defaults.lua
- crates/lua/src/options.rs
- crates/lua/src/runtime/gband/settings.lua
- crates/lua/src/runtime/meta/gband.lua
- crates/lua/tests/options.rs
- crates/lua/tests/settings.rs
- docs/internals/07-settings.md
- docs/internals/08-key-styles.md
- docs/internals/11-default-configs.md
- docs/plugins.md
- openspec/changes/band-ends/
- tests/lua/band_ends_spec.lua
- tests/lua/keylist_spec.lua
- tests/lua/loop_bands_spec.lua
- tests/lua/settings_spec.lua
- tests/lua/screenshots/settings_spec/
- tests/lua_specs.rs
- tests/settings.rs
