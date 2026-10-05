## Context

See proposal.md for the motivation. The relevant current state:

- `gband-core`'s `Column` holds `panes: Vec<PaneId>`, a `Proportion` width and a full-width flag. `geometry::tiles` splits a column's height equally. `Layout::apply(action)` handles every session action except open and close, which the server owns. `LayoutEvent` derives `Copy`.
- `gband-server`'s one session task owns the layout. `Session::publish` resizes every pane whose tile changed, then sends the new `Arc<State>` and bumps `changed`. `Pane::resize` skips a pane whose grid already has the size. `drive` selects over commands and pane exits.
- The client keeps a `View` per attachment. `Scene` carries the layout, the screen area and `viewport_cols`. `render::render` already draws each grid through `PseudoTerminal` into the tile's interior. That cuts a larger grid and leaves blank cells around a smaller one, and `cursor_position` hides a cursor outside the interior. The "Present the ribbon" scenarios for mismatched grids already hold, and this change only pins them with tests.
- `connection.rs` holds an `Attachment` per attached client, whose `Drop` decrements the client count and publishes `ClientDetached`.

This change depends on core-coverage-gaps, which adds tests to the same `crates/core/tests/` files. Implementing on top of it avoids two changes editing those files at once.

## Goals / Non-Goals

**Goals:**
- One session-wide quiet period that covers terminal resizes, held resize keys and camera moves alike.
- Panes whose heights are all automatic with weight 1 get exactly today's geometry, so every existing geometry and render test keeps passing unchanged.
- Pane heights behave as niri's window heights do, with every press moving a pane by the same number of rows.
- The server needs no knowledge of cameras. Clients tell it what they show.

**Non-Goals:**
- Animating the transition. The client already draws the old grid cut or padded, which is the summary's "clip/pad old grid" first step.
- A per-pane or per-client quiet period.
- Fixed-cell widths or heights.

## Decisions

### Width steps are exact fractions

`Proportion::step(self, step: Step) -> Proportion` adds or subtracts 1/10, clamps to `[1/10, 1]`, and reduces by the greatest common divisor. Starting from the presets, denominators stay within 2, 3, 10 and 30, so `u8` fields hold every reachable value. `Step` is a new `enum Step { Grow, Shrink }` in `layout.rs`. `Column::step_width` treats full width as `Proportion::WHOLE` and turns it off, as `cycle_width` does. Changes go through the existing `with_column`, so `ColumnWidthChanged` is emitted only when the pair `(width, full_width)` changes.

Alternative: store widths as a percentage (`u8` 1 to 100). Rejected. 1/3 has no exact percentage, so cycling and stepping would disagree about the presets.

### Heights follow niri's column model

The model is a port of niri's `WindowHeight::{Auto { weight }, Fixed}` and its `Column::set_window_height`, `reset_window_height`, `convert_heights_to_auto` and `update_tile_sizes` in `src/layout/scrolling.rs`. It has no gaps, and every tile's minimum height is `MIN_TILE_HEIGHT` (3 rows: the border and one row), beside `MIN_COLUMN_WIDTH`, in place of a window's minimum size. Preset heights (`switch-preset-window-height`), tabbed columns and window size constraints are left out.

`layout.rs` gains `enum PaneHeight { Auto(Weight), Fixed(u16) }` and `struct Weight { num: u16, den: u16 }`, always reduced. `Column` gains `heights: Vec<PaneHeight>`, kept the same length as `panes`. Every change to `panes` in `layout.rs` (`Column::new`, push on consume, insert and remove) goes through small `Column` methods that update both vectors together. Those methods give an entering pane `Auto(1)`, and reset the last automatic pane to weight 1, as niri's `add_tile_at` and `remove_tile_by_idx` do.

`Layout::apply` takes the screen area, `apply(action, area)`, and the server passes `self.area`. Growing or shrinking an automatic pane first converts every pane of the column to `Auto(tile rows / median tile rows)`. The pane then becomes `Fixed` at its current tile height plus or minus the step, clamped as the "Pane heights" requirement says. `geometry::tiles` clamps the fixed pane, then shares the remaining rows among the automatic panes by weight, raising any share under 3 rows. It scales the weights to their least common denominator in `u64`, so the shares are exact integer divisions. `LayoutEvent::PaneHeightsChanged { workspace, column, heights: Vec<PaneHeight> }` reports any change to a column's `heights`. `LayoutEvent` loses `Copy`, and `SessionEvent` is already only `Clone`.

Three deliberate differences from niri, none visible above one row:
- niri rounds each automatic height in turn, in pixels. gband keeps its existing rule: round down, then one leftover row each to the first panes. Automatic heights of weight 1 then give exactly today's split.
- niri measures a fixed window's step from its stored height. gband measures from the tile height the pane has now. The two differ only after the area shrank below a fixed height, and measuring the visible tile makes the first press take effect.
- niri's floor is a window's minimum size. gband's is 3 rows.

