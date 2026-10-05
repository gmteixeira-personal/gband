## Context

This change starts after `status-line` is archived. It is written against what that change delivers:
- The private `host` table that is passed to the bundled API chunks under `crates/lua/src/runtime/gband/`.
- `host.call`, `host.after_event` and `host.state()`, and `Runtime::set_state`, which the client calls before every emit, refresh and timer run.
- `hl.lua` with group resolution into `"#rrggbb"` strings or indexes.
- `gband.ui.width` and `gband.ui.truncate`.
- `color.rs` with `ColorSupport` and the style conversion.
- `Display` with its `ribbon: Rect`.

From the archived foundation, these are in place today:
- `crates/lua`: `Dispatch { Action, Spawn, Enter }` is queued while a callback runs and returned in `Outcome::dispatched`. A `LuaAction` value's `__call` takes no argument. `Controls::apply` turns each dispatch into a `Step` through `dispatch(display, action)`.
- `crates/core`: `Action::View(ViewAction)` holds only relative moves. `SessionCommand` holds the unresolved session actions, and `View::resolve` turns them into `SessionAction`, which always names the focused pane. `View::focus_pane` already handles the server's focus message.
- `crates/server`: every pane is a `Live` entry with a PTY, a killer and an exit watch. `Command::Action` carries a focus sender only for open pane. `connection::dispatch` forwards keys and pastes to the pane's PTY writer. The session is over when the layout is empty.
- The Lua runtime is rebuilt on every reload, so nothing that must outlive a reload can live in Lua state.

## Goals / Non-Goals

**Goals:**
- One Lua window model for both kinds. Floats and plugin panes share lines, scrolling, the cursor line, keys and callbacks. They differ only in where the drawn frame goes: to `render.rs` for a float, and to the server for a plugin pane.
- Plugin panes reuse the server's screen machinery: snapshots, updates, shown panes and resizes. The server stores a screen it never interprets, as it does for a program.
- Every pane operation a plugin performs travels the same path a key binding's does, so the server needs only two new session actions.

**Non-Goals:**
- Server-side rendering of lines or highlight groups. The server receives terminal output only.
- A per-viewer color downgrade of plugin pane contents.
- Partial content updates. Each content message carries the whole screen.
- Exposing `host` primitives as public API.

## Decisions

### Targets resolve in Rust against a pushed state
The state `Runtime::set_state` stores gains the client's `Layout` (as an `Arc`), the screen area, the view's band and focused pane, the ribbon size, and the map from plugin panes to window numbers. `control.rs`, a new Rust module, installs these:
- `gband.layout()` and `gband.view()`, which build fresh tables from that state.
- `gband.pane.*` and `gband.band.view`, which validate their numbers against it.

`LuaAction`'s `__call` accepts an optional target table. For a built-in session action, it builds the resolved `SessionAction` from the target and queues a new `Dispatch::Session(SessionAction)`, which `Controls::apply` sends unchanged. Without a target, the action queues `Dispatch::Action` as today and resolves against the view.
- Alternative: resolve targets in the client after the callback. Rejected: an unknown pane must be an error at the line of the call, and only the Lua side knows that line.

### New dispatch entries
`Dispatch` gains:
- `Session(SessionAction)`.
- `Input { pane, input }`, where the input is a key or a paste. It becomes `ClientMessage::Key` or `ClientMessage::Paste` naming that pane.
- `Window(WindowRequest)`, used by `win.lua` to open or close plugin panes.

`ViewAction` gains `FocusPane(PaneId)` and `ViewBand(BandId)`, so `gband.pane.focus` and `gband.band.view` queue `Dispatch::Action(Action::View(..))`. They then reach the view through the same `dispatch` function as a binding. `View` gains `view_band(BandId)`, which shares the band-switch code with `BandDown` and `BandUp`. `send_prefix` with a target becomes `Dispatch::Input` with the prefix key. Key names for `send_keys` are parsed by `keys.rs`. `send_text` maps each character to a `Key` with no modifiers, with `\n` and `\r` mapped to Enter and `\t` to Tab.

