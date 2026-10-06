## Context

`Session::apply` in `crates/server/src/session.rs` places an opened window, publishes the new state and, when the action asks for focus, calls `respond(Reply::Focus(window))` at once (line 323). The reply travels on the connection's `replies` channel. The connection loop in `crates/server/src/connection.rs` handles it in two places:

- The `replies` branch calls `sync` and then delivers the reply. `sync` sends the latest layout and the snapshots of its windows. When the window's program exited between `respond` and this point, the latest state no longer holds the window, so the client gets a layout without it, then a focus naming it.
- The `Barrier::Flush` branch, which the test channel's settle drives, delivers every queued reply first and syncs last, so a focus can even arrive before any layout holding the window.

`Sent::state` holds the state of the last layout sent to the client. After `sync`, a window in that state's layout has had its layout and its snapshot sent.

`Reply::Focus` also comes from `scripting.rs` (line 423), for a window the client's own Lua opened with focus. It uses the same channel, so the same fix covers it.

## Goals / Non-Goals

**Goals:**
- A focus message always follows a layout and a snapshot holding its window, and never follows a layout that does not hold it.
- `argument_list_program_runs_without_a_shell` stops failing intermittently.

**Non-Goals:**
- Changing the opened reply for plugin window requests, which must answer every request.
- Changing the test client's checks in `crates/test-support/src/lib.rs`. They check the protocol, and they are what found this.
- Any wire protocol change.

## Decisions

### Check the window against the layout just sent

A new `deliver_reply` in `connection.rs` takes the reply, the session handle, `Sent` and the writer. For `Reply::Focus(window)` it first calls `sync`, then sends the focus only when `sent.state` holds the window in its layout, and otherwise drops it with a debug log naming the window. Every other reply is delivered as today. Both branches call it.

Checking `sent.state`, not the session's current state, is what the requirement needs: the question is whether the layout this client last received holds the window, since that is what the client acts on.

*Alternative:* drop the reply in `Session` when the window closes. Rejected: the reply is already in the connection's channel by then, and the session does not know what each client has been sent.

*Alternative:* have the client ignore a focus for an unknown window and relax the test client. Rejected: the real client already ignores it, the spec forbids the message, and relaxing the test client would hide the next ordering bug.

### Sync before replies in the flush

The `Barrier::Flush` branch syncs before draining replies, then drains them through `deliver_reply`, then drains deliveries, and keeps its final sync, so the barrier still guarantees that everything queued has been sent.

### Regression test

A test in `crates/server/tests/programs.rs` opens a window running `true` with focus, 30 times in a row, pumping messages for 100 ms after each. The test client's `apply` already panics on a focus for a window it holds no snapshot of, which covers both a focus before the layout and a focus after a layout without the window. Each round ends with the layout holding only the first window. Before the fix, the test is checked to fail at least once in 20 runs, so it is known to catch the race.

### Keep the argument list test's window open

The focus fix does not by itself stop `argument_list_program_runs_without_a_shell` failing. When `printf` exits before the connection syncs, the server now drops the focus, and the test's `open_running` waits for it until it times out: 4 of 46 runs of the `programs` binary did. When the program exits before any layout holding it reaches the client, the client never receives its output at all, so no change to the waiting can make the test reliable.

The test runs `awk` with `BEGIN { printf "%s-%s", ARGV[1], ARGV[2]; getline line < "-" }`, `a` and `b` instead. It is still an argument list that no shell expands, it prints `a-b`, and it stays open reading its PTY until the session ends.

*Alternative:* `yes a-b`, which also stays open. Rejected: it writes without end, loading the server's emulator for the whole test.

*Alternative:* keep a window open after its program exits. Rejected: that changes the window lifecycle for a test's sake.

## Risks / Trade-offs

- [A client that relied on a focus for a window that closed at once] → no such client: the real client already ignores it, and the window is gone.
- [The extra `sync` before a focus in the flush] → `sync` sends nothing when the client is already current, so it costs one comparison.
- [The regression test is probabilistic] → it runs 30 rounds per run and is checked against the old code; the existing `argument_list_program_runs_without_a_shell` keeps running too.