Alternatives:
- Integer weights that each press raises by 1, the first draft of this change. Rejected. Each press moved the pane by a smaller share of the column, unlike niri's constant 10% of the height.
- Fixed heights as a proportion of the area's height. Rejected. niri keeps a fixed window's pixels when the output changes, and gband likewise keeps a fixed pane's rows when another client changes the area. A rows-over-area fraction also does not fit `Proportion`'s `u8` fields.
- `f64` weights, as niri has. Rejected. `Layout` derives `Eq`, the "no change, no event" rule compares layouts, and scenarios such as weight 10/7 must hold exactly.
- `panes: Vec<StackedPane { id, height }>`. Rejected. Every reader of `column.panes` in the client, the server, the view and the tests would change, for no behavioural gain.

### New actions

`SessionAction` gains `StepWidth { pane, step }`, `StepHeight { pane, step }` and `ResetHeight(PaneId)`. `SessionCommand` gains `StepWidth(Step)`, `StepHeight(Step)` and `ResetHeight`, and `View::resolve` maps them to the focused pane. `Layout::apply` handles all three, so the server's `act` needs no new arm, only the area argument. The bindings table adds `-`, `=`, `_`, `+` and `R`. The existing `matches` ignores Shift for character keys, so `+`, `_` and `R` match however the terminal reports Shift.

### One quiet period in the session task

`Session` gains `settle_at: Option<Instant>`. Every change of the area, the layout or the shown set sets it to `now + SETTLE`, where `SETTLE` is 100 ms. `publish` stops resizing and only sends the state. `drive` adds a `sleep_until(settle_at)` arm, enabled while it is `Some`. That arm calls `Session::settle`, which clears it and calls `Pane::resize` with each shown pane's terminal size. `Pane::resize` still skips an unchanged size, so only panes that differ get a SIGWINCH. A pane spawned by `open` is created at its terminal size, as today.

`Command::Area`'s `applied` reply keeps its meaning: the area is recorded and published. The attach path waits on it before its first sync, so the first layout carries the client's area. The first snapshot may still carry the old PTY size, and the next sync after the settle sends a snapshot at the new size, as any size change already does.

Alternatives:
- Debounce in the client before sending `Resize`. Rejected. The layout would lag too, and it would cover neither session actions nor other clients.
- A timer per pane. Rejected. The session task already serialises every change, so one deadline gives the same result with one timer.

### Clients report what they show

`View::shown(&self, scene) -> Vec<PaneId>` returns the panes of the viewed workspace whose tile meets the terminal, in layout order. `Scene::viewport_cols: u16` becomes `viewport: Size`, because the bottom edge matters as well as the right one. The client's `Display` keeps the last set it sent. After each batch of server messages, each key or view action, and each terminal resize, the attach loop computes `shown` and sends `ClientMessage::Shown(panes)` when it differs.

On the server, `dispatch` turns `Shown` into `Command::Shown { client, panes }`. `Attachment::drop` sends `Command::Shown { client, panes: Vec::new() }`, so a detach or disconnect withdraws the client's panes on every exit path. The session keeps `HashMap<u64, HashSet<PaneId>>`, drops identifiers it does not hold, and sets `settle_at` when a client's set changes. `settle` resizes only panes in the union.

Alternative: clients report their viewed workspace and camera, and the server computes visibility. Rejected. The server would need each client's terminal size as well as the area, and would duplicate the client's camera rule.

### Protocol version 4

`PROTOCOL_VERSION` becomes 4. `ClientMessage::Shown` is appended, and `Column`'s serialised form gains `heights`. postcard has no field tags, so a version 3 peer would misread the layout. The version bump makes both sides refuse instead.

### Test harness

`gband-test-support`'s `TestClient` gains `show(&mut self, panes: &[PaneId])` and `show_all()`, which sends the panes of its latest layout. The existing server tests that assert PTY sizes call `show_all()` after attaching and after opening panes. Their `wait_for` loops already absorb the 100 ms. A new `crates/server/tests/resize.rs` covers the "Pane resizes" scenarios. It counts SIGWINCH with a bash pane running `trap 'echo W >> file' WINCH` and an interruptible `read -t` loop, so each signal runs the trap at once rather than after a blocking `sleep`.

## Risks / Trade-offs

- [A pane is shown at its old size for at least 100 ms after every change] → This is the intent. The grid is cut or padded inside the correct border, and the size settles once.
- [A client that never sends `Shown` leaves every pane unresized] → Only this client and the test harness speak the protocol, and both send it. A version 3 client is refused at the handshake.
- [Bursts longer than the quiet period, such as a slow drag, still resize at each pause] → Acceptable. Each pause is a real stopping point, and the step count drops from one per event to one per pause.
- [`heights` and `panes` drift out of step] → Both change only through `Column` methods. A core test checks the lengths after every layout operation the existing tests run.
- [A fixed pane keeps its rows when another client shrinks the area, so the column can look different on the larger terminal afterwards] → This matches niri on an output mode change, and geometry clamps the fixed pane so the others keep 3 rows each.
- [SIGWINCH counting in tests is timing-sensitive] → The tests space their messages well inside 100 ms and assert on the final size before counting. Each count waits for the size, never for a fixed sleep.

## Migration Plan

Protocol version 4 makes a running version 3 server and a new client refuse each other. The existing debug-build restart path in `client-attach` replaces a server from the same executable path. No persisted state exists, so nothing else migrates. Rollback is reverting the change.
