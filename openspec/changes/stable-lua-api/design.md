## Context

See proposal.md for the motivation. The state this change starts from:

- `crates/lua/src/runtime.rs` builds a `host` table in the client only, from `ui::install` (`crates/lua/src/ui.rs`), `plugin_windows::install`, `bars::install` and `removed::install`, plus `colorschemes` and `bundled_themes`. It runs each entry of `bundled::API` (`crates/lua/src/bundled.rs`): `hl.lua`, `palette.lua`, `colorscheme.lua`, `bar.lua`, `win.lua`, `settings.lua` and `keystyle.lua`. Each one is called with `host`, and its return value is stored in `package.loaded["gband.<name>"]`. Then it calls `host.start_theme()`.
- The bundled searcher passes the same `host` to every module in `bundled::MODULES` through the `BIND` wrapper. `sidebar.lua` calls `host.state`, `host.on_state`, `host.error_item` and `host.loading`. `prompt.lua` calls `host.call`. The key list, the error list, the key form, the theme builder, the presets and the themes call nothing on `host`.
- The API modules also talk to each other through `host`. `hl.lua` publishes `host.hl`, which holds `drawn`, `snapshot`, `restore`, `quiet` and `clear`, and registers `host.client_styles`. `settings.lua` publishes `host.setting`, and `keystyle.lua` reads it. `colorscheme.lua` defines `host.start_theme`.
- `host.timer`, `host.cancel` and `host.events` have no Lua caller.
- The server and the test side get no `host`. Their `gband` comes from Rust alone. `gband.test` is the exception: `crates/harness/src/test.lua` gets a `host` table of its own from `crates/harness/src/runner.rs`, holding `register` and `wrap`.
- Rust already holds most of what these hooks expose: `ui::current_state` and `set_state`, the `guard::isolated` runner, `owner::current`, the presented bars and plugin window frames, and the palette and look state.
- Existing tests pin the observable behaviour of every module this change ports. They are `crates/lua/tests/{highlights,palette,colorschemes,themes,bars,plugin_windows,settings,keystyle,sidebar,prompt,errors}.rs`, the end-to-end tests under `tests/`, and the screen cases under `tests/lua/` and `examples/plugins/*/tests/`.

## Goals / Non-Goals

**Goals:**
- After this change, no Rust function is reachable from Lua except through a documented API path.
- Every existing test passes without editing its expectations. The only exceptions are tests that `require` an API module by name, or that assert on `host`.
- Each step of the work leaves the build and every test green, so the change can be stopped and resumed at any step.

**Non-Goals:**
- Reworking how plugin windows, bars or highlight groups behave. The port moves code across the boundary and keeps its rules.
- A Lua language server meta file, and API reference pages generated from code.
- Making the server or the test side load bundled consumer modules they do not load today.

## Decisions

### The boundary is the executable, and the API is implemented in Rust
Every function under `gband`, and every function of `gband.test`, is a Rust function installed by the executable. The Lua files that gband bundles are all consumers.

Alternatives considered:
- *Keep the API modules in Lua, but make them private.* They would not be `require`-able and not written to disk. This is the smallest change, and it hides `host` from users. But gband would still ship Lua that runs on a private channel, which is the situation the user asked to end. Every future API module would face the same temptation. It also keeps per-frame Lua-to-Rust traffic such as `client_styles`.
- *Promote the `host` hooks to documented low-level API, and keep the modules in Lua on top.* This turns presentation internals into a stable contract: frames, bar placement, drawn styles and window counters. Every later change to the renderer would then raise `gband.api_version`.

### Port the modules one at a time, in dependency order, with the old tests as the oracle
The order is `palette`, `hl`, `colorscheme`, `bar`, `win`, `keystyle`, the stored settings, then `gband.test`. Each port lands with its module's tests unchanged and green before the next one starts.

- Rust raises argument errors with the existing `ConfigError::raise` pattern. It reports at the caller's line, as the Lua `error(message, 2)` did. Messages are copied word for word, because tests assert on them.
- Callbacks into plugin code, such as `on_resize`, plugin window `keys`, `on_close` and `on_mouse`, run through `guard::isolated` with the bar's or window's owner, as `host.call` did.
- Lua-specific checks keep their exact meaning. Integer checks follow `math.type`, so `2.0` stays rejected where `is_integer` rejected it. Lists stay copied where the Lua copied them.
- `hl` in Rust removes the `client_styles` callback. The renderer reads the resolved groups directly.

The ported Lua files are deleted in the same step as each port, so the code never exists twice.

### New public API only where a bundled consumer needs it
The audit of `host` callers in consumer modules finds four needs. Each gets the smallest public form that is useful beyond its first consumer:

