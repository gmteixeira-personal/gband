## Why

window-names draws each window's name on its top border. Every screen reference with a tile then shows `sh` on its top border, so window-names regenerates every reference under `tests/lua/screenshots/`. Any later change to naming or title drawing would touch them all again, and a plugin author's `gband test` references would carry titles that their plugin never draws. Screen tests should show titles only where a test is about titles, and users who prefer bare borders should be able to turn them off.

## What Changes

- **Option**: a client option `window_titles`, a boolean, `true` by default. While it is `false`, the client draws no border title. Names are still kept, sent, read by Lua and renamed.
- **Environment**: the client reads `GBAND_WINDOW_TITLES` when it starts, as it reads `GBAND_ANIMATIONS`. `off` turns titles off whatever the option holds. Unset or `on` leaves the option in charge. Any other value is logged as a warning and read as unset.
- **`gband test` cases**: `g.start` takes `window_titles`, `false` by default. The runner sets `GBAND_WINDOW_TITLES=off` unless it is `true`. A case about titles asks for them.
- **gband's own end-to-end tests**: the harness environment sets `GBAND_WINDOW_TITLES=off` too, with a way for a test to remove it.
- **References**:
  - Every screen reference under `tests/lua/screenshots/` and `examples/plugins/*/tests/screenshots/` goes back to drawing no title. Each one matches its content before window-names, except the key list's `N` line.
  - `window_names_spec.lua` and `tests/window_names.rs` ask for titles and keep them.

Out of scope:
- Title highlight groups, title placement options, and turning off names themselves.
- Hiding titles per window or per band.

## Capabilities

### New Capabilities

### Modified Capabilities
- `configuration`: the client option `window_titles` in the options table.
- `window-names`: titles turned off by the option or by `GBAND_WINDOW_TITLES=off`.
- `plugin-testing`: `g.start`'s `window_titles` field and `GBAND_WINDOW_TITLES=off` in the case environment.

## Impact

- Lua options: `crates/lua/src/options.rs` declares `window_titles`, tested in `crates/lua/tests/options.rs`.
- Client: `crates/client/src/render.rs` skips the title when titles are off. The environment variable is read at start in `crates/client/src/lib.rs`, as `crates/client/src/animation.rs` reads `GBAND_ANIMATIONS`.
- Harness: `crates/harness/src/case.rs` (the `window_titles` field and the variable) and `crates/harness/src/env.rs` (the end-to-end environment).
- Tests: `tests/window_names.rs` and `tests/lua/window_names_spec.lua` opt in. The end-to-end assertions window-names changed for titles are restored. The references are regenerated.
- `README.md`, `docs/plugins.md` and `docs/testing.md`.
- No protocol change and no new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- window-names

### Expected Files
- openspec/changes/window-titles-option/
- crates/lua/src/options.rs
- crates/lua/tests/options.rs
- crates/client/src/lib.rs
- crates/client/src/render.rs
- crates/harness/src/case.rs
- crates/harness/src/env.rs
- tests/window_names.rs
- tests/lua/window_names_spec.lua
- tests/lua/screenshots/
- examples/plugins/agent-status/tests/screenshots/
- examples/plugins/hello/tests/screenshots/
- examples/plugins/window/tests/screenshots/
- README.md
- docs/plugins.md
- docs/testing.md
