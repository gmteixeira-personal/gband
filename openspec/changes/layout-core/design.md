## Context

The proposal gives the motivation. The server serves exactly one pane today: `crates/server/src/pane.rs` spawns one program into one PTY and one `vt100::Parser`, with a session-wide generation `watch` that wakes every client task. `crates/server/src/connection.rs` sends one snapshot and then diffs of that one screen. The client (`crates/client/src/lib.rs`) holds one parser and draws it full screen with `tui-term`'s `PseudoTerminal`. `crates/client/src/prefix.rs` is a two-entry prefix machine. `gband-core` holds only the key encoder, and depends on nothing but `serde` and `thiserror`.

The facts this design relies on are unchanged from `server-client-split`. `vt100::Screen` is `Clone`, `state_formatted()` reproduces a screen from scratch, and `state_diff(&prev)` reproduces it from `prev` when both have the same size. `postcard` is not self-describing, so any message change bumps `PROTOCOL_VERSION`.

## Goals / Non-Goals

**Goals:**
- A layout model in `gband-core` with no I/O, covered by unit tests that need no PTY, socket or terminal.
- One authoritative layout per session on the server, and an independent view per client.
- Ribbon rendering with edge clipping that can be tested against a `ratatui` `Buffer` without a terminal.
- A binding table shaped so that Lua can later replace it without touching the dispatch code.

**Non-Goals:**
- Animation. The camera jumps to its new position.
- Keeping a client's view across a detach. Every attach starts from the initial view.
- New panes starting in the focused pane's directory. Every pane starts in the server's directory.
- A vertical workspace transition. Only the viewed workspace is drawn.

## Decisions

### The core crate: data, geometry and view as separate modules

`gband-core` gains three modules, all pure and all `serde`-derived where they cross the wire:

| module | holds |
|---|---|
| `layout` | `PaneId`, `WorkspaceId`, `Proportion`, `ColumnWidth`, `Column`, `Workspace`, `Layout`, `Direction`, `SessionAction`, and the operations that change a layout |
| `geometry` | `Size`, `Tile`, and `tiles(&Workspace, area) -> Vec<Tile>`: each pane's strip rectangle and terminal size |
| `view` | `View`, `ViewAction`, focus movement, reconciliation after a layout change, and the camera |

`Layout` is the whole session state the server shares. It holds the workspaces and the counters that allocate pane and workspace identifiers, so that identifiers are never reused. The server allocates a pane identifier before spawning, because the program's environment carries `GBAND_PANE`. `Layout::apply(SessionAction)` covers every change except opening a pane, which is `Layout::open(PaneId, WorkspaceId, Option<PaneId>)` because the server must spawn the program first. `Layout::remove(PaneId)` runs when a program exits. Every operation re-establishes the two invariants, no empty column and exactly one trailing empty workspace, before it returns. Tests can then assert the invariants after any sequence of operations.

**Widths are exact fractions.** `Proportion { num: u8, den: u8 }` compares by cross-multiplication. The presets and the "smallest preset larger than the current width" rule are then exact, and tests compare equal values instead of floats. A fixed-cell width can later be added as a second `ColumnWidth` variant.

**Geometry is a function of a workspace and an area**, not stored state. The server calls it to size PTYs, and the client calls it, with the area from the latest layout message, to place tiles. Both sides compute identical tiles from identical inputs, so tile sizes never travel on the wire.

**The view lives in core even though only the client uses it.** Its rules are pure, and most of this change's edge cases sit in them: remembered focus, the removed focused pane and the removed viewed workspace. Testing them in core keeps the client loop thin. `View` keeps:

- the viewed `WorkspaceId`.
- per workspace, the focused `PaneId` and the camera position.
- a focus position (workspace index, column index, row index), refreshed whenever focus is set. When the focused pane vanishes, reconciliation clamps this position into the new layout.
- a monotonic counter with the tick at which each pane was last focused. Focusing a column picks its most recent pane.

`View::sync(&Layout, area, viewport_cols)` runs after every layout message and every terminal resize. It reconciles the workspace and focus, then runs the camera rule. `View::apply(ViewAction, ...)` and `View::focus_pane(PaneId, ...)` end the same way. Alternative: hold focus on the server per client. Rejected, because view actions would then wait for a round trip, which the summary's split between view and session exists to avoid.