### Core layout setters
`SessionAction` gains `SetWidth { pane, width: Proportion }` and `SetHeight { pane, height: PaneHeight }`. `Column` gains `set_width` and `set_height`. The weight normalisation at the start of `step_height` moves into a shared function, so a fixed height set by rows and one set by growing take the same path, and the clamp limits are shared too. Lua numbers become fractions through the existing option code that reads a width as the closest fraction with a denominator of at most 100.

`OpenPane` becomes `{ band, after, width: Option<Proportion>, focus: bool, content: PaneContent }` with `PaneContent::{ Program(Option<Program>), Plugin { request: u32 } }`. `Layout::open` takes the width. Bindings and `gband.spawn` send `focus: true` and `Program`, so their behavior is unchanged.

### Program-less panes on the server
`pane.rs` gains `Pane::blank(size)`: a `Pane` whose terminal has a grid and no PTY master. `resize` resizes only the grid, and `replace(output)` builds a fresh grid of the current size, feeds the output, swaps it in and bumps the generation. Because `catch_up` diffs from each client's checkpoint, a replaced screen reaches clients as an ordinary update, and as a snapshot after a resize.

`Session` keeps the plugin panes in a map of their own, `plugins: HashMap<PaneId, PluginPane { owner: u64, pane }>`, beside `panes: HashMap<PaneId, Live>`, so the code for killers, exits and SIGHUP stays untouched. `State::panes` still lists every pane's `PaneEntry` for `sync`. A plugin pane's entry has an input sender that drops what it receives, and that drop is how keys and pastes to plugin panes are discarded.
- `Command::Action` gains `client: u64`.
- Its focus sender becomes a reply sender of `Reply::{ Focus(PaneId), Opened { request, pane: Option<PaneId> } }`, which the connection writes as `ServerMessage::Focus` or `ServerMessage::Opened`.
- `Command::Content { client, pane, output }` and `Command::Leave { client }` are new. `Attachment::drop` sends `Leave`, which closes that client's plugin panes.
- `is_over` becomes "no `Live` pane remains". Ending a session drops its plugin panes first.

Alternative: a placeholder process, such as `cat`, behind a PTY. Rejected: input would reach it, its echo would fight the content, and it would keep a session alive.

### Windows live in a bundled Lua chunk
`win.lua` is a new API chunk, run with `host` like `statusline.lua`. It holds the window store: options, lines, `top`, `cursor`, kind, pane, content size, and the focused float and stacking order. It handles scrolling, the cursor line and the default keys. It lays out the visible rows into frames, using `gband.ui.width` and resolving groups through `hl.lua`. It calls the window callbacks through `host.call` with the window's owner. This matches the status line's split: the logic that needs highlight groups and widths is Lua, and Rust only draws or ships a finished frame.

New host primitives:

| primitive | purpose |
|---|---|
| `host.next_window()` | the next window number, from a counter that outlives reloads |
| `host.present_window(id, frame)` | store a float's placed box, border, title and cells, or a pane window's cells |
| `host.forget_window(id)` | drop a window's stored frame |
| `host.request(entry)` | queue `Dispatch::Window` to open or close a plugin pane |
| `host.parse_key(name)` | a canonical key string, as matching against bindings compares keys |
| `host.window_hooks(table)` | register the functions Rust calls: `opened`, `pane_resized`, `pane_closed`, `key`, `release`, `ribbon_resized`, `focused`, `pane_window`, and `flush`, which draws the windows changed during a run |

`host.after_event` becomes a list, so `win.lua` can clear the focused float on `FocusChanged` and `BandChanged`, and redraw on `HighlightChanged` and `ColorschemeChanged`, beside the status line.

