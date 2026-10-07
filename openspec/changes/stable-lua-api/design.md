## Context

See proposal.md for the motivation. The state this change starts from:

- `crates/lua/src/runtime.rs` builds a `host` table in the client only. It comes from `ui::install` (`crates/lua/src/ui.rs`), `plugin_windows::install`, `bars::install` and `removed::install`, plus `colorschemes` and `bundled_themes`. It then runs each entry of `bundled::API` (`crates/lua/src/bundled.rs`) with `host`: `hl.lua`, `palette.lua`, `colorscheme.lua`, `bar.lua`, `win.lua`, `settings.lua` and `keystyle.lua`. Each return value is stored in `package.loaded["gband.<name>"]`, and then `host.start_theme()` is called.
- The bundled searcher passes the same `host` to every module in `bundled::MODULES` through the `BIND` wrapper. Of those, `sidebar.lua` calls `host.state`, `host.on_state`, `host.error_item` and `host.loading`, and `prompt.lua` calls `host.call`. The key list, the error list, the key form, the theme builder, the presets and the themes never touch `host`.
- Lua modules also hand values to each other through `host`:
  - `hl.lua` writes `host.hl`, holding `drawn`, `snapshot`, `restore`, `clear` and the `quiet` flag, and registers `host.client_styles`.
  - `settings.lua` writes `host.setting`, holding `read` and `write`, and `keystyle.lua` reads it.
  - `colorscheme.lua` defines `host.start_theme`.
- `host.timer`, `host.cancel` and `host.events` exist in Rust, and timers fire from the client loop through `ui::fire_timers`, but no Lua calls them.
- `host.focus_window` queues the same `FocusWindow` view action as the public `gband.window.focus`, but without its check that the window is in the layout. The client can report a plugin window's drawn window before the runtime's layout holds it, so `gband.win.focus` needs the unchecked form.
- The test side gets its own `host` from `crates/harness/src/runner.rs`. It holds `register`, a Rust function, and `wrap`, which `test.lua` defines and Rust reads back.
- The Rust crate exposes the runtime's state to the client crate through methods such as `take_frames`, `take_bars`, `take_client_styles`, `take_palette`, `error_item_shown`, `take_settings_reopen` and `open_settings`. That interface sits between two Rust crates, not between Rust and Lua, and stays as it is.

## Goals / Non-Goals

**Goals:**
- No Lua file receives or reaches anything undocumented.
- Lua keeps every piece of logic it has today. The Rust diff is limited to renaming the install target, argument checks, multi-registration for `on_state`, `provide`, and the prelude call.
- Any bundled module, the prelude and the API modules included, can be replaced by a file in `user/lua/`.
- Every existing test passes without editing its expectations. The exceptions are tests that reach the private table or the runtime's private state.

**Non-Goals:**
- Making any primitive more general than it is today. Generalising one later is an addition, and additions do not raise `gband.api_version`.
- Moving existing Rust behaviour such as key maps, actions or options into Lua.
- Protecting gband from a broken override. A user who replaces `gband.win` owns the result, and a failing replacement fails the load, so the last good configuration stays.

## Decisions

### One table of primitives, `gband.core`
Everything that only exists for building API goes into `gband.core`. The user-facing Rust tables, such as `gband.layout`, `gband.view`, `gband.keymap`, `gband.action`, `gband.opt` and `gband.on`, stay where they are. A reader can tell the layers apart: `gband.core` is what the Lua API is built from, and the rest is what configuration normally uses.

Alternatives considered:
- *Spread the primitives into the user-facing tables*, for example `gband.win.present`. That mixes low-level and high-level functions in one table. It also makes replacing `gband.win` from Lua harder, because the replacement would have to keep Rust fields alive inside a table it rebuilds.
- *Keep the name `host`.* It is accurate, but the user chose `gband.core`.

### Each hook's place
| today | becomes | note |
|---|---|---|
| `owner`, `loading`, `failed`, `call`, `report`, `warn`, `load`, `bundled`, `colorschemes`, `bundled_themes`, `events`, `emit`, `after_event`, `state`, `timer`, `cancel`, `palette.set`, `palette.get`, `next_window`, `present_window`, `forget_window`, `request`, `parse_key`, `border`, `width`, `open_target`, `removed`, `place_bars`, `present_bars`, `dispatching` | `gband.core.<same name>` | same behaviour, plus argument checks |
| `on_state` | `gband.core.on_state` | holds a list instead of one slot, so more than one plugin can follow the state |
| `error_item` | `gband.core.error_marker` | renamed for what it records |
| `settings_reopen` | `gband.core.reopen_settings` | renamed to read as a verb |
| `window_hooks`, `bar_hooks`, `settings_hooks`, `client_styles` | `gband.core.provide("windows" \| "bars" \| "settings" \| "styles", …)` | one documented registration call |
| `focus_window` | `gband.core.focus_window` | not `gband.window.focus`, which rejects a window the layout does not hold yet; `win.lua` checks its own arguments first, so its error texts keep the `gband.win.focus` label |
| `host.hl` | exports `drawn`, `snapshot`, `restore`, `clear` and `quiet` of `require("gband.hl")` | written by `hl.lua`, read by `bar.lua`, `win.lua` and `colorscheme.lua` |
| `host.setting` | exports `read` and `write` of `require("gband.settings")` | also lets plugins save their own value files |
| `host.start_theme` | export `start` of `require("gband.colorscheme")` | called by the prelude |
| test `register`, `wrap` | test-side `gband.core.register` and `gband.core.wrap` | `wrap` registers the Lua wrapper Rust uses for raw driver functions |

