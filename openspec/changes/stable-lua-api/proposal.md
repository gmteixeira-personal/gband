## Why

gband's Lua API has two layers today. Configuration and plugins see the documented `gband` table. Every Lua file bundled with gband also gets a private `host` table, through the bundled module loader in `crates/lua/src/bundled.rs`. The sidebar, the Lua prompt, the settings window and the API modules `gband.win`, `gband.bar`, `gband.hl`, `gband.colorscheme`, `gband.palette`, `gband.settings` and `gband.keystyle` all call it. The test module `gband.test` gets a `host` table of its own as well. A copy of one of these files in `user/lua/` gets no `host` and breaks. So the bundled features cannot serve as the examples the README says they are, and a script author can only find out what a bundled feature relies on by reading gband's source.

gband should have one boundary. The executable, written in Rust, provides a stable, documented API. Every Lua file, bundled or the user's own, consumes only that API. Then anyone can script gband from the documentation and the bundled examples alone, without reading the Rust code or having the source.

## What Changes

- **One API, provided by the executable.** Every function, table and event a Lua script can use is part of `gband`, the `require`-able test module `gband.test`, or Lua's own standard library. Each is documented in `docs/plugins.md` or `docs/testing.md`. Nothing else is reachable: no private table, no hidden chunk argument, no undocumented field.
- **The `host` table is removed.** Bundled chunks are loaded like any other module and get no private argument. The API modules written in Lua today move into Rust, behind the same documented functions with the same behaviour: `gband.hl`, `gband.palette`, `gband.colorscheme`, `gband.bar`, `gband.win`, the stored-settings part of `gband.settings`, `gband.keystyle.use` and `gband.keystyle.saved`, and the test module `gband.test`. Their modules stop being `require`-able: `require("gband.win")` and the like stop working. They were never documented as modules.
- **Bundled features are API consumers.** The sidebar, the key list, the error list, the Lua prompt and rename box, the settings window and theme list, the two key style presets, the theme builder `gband.theme`, the key form `gband.keyform` and every bundled theme use only the documented API. Each one works unchanged when copied into a runtimepath entry. The settings window moves out of the `gband.settings` API into a bundled module of its own, `gband.settings.window`.
- **New public API**, for what the bundled features got from `host`:
  - `gband.eval(source, name)` compiles and runs one Lua source text as code of the configuration file would run, with its own instruction budget. It reports failures as configuration errors at `name` and the line. The Lua prompt runs its line with it.
  - The client event `ErrorsChanged` fires when the client's error list changes. The sidebar redraws its marker on it.
  - `gband.bar.mark_errors(id, shown)` says that a bar's lines show the error marker. While a shown bar marks errors, the client draws no error banner. The sidebar uses it in place of a private flag.
  - `gband.settings.save(name, value, opts)` saves one setting file in one step, and `opts.reopen` names the settings window line to reopen once the reload it causes succeeds. `gband.settings.open(opts)` takes that line as `opts.line`, and opens the window by requiring `gband.settings.window`.
- **The bundled sources ship with gband.** Each process writes every bundled Lua module, plugin and theme to `defaults/lua/gband/` and `defaults/colors/`, mirroring a runtimepath entry, as it already writes `defaults/init.lua` and the key style presets. Users can read them without the source, and copy one into `user/` to change it.
- **A stability rule.** The documented API is the whole public surface. A release that removes something documented, or changes what it does, raises `gband.api_version`. Additions do not. This change adds API and removes only undocumented internals, so `gband.api_version` stays `1`.
- **Enforced by tests.** One test fails when the client or server `gband` table holds a field that `docs/plugins.md` does not document. Another loads each bundled Lua file as a copy from `user/lua/` or `user/colors/` and checks that it behaves as the bundled one does. A third checks that no bundled chunk receives an argument.

Out of scope:
- New features in the API modules being ported. Their behaviour, error messages and error lines stay as their capabilities define them.
- Type annotation files for Lua language servers. A later change can generate them from the same documentation.
- Removing stale files from `defaults/` that an older build wrote.

## Capabilities

### New Capabilities
- `lua-api`: the boundary between the executable and Lua: the API the executable provides, documented in full, with nothing private; bundled Lua as consumers that work as copies; the bundled sources written to `defaults/`; the stability rule for `gband.api_version`; and the tests that enforce all of this.

### Modified Capabilities
- `plugins`: "Module lookup" drops the host table and lists the bundled modules after the change: the consumer modules only, adding `gband.theme`, `gband.theme.catppuccin` and `gband.settings.window`. A new "Evaluate source" requirement defines `gband.eval`.
- `configuration`: "Configuration directory" also writes the bundled modules and themes under `defaults/lua/gband/` and `defaults/colors/`. "Reporting configuration errors" hides the banner while any bar's error marker is drawn, not only the sidebar's.
- `lua-events`: "Built-in events" gains `ErrorsChanged`.
- `bars`: a new "Error marker bars" requirement defines `gband.bar.mark_errors`, and "Bars and reloads" clears the mark on a reload.
- `sidebar`: "Error marker" marks the sidebar's errors through `gband.bar.mark_errors` and redraws on `ErrorsChanged`.
- `lua-prompt`: "Running the line" runs the line with `gband.eval`.
- `settings`: a new "Save a setting" requirement defines `gband.settings.save`. "Settings window" places the window in the bundled module `gband.settings.window`, opened by `gband.settings.open`. "Reopen after a save" is driven by `opts.reopen`.

## Impact

- Lua crate: `crates/lua/src/bundled.rs` and `crates/lua/src/runtime.rs` lose the host table. `ui.rs`, `bars.rs`, `plugin_windows.rs` and `removed.rs` drop their host hooks, and `guard.rs`, `events.rs` and `directory.rs` gain the new API. New Rust modules implement highlights, the palette, colorschemes, bars, plugin windows, settings and key styles.
- Removed Lua: `crates/lua/src/runtime/gband/hl.lua`, `palette.lua`, `colorscheme.lua`, `bar.lua`, `win.lua` and `keystyle.lua`. `settings.lua` becomes `settings/window.lua`.
- Changed Lua: `sidebar.lua` and `prompt.lua` use only the public API.
- Harness: `crates/harness/src/test.lua` moves into `crates/harness/src/runner.rs`, or a module next to it.
- Tests: the existing tests of each ported module must pass unchanged. That is the evidence that behaviour did not change. New tests cover the documentation, the copies, the chunk arguments, `gband.eval`, `ErrorsChanged`, `mark_errors`, `settings.save` and the files under `defaults/`.
- Docs: `docs/plugins.md` gains an "API and stability" section and the new functions and event. `docs/testing.md` documents `gband.test` in full. The README's Scripting section drops the caveat that some bundled features cannot be copied.
- No protocol change and no new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- openspec/changes/stable-lua-api/
- README.md
- crates/client/src/lib.rs
- crates/harness/src/case.rs
- crates/harness/src/lib.rs
- crates/harness/src/runner.rs
- crates/harness/src/test.lua
- crates/harness/tests/api_docs.rs
- crates/lua/src/
- crates/lua/tests/
- docs/plugins.md
- docs/testing.md
- tests/config.rs
- tests/lua/bundled_copies_spec.lua
- tests/lua/screenshots/bundled_copies_spec/
- tests/lua_specs.rs
- tests/settings.rs
