## Why

Moving and resizing windows with the mouse needs navigation mode, so the direct key style cannot do it at all. niri drives windows with a held modifier, Mod with a drag or the wheel, which works without entering any mode and in every terminal, because mouse reports carry Ctrl, Alt and Shift. gband's Alt+click is already never forwarded to programs, so Alt is free for this. The two key styles have also drifted apart: the modal preset binds three mouse drags that the direct preset lacks. From now on, both presets must offer the same features.

## What Changes

- **Mod + mouse**: both key style presets bind, in `root` and in `prefix`, `mod+leftmouse` to `drag_window`, `mod+rightmouse` to `drag_resize_window` and `mod+middlemouse` to `drag_band`. With drag-bands, a vertical `drag_band` switches bands. They work in interactive mode and navigation mode, with either key style.
- **Mod + wheel**: both presets bind, in `root` and in `prefix`, `mod+wheeldown` to `focus_band_down` and `mod+wheelup` to `focus_band_up`.
- **New option `mouse_mod`**: names the modifiers that `mod` stands for in a mouse name, `"alt"` by default. Some Linux desktops take Alt+drag to move their own windows, so a user sets `gband.opt.mouse_mod = "ctrl+alt"` once instead of rebinding each name. `mod` resolves when the bindings are used, as `prefix` does, so the option and the preset can come in any order in `init.lua`. Each binding can still be changed or removed with `gband.keymap.set` and `gband.keymap.del`.
- **Wheel names**: `wheelup`, `wheeldown`, `wheelleft` and `wheelright` become mouse names that a binding can name. A wheel step that matches a binding in the active table runs it. An unbound wheel step goes to its target as today, in any table or mode, so a plain wheel keeps reaching programs. After a wheel binding runs, further steps of the same mouse name within 150 ms are discarded, so one flick does not skip several bands. While a gesture runs, wheel steps match no binding.
- **BREAKING**: gband's own text selection over a program that reports the mouse moves from Alt+drag to Ctrl+Alt+drag. An unbound press with Ctrl and Alt held is not forwarded. An unbound press with Alt alone is now forwarded.
- **BREAKING**: Alt+wheel no longer reaches the program under the pointer with the default bindings. It switches bands.
- **Parity fix**: the direct preset gains the modal preset's `prefix leftmouse`, `prefix rightmouse` and `prefix middlemouse`, so Ctrl+Space then a drag works with the direct style.
- **Preset parity**: a new key-style requirement says both presets bind the same names in `root` and `prefix` to the same actions, except bindings that enter or leave navigation mode. A test compares the two presets.

Out of scope:
- Horizontal mod+wheel bindings. niri maps them to column focus; they can follow later.
- A hold-to-navigate key style. It needs key release events, which the kitty keyboard protocol gives only where the terminal enables it, and support is not yet universal by default.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `mouse`: "Mouse names in key tables" lets a wheel step match a binding, with the cooldown; "Interactive mode defaults" moves the selection override from Alt to Ctrl+Alt; "Wheel" sends only unbound steps to their target.
- `configuration`: "Key names" adds the wheel names and the `mod` modifier; a new requirement adds the `mouse_mod` option.
- `client-attach`: "Default mouse bindings" lists the mod rows in `root` and `prefix` for both key styles, and the direct style's `prefix` mouse rows.
- `key-style`: "Key style presets" lets a preset bind mod mouse names in `root`; new "Preset parity" requirement.
- `plugin-windows`: "Mouse in plugin windows" gives a wheel step that runs a binding to the binding, not to the plugin window.

## Impact

- `crates/core/src/input.rs`: `MouseKey` names a wheel direction as well as a button.
- `crates/lua/src/keys.rs`, `crates/lua/src/keymap.rs`, `crates/lua/src/api.rs`: parsing of wheel names and of `mod`, and a chord that resolves `mod` at use.
- `crates/lua/src/options.rs`: the `mouse_mod` option.
- `crates/client/src/bindings.rs`: wheel lookup through the key tables, `mod` resolution, the parity test.
- `crates/client/src/mouse.rs`: wheel bindings, the cooldown, the Ctrl+Alt selection override.
- `crates/lua/src/runtime/gband/keystyle/modal.lua`, `direct.lua`: the new rows.
- Tests: `crates/lua/tests/keystyle.rs`, `tests/mouse.rs`, `tests/keystyle.rs`, `crates/client/tests/actions.rs`.
- Docs: `README.md`, `docs/plugins.md`.
- No protocol change: everything happens in the client.

## Coordination

### Author
- gmteixeira

### Depends On
- drag-bands

### Expected Files
- openspec/changes/alt-mouse/
- openspec/specs/mouse/spec.md
- crates/core/src/input.rs
- crates/lua/src/keys.rs
- crates/lua/src/api.rs
- crates/lua/src/options.rs
- crates/lua/src/defaults.lua
- crates/lua/src/events.rs
- crates/lua/src/runtime.rs
- crates/lua/src/runtime/gband/keyform.lua
- crates/lua/src/runtime/gband/keystyle/modal.lua
- crates/lua/src/runtime/gband/keystyle/direct.lua
- crates/lua/tests/commands.rs
- crates/lua/tests/config.rs
- crates/lua/tests/keymap.rs
- crates/lua/tests/keystyle.rs
- crates/lua/tests/options.rs
- crates/lua/tests/plugins.rs
- crates/client/src/bindings.rs
- crates/client/src/lib.rs
- crates/client/src/mouse.rs
- crates/client/tests/actions.rs
- tests/mouse.rs
- tests/keystyle.rs
- tests/lua/keylist_spec.lua
- README.md
- docs/plugins.md