When the focused float runs one of its `keys` functions, `win.lua` holds that float's focus: `FocusChanged` and `BandChanged` leave it focused. `Controls::press` releases the hold through a `release` window hook before it handles each key. A focus change that a float's own key caused, at once or through the server's later focus message, keeps the float focused. A key pressed outside the float still moves focus away from it.

- Alternative: the store in Rust with a Lua API. Rejected for the same reason the status line rejected it: highlight resolution and display widths already live in Lua, and a Rust store would need a second resolver.

### Window numbers and requests outlive the Lua state
The client owns a `Windows` value in `crates/client/src/windows.rs`, which holds:
- The next window number.
- The stacked floats, with their frames.
- The focused float.
- The map from pending request numbers to windows.
- The map from plugin panes to windows.

The request number of a plugin pane is its window number. On reload, `Controls::reload` asks `Windows` to close everything. It drops the floats, sends close pane for every known plugin pane, and marks pending requests as orphaned. An opened reply for an orphaned request is answered at once with close pane. The new runtime is seeded with the same counter, so numbers are never reused.

### Key routing
In `Controls::press`, `Command::Send(key)` first asks `Windows` for the focused window: the focused float, otherwise the window of the focused pane. When there is one, it calls `Runtime::window_key(win, key)`. That runs `win.lua`'s `key` hook, which tries the window's `keys` table, then the default keys. The key is never sent to the server. Pastes are dropped the same way. Root bindings, the prefix and named tables are handled earlier by `Leader`, so they always win.

### Drawing floats
`render::draw_frame` gains a floats pass after the tiles and before the banner. For each float in stack order, it fills the box with `Window`'s base style. It then draws the border with a ratatui `Block` styled from `WindowBorder`, the title spans, and the content cells. The frame arrives already placed: `win.lua` places boxes against the ribbon size in the pushed state, and places them again from the `ribbon_resized` hook. `render` only converts styles through `color.rs` and clips to the ribbon rectangle. The cursor is hidden whenever `Windows` has a focused float.

### Plugin pane contents
`Windows` encodes a pane window's frame as terminal output:
1. `ESC[0m ESC[2J ESC[?25l`.
2. For each row, a cursor move to its first column, then for each run of cells an SGR from the resolved style, followed by the text.
3. Rows filled with spaces in the base style.

Colors become `38;2;r;g;b` or `38;5;n`, and the same forms with 48 for backgrounds. After each runtime run, `Controls` takes the changed pane frames, keeps the latest per pane, and sends one `ClientMessage::Content` for each. When a snapshot of one of this client's plugin panes arrives with a new size, the client calls the `pane_resized` hook, which runs `on_resize` and presents a new frame at that size.

## Risks / Trade-offs

- [`status-line` may change its host table or its client-attach text before it is archived, and the client-attach delta here copies that text] → Task 1.1 diffs the archived `status-line` specs and host primitives against this change and reconciles the deltas and this design before any code.
- [A content frame encoded for the old size can reach the server after a resize] → The server feeds it into the current size and cuts it. The snapshot that follows the resize makes the owner encode and send the frame again.
- [Plugin pane contents carry truecolor SGR, and a viewer without truecolor sees the colors as its terminal maps them] → This is how a program's output behaves today. The window API documents that floats are downgraded and plugin panes are not.
- [A plugin that calls `set_lines` in a tight loop sends many content messages] → Content is taken once per runtime run and coalesced per pane, so one callback sends at most one message per pane.
- [An owner that hangs without disconnecting keeps its plugin panes alive] → This is the same exposure as a hung program. The user can close the pane with `close_pane` from any client.
- [Large line lists] → Only visible rows are laid out, so a frame costs its rows, not the list's length. Lines run under the callback's instruction limit.

## Migration Plan

The protocol version becomes 5. The handshake refuses a version 4 client meeting a version 5 server, and the reverse. The client handles that refusal as it handled the version 3 to 4 bump. No configuration changes: existing bindings and `gband.spawn` calls behave as before. Rollback is reverting the change.
