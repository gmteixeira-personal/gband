## 1. Workspace

- [ ] 1.1 Add a `[workspace]` table to the root `Cargo.toml` with `members = ["crates/*"]` and `resolver = "3"`, move `version`, `edition` and `rust-version` to `[workspace.package]` and inherit them in `[package]`, and verify that `cargo metadata --format-version 1 --no-deps` reports the root package `gband` with edition 2024
- [ ] 1.2 Move every dependency of the root package to `[workspace.dependencies]` with its existing version and features, add the `env-filter` feature to `tracing-subscriber`, and declare `gband-core`, `gband-protocol`, `gband-server` and `gband-client` there by path, then verify that the root `[workspace.dependencies]` lists all 20 external crates
- [ ] 1.3 Create `crates/core`, `crates/protocol`, `crates/server` and `crates/client`, each with a `Cargo.toml` naming its `gband-` package, inheriting the workspace package fields, and an empty `src/lib.rs`, and verify that `cargo metadata --format-version 1 --no-deps` lists the five workspace members
- [ ] 1.4 Give each member the dependencies in the design's role table, all with `.workspace = true`, and verify that `cargo tree -p gband-core -e normal --depth 1` shows only `serde` and `thiserror`
- [ ] 1.5 Run `cargo build --workspace` and `cargo tree -d -e normal`, and verify that the build succeeds and that `crossterm`, `ratatui` and `vt100` each appear in only one version

## 2. Command line

- [ ] 2.1 Replace `src/main.rs` with a clap derive `Cli` holding a required subcommand enum of `Server` and `Attach`, parsed before anything else runs, and verify that `cargo run -- --help` lists both subcommands and that `cargo run` with no arguments exits with status 2
- [ ] 2.2 Map `server` to the `server` role and `attach` to the `client` role, start logging for that role, record one `info` event `server started` or `client started` with the version and process id, and return, then verify that `cargo run -- server` exits with status 0 and prints nothing

## 3. Logging

- [ ] 3.1 Add `src/lib.rs` exposing `src/logging.rs`, with one function that takes the role, resolves the log directory from `directories::ProjectDirs` with the `state_dir()` then `data_local_dir()` fallback, creates it, and returns the non-blocking writer's guard, and verify that `cargo run -- server` with `XDG_STATE_HOME` set to a temporary directory writes `gband/log/server.<today>.log` there
- [ ] 3.2 Build the file appender with daily rotation, the role as prefix, `log` as suffix and at most 7 files, and format events without ANSI codes, then verify that the file written in 3.1 holds a `server started` line with no escape character
- [ ] 3.3 Read `GBAND_LOG` with `std::env::var`, parse it with `EnvFilter::try_new`, fall back to `info` on an error and record a `warn` event quoting the rejected value, and verify that `GBAND_LOG='[[[' cargo run -- server` exits with status 0 and logs that warning
- [ ] 3.4 Install a panic hook that records the message and location as an `error` event and then calls the previous hook, and verify it with the panic test of 4.3
- [ ] 3.5 On a failure to create the directory or open the file, print one line naming the directory and the cause to standard error and exit with status 1, and verify that `gband server` with `XDG_STATE_HOME` pointing at a read-only directory does so

## 4. Tests

- [ ] 4.1 Add integration tests under `tests/` that run `env!("CARGO_BIN_EXE_gband")` with `XDG_STATE_HOME` under `env!("CARGO_TARGET_TMPDIR")`, covering every scenario of the `command-line` spec and the placement, colour, level and setup-failure scenarios of the `logging` spec, and verify that `cargo test --workspace` passes
- [ ] 4.2 Add one test file under `tests/` for the retention scenario, which pre-creates 7 dated `server` files and one `client` file, calls the library's logging function, and asserts at most 7 `server` files remain and the `client` file is untouched, and verify that it passes
- [ ] 4.3 Add one test file under `tests/` for the panic scenario, which installs logging, panics in a spawned thread with the message `boom`, joins it, drops the guard, and asserts the file holds an `error` event with `boom` and the panic's file and line, and verify that it passes

## 5. Finish

- [ ] 5.1 Run `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all --check`, and verify that both exit with status 0
- [ ] 5.2 Check every new or changed Rust file against the repository's coding rules, which forbid narrative comments, and verify that no comment remains except one explaining an upstream API restriction
- [ ] 5.3 Run `cargo build --release` and verify that `target/release/gband` exists, as the README's Building section states
