## Context

This change starts after `status-line` and `key-hints` are archived, and is written against what they and `client-plugin-foundation` deliver.

`crates/lua` builds one Lua state per load: `lib.rs::evaluate` runs `runtime::install`, the init file, `source_plugins` and `finish`. `runtime.rs` hardcodes `SIDE = "client"`, and `plugin_files` lists `plugin/*.lua` then `plugin/client/*.lua`. `owner.rs`, `guard.rs`, `callbacks.rs` and `events.rs` give ownership, the instruction budget, guarded callbacks and `gband.on`. Status-line adds `bundled.rs`, the private `host` table and `runtime/gband/statusline.lua`, whose `host.state()` feeds the render context. mlua runs with `lua54, vendored, serialize, async, send`, so a `Lua` state and a `Config` are `Send`, and `watch` already builds a new `Config` on its own thread.

`src/main.rs::server` runs a full client load only to send `config.options.layout` through a `tokio::sync::watch` channel to `gband_server::run`, which runs on a multi-thread tokio runtime.

In `crates/server`, each pane reads its PTY on a std thread (`pane.rs::read_output` calling `Pane::process`) and writes input on another. Each session is owned by a `session::drive` task fed by `Command`s. The `Bus` (`event.rs`) is a broadcast of `Published { session, event }` carrying layout events, `PaneExited`, `ClientAttached` and `ClientDetached`; a pane's exit is published only after its remaining output was read. `connection.rs::attach` loops over client messages and the session's `changed`, `focus` and `ended` signals. Pane identifiers are per session: every layout counts from 1.

`crates/client` owns a `Controls { keymap, runtime, leader }` on a current-thread runtime. `Display::apply` handles server messages, and `react` diffs an `Observed` snapshot to emit events.

## Goals / Non-Goals

**Goals:**
- One Lua code base serves both sides. A `Side` value picks the API, the side file and the defaults; nothing else forks.
- No handler can stall a pane, a session or a connection. The server's Lua runs on its own thread and is fed through bounded queues.
- Everything Lua-visible that crosses the connection is a protocol message holding a plain data `Value`, so a relay or a later SSH transport carries it unchanged.
- Shared state lives in Rust, not in a Lua state, so it survives reloads and is read by connections without touching Lua.

**Non-Goals:**
- A Lua state per session. One server state serves every session; events carry the session name.
- Running client code in the server or server code in the client, even when both run on one machine.
- Pane titles or working directories as built-in state.
- Back-pressure from Lua onto panes. Output a slow handler cannot keep up with is dropped for that handler only.

## Decisions

### A `Side` parameter, not two crates
`gband_lua` gains `pub enum Side { Client, Server }`. `evaluate`, `install`, `load`, `load_defaults`, `defaults` and `watch` take it. `install` installs the shared API (`on`, `augroup`, `emit`, `cmd`, `opt`, `plugin`, `plugins`, `runtimepath`, `side`, `api_version`, `print`, `pane_state`, `action`) with each side's behaviour, then the side's own API: the client's keymap, bind, spawn, status line, highlights, colorschemes, `rpc` and local actions, or the server's `sessions` and `session`. The init file is `user/init.lua` or `user/server.lua`, and the default chunk is `defaults.lua` or a new `defaults_server.lua`.
- Alternative: a separate `gband_lua_server` crate. Rejected: the owner stack, guard, callbacks, events, options, manifest and runtimepath code would be duplicated or moved into a third crate for no behaviour difference.

### Side guard by a metatable on `gband`
After installing, each side sets a metatable on `gband` whose `__index` looks the key up in a static table of the other side's names and raises `gband.<name> is a <side> API; this is the <side>` at the caller's line. `gband.action` gets the same treatment for the view and client action names in the server. Fields that exist on both sides are real fields, so the metatable is only consulted for absent keys and costs nothing on the normal path.
- Alternative: leave the other side's names absent. Rejected: `gband.keymap.set` would fail with "attempt to index a nil value", which names neither the API nor the side.

