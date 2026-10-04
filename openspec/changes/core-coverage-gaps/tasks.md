## 1. Baseline

- [ ] 1.1 Confirm `cargo llvm-cov` runs. If it is not installed, ask the operator before installing `llvm-tools-preview` and `cargo-llvm-cov`
- [ ] 1.2 Run `cargo llvm-cov -p gband-core --show-missing-lines` and record the line and region percentages and the uncovered lines in `openspec/changes/core-coverage-gaps/coverage.md`

## 2. Layout

- [ ] 2.1 Opening a pane identifier that the layout already holds changes nothing and returns no events
- [ ] 2.2 `apply` with `OpenPane` or `ClosePane` changes nothing and returns no events
- [ ] 2.3 Cycling from a width that is not a preset moves to the next larger preset, and wraps to one third from above two thirds
- [ ] 2.4 `Proportion::of` rounds down, for example one third of 80 and two thirds of 1
- [ ] 2.5 `first_pane` and `workspace_index` on populated, empty and unknown workspaces
- [ ] 2.6 Expelling the middle pane of a stack of three, to each side, leaves the other two stacked in order

## 3. Geometry

- [ ] 3.1 `column_width` for a preset, full width, and a width below the three-cell minimum
- [ ] 3.2 A column holding more panes than the area has rows: pin the tile heights and positions, and list the behaviour in `coverage.md`
- [ ] 3.3 `Tile::span` matches the tile's position and width

## 4. View

- [ ] 4.1 `focus_pane` on a pane the layout does not hold changes neither the workspace nor the focus
- [ ] 4.2 A view action after the viewed workspace has been removed moves the view to the workspace in its place

## 5. Close the remaining gaps

- [ ] 5.1 Re-run the measurement and add a test for every line still reported as missing, or record why the public API cannot reach it
- [ ] 5.2 Record the final percentages, every pinned behaviour and every contradiction found in `coverage.md`

## 6. Verify

- [ ] 6.1 `cargo test -p gband-core`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` pass
- [ ] 6.2 `git diff --stat origin/dev` shows changes only under `crates/core/tests/` and `openspec/changes/core-coverage-gaps/`
