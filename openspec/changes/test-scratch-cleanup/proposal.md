## Why

gband's own tests leave their scratch directories in the system temporary directory. In one day, `/tmp` gathered about 1,300 `gband-*` entries holding 4.7 GB. The harness's `TestEnv` kills its server when dropped but never removes its root. The `gband-subcommands`, `gband-srv`, `gband-client-opener` and `gband-harness` directories are never removed at all. Directories named after the worktree outlive the worktree, and directories named after the process pile up with every run. Deleting them by hand does not stop the next run from filling `/tmp` again.

## What Changes

- A new workspace crate, `gband-scratch`, provides one scratch directory guard for every test. It creates `gband-<kind>-<name>-<pid>` under the system temporary directory and owns its removal.
- When a test passes, the guard removes its directory. It also removes every directory left by an earlier failing run of the same test, `gband-<kind>-<name>-<pid>` with a process ID no longer running.
- When a test fails, the guard keeps its directory and prints the path, so the failure can be inspected. The next passing run of that test removes it.
- Every test that makes a scratch directory under the system temporary directory uses the guard: the end-to-end `TestEnv`, the subcommand and plugin-testing tests, `gband-test-support`'s `runtime_dir` and `TestServer`, the client, Lua and harness tests, and the unit tests in `src/paths.rs`, `src/completions.rs`, `crates/harness/src/screenshot.rs`, `crates/client/src/bindings.rs`, `crates/lua/src/watch.rs` and `crates/lua/src/directory.rs`.
- Directory names no longer carry the worktree hash. The process ID keeps concurrent runs in different worktrees apart, so the FNV hashing in `tests/common/mod.rs` and `crates/test-support/src/lib.rs` goes.
- A workspace `clippy.toml` disallows `std::env::temp_dir`, so a new test cannot make a scratch directory without the guard. The guard itself, the `gband test` runner and the plugin-testing checks that look for the runner's leftovers allow it explicitly.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. Only gband's own tests change. The `gband test` runner already removes each case's tree whether it passed or not, as the plugin-testing capability requires ("Nothing left behind"), and it keeps that behaviour. The change sets `skip_specs: true`.

## Impact

- New crate `crates/scratch`, a dependency of `gband-test-support` and a dev-dependency of the root crate, `gband-client`, `gband-lua` and `gband-harness`.
- `gband-test-support`: `runtime_dir` returns the guard instead of a `PathBuf`, and `TestServer` owns it, which touches the callers in `crates/server/tests/` and `crates/client/tests/`.
- A new `clippy.toml` at the workspace root. `cargo clippy --all-targets -- -D warnings` enforces it.
- No change to the `gband` binary's behaviour, the protocol or any spec.
- Not covered: tests that write under `CARGO_TARGET_TMPDIR` (`tests/retention.rs`, `tests/panic.rs`, `tests/completions.rs` and the state half of `tests/subcommands.rs`). That directory lives inside `target/` and `cargo clean` removes it.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- Cargo.toml
- Cargo.lock
- clippy.toml
- crates/scratch/
- crates/test-support/Cargo.toml
- crates/test-support/src/lib.rs
- crates/server/Cargo.toml
- crates/server/src/lib.rs
- crates/server/tests/lifecycle.rs
- crates/server/tests/resize.rs
- crates/server/tests/scripting.rs
- crates/client/Cargo.toml
- crates/client/src/bindings.rs
- crates/client/tests/actions.rs
- crates/client/tests/bars.rs
- crates/client/tests/bridge.rs
- crates/client/tests/connect.rs
- crates/client/tests/events.rs
- crates/client/tests/local_actions.rs
- crates/client/tests/plugin_windows.rs
- crates/client/tests/sidebar.rs
- crates/lua/Cargo.toml
- crates/lua/src/watch.rs
- crates/lua/src/directory.rs
- crates/lua/tests/common/mod.rs
- crates/harness/Cargo.toml
- crates/harness/src/case.rs
- crates/harness/src/screenshot.rs
- crates/harness/tests/terminal.rs
- src/paths.rs
- src/completions.rs
- tests/common/mod.rs
- tests/subcommands.rs
- tests/plugin_testing.rs
- openspec/changes/test-scratch-cleanup/
