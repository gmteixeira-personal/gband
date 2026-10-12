## Why

The sidebar's `!` marker says an error exists, but a press on it does nothing, and no default key opens or clears the error list. A user sees the marker and has to type `gband.cmd.run("errors.open")` in the Lua prompt or bind a key first, as the tutorial's chapter 00 makes them do.

## What Changes

- A left press on the sidebar's error marker `!` opens the error list, as the `errors.open` action does.
- A right press on the error marker clears the errors, as the `errors.clear` action does.
- Either press does nothing when the `gband.errors` plugin is not set up, or when no marker is drawn.
- The modal and direct presets bind `e` in `prefix` to a function described `list the errors`, which dispatches `errors.open` when it is registered.
- The modal and direct presets bind `E` in `prefix` to a function described `clear the errors`, which dispatches `errors.clear` when it is registered.
- The floating preset binds neither key.
- The README, `docs/plugins.md` and tutorial chapter 00 describe the new press and keys, and stop telling the user to bind `e` by hand.

## Capabilities

### New Capabilities

### Modified Capabilities
- `sidebar`: "Band click" names the error marker press, and a new "Error marker press" requirement defines it.
- `client-attach`: "Key bindings" adds the `e` and `E` rows to the modal key style's table, which the direct key style shares.
- `key-style`: "Key style presets" says that the `e` and `E` bindings resolve the error list's actions when pressed, so the presets still leave `gband.errors` to the default configuration.

## Impact

- `crates/lua/src/runtime/gband/sidebar.lua`: the `MousePressed` handler takes left and right presses on the marker row.
- `crates/lua/src/runtime/gband/keystyle/modal.lua` and `keystyle/direct.lua`: two new `prefix` bindings each.
- Lua tests under `tests/lua/` and screenshots of the key list, whose `prefix` list grows by 2 lines.
- `README.md`, `docs/plugins.md`, `docs/tutorial/00-setup.md` and the tutorial examples that bind `prefix e` by hand.
- A user configuration that already binds `prefix e` or `prefix E` keeps its own binding, since a later `gband.keymap.set` replaces the preset's.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- README.md
- crates/client/src/bindings.rs
- crates/lua/src/runtime/gband/keystyle/direct.lua
- crates/lua/src/runtime/gband/keystyle/modal.lua
- crates/lua/src/runtime/gband/sidebar.lua
- crates/lua/tests/config.rs
- crates/lua/tests/keystyle.rs
- crates/lua/tests/plugins.rs
- docs/internals/08-key-styles.md
- docs/internals/09-sidebar-and-errors.md
- docs/plugins.md
- docs/tutorial/00-setup.md
- examples/tutorial/00-setup/tests/setup_spec.lua
- examples/tutorial/00-setup/user/init.lua
- examples/tutorial/01-keys/user/init.lua
- examples/tutorial/02-actions/user/init.lua
- examples/tutorial/03-options/user/init.lua
- examples/tutorial/04-events/user/init.lua
- examples/tutorial/05-layout/user/init.lua
- examples/tutorial/06-colors/user/init.lua
- examples/tutorial/07-plugin-windows/user/init.lua
- examples/tutorial/08-bars/user/init.lua
- examples/tutorial/09-plugin/user/init.lua
- examples/tutorial/10-server/user/init.lua
- examples/tutorial/11-testing/user/init.lua
- tests/lua/keystyle_spec.lua
- tests/lua/prompt_spec.lua
- tests/lua/screenshots/keylist_spec/default-list--last-page.txt
- tests/lua/screenshots/prompt_spec/the-key-list-holds-the-prompt-between-the-key-list-and-detach--hint.txt
- tests/lua/screenshots/sidebar_spec/left-click-on-the-error-marker-opens-the-error-list--error-list-from-the-marker.txt
- tests/lua/sidebar_spec.lua
- openspec/changes/error-list-shortcuts/
