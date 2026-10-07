## 1. Deltas on the current specs

- [x] 1.1 Re-copy plugins' "Module lookup" and "Side guard" and configuration's "Configuration directory" from the main specs on `dev`. Re-apply this change's edits to each: the bundled module list with the prelude and the API modules, override and no host table, `core` on the client and the test side, and the sources under `defaults/`. Check the list against `bundled::MODULES` and `bundled::API` on `dev`. Verify with `openspec validate stable-lua-api --strict`, and by diffing each requirement against the main spec, which must show only these edits

## 2. `gband.core` in Rust

- [x] 2.1 Install the hooks of `ui::install` (`crates/lua/src/ui.rs`) into a new client table `gband.core` instead of `host`, under the names design.md gives. Make `on_state` hold a list, rename `error_item`'s counterpart to `error_marker` and `settings_reopen` to `reopen_settings`, and add argument checks with `ConfigError::raise`. During the transition, keep `host` as an alias of `gband.core` so that each Lua file can move on its own. Verify with a new `crates/lua/tests/core.rs` that covers "Owner inside a plugin", "Isolated call", "Wrong argument", "Timer" and "Several state functions", and with `cargo test -p gband-lua` passing unchanged
- [x] 2.2 Do the same for `plugin_windows::install`, `bars::install` and `removed::install`. Keep `focus_window` as `gband.core.focus_window`, which skips the layout check of `gband.window.focus`. The checks on frames, bar lists, slots and request entries name the primitive and the field. Verify the argument errors of each in `crates/lua/tests/core.rs`, and that `cargo test -p gband-lua` passes
- [x] 2.3 Add `gband.core.provide(kind, implementation)` for `windows`, `bars`, `settings` and `styles`, replacing `window_hooks`, `bar_hooks`, `settings_hooks` and `client_styles`, and clear every provider when a load starts. Verify "Unknown kind", that each kind with no provider leaves its feature absent, and that reloads with an open plugin window and a bar keep both drawn, in `crates/lua/tests/core.rs`, `crates/lua/tests/plugin_windows.rs` and `crates/lua/tests/bars.rs`
- [x] 2.4 Add `core` to the client-only fields and to the test side in `crates/lua/src/sides.rs`. Verify "Core primitives in the server" and the existing "Side guard" scenarios in `crates/lua/tests/plugins.rs`

## 3. Lua on the public API

- [x] 3.1 Switch `hl.lua`, `palette.lua` and `colorscheme.lua` from `host` to `gband.core`, and return their exports from each module's chunk. `hl.lua` exports `drawn`, `snapshot`, `restore`, `clear` and `quiet`, and `colorscheme.lua` exports `start`. Register the styles provider with `gband.core.provide("styles", ...)`. `colorscheme.lua` reads hl's exports through `require("gband.hl")`. Verify `cargo test -p gband-lua --test highlights --test palette --test colorschemes --test themes` passes unchanged
- [x] 3.2 Switch `bar.lua` and `win.lua` to `gband.core`, `require("gband.hl")` and `gband.core.focus_window`, registering their providers with `gband.core.provide`. Keep `win.lua`'s own argument checks ahead of `gband.core.focus_window`, so error texts are unchanged. Verify that `gband.win.focus` still focuses a tiled plugin window whose window the runtime's layout does not hold yet. Verify `cargo test -p gband-lua --test bars --test plugin_windows --test errors`, `cargo test --test plugin_windows --test mouse --test floating`, and the window example's cases in `cargo test --test lua_specs`, all passing unchanged
- [x] 3.3 Switch `settings.lua` to `gband.core.reopen_settings` and `gband.core.provide("settings", ...)`, and export `read` and `write`. Switch `keystyle.lua` to `require("gband.settings").read`. Verify `cargo test -p gband-lua --test settings --test keystyle` and `cargo test --test settings --test keystyle` pass. The only edits allowed are in assertions that read the runtime's private reopen state
- [x] 3.4 Switch `sidebar.lua` to `gband.core.state`, `gband.core.on_state`, `gband.core.error_marker` and `gband.core.loading`, and `prompt.lua` to `gband.core.call`. Verify `cargo test -p gband-lua --test sidebar --test prompt` and `cargo test --test sidebar --test prompt` pass. The only edit allowed is `error_item_shown` becoming its renamed counterpart
- [x] 3.5 Switch `crates/harness/src/test.lua` to `gband.core.register` and `gband.core.wrap`, which `crates/harness/src/runner.rs` installs on the test side in place of its `host` table. Verify `cargo test --test plugin_testing --test lua_specs` passes unchanged, together with the example plugins' cases it runs

