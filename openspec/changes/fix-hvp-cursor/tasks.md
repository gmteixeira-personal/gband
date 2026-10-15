## 1. Patch vt100 for HVP

- [ ] 1.1 Fork `vt100` 0.16.2 and add `'f' => self.screen.cup(canonicalize_params_2(params, 1, 1))` beside the `'H'` arm in `src/perform.rs`. Tag the fork `0.16.2-hvp`.
- [ ] 1.2 Add `[patch.crates-io] vt100 = { git = "<fork url>", tag = "0.16.2-hvp" }` to the workspace `Cargo.toml`. Run `cargo update -p vt100` so `Cargo.lock` records the fork. Verify `cargo build --workspace`.

## 2. Lock HVP in the emulator contract

- [ ] 2.1 Add an `hvp_positions_like_cup` contract test to `crates/emulator/tests/contract.rs`, generic over `E: Emulator` like `ln_characters`: feed `\e[2;5fX`, assert `X` at row 2 col 5, and assert `\e[f` then text lands at row 1 col 1.
- [ ] 2.2 Instantiate it for `Grid`, matching `grid_reproduces_ln_characters`. Verify `cargo test -p gband-emulator --test contract`.

## 3. Confirm btop

- [ ] 3.1 Run btop in a gband window and confirm the CPU, memory and network panels draw in place, not scrambled. Manual, visual.
