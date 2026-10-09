## Context

See proposal.md for the failure and its trace.

Today a key or paste takes this path through the server:

1. `dispatch` in `crates/server/src/connection.rs` runs on a tokio worker thread. It calls `forward`, which looks up the window in the session state and sends the input to that window's unbounded input channel.
2. The window's input thread (`spawn_input` in `crates/server/src/window.rs`) wakes, encodes the input and writes it to the PTY.
3. Back in `dispatch`, once `forward` returns true, `taps.notice` sends `Input::Notice` to the Lua thread's `std::sync::mpsc` channel.

Program output takes another thread: the window's reader thread (`read_output`) calls `taps.output`, which sends `Input::Output` to the same channel. The Lua thread handles that channel's inputs in the order they were sent.

Steps 2 and 3 race. The input thread and the program can finish before step 3 runs, and the program's answer then enters the channel before the notice.

## Goals / Non-Goals

**Goals:**
- A key's or paste's notice enters the Lua channel before the input can reach the PTY.
- A key or paste for a window outside the layout still emits no notice.

**Non-Goals:**
- Any change to the notice budget, to output taps or to the Lua thread.
- Ordering of mouse input, which emits no notice.

## Decisions

**Queue the notice between the lookup and the send.** `forward` takes the window entry from the session state, then calls a hook, then sends the input. `dispatch` passes `taps.notice` as the hook for keys and pastes, and nothing for mouse events. Program output caused by the input cannot exist before the input is sent, so the notice is ahead of it in the channel. The single Lua channel keeps that order to the handlers.

Alternatives considered:
- *Notice before `forward`, unconditionally.* Rejected: a key for a window that is not in the layout would emit a `WindowInput` for a window no handler can find.
- *Have the input thread send the notice after writing.* Rejected: it reverses the order, which is the bug, and it would need the client number in the window input.
- *Timestamp inputs and outputs and reorder in the Lua thread.* Rejected: it adds buffering and a delay to every output for an ordering one queue already gives.

**Keep a notice whose send fails.** When the window's input channel is closed, its writer has stopped and the window is leaving the layout. The notice is already queued. A `WindowInput` for that window lets a handler clear state the window is about to lose anyway, so it is not withdrawn. The spec only orders a notice before output, and a window whose writer stopped produces no output from that input.

**Regression test in `crates/server/tests/scripting.rs`.** A server Lua config counts `WindowInput` calls for window 1. Its `WindowOutput` handler keeps a tail per window, and when the tail first holds `input`, it emits the count. The test pastes `printf 'in%s\n' put` and presses Enter with `type_line`, which sends one paste and one key, so the count must be 2. The echo of the typed line holds `in%s`, never `input`. Before the fix the count is 1 only when the race is lost, so the test fails intermittently under load. After the fix it cannot fail.

**Agent status spec waits for the prompt.** `g.wait_text("$")` after `g.start` in "a prompt marks the window waiting" and "typing clears the waiting state", as `tests/lua/settings_spec.lua` does. The later cases type into a new window only after `g.settle` or not at all, and stay as they are.

## Risks / Trade-offs

- [The test cannot make the old race happen on demand, so it guards the fix only statistically.] → The fix is structural: the notice is queued before the send in the same function, and review can check that order directly.
- [A notice dropped by the 1024-event budget gives no ordering at all.] → Unchanged behaviour, already specified in "Handlers never slow a session".
- [reload-and-control adds `hub.typed(client)` to the Key, Paste and Mouse branches of `dispatch`.] → This change depends on reload-and-control and starts from its `dispatch`. `hub.typed` stays first in each branch, before `forward`, and the reorder does not affect it.
