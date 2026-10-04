## Why

`gband-core` holds the layout, view, geometry and input rules that every other crate builds on, so a regression there reaches the whole program. Its 79 tests cover the main paths, but some branches have no test. A refusal to open a pane twice, a non-preset column width, a column with more panes than rows, and focusing a pane that no longer exists are examples. Nothing measures coverage today, so these gaps are found only by reading the code. Lua and the event subscribers will soon drive core from new places, and they will reach these branches first.

The other two parts of the original request already exist. foundation-seams added integration tests that start a real server and run `printf` through it (`crates/server/tests/programs.rs`, `sync.rs`, `tests/attach.rs`). It also added `insta` snapshot tests of ratatui `TestBackend` buffers (`crates/client/tests/render.rs`). This change does not repeat them.

## What Changes

- Measure the line coverage of `crates/core/src/` with `cargo llvm-cov`, before and after the change.
- Add unit tests to the existing `crates/core/tests/` files for every uncovered line, branch and public function the measurement reports. The gaps known from reading the code:
  - `layout`: opening a pane identifier that is already in the layout; `apply` with `OpenPane` or `ClosePane`, which the layout leaves to the server; cycling the width from a proportion that is not a preset; `Proportion::of` rounding; `first_pane` and `workspace_index`; expelling the middle pane of a stack of three.
  - `geometry`: `column_width` on its own; a column holding more panes than the area has rows; `Tile::span`.
  - `view`: `focus_pane` on a pane the layout does not hold; `apply` after the viewed workspace has gone.
- Change no production code. A new test that fails exposes a defect. It is reported and fixed in a separate change, not here.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. The tests pin behaviour that the current specs already describe or that the code already has. No requirement changes.

## Impact

- Code: none outside `crates/core/tests/`.
- Tests: new tests in `crates/core/tests/layout.rs`, `view.rs` and `geometry.rs`.
- Dependencies: none. `cargo-llvm-cov` is a developer tool installed with `cargo install`. It is not a dependency of any crate.
- Out of scope: property-based tests, end-to-end render snapshots, coverage of crates other than `gband-core`, and a coverage gate in CI, which the repository does not have.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- crates/core/tests/layout.rs
- crates/core/tests/view.rs
- crates/core/tests/geometry.rs
- crates/core/tests/events.rs
- openspec/changes/core-coverage-gaps/
