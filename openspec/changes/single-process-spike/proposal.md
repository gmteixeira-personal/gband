## Why

The crates gband depends on, `portable-pty`, `vt100`, `tui-term`, `ratatui` and `crossterm`, have never run together. Build step 1 of `summary.txt` splits them across a server and a client, which makes any problem with one of them harder to isolate. One process that hosts one shell full screen shows whether the stack works and where it breaks for the programs gband exists to host, nvim and Claude Code. The spike also needs a key encoder, the code that turns a terminal key event into the bytes a PTY expects. That encoder is permanent: every later pane uses it.

## What Changes

- Add a key and paste encoder to `gband-core`. It is a pure function of a key and the pane's input modes, and it returns the bytes to write to the PTY. It covers printable characters, control characters, Alt as an ESC prefix, editing keys, cursor keys under application cursor mode, tilde and function keys with xterm modifier parameters, and bracketed paste. Table-driven tests cover each of these.
- Add a translation in `gband-client` from a `crossterm` key event to the encoder's key type.
- Add a throwaway example, `examples/spike.rs`, in the root package. It spawns the user's shell in a PTY, feeds its output into a `vt100` grid and renders the grid full screen with `tui-term`. It forwards keys, pastes and resizes to the PTY and logs every escape sequence `vt100` does not handle. The example is not a `gband` subcommand, so the `command-line` capability is unchanged.
- Run nvim and Claude Code inside the spike and record what breaks in a Findings section of this change's `design.md`. Each serious problem is named there as a follow-up change. This change does not fix those problems.

## Capabilities

### New Capabilities

- `input-encoding`: the bytes gband writes to a pane's PTY for each key and each paste, given the pane's input modes, and which terminal key events become keys.

### Modified Capabilities

None.

## Impact

- New files: `crates/core/src/input.rs`, `crates/client/src/input.rs`, `examples/spike.rs`, and tests under `crates/core/tests/` and `crates/client/tests/`.
- Modified files: `crates/core/src/lib.rs`, `crates/client/src/lib.rs`, `crates/client/Cargo.toml`, `Cargo.toml` and `Cargo.lock`.
- Dependencies: `gband-client` gains a direct dependency on `gband-core`. The root package gains dev-dependencies on `portable-pty`, `vt100`, `tui-term`, `ratatui`, `crossterm` and `gband-core` for the example. No new crate enters the dependency graph.
- `gband-core` gains no dependency, so it stays free of I/O crates.
- The `gband` binary does not change.

## Coordination

### Author
- gmteixeira

### Depends On
- workspace-skeleton

### Expected Files
- Cargo.lock
- Cargo.toml
- crates/client/Cargo.toml
- crates/client/src/input.rs
- crates/client/src/lib.rs
- crates/client/tests/key_translation.rs
- crates/core/src/input.rs
- crates/core/src/lib.rs
- crates/core/tests/input_encoding.rs
- examples/spike.rs
- openspec/changes/single-process-spike/
