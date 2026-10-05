# gband-core coverage

Measured with `cargo llvm-cov -p gband-core --show-missing-lines` (cargo-llvm-cov 0.9.1, Fedora `llvm` 22, rustc 1.98.1). On a toolchain without rustup, set `LLVM_COV=/usr/bin/llvm-cov` and `LLVM_PROFDATA=/usr/bin/llvm-profdata`.

## Baseline

| file | lines | missed lines | regions | missed regions |
|---|---|---|---|---|
| geometry.rs | 92.00% | 6 | 96.39% | 3 |
| input.rs | 96.84% | 3 | 97.83% | 4 |
| layout.rs | 94.33% | 14 | 94.75% | 18 |
| view.rs | 97.65% | 4 | 96.28% | 12 |
| total | 95.40% | 27 | 96.03% | 37 |

Uncovered lines:

- `geometry.rs`: 44-49, `Tile::span`
- `input.rs`: 14-16, `Key::plain`
- `layout.rs`: 14-16 and 20-22, the `Display` impls of `PaneId` and `WorkspaceId`; 37-39, `Proportion::new` called at run time; 162-164, `Layout::default`; 275, `apply` with `OpenPane` or `ClosePane`; 295, a width change that changes nothing
- `view.rs`: 93-94, `apply` after the viewed workspace has gone; 128, `focus_pane` on an unknown pane; 155, a stale focus on a workspace with no columns

## Final

| file | lines | missed lines | regions | missed regions |
|---|---|---|---|---|
| geometry.rs | 100.00% | 0 | 100.00% | 0 |
| input.rs | 100.00% | 0 | 100.00% | 0 |
| layout.rs | 99.60% | 1 | 99.71% | 1 |
| view.rs | 99.41% | 1 | 99.07% | 3 |
| total | 99.66% | 2 | 99.57% | 4 |

Functions: 100%, up from 93.62%. Tests: 86, up from 70.

## Unreachable from the public API

No test covers these. They are dead code for any layout built through `Layout`'s methods. The code is unchanged, because this change modifies nothing in `src/`.

- `layout.rs:295`, `with_column` returning no events when a width change changes nothing. Toggling full width always flips the flag. Cycling always clears the flag and picks a preset strictly larger than the current width, or one third when none is larger, which needs a current width of at least two thirds. So the column always changes, even for a deserialized width with a zero denominator.
- `view.rs:155` and the `?` at `view.rs:187`, a remembered focus on a workspace that has no columns. A view only remembers a pane that its workspace held at the time, so that workspace was not the last one. The layout removes every empty workspace except the last, and only ever adds workspaces at the end. A workspace that held a pane therefore never comes back empty, and `clamped` never meets an empty workspace.
- Cycling from a width that is not a preset, dropped from the tasks. A column's width only ever comes from `Column::new` (one half) or from cycling (a preset), and `Layout` keeps its workspaces private. Only a deserialized `Layout` could hold another width, and testing that needs a serde format as a new dev-dependency. resize-semantics replaces this cycling with stepped widths.

## Pinned behaviour

No spec states these. The tests pin what the code does now.

- A view action applied after the viewed workspace was removed only moves the view to the workspace in its place. The action itself is dropped (`view_action_after_the_viewed_workspace_is_removed_only_moves_the_view`).
- A focus move applied after the focused pane closed, without a sync in between, is dropped. Focus lands where a sync would put it, so `FocusLeft` lands on the right-hand neighbour (`focus_move_after_an_unsynced_close_lands_where_a_sync_would`).

## Specified, but worth a look

- A column holding more panes than the area has rows gives the extra panes tiles 0 rows high, placed below the area (`panes_beyond_the_area_rows_get_no_height`). This follows the layout spec's height rule. Each such pane's terminal is still 1 row, because a terminal size is at least 1 by 1.

## Contradictions

None. Every new test passed against the current code.