## 4. Prelude and loader

- [x] 4.1 Add `crates/lua/src/runtime/gband/prelude.lua`, which requires the seven API modules in order and calls `require("gband.colorscheme").start()`. Merge `bundled::API` into `bundled::MODULES`, and replace the API loop and the `start_theme` call in `crates/lua/src/runtime.rs` with `require("gband.prelude")`. Verify "Defaults through the prelude", "API module replaced", "Broken prelude override", "API module is a module" and "Replaced bar implementation" in `crates/lua/tests/plugins.rs` and `crates/lua/tests/core.rs`
- [x] 4.2 Remove `host`. Delete the alias from 2.1, the `BIND` wrapper and the host parameter of `install_searcher` in `crates/lua/src/bundled.rs`, and the `host` construction in `runtime.rs`. Verify "Bundled chunk receives nothing private" and "Bundled chunk arguments" in `crates/lua/tests/plugins.rs`, no match from `rg -n '\bhost\b' crates/lua/src/runtime crates/harness/src/test.lua`, and `cargo test --workspace`

## 5. Bundled sources on disk

- [x] 5.1 In `crates/lua/src/directory.rs`, create `defaults/lua/gband` and `defaults/colors`. Write every `bundled::MODULES` entry other than the key style presets to `defaults/lua/gband/<path>`, and every `bundled::COLORS` entry to `defaults/colors/<name>.lua`, through `write_defaults`. Verify "Sources written", "First run" and "Edited defaults are restored" in the module's tests and `tests/config.rs`, and "Copies have no effect" and "Copy to override" in `tests/config.rs`

## 6. Tests that enforce the boundary

- [x] 6.1 Add `crates/lua/tests/api_docs.rs`, which walks the client and the server `gband` and the export tables of the prelude's modules with raw `next`. Add `crates/harness/tests/api_docs.rs`, which walks the test side's `gband.core` and `require("gband.test")`. Use the matching rule in design.md. Verify both pass once group 7 has filled the documentation, and that adding an undocumented field to `gband.core` makes the first one fail
- [x] 6.2 Add a `bundled_copies` field to the harness case in `crates/harness/src/case.rs`. It writes every bundled module and colorscheme into the case's `user/lua/` and `user/colors/` before start. Run the Lua spec suite again with it in `tests/lua_specs.rs`. Add "Copied sidebar", "Copied prompt" and "Copied theme" as named cases in a new `tests/lua/bundled_copies_spec.lua`, with references under `tests/lua/screenshots/bundled_copies_spec/`. Verify every case passes with the same screens

## 7. Documentation

- [x] 7.1 In `docs/plugins.md`, add three things:
  - a "Core primitives: `gband.core`" section with every primitive's arguments, return values, errors and effect, the frame, bar, slot and entry fields, and each provider kind with the functions the client calls and when;
  - the exports of `gband.hl`, `gband.settings` and `gband.colorscheme`, and the prelude and how to override a bundled module;
  - an "API and stability" section with the `gband.api_version` rule and an empty change list for version 1.

  Document every built-in action. Fill each gap that 6.1 reports. Verify `cargo test -p gband-lua --test api_docs`
- [x] 7.2 In `docs/testing.md`, document the test side's `gband.core.register` and `gband.core.wrap` and every field of `gband.test` that 6.1 reports as missing. Verify `cargo test -p gband-harness --test api_docs`
- [x] 7.3 In `README.md`, rewrite the Scripting section's account of the bundled Lua. Every bundled file, the API modules included, uses only documented API, and can be copied from `defaults/lua/gband/` or `defaults/colors/` into `user/` to change or replace it. Drop the caveat about the sidebar, the prompt and the settings window, and describe `defaults/lua` and `defaults/colors` in the Configuration section. Verify each path named exists after a fresh start in a scratch `XDG_CONFIG_HOME`

## 8. Final check

- [x] 8.1 Run `cargo test --workspace`, `cargo clippy --workspace --all-targets` and `openspec validate stable-lua-api --strict`. Confirm that `git diff --no-ext-diff --stat dev -- tests/lua/screenshots examples` shows no changed reference, and that `git diff --no-ext-diff --stat dev -- crates/lua/src/runtime` shows Lua edits limited to the `host` swaps, the exports, the providers and `prelude.lua`