The primitives with no caller stay and become public: Rust already pays for them, and the user wants Lua to be as capable as possible.

Alternatives considered:
- *Replace the sidebar's hooks with new high-level API*, such as an `ErrorsChanged` event and a per-bar error mark. It adds Rust for a need the existing primitives already meet. An earlier draft of this change took that route, and the user rejected it.

### Providers instead of single-slot hooks
`provide(kind, implementation)` names the protocol Rust calls into. It replaces four ad hoc registration functions. A provider is cleared on every reload, because the prelude registers it again as each load starts. Before this change, a hook registered once at startup lived for the whole process. Clearing on reload is what lets a `user/lua/gband/win.lua` added later take over at the next reload. A kind with no provider degrades to "feature absent", which is what an empty `Option` slot already does in `bars.rs`, `plugin_windows.rs` and `ui.rs`.

### Startup is a Lua module, `gband.prelude`
`runtime.rs` calls `require("gband.prelude")` where it now loops over `bundled::API` and calls `start_theme`. The prelude is a few lines of Lua: the seven `require`s in today's order, then `require("gband.colorscheme").start()`. Because `require` searches the runtimepath before the bundled modules, overrides work for every API module with no special case. `bundled::API` merges into `bundled::MODULES`.

The order stays `hl`, `palette`, `colorscheme`, `bar`, `win`, `settings`, `keystyle`, because the modules call into each other at load time. `start` runs after `settings` has loaded, as `start_theme` does today.

### Argument checks are the only new Rust logic
Today the primitives trust their callers, which were all gband's own Lua. Once public, each one checks its arguments with the existing `ConfigError::raise` pattern and reports at the caller's line. A bad call is then a configuration error, never a panic or a confusing `mlua` conversion error. The structured inputs, the window frames, bar lists, slots and request entries, get their documented fields checked by the readers that already exist (`read_frame`, `read_bar`, `read_slot`, `request`). These now produce messages that name the primitive and the field.

### Bundled sources are written from the tables the loader reads
`directory.rs` writes `defaults/lua/gband/<path>` for every entry of `bundled::MODULES`, now including the prelude and the API modules but not the key style presets. It writes `defaults/colors/<name>.lua` for every entry of `bundled::COLORS`. Both use the same atomic `write_defaults` as `defaults/init.lua`. The presets stay at `defaults/keystyle/`, the path the README and the key-style specification already document. `defaults` stays off the runtimepath.

### Tests enforce the boundary
- **Documentation test** (`crates/lua/tests/api_docs.rs`). It builds a client and a server state with no configuration. It walks `gband` and the export tables of the prelude's modules with raw `next`, skipping metatables, and collects every path to a function or table. A path counts as documented when `docs/plugins.md` contains it in full, or when its last segment appears in backticks inside the section whose heading names its parent, as the action tables list `focus_column_left`. `crates/harness/tests/api_docs.rs` does the same for the test side's `gband.core` and `gband.test` against `docs/testing.md`.
- **Arguments test.** It requires each bundled module through a probe that records `...`, and compares the result with a runtimepath module.
- **Copy suite** (`tests/lua_specs.rs`). A new `bundled_copies` field on the harness case writes every bundled module and colorscheme into the case's `user/lua/` and `user/colors/`. The suite runs once more with it set, and the screens must match.

### The version stays 1
Everything this change adds is new documented API. What it removes, `host` and the `BIND` argument, was never documented. So under the new rule `gband.api_version` stays 1.

## Risks / Trade-offs

- [Freezing presentation primitives such as frames, bar lists and the styles provider means a later renderer change that alters them raises `gband.api_version`] → This is accepted: it is the price of letting Lua own the drawing logic. Keep the documented fields to what exists, and prefer additive fields when the renderer grows.
- [A user override of an API module can break the client] → A failing override fails the load, and the configuration capability's "Last good configuration" applies, as with any init file error. The error names the override's path, so the cause is visible.
- [`provide` cleared on reload changes when hooks exist during a reload] → The prelude registers the providers before the init file, as early as the bundled modules registered them before. Tests of reloads with open plugin windows and bars guard the transition.
- [Documented primitives invite uses gband's own Lua never made, which exposes untested paths] → Argument checks reject malformed input, and each primitive gets at least one direct test.
- [The documentation test is too strict or too loose] → The matching rule is simple and lives in one file. Fields that should not be public get removed from `gband`, not allow-listed.
- [The copy suite doubles the Lua spec run time] → It runs the same cases with another file layout. It can be narrowed to the example cases if run time matters.

## Migration Plan

No action is needed from users. Documented API keeps working, and `gband.api_version` stays 1. Rollback is a revert of the change's commits. No protocol message, file format or saved setting changes.
