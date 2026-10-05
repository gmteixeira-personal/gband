## Context

See proposal.md for why. Three things exist today:

- `tests/common/mod.rs` holds `TestEnv`, which builds an isolated XDG tree, and `Attached`, which runs `gband attach` in a `portable-pty` PTY and feeds its output to a `gband-emulator` `Grid` on a reader thread. Seven test files use it. It does not answer terminal queries: the emulator's `write_back` bytes are never written to the PTY.
- `crates/emulator/src/callbacks.rs` already sees `unhandled_osc` and `audible_bell`, and only logs them.
- The client's loop in `crates/client/src/lib.rs` is one `tokio::select!` over server messages, terminal events from a crossterm reader thread, reloads, timers and animation frames. crossterm parses terminal input into events on its own thread, so the client never sees how many bytes it read.

This change builds on `split-plugin-runtime`: the server's `gband-lua` thread with its `Input` channel, the `Side` parameter in `crates/lua`, the side guard metatable, and the plain data `Value` in `crates/protocol`. It also builds on `plugin-windows`, whose `gband.layout()` and `gband.view()` the examples and scenarios use, and whose `tests/common/mod.rs` edits must land before that file moves.

The `plugin-bridge` and `plugins` deltas modify requirements that only exist once `split-plugin-runtime` is archived. `openspec validate` notes that archive would refuse them today. The dependency order makes them valid by the time this change archives.

## Goals / Non-Goals

**Goals:**
- One runner that gband's own tests, plugin authors and Claude use alike.
- Tests that read what a user's terminal shows, without a wire protocol change.
- Waiting on gband's own work without sleeps.

**Non-Goals:**
- Running cases in parallel. Cases run one at a time, so the report's order is the run's order.
- A stable Rust API for the harness crate. Only the Lua API, the CLI and the screenshot format are contracts.
- Settling on programs in panes. `g.wait` covers them.

## Decisions

### A new shipping crate, `crates/harness`
`gband test` ships in the binary, so the runner cannot live in `gband-test-support`, which is dev-only. The new crate `gband-harness` holds:

- `env.rs`: the case tree, environment, plugin links and opener stubs. This is `TestEnv` moved out of `tests/common`.
- `terminal.rs`: the PTY host. This is `Attached`, plus writing the emulator's `write_back` to the PTY so the runner answers terminal queries as a real terminal does, plus the recorded OSC events.
- `screenshot.rs`: rendering, references and the row-keyed diff.
- `channel.rs`: the runner's end of the test channel.
- `runner.rs`: discovery, the test-side Lua state, the `gband.test` module, the report.

`tests/common/mod.rs` keeps the process helpers it alone uses, such as `children` and `session_of`, and re-exports the rest from `gband-harness`, so the seven test files only change their imports.

- Alternative: the runner in `src/` beside `main.rs`. Rejected: the Rust tests could not share it without depending on the binary crate.

### The test side in `crates/lua`
`split-plugin-runtime`'s `Side` gains `Test`. A test-side state installs only `side` and `api_version` under the side guard's metatable, whose error now names both sides for a field the client and the server share. The harness registers `package.preload["gband.test"]`: a bundled Lua chunk for `case`, the assertions and the handle's shape, over Rust host functions for everything that touches a process. `package.path` gains the test file's directory. `print` is replaced in the test-side state by one that writes to the runner's standard output.

The test-side Lua runs on a plain thread. Channel I/O and the PTY run on a tokio runtime the runner owns. Host functions block on a oneshot answer. The case time limit is an instruction hook that checks a deadline every 10,000 instructions, plus the same deadline passed to every blocking host call.

### The test channel is a separate socket, not the wire protocol
The runner listens on `<case tree>/channel.sock` in a 0700 directory. The client and the server connect when `GBAND_TEST_SOCKET` is set, before loading configuration, check the listener with `SO_PEERCRED`, and send `Hello { role }`. The runner answers `Start { time }`, so frozen time is in place for the first load.

Messages are a new `gband_protocol::test` module framed by the existing `MessageReader` and `MessageWriter`. Results reuse `Value`.

| runner to process | process to runner |
|---|---|
| `Start { time }` | `Hello { role }` |
| `Eval { id, source, args }` | `Answer { id, result }` |
| `Settle { round, markers }` | `Settled { round, sent }` |
| `Reload { id }` | `Reloaded { id, error }` |
| `SetTime { time }` | |

The runner always starts `gband attach` from its own executable, and the client starts the server from its own, so both ends are one build. The channel needs no version handshake.

- Alternative: an `Eval` message relayed by the server over the wire protocol. Rejected in exploration: it puts Lua code on the connection that `plugin-bridge` keeps code-free, and a later SSH transport would carry it to remote clients.

