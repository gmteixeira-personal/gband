## Why

`bootstrap-rust-crate` gives gband one binary crate with the full dependency set and an empty `main`. The design in `summary.txt` splits gband into a pure-logic core, a wire protocol, a server and a client, and every later change lands in one of them. Creating that split now, before any code exists, costs nothing to move. Logging comes with it: the server runs in the background and the client owns the terminal in raw mode, so neither can print diagnostics, and a file log is the only way to see what either did.

## What Changes

- Turn the repository root into a Cargo workspace. The root package stays the `gband` binary, so `target/release/gband` and the README's build steps are unchanged.
- Add four empty library crates under `crates/`: `gband-core`, `gband-protocol`, `gband-server` and `gband-client`.
- Move the dependency set from the root package into `[workspace.dependencies]`, and give each crate the dependencies of its role, so the resolved graph stays the one `bootstrap-rust-crate` verified.
- Give the `gband` binary a clap command line with two subcommands, `server` and `attach`. Both are stubs: each starts logging, records that it started, and exits successfully.
- Log every `gband` process to a daily-rotated file under the XDG state directory, one file series per role, with the level set by the `GBAND_LOG` environment variable. Panics are logged too.

## Capabilities

### New Capabilities

- `command-line`: the `gband` binary's subcommands, help output and exit statuses.
- `logging`: where and how `gband` processes write their diagnostic log, how its level is chosen, and what happens when the log cannot be opened.

### Modified Capabilities

None.

## Impact

- Modified files: `Cargo.toml`, `Cargo.lock`, `src/main.rs`.
- New files: `crates/core/`, `crates/protocol/`, `crates/server/`, `crates/client/`, each a `Cargo.toml` and an empty `src/lib.rs`; `src/lib.rs` and `src/logging.rs` in the root package; integration tests under `tests/`.
- Runtime: `gband` now creates `$XDG_STATE_HOME/gband/log/` (by default `~/.local/state/gband/log/`) when a subcommand runs.
- No new dependencies. Every crate used here is already in the set `bootstrap-rust-crate` added.

## Coordination

### Author
- gmteixeira

### Depends On
- bootstrap-rust-crate

### Expected Files
- Cargo.lock
- Cargo.toml
- crates/client/
- crates/core/
- crates/protocol/
- crates/server/
- openspec/changes/workspace-skeleton/
- src/lib.rs
- src/logging.rs
- src/main.rs
- tests/