### Manifests run in an empty environment
`runtime.rs` replaces `plugin_files` with `manifests`: for each runtimepath entry after `user`, if `plugin.lua` exists, load it with `Chunk::set_environment` set to a new empty table, under `guard::run` owned by the entry's name, and validate the returned table. A valid manifest is kept in a `Manifests` app-data list that backs `gband.plugins()`. Side files are then sourced in runtimepath order, one per plugin. An entry holding `client.lua` or `server.lua` but no manifest reports a plugin error.
- Alternative: the manifest as a field of the side file's return value. Rejected: the server would have to run `client.lua` to learn the client requirement, which is exactly the cross-side evaluation this change removes.

### `version.rs` parses versions and requirements
A small parser for `N[.N[.N]]` and comma-separated comparisons, shared by manifest validation in both sides and the client's requirement check. No dependency is added: the grammar is five operators and one to three integers.

### Plain data `Value` in the protocol crate
`crates/protocol` gains `Value { Nil, Bool(bool), Int(i64), Float(f64), Bytes(Vec<u8>), Table(Vec<(Key, Value)>) }` and `Key { Int(i64), Bytes(Vec<u8>) }`, serialised by postcard like every message. `crates/lua/src/value.rs` converts a Lua value to a `Value` with a path stack for errors (`data.cb.run`), a visited set for cycles, a depth limit of 32 and a size check with `postcard::experimental::serialized_size` or an encode into a counting writer against 1 MiB. The reverse conversion always builds new tables.
- Alternative: mlua's `serialize` feature with serde_json-like values. Rejected: it cannot keep integer and string keys apart, nor carry arbitrary bytes, and it would put a serde data model into the protocol.

### The server's Lua thread
`crates/server` gains `scripting.rs`. A std thread named `gband-lua` owns the server `Config` and its `Runtime`. It reads one `std::sync::mpsc` channel of `Input`:

| input | from |
|---|---|
| `Bus(Published)` | a tokio task subscribed to the session bus, which also counts lag |
| `Output { session, pane, bytes }` | the pane's read thread, through the tap |
| `Input { session, pane, client }` | `connection.rs::forward` |
| `Session(Created/Ended, name)` | the registry |
| `Call { session, client, id, name, args, reply }` | `connection.rs` on a command message |
| `Reload(Config)` | the configuration watcher |

For each input it emits the matching event or runs the command, then drains the `Outcome`'s dispatched effects: session actions are sent to the registry, which forwards them to the session's `drive` task like a client action without a focus reply; emits, state changes and server errors go to the hub. Running Lua on a std thread keeps a 100M-instruction handler off the tokio workers.

Ordering: the tap sends a chunk before `Pane::process` returns, and the session publishes `PaneExited` only after the reader has finished, so a pane's last `Output` reaches the channel before its exit event. Layout events and exits of one session come from one bus in order.

Load: the tap queue is a shared `AtomicUsize` of undelivered bytes. The read thread adds before sending and drops the chunk, counting it, when the total would pass 4 MiB; the Lua thread subtracts on receipt and logs the dropped count when it next delivers. Other inputs share a bounded budget of 1024, dropping the oldest bus events through the broadcast's own lag. Two `AtomicBool`s, set after each load from whether `PaneOutput` and `PaneInput` handlers exist, let the read thread and the connection skip the send altogether.
- Alternative: a tokio task with `spawn_blocking` per event. Rejected: event order across calls would be lost, and the Lua state would move between threads for every event.

### The hub holds shared state
`crates/server/src/hub.rs` holds, behind one `Mutex`: pane states keyed by `(session, PaneId)` as `BTreeMap<String, Value>` with their encoded sizes; the queue of at most 256 undelivered events with their target; the set of attached clients per session; the latest server error; and the requirements list. A `broadcast` channel carries `Delivery { session: Option<String>, message }` to connections.

On attach, `connection.rs` locks the hub, registers the client, copies the session's states, the requirements, the error and the matching queued events, and subscribes to the broadcast, all under the lock, so no change falls between the copy and the subscription. It sends them after the layout and snapshots, in the order the wire protocol requires. Detach unregisters under the lock. An emit checks the attached set under the same lock to choose between broadcast and queue, so an event is either delivered or queued, never both or neither.

The Lua proxy returned by `gband.pane_state(session, pane)` is a userdata-free table with a metatable whose `__index`, `__newindex` and `__pairs` call host functions that lock the hub. A pane's state is removed when the hub sees its `PaneClosed`. States are in Rust, so a reload keeps them.
- Alternative: keep states in the Lua state and rebuild them after a reload. Rejected: a failed reload or a crashed handler would lose shared state, and connections would need the Lua thread to answer every attach.

