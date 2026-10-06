## Context

See proposal.md for why. The code today:

- **Client input.** One thread reads `crossterm::event::read()` into a channel (`crates/client/src/lib.rs:1266`). The attach loop handles `Key`, `Paste`, `Resize` and the focus marker, and drops `Event::Mouse` (`lib.rs:1259`). `TerminalGuard::enter` enables only bracketed paste (`lib.rs:132`).
- **Key path.** `Leader::handle` (`crates/client/src/bindings.rs`) matches a `Key` against `Chord::Key | Chord::Prefix` (`crates/lua/src/api.rs:16`). It returns `Send`, `Run` or `Discard`. Keys are encoded on the server: `ClientMessage::Key` becomes `Input::Key` in the PTY writer (`crates/server/src/window.rs:293`), encoded with `encode_key(key, modes)`. `Modes` holds only DECCKM and bracketed paste (`crates/core/src/input.rs:76`).
- **Emulator.** vt100 0.16 tracks mouse tracking mode and encoding, and `state_formatted` and `state_diff` carry them. The client's grids, rebuilt from snapshots and updates, therefore already know each window's mouse mode. Scrollback length is 0.
- **Geometry.** The client computes every screen rectangle itself inside `render()` (`crates/client/src/render.rs:159`), from `Drawn` (`animation.rs:218`), the view's camera and the floating stacking order. There is no hit test. `WindowBox::contains` exists (`crates/core/src/geometry.rs:77`).
- **Actions.** `SetWidth`, `SetHeight` and `SetPosition` already exist as session actions, reached only from Lua. There is no action that inserts a window at an arbitrary place. The camera is client-only (`BandView.camera`, `crates/core/src/view.rs:39`) and moves only through `settle()` and `CenterColumn`.
- **Clipboard.** `gband.clipboard` writes OSC 52 through `Dispatch::Write` (`crates/lua/src/bridge.rs:247`).
- **Plugin windows.** Floating plugin windows live only in Lua (`crates/lua/src/runtime/gband/win.lua`) with `row`, `col`, `width` and `height` in ribbon cells. Tiled ones are layout windows.
- **Other changes.** navigation-mode, a dependency, makes `prefix` a mode. `Leader` gains mode handling there, and this change builds on it. sidebars-borders-steps, loop-bands and lua-prompt change files this change also touches: the ribbon's origin, the strip's wrapping, and `win.lua`. None of them is a dependency.

## Goals / Non-Goals

**Goals:**
- One hit test that always agrees with the frame on screen, whatever the animation, side bars or looping strip draw.
- Mouse buttons ride the existing key-table machinery, so modes, user bindings and Lua functions work for the mouse with no second dispatcher.
- Gestures are client-side state machines that only ever send existing or new session actions. The server learns of no gesture.

**Non-Goals:**
- A server-side notion of a pointer, or showing one client's drag to another client before it sends a change.
- Scrollback, which a later change can add without touching the gesture code.

## Decisions

### Mouse names are a chord kind, not new key codes
Add `MouseKey { trigger, modifiers }` to `crates/core/src/input.rs`, where `trigger` is `Left | Middle | Right`. The wheel has no `MouseKey`. Add `Chord::Mouse(MouseKey)` beside `Chord::Key`. `parse_key` in `crates/lua/src/keys.rs` returns a small enum of either a key or a mouse key. Every caller that must reject mouse names says so explicitly: the `prefix` option and plugin windows' `keys`.

`Leader` gains `handle_mouse(keymap, MouseKey) -> Command` with the same table and mode rules as `handle`. `Command::Send` becomes "apply the default" for a mouse key.

*Alternative:* add mouse variants to `KeyCode`. Rejected because `Key` is serialized in `ClientMessage::Key`, so a mouse "key" could then reach the server as keyboard input, and every exhaustive match over `KeyCode`, including the encoder, would need a dead arm.

### Hit-testing from the regions the renderer draws
Extract the rectangle computation from `render()` into a pure function, `regions(drawn, view, plugin floats, ribbon) -> Vec<Region>`. Each `Region` carries its target, box, content rectangle and z order. `render()` draws from it. `Display` keeps the last presented regions, and the hit test walks them top-down.

The drop place for a lifted tile is also computed from the drawn column spans in the regions, not from strip arithmetic. Side bars, a looping strip or animation offsets then cannot make the click and the picture disagree.

*Alternative:* recompute geometry from layout and view at click time. Rejected because it disagrees with the screen mid-animation, and it would need to replicate loop-bands' and sidebars' drawing rules.

### A client `mouse` module owns pointer state
Add `crates/client/src/mouse.rs`, which holds:
- the button-down state and its grabbing window, for forwarding;
- the selection: window, anchor and head in content cells;
- the copy buffer, which lives outside the Lua runtime so a reload keeps it;
- the active gesture, an enum `Move`, `Lift`, `Resize` or `Slide` holding the press geometry snapshot.

`Controls` routes `Event::Mouse` there:
1. Convert the event and resolve its target.
2. Run `Leader::handle_mouse` on presses only.
3. Run the binding, the default or the gesture.
4. Queue the Lua mouse event.

A wheel step skips the leader and the gesture. It goes straight to its target's window as a `ClientMessage::Mouse`, or to the plugin window's hook.

Motion events are coalesced: an event whose cell equals the previous one is dropped before any work. Mode 1003 reports every pixel row on some terminals.

The drag actions are `ClientAction::DragWindow`, `DragResize` and `DragBand`. `dispatch()` reads the "current press" slot, which is set only while a press is being handled. Outside a press the slot is empty and the actions do nothing.

