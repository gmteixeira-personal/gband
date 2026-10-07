## Why

gband's Lua API has two layers today. Configuration and plugins see the documented `gband` table. Every Lua file bundled with gband also gets a private `host` table, through the bundled module loader in `crates/lua/src/bundled.rs`. The sidebar, the Lua prompt and the API modules call it: `gband.win`, `gband.bar`, `gband.hl`, `gband.colorscheme`, `gband.palette`, `gband.settings` and `gband.keystyle`. The test module `gband.test` gets a `host` table of its own. A copy of one of these files in `user/lua/` gets no `host` and breaks. So the bundled Lua cannot serve as the examples the README says it is. A script author can only learn what it relies on by reading gband's Rust source, and can never replace it.

gband should keep its logic in Lua and make Lua as powerful as possible, with as little Rust as possible. Rust exposes a small, stable, documented set of primitives. Every Lua file, bundled or the user's own, uses only documented API. Then anyone can script gband, or replace any bundled part of it, from the documentation and the bundled sources alone.

## What Changes

- **No Lua logic moves into Rust.** The API modules `gband.hl`, `gband.palette`, `gband.colorscheme`, `gband.bar`, `gband.win`, `gband.settings` and `gband.keystyle`, the bundled plugins, the key style presets, the themes and `gband.test` stay Lua.
- **`gband.core`: the private hooks become public primitives.** Every function of today's `host` table becomes a documented, stable function in the new client table `gband.core`. Each one keeps its current behaviour, and gains argument checks and a documented name. Examples: the current plugin, isolated calls with an owner, error reports, loading a file, emitting a built-in event, the view state, timers, the terminal palette, presenting plugin window frames and bars, placing bars, the error marker flag, key name parsing, and the border and width validators. Three kinds of hook are not carried over:
  - A hook that duplicates existing public API is dropped, and its callers use that API: `focus_window` becomes `gband.window.focus`.
  - Hooks with no caller, `timer`, `cancel` and `events`, become public rather than being deleted, because Rust already has them.
  - The single-slot registrations, `window_hooks`, `bar_hooks`, `settings_hooks` and `client_styles`, become one documented call, `gband.core.provide(kind, implementation)`. A user's implementation can replace the bundled one for plugin windows, bars, the settings window or the drawn styles.
- **The hand-offs between Lua modules become documented module exports**, in place of fields the modules wrote into `host`: `require("gband.hl")` exports `drawn`, `snapshot`, `restore`, `quiet` and `clear`; `require("gband.settings")` exports `read` and `write`; and `require("gband.colorscheme")` exports `start`.
- **The Lua prelude.** Before the init file, the client requires one bundled module, `gband.prelude`. It requires the API modules in order and loads the start theme. Rust does nothing more. Every module is found through the ordinary module search, so a copy in `user/lua/gband/` replaces any bundled module, API modules and the prelude included.
- **The test side** gets `gband.core.register` and `gband.core.wrap`, and `gband.test` uses only those.
- **The bundled sources ship with gband.** Each process writes every bundled module, the prelude, the API modules and the plugins to `defaults/lua/gband/`, and every theme to `defaults/colors/`. The layout mirrors a runtimepath entry, next to the files it already writes there.
- **A stability rule.** The documented API is the whole public surface: the Rust tables, `gband.core`, the Lua API modules and their exports. A release that removes something documented, or changes what it does, raises `gband.api_version`. Additions do not. This change only adds documented API and removes the undocumented `host`, so `gband.api_version` stays `1`.
- **Enforced by tests.** Three tests guard the boundary:
  - One fails when a field of `gband`, `gband.core` or a documented module export lacks documentation.
  - One checks that no chunk receives an argument a runtimepath module would not.
  - One runs the Lua spec suite again with every bundled file copied into `user/`.

Out of scope:
- Moving Rust code that already exists into Lua. A later change can do that, built on `gband.core`.
- Changing what any primitive does. This change names, checks and documents what exists.
- Type annotation files for Lua language servers.
- Removing stale files from `defaults/` that an older build wrote.

## Capabilities

### New Capabilities
- `lua-api`: the boundary between the executable and Lua. It covers the primitives in `gband.core` and `gband.core.provide`, the prelude, the bundled Lua as consumers that work as copies, the complete documentation, the bundled sources in `defaults/`, the stability rule for `gband.api_version`, and the tests that enforce all of this.

### Modified Capabilities
- `plugins`: "Module lookup" passes no host table, lists the API modules and `gband.prelude` among the bundled modules, and lets a runtimepath module override any of them. "Side guard" adds `core` to the client and the test side.
- `configuration`: "Configuration directory" also writes the bundled modules and themes under `defaults/lua/gband/` and `defaults/colors/`.

## Impact

- Lua crate, Rust side:
  - `crates/lua/src/ui.rs`, `bars.rs`, `plugin_windows.rs` and `removed.rs` install their functions into `gband.core` instead of `host`, with argument checks.
  - `crates/lua/src/runtime.rs` replaces the `bundled::API` loop and `start_theme` with `require("gband.prelude")`.
  - `crates/lua/src/bundled.rs` drops `BIND` and the host parameter.
  - `crates/lua/src/directory.rs` writes the sources to `defaults/`, and `sides.rs` guards `core`.
- Lua crate, Lua side: every file under `crates/lua/src/runtime/gband/` that calls `host` switches to `gband.core`, to existing public API or to a module export. The new `prelude.lua` holds the load order.
- Harness: `crates/harness/src/runner.rs` installs `gband.core.register` and `gband.core.wrap`, and `crates/harness/src/test.lua` uses them.
- Tests: existing tests pass unchanged, apart from tests that reach into the runtime's private state. New tests cover the documentation, the chunk arguments, the copies, `gband.core` argument checks, `provide`, the prelude override and the files under `defaults/`.
- Docs: `docs/plugins.md` gains a "Core primitives: `gband.core`" section, the module exports, and an "API and stability" section. `docs/testing.md` documents the test-side primitives. The README's Scripting section says every bundled file can be copied and replaced.
- No protocol change and no new dependency. The Rust added is argument checks and a renamed install table.

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
