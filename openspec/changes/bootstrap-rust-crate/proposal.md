## Why

gband has no code yet. Every later change needs a Cargo crate to build in, and the dependency set decides the architecture: how panes run programs, how they render, how the server and clients talk, and how Lua scripts the whole thing. Fixing that set in one change lets later changes build features instead of debating libraries.

## What Changes

- Create a single binary crate named `gband` at the repository root, with `Cargo.toml`, `Cargo.lock` and a placeholder `src/main.rs`.
- Add the dependency set, grouped by role:
  - **Terminal core**: `portable-pty` spawns programs in PTYs, `vt100` keeps each pane's screen grid, `tui-term` and `ratatui` render panes, and `crossterm` is ratatui's backend and reads input from the host terminal. This is the combination herdr already proves in production.
  - **Server/client plumbing**: `tokio` (full) for async PTY reads, the Unix socket and timers; `serde` (derive) for everything that crosses a boundary; `postcard` (use-std) for the compact internal client/server protocol; `serde_json` for the external CLI, the JSON event stream and session snapshots, which humans and scripts read.
  - **Lua**: `mlua` with `lua54`, `vendored`, `serialize`, `async` and `send`. Vendored means users need no system Lua. Serialize turns Lua config tables into typed Rust structs.
  - **Command line**: `clap` (derive).
  - **Agent awareness**: `rustix` (process, termios), to inspect the processes running in each pane.
  - **Desktop notifications**: `notify-rust`.
  - **File watching**: `notify`, to react to config file changes.
  - **Logging**: `tracing`, `tracing-subscriber` and `tracing-appender`. The server has no terminal to print to, so it logs to files.
  - **Paths and errors**: `directories` for XDG paths, `thiserror` and `anyhow` for errors.
- Ignore the Cargo build directory `/target` in `.gitignore`.

## Capabilities

### New Capabilities

None. This change adds build tooling and dependencies only. The binary does nothing observable yet, so no spec changes. The change sets `skip_specs: true`.

### Modified Capabilities

None.

## Impact

- New files: `Cargo.toml`, `Cargo.lock`, `src/main.rs`.
- Modified files: `.gitignore`, and `README.md` for the C compiler build requirement.
- Build requirements: a stable Rust toolchain no older than the highest `rust-version` among the dependencies, and a C compiler for the vendored Lua.
- The README's existing build instructions (`cargo build --release`) start to work.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- .gitignore
- Cargo.lock
- Cargo.toml
- README.md
- openspec/changes/bootstrap-rust-crate/
- src/main.rs
