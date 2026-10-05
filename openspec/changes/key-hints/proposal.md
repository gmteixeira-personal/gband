## Why

A user has to remember every binding: nothing on screen says which keys work in the active key table or what they do. The status line gives a place to show it. A key hints segment, like the hint bar of zellij, lists the active table's keys with a short label each and changes as tables change, so a new user can find the prefix key and what follows it without reading the configuration.

## What Changes

- A bundled segment plugin `gband.statusline.hints`, plugin name `hints`, built on the status line component API like `band`, `mode` and `position`. It shows one hint per binding of the active key table: the key, then a label. In `root` it starts with the prefix key labelled `prefix`, followed by the bindings of `root`.
- Keys are shown in a short form: modifiers as `C-`, `A-` and `S-`, named keys in lowercase, so `ctrl+space` shows as `C-space`. A `prefix` key in a table other than `root` shows as the key the `prefix` option names.
- Labels: a binding's own `desc` when it differs from its action's description, otherwise a short built-in label for each built-in action (`left`, `right`, `new`, `close`, `width` and so on), otherwise the registered action's description or full name. The option `labels` overrides the label of any action by name. A binding to a function with no description is left out.
- The segment fits itself to the width the status line leaves it: it shows the leading hints that fit, ends with `…` when some are left out, and hides itself when not even one fits.
- Highlight groups `KeyHintKey` (default link `StatusLineAccent`) and `KeyHintLabel` (default link `StatusLineSegment`).
- A new component field `fill` in the status line. A fill component renders after the other components of the same trigger, and renders again whenever the width left to it changes. The hints segment uses it, so its fit follows the mode and position segments without a stale width.
- The default configuration sets up `hints` after `mode`, so the default status line shows `C-space prefix` in `root` and the prefix table's hints after Ctrl+Space.
- `docs/plugins.md` documents the segment, its options and groups, and the `fill` field.

Out of scope: a separate status line row for hints, hints for key sequences longer than two keys, grouping or paging hints, mouse input on hints, and changing the descriptions the default configuration gives its bindings.

## Capabilities

### New Capabilities

- `key-hints`: the bundled hints segment, its key form, its labels and their order of precedence, fitting to the available width, its options, its highlight groups, and its setup by the default configuration.

### Modified Capabilities

- `status-line`: the component field `fill`, and the render trigger that renders fill components after the others and again when their available width changes.
- `configuration`: the default configuration sets up `gband.statusline.hints` between `mode` and `position`.

## Impact

- `crates/lua`: a new embedded `src/runtime/gband/statusline/hints.lua`, registered in the bundled module table. `statusline.lua` gains the `fill` field and the fill pass after each render pass and width change. `defaults.lua` gains one `gband.plugin` call.
- New `crates/lua/tests/key_hints.rs`. The status line tests gain `fill` cases.
- Client snapshots and integration tests of the default status line change, since the default line now shows the hints.
- `docs/plugins.md` and `README.md`.
- No Rust host primitive, protocol or server change.

## Coordination

### Author
- gmteixeira

### Depends On
- status-line

### Expected Files
- crates/lua/src/runtime/gband/statusline.lua
- crates/lua/src/runtime/gband/statusline/hints.lua
- crates/lua/src/bundled.rs
- crates/lua/src/defaults.lua
- crates/lua/tests/statusline.rs
- crates/lua/tests/key_hints.rs
- crates/lua/tests/config.rs
- crates/client/tests/statusline.rs
- crates/client/tests/snapshots/
- tests/statusline.rs
- docs/plugins.md
- README.md
- openspec/changes/key-hints/
