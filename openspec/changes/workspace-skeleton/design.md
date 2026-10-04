## Context

This change starts from the state `bootstrap-rust-crate` leaves: one binary package `gband` at the repository root, edition 2024, `rust-version = "1.89"`, the 20-crate dependency set in `[dependencies]`, `Cargo.lock` committed, and `src/main.rs` holding `fn main() {}`. That change chose one crate and noted that a workspace could come later without changing the dependency choices. See proposal.md for why the split comes now.

`summary.txt` at the repository root is the architecture guide. It names the binary `yourmux`; that name is a placeholder, and the binary stays `gband`.

## Goals / Non-Goals

**Goals:**
- A workspace in which `cargo build --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` pass.
- Each library crate owns the dependencies of its role from the start, so later changes add code to a crate without editing manifests across the workspace.
- A logging setup the server, the client and every later subcommand reuse unchanged.

**Non-Goals:**
- Any server, client, PTY, protocol or layout behavior. The four library crates hold an empty `src/lib.rs`.
- The `lua` crate from `summary.txt`. It gets its own change when the Lua API is designed.
- Subcommands other than `server` and `attach`: `msg`, `agent-event`, `proxy` and `install-hooks` come with the features they serve.
- Daemonizing `gband server`, or `gband attach` starting a server. Both belong to the change that implements them.
- A `.claude/flow-gate` script.

## Decisions

### The root package stays the binary and becomes the workspace root

`Cargo.toml` at the root keeps `[package] name = "gband"` and gains a `[workspace]` table listing `crates/*`. `src/main.rs` stays where it is.

Alternative: a virtual manifest at the root with the binary moved to `crates/gband/`. That is tidier, but moves `src/` and changes nothing a user sees. The root package keeps `cargo run` working with no `-p` flag and leaves the README's build steps and `target/release/gband` untouched.

### Crate directories and package names

| directory | package | role |
|---|---|---|
| `crates/core` | `gband-core` | layout, ribbon, agent states; pure logic, no I/O |
| `crates/protocol` | `gband-protocol` | wire messages and versioning |
| `crates/server` | `gband-server` | pane host, services, behavior Lua host |
| `crates/client` | `gband-client` | transports, rendering, presentation Lua host |

The directories use the short names from `summary.txt`. The packages carry a `gband-` prefix because a package named `core` shadows Rust's built-in `core` crate, and the prefix keeps all four names consistent. Code imports them as `gband_core`, `gband_protocol` and so on.

### Shared package fields and dependency versions live in the workspace

`[workspace.package]` holds `version`, `edition` and `rust-version`, and every member inherits them with `.workspace = true`. All 20 dependencies move to `[workspace.dependencies]` with the exact versions and features `bootstrap-rust-crate` resolved, and each member names the ones it uses with `<dep>.workspace = true`. The internal crates are also declared there by path, so a member depends on `gband-core.workspace = true`.

The workspace sets `resolver = "3"`, the edition 2024 default, explicitly, so the resolver does not change silently if the root package's edition does.

### Dependencies by role

| package | dependencies |
|---|---|
| `gband-core` | `serde`, `thiserror` |
| `gband-protocol` | `gband-core`, `serde`, `postcard`, `serde_json`, `thiserror` |
| `gband-server` | `gband-core`, `gband-protocol`, `portable-pty`, `vt100`, `tokio`, `mlua`, `rustix`, `notify`, `tracing`, `thiserror` |
| `gband-client` | `gband-protocol`, `ratatui`, `crossterm`, `tui-term`, `vt100`, `tokio`, `mlua`, `notify-rust`, `tracing`, `thiserror` |
| `gband` | `gband-server`, `gband-client`, `clap`, `directories`, `tracing`, `tracing-subscriber`, `tracing-appender`, `anyhow` |

Every one of the 20 crates stays in at least one member, so `Cargo.lock` resolves the same graph `bootstrap-rust-crate` verified, and the single-version check on `crossterm`, `ratatui` and `vt100` stays meaningful.

`gband-core` stays free of I/O crates: no `tokio`, no `portable-pty`, no `mlua`. That is the boundary which keeps it unit-testable without a terminal.

`tracing-subscriber` gains the `env-filter` feature, which `GBAND_LOG` needs. It is a feature of a crate already in the set, not a new crate.

Alternative: leave every dependency on the binary and move each one when its first user appears. That leaves the libraries' manifests empty now, but drops most of the graph from `Cargo.lock` until code arrives, and spreads manifest edits over every later change.

