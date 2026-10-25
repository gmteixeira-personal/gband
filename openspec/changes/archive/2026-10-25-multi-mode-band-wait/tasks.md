## 1. Wait for the new band's window

- [x] 1.1 In `tests/multi_mode.rs`, add `viewed_window`, which returns `gband.view().window` as an `Option<u32>`, and make `focused_window` unwrap it. Verify with `cargo test --test multi_mode`
- [x] 1.2 In `tests/multi_mode.rs`, add `open_band` beside `open_column`: it sends prefix `u n Enter`, waits until `viewed_window` is a window other than the one focused before, waits for the prompt and returns the focused window. Make `band_change_moves_the_input` and `other_bands_receive_nothing` call it in place of sending the keys and reading `focused_window` at once. Verify by running the `multi_mode` test binary 40 times idle and 40 times with every core busy, and check that every run passes

## 2. Gate

- [x] 2.1 Run `.claude/flow-gate` and check that fmt, clippy and the workspace tests pass

## Workflow follow-up

- Sync the change's specs (none, since `skip_specs` is set) and archive the change.
