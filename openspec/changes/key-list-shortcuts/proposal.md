## Why

The key list shows each binding's key, but running one means moving the cursor line to it and pressing Enter. A user who reads `l` beside "focus the column to the right" expects to press `l` there and then. Pressing a line's own key should run it at once, the way a menu's accelerator works.

## What Changes

- **Direct keys in the key list**: while the key list is focused, pressing the key of one of its lines moves the cursor line to that line and runs it, exactly as moving there and pressing Enter does. The list stays open and focused, a binding that closes the focused plugin window closes the list, and the line of `keylist.open` does nothing beyond moving the cursor line.
- **Bindings win over the list's movement keys**: `j` and `k`, and any other key a line shows, run that line. They no longer move the cursor line when the `prefix` table binds them.
- **The list's own keys**: Up, Down, PageUp, PageDown, Home and End move the cursor line, Enter runs the cursor line, and Escape closes the list. A line whose key is one of these runs only through Enter, as does the line of the prefix key, which the prefix key never reaches because it enters the `prefix` table first.
- **BREAKING**: with the default configuration, `j` and `k` in the key list run "focus the window below" and "focus the window above" instead of moving the cursor line. Only Up and Down move it. There is no other key for that.

Out of scope:
- Another way to move the cursor line with `j` and `k`, or with a modifier on them.
- Direct keys in any other plugin window. The change uses the `keys` option every plugin window already has.
- Lines of tables other than `prefix`, filtering, and mouse input.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `key-list`: a line's own key moves the cursor line to it and runs it, the list keeps Up, Down, PageUp, PageDown, Home, End, Enter and Escape for itself, and `j` and `k` no longer move the cursor line when they are bound.

## Impact

- `crates/lua/src/runtime/gband/keylist.lua`: one `keys` entry per line whose key is not one of the list's own keys, each moving the cursor line and running the line as Enter does.
- `tests/lua/keylist_spec.lua` and its screenshots: moving through the list with Up and Down, and new cases for direct keys.
- The end-to-end key list case that navigation-mode adds, which moves the cursor line with `j`.
- `README.md` and `docs/plugins.md`: the key list's keys.
- No Rust change, no protocol change, and no new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- navigation-mode

### Expected Files
- README.md
- crates/lua/src/runtime/gband/keylist.lua
- docs/plugins.md
- tests/lua/keylist_spec.lua
- tests/lua/screenshots/keylist_spec/
- tests/navigation.rs
- openspec/changes/key-list-shortcuts/