### The client and the server join the channel through their existing loops
- **Client:** the channel is one more `tokio::select!` branch. `Eval` runs the chunk through the `Runtime` as a callback that belongs to no plugin, and its dispatched steps go through `perform` like a key's. `Reload` calls `gband_lua::load` and `controls.reload` synchronously.
- **Server:** a task per channel connection. `Eval`, `Reload` and `SetTime` become new `Input` variants on the `gband-lua` thread's channel, each carrying a oneshot reply. Ordering with other inputs follows from the one channel.
- **Panes:** `pane.rs` removes `GBAND_TEST_SOCKET` from the command it builds, next to where it sets `GBAND_PANE`.

### Settle uses in-band markers
Byte counts are not available on the input side, because crossterm parses input on its own thread. Order is preserved within each direction of the PTY, so the runner uses markers in both directions:

1. **Input marker.** After its input, the runner writes `ESC [ I`, which crossterm parses as `FocusGained` whether or not focus reporting is on. A client on a test channel counts `FocusGained` events as markers and does nothing else with them. `Settle { markers }` tells it how many the runner has written. The client answers once it has handled that many, which means it has handled every key before them.
2. **Server barrier.** The runner then sends the server `Settle`. Each connection task reads everything readable from its client and forwards it. Bytes the client wrote before answering are already in the socket buffer. A barrier message then follows those through the session `drive` task's queue, the `gband-lua` input channel and each connection's outgoing queue, all FIFOs.
3. **Client draw.** The runner sends the client `Settle` again. The client handles every readable server message, waits out any animation, draws, and writes `ESC ] 7777 ; settle ; <round> BEL` after the frame. The runner's emulator callbacks see the OSC, so every byte before it has been parsed.
4. **Rounds.** `Settled { sent }` reports how many messages each side sent since the previous round. While either is non-zero, the runner repeats steps 2 and 3, up to ten rounds.

- Alternative: a counting writer around the client's stdout and byte counts for the output side. Rejected: it covers only one direction, and an in-band marker needs no new I/O path.
- Alternative: polling the screen until it stops changing. Rejected: this is the sleep-and-hope approach that settle exists to replace.

### Frozen time wraps `os.time` and `os.date`
Every Lua state `crates/lua` creates on a side with a frozen clock gets `os.time` and `os.date` replaced. A Rust closure reads an `Arc<AtomicI64>` holding the instant, or a sentinel for unset, and calls the original with the instant when the caller gave no time. `SetTime` stores into the same atomic, so a later reload keeps it. Timers keep `Instant`. `TZ=UTC` in the case environment makes `os.date` independent of the host's zone.

### Screenshots compare row by row
A screen has a fixed number of rows, so the diff compares the header, then each row's text and that row's style runs, keyed by row number. It prints `-` and `+` lines only for differing rows. No diff library is needed, which keeps the proposal's "no new dependency". OSC 52 is base64. The runner decodes it with a small local decoder rather than adding `base64` as a direct dependency.

### Recording is opt-in in `gband-emulator`
`Callbacks` gains an optional recorder for notifications, clipboard writes, bells and settle markers. Only the harness turns it on. The client's per-pane emulators would otherwise accumulate every OSC their programs write.

### gband's own Lua tests
`tests/lua/statusline_spec.lua` covers the bundled modules, and `examples/plugins/*/tests/` covers each sample. `tests/lua_specs.rs` runs `gband test` on each set with the built binary and asserts exit status 0, printing the report on failure. References are committed. A change to a default colour or segment updates them with `--update` in the same commit.

### The project skill
`.claude/skills/gband-test/SKILL.md` describes the loop: write a chunk to `gband test - --show` against the plugin's directory, read the screenshot, edit, rerun, then promote the chunk into `tests/*_spec.lua` and accept references with `--update`. It points at `docs/testing.md` for the API, so the API is documented in one place.

## Risks / Trade-offs

- [`FocusGained` is spent as a marker] → Only a client on a test channel treats it so. If gband later reacts to focus events, the marker moves to another inert sequence. The spec does not name the marker.
- [A settle round count can be exceeded by handlers that keep triggering each other] → After ten rounds the settle fails and names the step. This matches the client's own ten-round event limit.
- [References churn when defaults change] → `styles = false` for cases about text. Defaults changes update references in the same change, with the diff in review.
- [Same-user code can drive a client that has the variable] → Same-user code can already ptrace it. `SO_PEERCRED` refuses another user's socket, and the server strips the variable from panes, so a nested gband does not join the channel by accident.
- [Shipping the runner grows the binary] → It adds no dependency. The runner reuses `portable-pty`, `gband-emulator` and `mlua`, which the binary already links.
- [`/bin/sh` prompts differ across systems] → `PS1='$ '` in the case environment. `INPUTRC=/dev/null` keeps readline quiet.

## Migration Plan

Additive for users. Inside the repository, `tests/common` becomes a re-export in the same commit that creates the crate, so every test keeps compiling. Rollback is reverting the change. No stored data or protocol version is involved.
