## Why

btop renders scrambled inside a gband window. btop positions every cell with HVP (`\e[<row>;<col>f`), 626 of 626 moves in a 3-second capture, and never uses CUP (`\e[<row>;<col>H`). The `vt100` 0.16.2 emulator handles `H` but has no `f` arm (`perform.rs:103-116`), so every `f` falls to `unhandled_csi`, which never moves the cursor. All 626 jumps are dropped and btop's absolutely-positioned cells pile up wherever the cursor drifted. HVP and CUP are identical per ECMA-48, so this is a parsing gap, and any HVP-positioning program is affected.

## What Changes

- The emulator moves the cursor on `\e[<row>;<col>f` (HVP) identically to `\e[<row>;<col>H` (CUP), with the same 1-based defaults.
- `vt100` 0.16.2 gains the missing `f` dispatch. The crate exposes cursor positioning only as `pub(crate) cup`, so the fix patches the library rather than handling `f` in gband's `unhandled_csi`. The design chooses between a `[patch.crates-io]` fork and vendoring.
- A terminal-emulator scenario covers HVP positioning so the gap cannot return unnoticed.

## Capabilities

### New Capabilities

### Modified Capabilities
- `terminal-emulator`: add a requirement that the emulator positions the cursor on HVP (`\e[<row>;<col>f`) exactly as it does on CUP, including the 1-based row and column defaults when a parameter is omitted.

## Impact

- `crates/emulator`: the vt100-backed `Vt100` now positions on `f`. No change to the `Emulator` trait or its callers.
- `Cargo.toml`: a `[patch.crates-io]` entry or a vendored `vt100`, pinned to a fork of 0.16.2 with the one-line `f` dispatch added.
- Programs fixed: btop and any other that positions with HVP. Programs that use CUP (nvim, most TUIs) are unaffected, which is why only btop scrambled.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- Cargo.toml
- Cargo.lock
- crates/emulator/tests/contract.rs
- openspec/changes/fix-hvp-cursor/
