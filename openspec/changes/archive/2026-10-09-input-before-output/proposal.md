## Why

The server can deliver a window's `WindowInput` to its Lua handlers after the `WindowOutput` that the same key caused. `dispatch` in `crates/server/src/connection.rs` hands a key or paste to the window's PTY writer thread first and queues the notice for the Lua thread only afterwards. When the connection's worker thread is descheduled between the two steps, the program reads the key, answers, and the PTY reader thread queues that answer ahead of the notice. A trace on `dev` at 201428e captured a 3.6 ms gap: the agent status sample set `agent = "waiting"` from the prompt and then cleared it on the late `WindowInput` for the Enter that printed the prompt. This breaks the server-runtime rule that a session's events reach the handlers in the order the session applied the changes. Under CPU load, `examples/plugins/agent-status/tests/agent_status_spec.lua` failed 16 of 60 runs on `dev`, the same rate as on the reload-and-control branch.

The same spec has a second, independent weakness. Its first two cases type with `g.run` right after `g.start`, before bash prints its first prompt. The terminal echoes the typed-ahead characters before the prompt appears, the line reads `pr$ printf ...`, and the case that counts prompts gives up at line 29. That shape accounts for 4 of the 16 failures.

## What Changes

- For a key or paste, the server queues the `WindowInput` notice for the Lua thread before it writes the input to the window's PTY. Notices and window output travel to the one Lua thread on one queue, so the notice then always comes before any output the program writes after reading that input.
- A key or paste for a window that is not in the layout still emits no `WindowInput`. A notice queued for a window whose PTY writer has just stopped is kept: that window is leaving, and a handler that clears its state does no harm.
- Mouse input is unchanged: it emits no `WindowInput`.
- The server-runtime spec states the guarantee and gains a scenario for it. `docs/plugins.md` states it beside the existing ordering rule.
- A server test checks that the `WindowInput` notices of a typed line have all reached the handlers before the output the line produced.
- The agent status spec waits for the shell prompt with `g.wait_text("$")` after `g.start` and before `g.run` in its first two cases.

Out of scope:
- Ordering between different windows, or between a window's input and output that the program wrote before reading the input.
- Dropped notices: when more than 1024 events wait for the handlers, the server still drops them as "Handlers never slow a session" defines.
- The flake in `tests/control.rs` and the other end-to-end tests that wait for a stale focused window, which a separate change covers.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `server-runtime`: "Server events" states that a key's or paste's `WindowInput` reaches the handlers before any `WindowOutput` holding output the program wrote after reading that input, with a new scenario.

## Impact

- `crates/server/src/connection.rs`: `dispatch` and `forward` reorder the notice and the write for keys and pastes.
- `crates/server/tests/scripting.rs`: a new ordering test.
- `examples/plugins/agent-status/tests/agent_status_spec.lua`: prompt waits in the first two cases.
- `docs/plugins.md`: the ordering sentence.
- No wire protocol change, no Lua API change, no change to `gband.api_version`.

## Coordination

### Author
- gmteixeira

### Depends On
- reload-and-control

### Expected Files
- openspec/changes/input-before-output/
- crates/server/src/connection.rs
- crates/server/tests/scripting.rs
- docs/plugins.md
- examples/plugins/agent-status/tests/agent_status_spec.lua
