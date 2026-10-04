## Context

The repository holds only a README, OpenSpec and Claude Code configuration. There is no `Cargo.toml`. See proposal.md for why the crate and its dependency set come first.

The full dependency set was resolved and compiled in a throwaway crate with `cargo add` (the exact commands from the request) and `cargo check`, on rustc 1.98.1. It resolves and compiles cleanly. The versions it picked:

| crate | version | features |
|---|---|---|
| portable-pty | 0.9.0 | |
| vt100 | 0.16.2 | |
| tui-term | 0.3.4 | |
| ratatui | 0.30.2 | |
| crossterm | 0.29.0 | |
| tokio | 1.53.2 | full |
| serde | 1.0.229 | derive |
| postcard | 1.1.3 | use-std |
| serde_json | 1.0.151 | |
| clap | 4.6.7 | derive |
| mlua | 0.12.2 | lua54, vendored, serialize, async, send |
| rustix | 1.1.5 | process, termios |
| notify-rust | 4.18.1 | |
| notify | 8.2.0 | |
| directories | 6.0.0 | |
| tracing | 0.1.44 | |
| tracing-subscriber | 0.3.23 | |
| tracing-appender | 0.2.5 | |
| thiserror | 2.0.21 | |
| anyhow | 1.0.104 | |

## Goals / Non-Goals

**Goals:**
- A crate that builds with `cargo build` and passes `cargo clippy` with the whole dependency set in place.
- One version each of `crossterm`, `ratatui` and `vt100` in the dependency graph, so the widgets, the backend and the grid all agree on their types.

**Non-Goals:**
- Any runtime behavior. `src/main.rs` is an empty entry point.
- Module layout, the client/server split, the protocol or the Lua API. Each of these gets its own change.
- A `.claude/flow-gate` script. It becomes useful once there is code to test, and a later change adds it.

## Decisions

### One binary crate at the repository root

The crate is a single binary named `gband`, created with `cargo init --name gband` in the repository root, edition 2024. One binary serves as client, server and CLI, which is how tmux and herdr ship.

Alternative: a Cargo workspace with separate `gband-server`, `gband-client` and `gband-protocol` crates. That split costs build configuration before there is code to split. A workspace can be introduced later without changing the dependency choices.

### Add dependencies with `cargo add`, commit `Cargo.lock`

The dependencies are added with the `cargo add` commands from the request, so each resolves to its latest stable release, as listed in the Context table. `Cargo.lock` is committed, because gband is a binary and every build should use the same resolved graph.

`notify` resolves to 8.2.0 although 9.0.0 release candidates exist. `cargo add` skips pre-releases, and the change keeps it that way.

### Declare `rust-version`

`Cargo.toml` sets `rust-version` to the highest minimum supported Rust version among the direct dependencies, which is currently 1.89 (from `notify-rust`). A too-old toolchain then fails with one clear message instead of a compile error deep in a dependency.

### Direct `crossterm` dependency, kept in step with ratatui

ratatui 0.30 reaches crossterm through `ratatui-crossterm`, and the probe showed both resolving to crossterm 0.29.0. gband keeps its own direct `crossterm` dependency for reading input, because it uses crossterm's event API directly.

Alternative: use the `ratatui::crossterm` re-export only. That avoids the risk of two versions but ties input handling to whatever ratatui re-exports. The direct dependency is kept, and the version check below guards it.

### Lua 5.4, not LuaJIT

mlua uses the `lua54` feature. Lua 5.4 has native integers and is the current reference implementation, and the vendored build works on every platform Rust targets.

Alternative: `luajit` is faster. Config and scripting hooks are not hot paths, and LuaJIT is stuck on Lua 5.1 semantics. Switching is a feature flag change later if profiling shows a need.

### `notify` is for watching config files

The request lists `notify` without a stated role. This design assumes it watches the Lua config so changes reload without a restart, as niri reloads its config. If that assumption is wrong, the dependency still costs nothing until code uses it.

## Risks / Trade-offs

- [The vendored Lua build needs a C compiler] → Note it in the README build requirements. Fedora and most developer setups have `cc` already.
- [Two crossterm versions appear after a future ratatui upgrade, and the widgets and the input reader disagree on types] → Task 2.3 runs `cargo tree -d` and checks for a single crossterm, ratatui and vt100. Repeat that check on every ratatui or tui-term upgrade.
- [`notify-rust` pulls in a D-Bus stack, which adds build time] → Accepted. Desktop notifications are a stated goal.
- [The dependency graph already duplicates `thiserror` (1.x and 2.x), `syn` and `hashbrown` through transitive dependencies] → Accepted. These duplicates are not visible through gband's own types.
- [`tracing-subscriber` is added without the `env-filter` feature, so `RUST_LOG`-style filtering is unavailable] → Add the feature in the change that sets up logging, if that change wants it.