### Command line

`src/main.rs` parses a clap derive `Cli` with a required `Command` enum of `Server` and `Attach`. clap's defaults supply the help, `--version`, the usage on standard error and exit status 2 for a missing or unknown subcommand, which is what the `command-line` spec requires. Argument parsing runs before logging starts, so an invalid invocation creates no log file.

Each subcommand maps to a role, `server` or `client`, which names its log file series. The stub then records one `info` event, `server started` or `client started`, with the version and process id, and returns.

### Logging module in the root package

Logging lives in `src/logging.rs`, exposed through a library target `src/lib.rs` of the root package, with one function that takes the role and returns a guard to hold for the life of the process. `src/main.rs` calls it through the library. The library target exists so integration tests can install logging in a process of their own, which a binary-only package does not allow.

- **Directory**: `directories::ProjectDirs::from("", "", "gband")`, then `state_dir()` joined with `log`, falling back to `data_local_dir()` when `state_dir()` is `None` (macOS and Windows). `directories` already honours `XDG_STATE_HOME` and ignores a relative value, as the XDG specification requires. The directory is created with `create_dir_all`.
- **Files**: `tracing_appender::rolling::Builder` with `Rotation::DAILY`, `filename_prefix(role)`, `filename_suffix("log")` and `max_log_files(7)`, which yields `server.2026-10-04.log` and prunes only files of the same prefix and suffix. `Builder::build` returns an error when the file cannot be opened; it is reported with the directory and the program exits with status 1.
- **Writer**: `tracing_appender::non_blocking`, so a slow disk never stalls the server's event loop. The returned `WorkerGuard` lives in `main` and flushes on drop.
- **Format**: `tracing_subscriber::fmt` with `with_ansi(false)` and the default timestamp, level and target.
- **Filter**: `GBAND_LOG` is read with `std::env::var` and parsed with `EnvFilter::try_new`. An unset variable means `info` silently. On a parse error the filter falls back to `info`, and after the subscriber is installed one `warn` event quotes the rejected value. `EnvFilter::try_from_env` is not used because it returns the same error for an unset variable as for an invalid one.
- **Panics**: a panic hook records the panic message and `Location` as an `error` event, then calls the previous hook. In the stubs the main thread returns normally, so the guard drops and flushes. A panic on the main thread unwinds through `main`, which also drops the guard.

The setup error is printed by `main` itself with `eprintln!`, not through `tracing`, because the subscriber does not exist yet. That is the one place `gband` writes to standard error outside clap.

Alternative: put logging in a shared crate. Only the binary installs a global subscriber; the libraries only emit events through `tracing`. A crate of its own is premature until a second binary exists.

Alternative: one log file for all roles. Server and client run at the same time, often on different machines over SSH, and interleaving them makes both harder to read.

### Tests

The spec's command-line, placement, level and setup-failure scenarios are integration tests in `tests/` that run the built binary with `env!("CARGO_BIN_EXE_gband")`, `XDG_STATE_HOME` pointed at a temporary directory, and `GBAND_LOG` set or removed per case. The retention and panic scenarios cannot be reached through the stub subcommands, so they call the library's logging function directly. A process holds one global subscriber, so each of those tests lives in its own file under `tests/`, which Cargo builds as its own process. No new dev-dependency is added: temporary directories are made under `env!("CARGO_TARGET_TMPDIR")`.

## Risks / Trade-offs

- [Libraries declare dependencies no code uses yet, so a reader cannot tell a needed dependency from a stale one] → The role table above is the record. Each later change that adds code to a crate checks it against that crate's manifest.
- [`tracing-appender`'s file names or pruning differ from the spec across versions] → The pinned 0.2.5 produces `<prefix>.<date>.<suffix>`; the retention unit test fails if an upgrade changes it.
- [The non-blocking writer drops events when its buffer fills] → Accepted for a background server. The default buffer holds 128 000 lines, far above what the stubs write. A later change can switch to `NonBlockingBuilder::lossy(false)` if loss is ever observed.
- [`max_log_files` prunes only when a new file is created, so a server that runs for months still keeps at most 7 files but checks only at each daily roll] → That is the behavior the spec states.
- [`bootstrap-rust-crate` changes its manifest before it is archived] → This change depends on it and starts from what `<base>` holds after it merges, so implementation reads the merged `Cargo.toml` rather than this design's copy.

## Migration Plan

None. Nothing has been released and no user has a config or log directory to move.
