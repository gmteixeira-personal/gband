## Context

Tests make scratch directories under `std::env::temp_dir()` at about twenty sites, each with its own naming and cleanup. See proposal.md for what leaks. Today they fall into three groups:

- **Never removed**: `TestEnv` in `crates/harness/src/env.rs` removes an old root when it starts and kills the server and shells when dropped, but never removes the root. That covers `gband-e2e-*` through `tests/common/mod.rs` and `gband-harness-*` through `crates/harness/tests/terminal.rs`. `gband-subcommands-*` in `tests/subcommands.rs`, `gband-srv-*` in `crates/test-support/src/lib.rs` and `gband-client-opener-*` in `crates/client/tests/local_actions.rs` likewise remove only at the start of the next run.
- **Removed on drop, even on failure**: the `Scratch` types in the client and Lua tests, `crates/lua/src/watch.rs` and `crates/lua/src/directory.rs`, the `Client` in `crates/client/tests/local_actions.rs`, and the `Project` in `tests/plugin_testing.rs`.
- **Removed by the test's last line**: `src/paths.rs`, `src/completions.rs`, `crates/harness/src/screenshot.rs` and `crates/client/src/bindings.rs`. A failing assertion skips that line, and the directory, named by the process ID, is never visited again.

`gband-e2e`, `gband-subcommands`, `gband-pt` and `gband-srv` names carry an FNV hash of `CARGO_MANIFEST_DIR`. The hash keeps concurrent runs in different worktrees apart while keeping one stable name per worktree.

The `gband test` runner, `crates/harness/src/case.rs`, names each case `gband-test-<pid>-<n>` and removes it when the case ends, pass or fail. `tests/plugin_testing.rs` asserts that. It stays as it is.

Every crate can take a dependency on a new leaf crate. `gband-test-support` depends on the server and client crates, so the Lua crate cannot use it, and the harness is a runtime dependency of the binary.

## Goals / Non-Goals

**Goals:**
- A test that passes leaves nothing under the system temporary directory.
- A test that fails keeps its directory until that test next passes.
- A new test cannot make a scratch directory there without the guard.

**Non-Goals:**
- Directories under `CARGO_TARGET_TMPDIR`, which live in `target/`.
- The `gband test` runner's case trees, which it already removes.
- Directories left by a test that is never run again, or by a process killed with SIGKILL. Both stay until removed by hand.

## Decisions

### One guard in a new leaf crate

`crates/scratch`, package `gband-scratch`, depends only on `rustix`. It exports `Scratch`:

- `Scratch::new(kind, name)` creates `<temp>/gband-<kind>-<name>-<pid>` with mode 0700, after removing any directory already at that path.
- `Scratch` implements `Deref<Target = Path>` and `AsRef<Path>`, so most callers that took a `&Path` keep compiling unchanged.
- On drop, it removes its directory if the thread is not panicking. It then removes every sibling `gband-<kind>-<name>-<digits>` whose process ID is not running.
- On drop while `std::thread::panicking()`, it keeps the directory and writes `kept <path>` to standard error. The test harness shows that line with the failure's output.

Alternatives:
- Put the guard in `gband-test-support`. Rejected: the Lua crate and the binary's unit tests cannot depend on it without a cycle or a second copy of a crate.
- Put it in `gband-harness`. Rejected: the harness ships in the binary, and the guard is test-only.
- Use the `tempfile` crate. Rejected: its `TempDir` removes on drop even while panicking, and has no way to find an earlier run's directory.

### Names carry the process ID, and passing runs sweep stale ones

The sibling sweep needs a name that is stable across runs of one test and different between concurrent runs. `gband-<kind>-<name>` is stable, and the trailing process ID separates concurrent runs. The worktree hash then serves no purpose. Directories from a removed worktree carry a dead process ID, so the next passing run of the same test in any worktree sweeps them.

A sibling matches only when the text after `gband-<kind>-<name>-` is all digits. So `gband-e2e-config` does not sweep `gband-e2e-config-default-1234`. Liveness uses `rustix::process::test_kill_process`. Only `ESRCH` counts as dead, so a process of another user counts as running.

Alternative: keep stable per-worktree names with no process ID. Each run removes the previous directory at start, so a failure is kept until the next run. Rejected: directories of removed worktrees are never visited again, and concurrent runs in one worktree collide.

### Removal survives read-only trees

Some tests drop write permission inside their tree, for example `tests/subcommands.rs` on the runtime directory. When `remove_dir_all` fails, the guard walks the tree, sets every directory to 0700 and tries once more. A second failure is ignored, because a guard must not panic in `Drop`.

### Owners drop the processes before the directory

The guard is the last field of every struct that owns one and runs processes in it, so Rust's field drop order kills the server and shells first. The end-to-end `TestEnv` in `tests/common/mod.rs` holds the harness `TestEnv` first and its `Scratch` second. `gband-test-support`'s `TestServer` holds its `Scratch` after the server handle. `runtime_dir` returns `Scratch`, and `TestServer::start_in` and `start_with` take one.

### Clippy keeps it that way

A workspace `clippy.toml` lists `std::env::temp_dir` under `disallowed-methods`, with a reason that names `gband_scratch::Scratch`. The gate already runs `cargo clippy --all-targets -- -D warnings`. `#[allow(clippy::disallowed_methods)]` goes on the guard's own call, on `scratch` in `crates/harness/src/case.rs`, and on the two `tests/plugin_testing.rs` tests that scan the temporary directory for the runner's leftovers.

Alternatives:
- A project rule in `CLAUDE.md` or a memory. Rejected: a rule asks to be remembered, and a lint fails the build.
- A test that greps the sources. Rejected: clippy already resolves paths and handles aliases.

## Risks / Trade-offs

- [A reused process ID makes a dead run's directory look alive] → The sweep skips it. The next passing run whose ID is free removes it.
- [A guard dropped on a thread other than the failing test's sees no panic] → It removes the directory, and that failure loses its tree. Keep guards owned by the test function or by structs it owns, never moved into spawned tasks.
- [Two tests in one process sharing kind and name] → They would share a directory, as they would today. Names stay unique per test.
- [A failing test that is renamed or deleted keeps its directory forever] → Accepted. It is one directory, not one per run.

## Migration Plan

Land the crate and the lint together with the migration, so `-D warnings` passes on the merge. After the merge, `rm -rf /tmp/gband-*-<8 hex digits>*` removes the hash-named directories that no run will sweep. Rollback is a revert.
