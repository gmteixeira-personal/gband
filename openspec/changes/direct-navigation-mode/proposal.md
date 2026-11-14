## Why

The direct key style runs one action per prefix key. Moving across five columns costs ten keys. The modal style's navigation mode keeps keys active until Escape, but switching to it changes every other key. Direct users want both: one-shot keys by default, and a sticky navigation mode a key away.

## What Changes

- The direct preset binds `space` in `prefix`. Ctrl+Space then Space enters a new mode, the table `navigation`, labelled `navigation`.
- The `navigation` table holds the modal preset's `prefix` bindings, entry for entry, in the same order. Every key behaves as in modal navigation mode:
  - Escape and Enter return to interactive mode.
  - `n` opens a window, and returns to interactive mode only while `I on new` is on.
  - `m` toggles multi mode, then returns to interactive mode.
  - Ctrl+Space sends the prefix key, then returns to interactive mode.
  - `a` and `b` enter agents mode and bands mode.
  - `?`, `:`, `N`, `s`, `!` and `e` open their windows and return to interactive mode, as they do from modal navigation mode.
  - Mouse names and unbound keys follow the client-attach capability's rules for a mode: the mode stays active.
- A new bundled module, `gband.navigation`, makes those bindings in a named table. The modal preset calls it for `prefix`, and the direct preset calls it for `navigation`. One source keeps the two tables equal.
- `gband.jump.setup` takes an optional table name, defaulting to `prefix`, and binds `a` and `b` there.
- The key list lists the table active when it opens, or `prefix` when `root` was active. Opened from direct navigation mode, its title is `navigation keys`.
- The sidebar shows `N` in direct navigation mode, through the existing `gband.keymap.label` rule.
- The settings window shows the `I on new` line with the direct style too, because the setting now changes direct navigation mode's `n`. The direct style's settings window grows from five lines to six.
- "Preset parity" allows one more difference: the direct preset's `space` in `prefix`.
- Direct `prefix` keys other than `space` keep their one-shot behaviour.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `key-style`: "Direct preset" declares the `navigation` mode and binds `prefix space`. "Modal preset" makes its bindings through `gband.navigation`. "Preset parity" adds the `space` difference and requires `navigation` to equal modal `prefix`.
- `client-attach`: "Key bindings" gives the direct style's `space` row and its `navigation` table.
- `jump-modes`: "Jump mode tables" lets `setup` bind `a` and `b` in a named table.
- `key-list`: "Key list window" lists the active table's bindings when a table other than `root` is active.
- `settings`: "Settings window" and "Saved I on new" show and apply `I on new` with the direct style.

## Impact

- `crates/lua/src/runtime/gband/navigation.lua`: new module.
- `crates/lua/src/bundled.rs`: registers the module.
- `crates/lua/src/runtime/gband/keystyle/modal.lua` and `direct.lua`: call the module. Direct adds the mode and `space`.
- `crates/lua/src/runtime/gband/jump.lua`, `keylist.lua` and `settings.lua`: the table argument, the listed table and the `I on new` line.
- `crates/lua/src/runtime/meta/gband.lua`: types for `gband.navigation` and the `setup` argument.
- `docs/internals/08-key-styles.md`, `docs/internals/07-settings.md`, `docs/plugins.md`, `docs/tutorial/00-setup.md` and `README.md`.
- Tests in `crates/lua/tests/`, `crates/client/src/bindings.rs`, `tests/keystyle.rs`, `tests/settings.rs`, `tests/lua/keystyle_spec.lua` and `tests/lua/settings_spec.lua`.
- No protocol or Rust keymap change. The client already keeps any declared mode active.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- README.md
- crates/client/src/bindings.rs
- crates/harness/src/case.rs
- crates/lua/src/bundled.rs
- crates/lua/src/runtime/gband/jump.lua
- crates/lua/src/runtime/gband/keylist.lua
- crates/lua/src/runtime/gband/keystyle/direct.lua
- crates/lua/src/runtime/gband/keystyle/modal.lua
- crates/lua/src/runtime/gband/navigation.lua
- crates/lua/src/runtime/gband/prelude.lua
- crates/lua/src/runtime/gband/settings.lua
- crates/lua/src/runtime/meta/gband.lua
- crates/lua/src/sides.rs
- crates/lua/tests/config.rs
- crates/lua/tests/keystyle.rs
- crates/lua/tests/settings.rs
- docs/internals/00-boundary.md
- docs/internals/07-settings.md
- docs/internals/08-key-styles.md
- docs/internals/10-keylist-and-prompt.md
- docs/internals/12-own-prelude.md
- docs/plugins.md
- docs/testing.md
- docs/tutorial/00-setup.md
- examples/internals/12-own-prelude/user/lua/gband/prelude.lua
- openspec/changes/direct-navigation-mode/
- tests/keystyle.rs
- tests/lua/agents_spec.lua
- tests/lua/floating_spec.lua
- tests/lua/keylist_spec.lua
- tests/lua/keystyle_spec.lua
- tests/lua/screenshots/settings_spec/the-settings-window-with-the-direct-key-style--direct.txt
- tests/lua/settings_spec.lua
- tests/settings.rs
