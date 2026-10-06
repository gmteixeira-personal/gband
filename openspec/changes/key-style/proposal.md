## Why

navigation-mode makes the leader enter a mode where layout keys repeat until Escape. Users who come from tmux expect the leader, one key, and typing again. Neither style suits everyone, and a new user should pick one on the first start without writing Lua. Each style should live in its own Lua file, so a user can read exactly what a style binds and copy it into a configuration.

## What Changes

- **Two key style presets**, each a bundled Lua file of plain top-level calls that reads as an example:
  - `gband.keystyle.modal`: navigation mode. It declares the `prefix` mode labelled `navigation` and makes the default bindings that navigation-mode put in the default configuration.
  - `gband.keystyle.direct`: the leader, then one key per action, then back to typing. It binds the same keys in `prefix` with no mode and no Escape or Enter binding. `n` opens a window and Ctrl+Space sends the prefix key, both as plain actions.
  - Both set up the bundled plugins whose actions they bind, `gband.keylist` and `gband.prompt`. Both bind `?` to the key list and `:` to the Lua prompt, and bind every action they give to `h`, `j`, `k` or `l` to the matching arrow key too.
- **`gband.keystyle`**, a new client API:
  - `gband.keystyle.use(style)`, while the configuration loads, makes the bindings of `"modal"` or `"direct"` and returns the style's name. With no argument it uses the saved style, or modal when none is saved. It runs once per load. A user's own `init.lua` can call it.
  - `gband.keystyle.saved()` returns the saved style, or nil.
  - `gband.keystyle.choose()`, in a callback, returns to `root` and opens the key style chooser: a floating plugin window with one line per style and a cursor line. j, k and the arrow keys move. Enter saves the style. Escape and `q` close it and save nothing. Users can call it again later, for example from the `:` Lua prompt.
- **Saved choice**: `user/keystyle.lua` holds `return "direct"` or `return "modal"`.
  - Saving replaces the file in one step. Its name ends in `.lua`, so saving reloads the configuration, which applies the new style.
  - Nothing ever writes `user/init.lua`.
  - A file that does not return a style name reads as no saved style.
  - When saving fails, the client shows an error and the style in use stays.
- **`gband.config_dir`**: the configuration directory's path on the client and the server, or nil when there is none.
- **Default configuration**:
  - It calls `gband.keystyle.use()` instead of binding keys itself, and no longer sets up `gband.keylist` or `gband.prompt`.
  - On `Attached`, it opens the chooser when a configuration directory exists and no style is saved. `Attached` runs once per client and never after a reload, so the offer appears once per start.
  - Dismissing the chooser keeps modal for the session, and the next start offers it again.
- **Defaults directory**: gband writes the two presets to `defaults/keystyle/modal.lua` and `defaults/keystyle/direct.lua`, beside `defaults/init.lua`, for the user to read and copy.
- **Tests**: every `gband test` case saves the modal style unless `g.start` says otherwise, so a case with the default configuration starts with no chooser open. `g.start` takes `keystyle`, which is `"modal"`, `"direct"` or `false`.

Out of scope:
- Swapping bindings without a reload, and a style per client or per session.
- More styles, and choosing other preferences, such as the prefix key or a colorscheme, in the chooser.
- A default key binding for `gband.keystyle.choose()`.
- Changing an existing `user/init.lua`. A configuration that does not call `gband.keystyle.use()` keeps its own bindings, and `choose()` then only saves the choice.

## Capabilities

### New Capabilities

- `key-style`: the two presets, `gband.keystyle.use`, `saved` and `choose`, the saved choice file, the chooser, and the offer on first start.

### Modified Capabilities

- `configuration`: the default configuration binds through `gband.keystyle.use()` and offers the chooser. The presets are written to `defaults/keystyle/`. `gband.config_dir` is added.
- `client-attach`: the default bindings depend on the key style, with a direct table beside navigation mode's.
- `plugins`: `keystyle` joins the client-only fields of the side guard.
- `plugin-testing`: a case saves a key style, modal by default, and `g.start` takes `keystyle`.
- `lua-prompt`: the presets, not the default configuration, set up `gband.prompt` and bind `prefix :`.

## Impact

- `crates/lua/src/runtime/gband/keystyle.lua`, a new client API file, and `crates/lua/src/runtime/gband/keystyle/modal.lua` and `direct.lua`, two new bundled modules, listed in `crates/lua/src/bundled.rs`.
- `crates/lua/src/defaults.lua`: the bindings and the `gband.keylist` and `gband.prompt` setups move into the presets, and the `Attached` offer is added.
- `crates/lua/src/runtime.rs`: `gband.config_dir`. `crates/lua/src/sides.rs`: `keystyle` is client only.
- `crates/lua/src/directory.rs` and `lib.rs`: the presets are written to `defaults/keystyle/`.
- `crates/client/src/bindings.rs`: the default key table tests cover both styles.
- `crates/harness/src/case.rs` and `runner.rs`: the `keystyle` option. `tests/common/mod.rs`: end-to-end tests save the modal style unless they ask for none.
- Tests in `crates/lua/tests/`, `tests/` and `tests/lua/`.
- `README.md`, `docs/plugins.md` and `docs/testing.md`.
- No protocol change and no new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- navigation-mode
- lua-prompt

### Expected Files
- openspec/changes/key-style/
- crates/lua/src/runtime.rs
- crates/lua/src/sides.rs
- crates/lua/src/bundled.rs
- crates/lua/src/lib.rs
- crates/lua/src/directory.rs
- crates/lua/src/defaults.lua
- crates/lua/src/runtime/gband/keystyle.lua
- crates/lua/src/runtime/gband/keystyle/
- crates/lua/tests/keystyle.rs
- crates/lua/tests/config.rs
- crates/client/src/bindings.rs
- crates/harness/src/case.rs
- crates/harness/src/runner.rs
- tests/common/mod.rs
- tests/config.rs
- tests/keystyle.rs
- tests/plugin_testing.rs
- tests/lua/keystyle_spec.lua
- tests/lua/screenshots/keystyle_spec/
- README.md
- docs/plugins.md
- docs/testing.md
