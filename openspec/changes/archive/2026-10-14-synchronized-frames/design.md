## Context

`draw` at `crates/client/src/lib.rs:2258` calls `terminal.draw` on a `DefaultTerminal`, a ratatui `Terminal<CrosstermBackend<Stdout>>` made by `ratatui::init()`. `Display::draw` sets the frame's cursor in `crates/client/src/render.rs:166` when the ribbon presentation shows it, and leaves it unset otherwise.

ratatui-core 0.1.2's `apply_buffer_with_cursor` (`src/terminal/render.rs:288`) runs in this order:

1. `Terminal::flush` sends the buffer diff through `CrosstermBackend::draw`, which only queues: a `MoveTo` per run of changed cells, colours, attributes and symbols.
2. With a frame cursor it calls `show_cursor`, then `set_cursor_position`. Without one it calls `hide_cursor`. ratatui-crossterm 0.1.2 implements each with `execute!`, so each writes and flushes on its own.
3. `backend.flush()`.

Standard output is a `LineWriter` with a 1 KiB buffer, so a frame larger than that reaches the terminal in several writes even before step 2. Nothing in the client writes `\e[?2026h`. `TerminalGuard` in the same file enables bracketed paste and mouse reporting on entry, and its `drop` disables them, calls `ratatui::restore()` and shows the cursor.

crossterm 0.29's `terminal::BeginSynchronizedUpdate` and `EndSynchronizedUpdate` write `\e[?2026h` and `\e[?2026l`.

## Goals / Non-Goals

**Goals:**
- A terminal with mode 2026 shows each frame in one paint, cursor included.
- A terminal without mode 2026 never shows the cursor on a changed cell while a frame is written.

**Non-Goals:**
- Fewer frames. The client still draws on every screen change, and ratatui's diff still limits each frame to changed cells.
- Detecting mode 2026 support with a DECRQM query. Unsupported terminals ignore the markers.
- Changing ratatui's Show-then-MoveTo order for terminals without mode 2026.

## Decisions

### Markers around `terminal.draw` in `draw`

The wrapper queues `BeginSynchronizedUpdate` and `Hide` on the backend's writer, calls `terminal.draw`, then queues `EndSynchronizedUpdate` and flushes, whether `terminal.draw` succeeded or not. The draw's error is returned after the end marker; an error from writing the end marker is returned only when the draw succeeded.

`draw` is the only caller of `terminal.draw`, so one place covers every frame, resizes included: ratatui's autoresize clears the screen inside `terminal.draw`, and that clear now lands inside the same update as the redrawn frame.

Alternative: a custom ratatui `Backend` that wraps `CrosstermBackend` and adds the markers in its own `flush`. ratatui calls `backend.flush` several times per frame through the `execute!` calls, so the wrapper would need to track frame boundaries itself. Rejected as more code for the same bytes.

### Hide before the frame

`Hide` goes right after the begin marker. ratatui then shows the cursor only in step 2, after every changed cell is written. With mode 2026 the Hide makes no visible difference. Without it, the cursor is hidden while ratatui walks the changed cells, which removes most of the flashing.

ratatui still sends Show before `MoveTo`, so without mode 2026 the cursor can appear for one paint on the last changed cell. Moving cursor placement out of ratatui (leave the frame cursor unset, then queue `MoveTo` and `Show` in `draw`) would close that gap too. It is left out: Ghostty, kitty, WezTerm, foot, Alacritty and iTerm2 support mode 2026, and the gap is two consecutive writes.

### A public wrapper over any writer

The markers live in a new public function, `synchronized`, generic over `W: Write`, that takes a `Terminal<CrosstermBackend<W>>` and a render callback. `draw` calls it with `|frame| display.draw(frame, now)`. A test in `crates/client/tests/frames.rs` then needs no `Display`: it builds `Terminal::with_options(CrosstermBackend::new(Vec::new()), TerminalOptions { viewport: Viewport::Fixed(area) })`, which queries no terminal size, renders a callback that sets cells and the cursor, and reads the bytes from `terminal.backend().writer()`. The failure case uses a writer that returns an error once it has accepted a set number of bytes, and records what it accepted.

Alternative: a `#[cfg(test)]` module in `lib.rs` around a private `draw`. The client crate keeps its rendering tests under `crates/client/tests/`, so the wrapper is public instead.

### End marker on exit

`TerminalGuard::drop` writes `EndSynchronizedUpdate` first, before mouse reporting is turned off. It runs after a panic inside `terminal.draw` too, where `draw` never reached its own end marker. Writing `\e[?2026l` with no update open is a no-op in every terminal that supports the mode.

## Risks / Trade-offs

- [A terminal that supports mode 2026 holds output until the end marker] → `draw` writes the end marker on every return path, `TerminalGuard` writes it on exit, and Ghostty and kitty end a held update on their own after a timeout.
- [tmux or another multiplexer between gband and the terminal may drop or forward the markers] → Either way the frame arrives, as it does today.
- [Hide on every frame adds 6 bytes per frame] → Negligible beside the cell bytes of any frame that changes a cell.