- **`gband.eval(source, name)`, for `host.call` in the prompt.** It takes source text, not a function. Source text that belongs to no plugin matches how the configuration file runs, and the prompt specification already requires that. A general `gband.isolate(fn)` would let any plugin run its own closures as code of no plugin, which blurs who owns a registered name. It would also need rules for upvalues that cross the budget boundary.
- **`ErrorsChanged`, for `host.on_state` and the error field of `host.state`.** The other inputs of `on_state` already have public events: `KeyTableChanged`, `BandChanged` and `LayoutChanged`. The payload carries the list, so a handler needs no second call. The event is emitted at most once per unit of handling, the same moment `set_state` compares states today. A failing handler does not emit it again.
- **`gband.bar.mark_errors(id, shown)`, for `host.error_item`.** The mark belongs to a bar, not to the client. A hidden or removed bar stops counting without the plugin noticing, as the sidebar specification requires today. A global switch such as `gband.ui.error_banner(false)` would make every plugin track its own visibility, and two marking bars would fight over it.
- **`gband.settings.save` with `opts.reopen`, and `gband.settings.open(opts)`, for `host.setting`, `host.settings_reopen` and `host.settings_hooks`.** Saving files and reopening after a reload are Rust's job: Rust owns the reload. The window is a consumer. `open` finds it with an ordinary `require("gband.settings.window")`, so a user's copy replaces it. The alternative was a general store that survives a reload. It was rejected for now: it has one consumer, and it can be added later without breaking anything.

`host.loading` gets no public form. The sidebar's `setup` only skipped drawing while loading. A bar added while loading gets its first `on_resize` after the load ends, as the bars specification defines, so the sidebar draws from `on_resize` and from events alone.

### The settings window becomes `gband.settings.window`
The window and theme list code of `settings.lua` moves to `crates/lua/src/runtime/gband/settings/window.lua`. It is a module that returns `{ open = function(opts) ... end }`, not a plugin with `setup`, because it registers nothing and is opened on demand. `SettingsLabel` keeps its default in Rust's defaults layer, as the "Settings group" requirement asks. It is defined before the init file runs, whether or not the window module was ever required. The naming follows the `gband.keystyle.modal` precedent of a bundled submodule under an API name.

### Bundled sources are written from the same tables the loader reads
`directory.rs` writes `defaults/lua/gband/<path>` for every entry of `bundled::MODULES` except the key style presets, and `defaults/colors/<name>.lua` for every entry of `bundled::COLORS`. It writes them in the same atomic way as `defaults/init.lua`. The presets stay at `defaults/keystyle/`, the path the README and key-style specification already document, so no file is written twice. `defaults` stays off the runtimepath, so the copies never load.

### Tests enforce the boundary
- **Documentation test** (`crates/lua/tests/api_docs.rs`). It builds a client and a server state with no configuration and walks `gband` with raw `next`. It skips metatables, so proxy tables such as `gband.opt` contribute only their raw fields. It collects every path to a function or table. A path counts as documented when `docs/plugins.md` contains it in full, or when its last segment appears in backticks inside the section whose heading names its parent, as the action tables list `focus_column_left`. A second test walks `require("gband.test")` against `docs/testing.md`. Writing the missing entries is part of the work: every built-in action goes into `docs/plugins.md`.
- **Arguments test.** It requires each bundled module through a probe that records `...`, and checks the result against a runtimepath module.
- **Copy suite** (`tests/lua_specs.rs`). It runs the Lua spec suite a second time with every bundled module and colorscheme written into each case's `user/lua/` and `user/colors/`, through a new field on the harness case. The screens must match the first run.

### The version stays 1
This change adds documented API and removes only paths no documentation described: the `require`-able API modules and `host`. Under the rule it introduces, `gband.api_version` stays 1. The "API and stability" section of `docs/plugins.md` states the rule and starts the change list empty.

## Risks / Trade-offs

- [A port changes behaviour in a way no test covers] → Port one module per step, with no edits to existing expectations. Review each step by reading the deleted Lua next to the new Rust. Add a test before porting any behaviour that no test covers yet.
- [The Rust crate grows by roughly 2,000 lines] → Accepted. The code gets direct access to client state, and per-frame calls from Lua into Rust go away.
- [An error message or error line drifts in the port] → Tests assert exact texts and lines. Each Rust raise mirrors the Lua `error(..., 2)` level.
- [Users who called `require("gband.win")` or a similar API module break] → Those modules were never documented. The README and the guide only ever showed the `gband.*` paths. The breakage is noted in the plugin guide's "API and stability" section.
- [The documentation test is too strict or too loose] → The matching rule is simple and lives in one test file. Paths that should not be public get removed from `gband` rather than allow-listed.
- [Stale files remain in `defaults/` after a module is renamed in a later build] → Accepted. They never load, and the README says edits to `defaults/` have no effect.
- [The copy suite doubles the Lua spec run time] → It runs the same cases with a different file layout. If time matters, it can be limited to the cases of the features that ship as examples.

## Migration Plan

No action is needed from users. A configuration or plugin that uses only documented API keeps working, and `gband.api_version` stays 1. Rollback is a revert of the change's commits. No protocol message, file format or saved setting changes.