### Session actions carry their target

A session action names its pane, or for opening a pane the workspace and the pane to open after: `SessionAction::{OpenPane { workspace, after }, ClosePane(pane), ConsumeOrExpel { pane, direction }, CycleWidth(pane), ToggleFullWidth(pane)}`. The client resolves its own focus into the action before sending it. Focus is per client, so the server has no "focused pane" of its own to apply an action to. An action whose target has left the layout by the time it arrives is ignored, because a stale target is a race, not an error.

### Wire protocol version 2

```rust
enum ClientMessage { Key { pane: PaneId, key: Key }, Paste { pane: PaneId, text: String }, Resize { cols, rows }, Action(SessionAction), Detach }
enum ServerMessage { Info { .. }, Layout { cols, rows, layout: Layout }, Snapshot { pane, cols, rows, contents }, Update { pane, contents }, Focus(PaneId), Exited }
```

`Hello` and `HelloReply` are untouched, and their pinned-bytes test stays as it is. `PROTOCOL_VERSION` becomes 2. A debug client then replaces a version 1 server through the existing handshake path, and a release client reports the mismatch.

The whole layout travels on every change, rather than as a diff. It is a few hundred bytes for any realistic session, it changes only on user actions and program exits, and a full value cannot drift out of step the way a stream of edits can.

### Server structure

```
pane threads (per pane) ──bytes──▶ Pane { Mutex<Parser>, generation: AtomicU64 } ──bump──▶ watch<u64> changed
pane exit thread ──PaneExited(id)──▶ ┐
client tasks ──Hello/Resize/Action──▶ session task ──publish──▶ watch<Arc<State>>  (layout, area, panes)
SIGTERM ──Terminate──────────────────┘          └──oneshot Focus(id)──▶ requesting client task
client task: select { changed, focus reply, ended, frame read }
```

- **One session task owns the layout.** It receives commands over an `mpsc` channel: a client's size, a session action with an optional focus reply channel, a pane exit, and SIGTERM. Serialising every change through one task gives the single order across clients that the spec requires, without holding a lock across a spawn. After each change it recomputes tiles for the area. It resizes only the panes whose terminal size changed, then publishes a new `Arc<State>` and bumps `changed`.
- **Panes keep their threads.** Each pane has a reader thread, a child-wait thread and an input thread, as the one pane has today. The child-wait thread waits for the reader to drain, for up to 500 ms, then sends `PaneExited`. The session task removes the pane, and when no pane remains it sets `ended`. The existing farewell and socket-removal path then runs unchanged.
- **Keys bypass the session task.** A client task looks the pane up in the latest published `State` and sends to that pane's input channel. Keys are the hot path, and per-pane order holds because each pane has one input channel. A key for a pane missing from `State` is dropped.
- **Each client task tracks what it last sent**: the `Arc<State>` pointer, and for each pane its generation and `Screen`. On every wake it sends `Layout` if the state pointer changed. For each pane it then sends a snapshot when the pane is new to this client or its size changed, or an update when the pane's generation moved and the diff is not empty. Pane generations let a wake skip panes that did not change, so one busy pane does not make every client diff every pane. Coalescing through `watch` works per client exactly as before.
- **The focus reply** is a `oneshot` the session task completes after it publishes the state holding the new pane. On receiving it, the client task first runs its normal sync, which sends the layout and the new pane's snapshot, and then sends `Focus`. The client therefore always knows the pane before it is told to focus it.
- **Closing a pane** sends SIGHUP through the pane's `ChildKiller` and starts the existing `kill_if_running` grace timer for that pane. **SIGTERM** does the same for every pane. Both then wait for the ordinary `PaneExited` path, so a closed pane and an exited one leave the layout the same way.
- **The screen area** is the most recent hello or resize from any client, and starts at 80×24. This replaces the per-PTY rule. `Pane::resize` keeps its skip-if-unchanged check, so a program sees SIGWINCH only when its own tile changed.

### Client structure

