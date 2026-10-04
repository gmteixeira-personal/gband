# gband

gband is a terminal multiplexer inspired by the [niri](https://github.com/YaLTeR/niri) window manager.
It places panes on an infinitely side-scrolling strip and groups strips into workspaces.
It is written in Rust and scriptable with Lua.

## Status

gband is in early development and is not usable yet.

## How it works

Multiplexers such as tmux and Zellij split a fixed screen area.
Each new pane shrinks the panes already on screen.

gband follows niri's scrollable tiling model instead:

- Panes sit in columns on a strip that extends without limit to the left and right.
- A new pane adds a column to the strip, and existing panes keep their size.
- The view scrolls sideways to follow focus, so the focused pane is always on screen.
- A column can hold several panes stacked vertically.
- Each workspace has its own strip.
  Workspaces stack vertically, and you move between them up and down.

## Scripting

gband embeds a Lua runtime.
Configuration and automation are Lua scripts, so key bindings, layout behavior and custom commands are code you can change.

## Building

Install a stable Rust toolchain, version 1.89 or newer, and a C compiler such as `gcc` or `clang`.
The C compiler builds the Lua runtime that gband bundles, so no system Lua is needed.
Then run:

```sh
git clone git@github.com:gmteixeira-personal/gband.git
cd gband
cargo build --release
```

The binary is `target/release/gband`.

## Branches

- `dev` is the default branch.
  All development happens there.
- Every other working branch forks from `dev` and merges back into it.
- `main` holds releases only.
  It moves forward only when a release is made from `dev`.

Branch from `dev` and open pull requests against `dev`.
Never commit to `main` directly or branch from it.

## Acknowledgements

The layout model comes from [niri](https://github.com/YaLTeR/niri), a scrollable-tiling Wayland compositor.