### Server commands answer through the connection
A command message becomes `Input::Call` with a oneshot `reply` and the connection's existing focus sender. `ctx.focus(pane)` pushes onto that sender, so the focus reaches only the caller, through the same `ServerMessage::Focus` path open pane uses. The result or error comes back on `reply` and the connection sends `ServerMessage::Result`. Calls without a connection (`gband.cmd.run` in the server) have no sender, and `ctx.focus` does nothing.

### Client side of the bridge
`Display::apply` handles the new messages:
- `PaneState` updates a `BTreeMap<PaneId, BTreeMap<String, Value>>` in `Display`, pruned on each layout. Changes after the attach batch become `Event::PaneStateChanged`; the attach batch is applied silently, using the same `first_view` flag as the other events.
- `Event` becomes `Event::ServerEvent`; the `pattern` match in `events.rs` extends to it.
- `Result` looks up a pending `call id -> CallbackId` map in `Controls` and calls the callback through a new `Runtime::call_with(id, args)`.
- `Requirements` is checked against `Config`'s manifests with `version.rs`; each failure becomes a `ConfigError` added to the error display.
- `ServerError` becomes a `ConfigError` with a `server: ` prefix.

`gband.pane_state(pane)` and the render context's `panes` read the map through `host.state()`, which status-line already feeds from `Display`. The map is copied into Lua only when a render or a call asks for it.

`gband.rpc`, `notify`, `bell`, `clipboard` and `open` queue new `Dispatch` variants. The client loop writes the escape sequences into the terminal output after the frame it is drawing, never inside it, and spawns the opener with `std::process::Command` with null stdio and no wait. Notification text is sanitised in Rust before it is written.

### Server options move to `user/server.lua`
`options.rs` gains a side for each built-in option. The client table drops `default_column_width` and `width_presets` and gains `notify_style`; the server table holds the two widths. The client still parses a width option's name only to report "a server option" in its error. `src/main.rs::server` loads the server side, sends `options.layout` through the existing `watch` channel on every successful load, and hands the `Config` to the scripting thread.
- Alternative: keep reading the widths from `user/init.lua` for compatibility. Rejected by the user: the server would keep evaluating client code.

### Protocol version 6
New `ClientMessage::Command { id, name, args }` and `ServerMessage::{Event, PaneState, Result, Requirements, ServerError}`. Version 5 is taken by `plugin-windows`, which landed first. The version check already refuses a version 5 peer, and the client replaces a server of another build as it does today.

## Risks / Trade-offs

- [`PaneOutput` costs a copy of every chunk while a handler exists] → The copy is skipped when no handler is registered, and the byte budget bounds memory. Handlers see raw bytes, so plugins must match across chunk boundaries; the guide and the example keep a short tail per pane.
- [A runaway server handler delays commands and events for up to the instruction limit] → The limit stops it and marks its plugin failed. Panes and screens are unaffected.
- [Pane state keys are global, so two plugins can clobber one key] → Documented: a plugin prefixes keys it does not mean to share. Global keys are the point of shared state such as `agent`.
- [The status-line delta modifies a capability that does not exist on `<base>` yet] → It depends on `status-line`, which creates it; validation reports this as information until then.
- [`plugin-windows` landed first with protocol version 5 and client-only APIs (`gband.layout`, `gband.view`, `gband.pane`, `gband.band`, `gband.win`)] → This change takes version 6, and the side guard's client list includes those names.
- [Breaking layout and option moves] → No third-party plugin exists yet. The guide documents the move, and an option set on the wrong side is reported naming the side that owns it.

## Migration Plan

1. Move every `plugin/*.lua` and `plugin/client/*.lua` in tests, examples and docs to a `client.lua` with a `plugin.lua` manifest.
2. Move width options in tests from `user/init.lua` to `user/server.lua`.
3. Ship the server runtime, protocol version 6 and the client bridge together; a version 5 client is refused and replaced as today.

Rollback is reverting the change: no on-disk format outside `defaults/server.lua` is introduced, and that file is rewritten on each start.