- **`bindings.rs` replaces `prefix.rs`.** It holds `const BINDINGS: &[(KeyCode, Modifiers, Binding)]` and a two-state `Leader` machine. The machine returns `Command::{Send(Key), View(ViewAction), Session(SessionCommand), Detach, SendPrefix, Discard}`. `SessionCommand` is an action without a target, and the main loop resolves it against the view. A `Char` entry matches on character, Ctrl and Alt and ignores Shift, so `D` matches whether or not a terminal reports Shift with an uppercase letter. The table is data, and a later Lua change replaces where it comes from rather than how it is matched.
- **The client keeps** `HashMap<PaneId, vt100::Parser>`, the latest `(Layout, area)` and a `View`. A `Layout` message replaces the layout, drops parsers for panes it no longer holds, and calls `view.sync`. `Focus` calls `view.focus_pane`. A terminal resize sends `Resize` and calls `view.sync` with the new width.
- **`render.rs` draws into a `Buffer` and returns the cursor.** For each tile of the viewed workspace that overlaps `[camera, camera + width)`, it renders a bordered `Block` and a `PseudoTerminal` into a scratch `Buffer` of the tile's full size at the origin. It then copies the cells that fall inside the terminal into the frame buffer at the tile's x position less the camera. Rendering the whole tile and then cutting it keeps clipping exact for borders and wide characters alike. It also avoids teaching `PseudoTerminal` about offsets, which it does not support. The cost is one scratch render for each visible tile per frame, and few tiles are ever visible. The focused tile's border is bold in the terminal's default foreground, and other borders are dim, so the distinction holds in any colour scheme without a theme. Alternative: one-cell separators instead of borders. Rejected, because adjacent stacked and side-by-side panes would then be indistinguishable without colour, and the summary plans column chrome later anyway.

### Testing

- `crates/core/tests/layout.rs`: every layout scenario in the specs, plus invariant checks after random-ish sequences of open, remove, consume and expel built from a fixed seed table, with no new dependency.
- `crates/core/tests/geometry.rs` and `crates/core/tests/view.rs`: the tile and camera scenarios with their exact numbers, focus memory, and reconciliation after removals.
- `crates/protocol/tests/messages.rs`: round trips of every new message, including a two-workspace layout. The existing hello pinned bytes are kept.
- `crates/server/tests/`: the harness's `TestClient` keeps a parser per pane and the latest layout. New tests in `crates/server/tests/panes.rs` cover opening a pane, the focus reply going only to its requester, `GBAND_PANE`, keys reaching only their pane, closing with SIGHUP and the SIGKILL fallback, one of two shells exiting, the area rule resizing only changed panes, and width cycling resizing the PTY. The existing sync and lifecycle tests move to pane-addressed messages.
- `crates/client/src/bindings.rs` unit tests: the table, `D` with and without Shift, `d` discarded, Ctrl+A twice. `crates/client/tests/render.rs`: two columns side by side, left-edge clipping, a tile taller than the terminal, an empty workspace, and the focused border style, all against a `Buffer`.
- `tests/attach.rs`: detach becomes `\x01D`. New end-to-end tests open a pane with `\x01\r`, check `echo $GBAND_PANE` in it, focus left with `\x01h` and type there, close with `\x01q`, and move to the second workspace and back.

## Risks / Trade-offs

- [The first pane fills only the left half of the terminal, as niri's default column width does] → Leader `f` makes it full width, and a configurable default width comes with Lua.
- [Borders cost two columns and two rows per pane] → They make focus and pane edges visible without colour. A borderless option can come with the theme work.
- [A client whose terminal differs from the screen area sees tiles that do not fit] → The same trade as today's "pane larger than the terminal": the latest client decides, and others see a clipped or padded view.
- [Every attach resets focus to the first workspace's first pane] → Accepted for now. Restoring a view needs per-client identity, which arrives with reconnection.
- [Closing the last pane of a middle workspace removes it at once, while niri keeps a focused empty workspace until focus leaves] → The server does not know which workspace each client views. The client lands on the workspace that took its place, which is usually the empty one where a new pane can open.
- [A burst of output from many panes wakes every client task for every pane] → Pane generations make each wake skip unchanged panes, so the cost follows the panes that changed.
- [Protocol 2 makes every running version 1 session unreachable from a new release client] → The release client already names `gband kill-server` in its error. No release has shipped.

## Migration Plan

A debug client replaces a version 1 server on attach, and its shell is lost, as the existing build-mismatch behaviour intends. Rollback is reverting the change.