### Gestures send at most one change per frame
A gesture computes its target geometry on every motion but writes it to a pending slot. The frame tick flushes the slot by sending `SetPosition`, `SetWidth` or `SetHeight` only when the value differs from the last one sent. Live resizing then costs one `SIGWINCH` per frame at most.

Widths are sent as `cells / area_width` in lowest terms, using the session's screen area from the layout, as floating placement already does. Each sent width then maps back to exactly the intended cell count.

The left-edge resize of a tile and the band slide move the camera. Both set a camera override on the viewed band that `settle()` leaves alone until the gesture ends. At the release, the slide applies the release rule from the mouse spec. The resize clears the override and lets the policy apply from the moved camera.

### Moving to a place is one new session action
`SessionAction::MoveToPlace { window, reference, place: ColumnLeft | ColumnRight | Above | Below }` is applied in `Layout::apply` (`crates/core/src/layout.rs`) as "remove as close does, then insert".

Naming a reference window, rather than column indices, makes the action robust to concurrent edits. If the reference window has gone, the existing "names a window no longer in the layout" rule drops the action.

*Alternative:* chain the existing move-column, consume and expel actions from the client. Rejected because it needs several round trips, animates through intermediate layouts, and races with other clients.

### Mouse reports are encoded on the server
`ClientMessage::Mouse { window, event }` carries the event in content cells. The server's PTY writer gains `Input::Mouse` and encodes it with `encode_mouse(event, modes)` in `crates/core/src/input.rs`. `Modes` gains `mouse_tracking` and `mouse_encoding`, filled from the vt100 screen in `crates/emulator/src/lib.rs`.

This mirrors keys: the server's grid is authoritative for modes at write time. The client still reads its own grid's mode to decide between forwarding and selecting, because that decision is UI and must not wait for a round trip. A mode change racing a click can at worst produce one dropped report.

### Alt, not Shift, overrides forwarding
Kitty, Alacritty, WezTerm, foot and VTE keep Shift with a mouse event for their own selection and never report it while mouse reporting is on. A Shift override would therefore never reach gband. Alt reaches gband in those terminals, and Shift+drag stays the terminal's native selection, which is a second, independent way to copy.

### The wheel bypasses gband
Every wheel step is sent to the window under the pointer, whatever the mode, the modifiers or a running gesture. gband binds nothing to it, so a program never loses a scroll to gband. The client does not check the window's mouse mode before sending a wheel step. The server's encoder already drops steps a mode does not report, and checking twice would only add a way for the two sides to disagree.

Plugin windows keep the wheel: they have no program to receive it. A floating plugin window scrolls, or passes the step to its `on_mouse`. `MouseScrolled` lets plugins observe the wheel without taking it, since events never consume.

*Alternative:* niri's Mod+wheel band and column switching in navigation mode. Rejected: the wheel must always reach programs.

### Mouse capture is written explicitly
The client writes `?1000h ?1002h ?1003h ?1006h` itself rather than crossterm's `EnableMouseCapture`, which also enables 1015 (urxvt). It disables them in reverse in `TerminalGuard`'s drop, before leaving the alternate screen, so the spec'd bytes are exact. crossterm still parses the SGR reports.

### Selection text comes from the emulator
Add `Emulator::text_between(start, end) -> String` backed by vt100's `contents_between`. vt100 already joins wrapped rows without a newline and trims trailing blanks per row. Tests pin both rules from the spec. Selection drawing swaps foreground and background of the selected cells after the tile's content is drawn, in this client's render only.

### Plugin windows: mouse defaults in Lua
`win.lua` gains `on_mouse` validation and a `hooks.mouse(id, event)` entry that the client calls for plugin window targets. With no `on_mouse`, the hook applies the defaults by reusing `move()`, the code that serves Up and Down, and a "set cursor to line" helper. Moving and resizing floating plugin windows reuses `set_config`'s placement and its `on_resize` dispatch. The gesture calls a host function that sets `row`, `col`, `width` and `height` in one step, so `on_resize` runs once per new size.

### Hints and key list skip mouse bindings in Lua
`gband.keymap.list` entries are unchanged. A predicate in the existing `gband.keyform` module, `is_mouse(name)`, tests the canonical key name. `statusline/hints.lua` and `keylist.lua` use it. This keeps navigation-mode's and key-list-shortcuts' code paths intact apart from one filter each.

## Risks / Trade-offs

- **Lost native selection.** While attached, a plain drag in the host terminal no longer selects. → Shift+drag still selects natively in the common terminals, and gband's own selection copies with OSC 52. The README says so.
- **OSC 52 disabled in the host terminal.** Copying then reaches only the copy buffer. → Right-click paste still works inside gband. The README names the terminal settings to check.
- **Contended specs.** sidebars-borders-steps also modifies wire-protocol's "Client messages" and "Handshake". Whichever change archives second replaces the requirement text wholesale. → Before `/ready`, rebuild both MODIFIED blocks from the then-current main spec and re-apply this change's edits. Raise the protocol version to one above `dev`'s value at that time, 9 if sidebars-borders-steps landed first.
- **loop-bands and sidebars change the ribbon.** → The hit test and the drop place read rendered regions, so they follow whichever lands first. Whichever lands second only needs to keep `regions()` as the renderer's source of rectangles.
- **Event volume.** Any-motion tracking floods events. → Same-cell motion is dropped at intake, and gesture sends are flushed per frame.
- **Resizing a program live sends many SIGWINCH.** → At most one per frame, and only on a cell change.
- **A gesture racing another client's change.** → A gesture whose window leaves the layout ends at once. Size and position sends are absolute, so the last one wins cleanly.

## Migration Plan

The protocol version bump makes old clients refuse new servers through the existing handshake, and the running server's mismatch note applies. No configuration migration is needed. User configurations gain the navigation mode mouse bindings from the defaults unless they replace the `prefix` table.
