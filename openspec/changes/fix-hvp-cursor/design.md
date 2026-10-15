## Context

btop positions every cell with HVP, `\e[<row>;<col>f`: 626 of 626 moves in a 3-second capture at 100x30, zero CUP (`\e[<row>;<col>H`). The `vt100` 0.16.2 CSI dispatch in `perform.rs:103-116` maps `H` to `cup` but has no `f` arm, so each `f` reaches `unhandled_csi` and never moves the cursor. btop's absolute layout then collapses onto a drifting cursor and renders scrambled. CUP-positioning programs (nvim, most TUIs) are unaffected.

## Goals

- `\e[<row>;<col>f` moves the cursor identically to `\e[<row>;<col>H`.
- No change to the `Emulator` trait or its callers.
- A contract test locks HVP so the gap cannot return.

## Decision: patch vt100, do not route through gband

`vt100::Screen` exposes cursor positioning only as `pub(crate) fn cup` (`screen.rs:1047`). `unhandled_csi` receives `&mut Screen` but cannot call `cup`, and no public setter reaches the grid cursor. gband cannot add `f` from its own callbacks. The fix is one arm in vt100's dispatch:

```rust
'f' => self.screen.cup(canonicalize_params_2(params, 1, 1)),
```

placed beside the `'H'` arm, with the same `canonicalize_params_2(params, 1, 1)` defaults.

## Decision: a git fork over vendoring

The workspace pins `vt100 = "0.16.2"` from crates.io with no `[patch]`. Redirect it to a fork of 0.16.2 carrying only the `f` arm:

```toml
[patch.crates-io]
vt100 = { git = "https://github.com/gmteixeira-personal/vt100-rust", tag = "0.16.2-hvp" }
```

The fork keeps the footprint to `Cargo.toml` and `Cargo.lock`. Vendoring a patched copy under `crates/vt100/` is the fallback when an external fork is unwanted; it adds the whole crate tree and a path `[patch]` instead. The fork is chosen because the diff is one line and the vendored tree is otherwise dead weight.

## Risks

- The upstream `alacritty_terminal` move named in `summary.txt` would drop the fork later. This change does not block that: it adds one requirement and one test that any emulator must pass.
- A git `[patch]` fetches the fork at build time. Offline builds need the fork cached, same as any git dependency.
