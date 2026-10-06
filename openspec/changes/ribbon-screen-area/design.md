## Context

`Display` in `crates/client/src/lib.rs` keeps `terminal`, the terminal's size, and `ribbon`, the rectangle `place_bars` leaves. `reported_size()` returns `terminal`, and `set_size` returns a size to report only when the terminal itself changed size. `set_bars` snaps the view when the ribbon moves and never reports. `connect` reads `crossterm::terminal::size()` and sends it in the `Hello`. The server takes whatever size the hello and each `Resize` carry as the session's screen area, so it needs no change.

The configuration is loaded in `src/main.rs` before `gband_client::run` connects, so the bars that `setup` calls add, including the default sidebar's, already exist in the Lua runtime when the handshake is sent. They reach `Display` only through `Controls`, via `runtime.take_bars()`, after `attach` builds the display.

## Goals / Non-Goals

**Goals:**
- One rule for the reported size: the ribbon area's size, from the hello on.
- Every ribbon size change becomes exactly one `Resize`, whether the terminal or the bars caused it.

**Non-Goals:**
- No per-client screen area. The session keeps one screen area, set by the latest reporting client, as before. A client whose bars differ from the latest reporter's still sees tiles measured for that client.
- No wire protocol change and no version bump. The hello and resize keep their fields.
- No change to the layout or floating-windows geometry. Full width stays "the area's width". Only the area changes.

## Decisions

**The reported size is the ribbon's size, not a second field.** The alternative was to send the ribbon width alongside the terminal size and let only full width use it. That gives two widths with different meanings, so 1/2 plus 1/2 would no longer fill the space full width fills. niri measures every proportion against the working area, and gband now does the same with its bars.

**`Display::reported_size()` returns the ribbon's size.** `set_size` and `set_bars` both return the new reported size when it differs from the last reported one. Both callers, `Controls::resize` and the bar refresh in `react`, push `resize_step` for it. A bar moved to the other side changes `ribbon.x` but not its size, so it snaps the view and reports nothing. The refresh in `react` already folds a callback's or a load's bar changes into one `set_bars` call, and at most one `set_bars` follows it, so a single callback or load yields at most one report. Compare against the last reported size, not the previous ribbon, so that the two `set_bars` calls in one `react` cannot report twice.

**Place the bars before the handshake.** `run` builds the `Display` from the terminal's size and hands it the bars the loaded configuration already holds, before `connect`, which then sends `display.reported_size()` in the hello. `connect` takes the size as a parameter in place of reading the terminal itself. Otherwise a new session would start at the terminal's size and resize its first window 100 ms later, a visible SIGWINCH on every start. `attach` takes the prepared display and controls instead of building its own.

**The two "Key bindings" scenarios are deferred, not left wrong.** "Repeated resize" and "Grow the column" measure tiles against an 80-column area beside the default sidebar. Three open changes also MODIFY "Key bindings", and a MODIFIED block replaces the whole requirement, so a copy taken now would drop their edits at archive. The task that fixes them copies the requirement from the latest `origin/dev`, at implementation time, and adds "the client has no bar" to both scenarios, so their numbers stay true.

## Risks / Trade-offs

- [Nearly every window beside the default sidebar is one column narrower, so most saved screenshots under `tests/lua/screenshots/` and `examples/plugins/*/tests/screenshots/` change] → Regenerate them in one pass after the code change, and review each diff for exactly a one-column shift of tile borders, with no other change.
- [End-to-end tests that measure tiles beside the default sidebar, such as `tests/attach.rs` and `tests/navigation.rs`, assert 80-column numbers] → Restate each one for the 79-column area, or give it a no-bar configuration where the test is not about bars, matching the spec's scenarios.
- [Toggling or resizing a bar now sends SIGWINCH to every shown window] → This is intended. The server's 100 ms settle already folds a burst of changes into one PTY resize.
- [settings-themes, still open, declares `tests/lua/screenshots/`, `crates/client/src/lib.rs` and `crates/client/tests/`] → Expect merge conflicts in screenshots at integration. Regenerating them after merging `origin/dev` resolves them.
- [Two clients with different bars take turns setting the area, so each one's report resizes the shared windows] → This is unchanged behaviour for clients with different terminal sizes, and now also covers different bars.

## Migration Plan

The protocol is unchanged, so an old server keeps working with a new client: it just receives the narrower size. Rollback is a revert of the client change.
