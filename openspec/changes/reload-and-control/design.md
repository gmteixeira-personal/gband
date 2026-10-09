## Context

See proposal.md for the motivation and specs/ for the behaviour. The pieces this change builds on:

- **Loading.** `src/main.rs` loads each side's configuration with `gband_lua::load`, which reads the `user`, `defaults` and plugins directories every time. The watcher (`gband_lua::watch`) calls it on a change under `user/`. Only the test channel holds a loader closure that can load on demand (`TestChannel::reload` on both sides). The client applies a load with `Controls::reload`, which already returns to `root`, closes plugin windows and reopens the settings window from `display.settings_line`. The server applies one with `Input::Reload` on its scripting thread, and `Input::Barrier` waits until it took effect.
- **Running Lua on demand.** The test channel's `Eval` is handled by `Controls::eval` in the client and `Input::Eval` in the server. Both already return plain data or an error message and keep chunk errors out of the error list.
- **Connections.** `crates/server/src/connection.rs` reads one request after the hello: `Attach`, `ListSessions` or `KillSession`. A non-attach request is answered and the connection closes. Every connection gets an id from `NEXT_ID`, and attached clients are kept in `Hub::clients` with a sender per client and their session.
- **Errors.** The hub keeps only the latest server error (`Inner::error`) and broadcasts each one as `ServerMessage::ServerError`. The client keeps `Display::errors` as plain strings, server errors prefixed with `server: `.
- **Selection.** `Selection::resolve` already reads `GBAND`, so every subcommand run in a window reaches that window's server.

## Goals / Non-Goals

**Goals:**
- One reload path per side, used by the watcher, the reload action, a control request and the test channel, so every load is counted the same way.
- Control requests reuse the test channel's eval and reload handlers rather than adding a second way to run Lua.

**Non-Goals:**
- No remote action subcommand (`gband action <name>`): `eval` covers it with `gband.action.<name>()`.
- No `gband mcp` server, no socket other than the server's own, no escape-sequence channel.
- No change to how a file change reloads, other than counting.
- No client-side wait for remote control in the Lua API: Lua cannot call `gband reload`.

## Decisions

### 1. A loader for each side, always present
`main.rs` builds one `Loader` closure per side from `Locations` (or the defaults), and passes it to the client in `Configuration` and to the server in `ServerConfig`, whether or not a test channel is joined. The watcher, the test channel and the forced reload all call it. The server-side closure keeps updating `options_tx`, as both of today's closures do.

*Alternative:* keep loaders inside the test channel and add a separate one. Rejected: two copies of the same closure, and the load counter would need two increments.

### 2. Load numbers live with the state that applies loads
The client increments a counter in `Controls::reload` and starts at 1 in `Controls::new` from `run`. Because `Controls::reload` replaces `*self`, the counter is carried across as `awaiting` and `pending` are. The server increments in its scripting thread when it handles `Input::Reload`, and reports it in the answer to `Input::Barrier`. A failed load counts, as the spec says.

### 3. Protocol: one control request, forwarded per client
New types in `crates/protocol/src/message.rs`, with `PROTOCOL_VERSION` 12:

```
ClientMessage::Control { session, target: Target, operation: Operation }   // a request
ClientMessage::Reload                                                      // attached
ClientMessage::ControlAnswer { call: u64, answer: Answer }                 // attached
ServerMessage::Reloaded
ServerMessage::Control { call: u64, operation: Operation }
ServerMessage::ControlResults(Vec<Entry>)

enum Target { Server, EveryClient, Client(u64), Chosen }
enum Operation { Reload, Errors, Eval { source, args }, Command { name, args } }
enum Answer { Loaded { load, error }, Errors { load, errors }, Values(Result<Vec<Value>, String>) }
struct Entry { process: Process, answer: Option<Answer> }   // None: no answer
```

`Target::Chosen` lets the server pick the client with the information only it has. A server command reuses `Answer::Values` with one value.

*Alternative:* one request type per subcommand. Rejected: four request variants with the same forwarding logic.

### 4. The server gathers answers in the connection task
`connection.rs` handles `Control` like `KillSession`: it resolves the session, runs the server's part through the scripting thread (`Input::Reload` then `Input::Barrier`, `Input::Eval`, `Input::Call` with the chosen client as caller, or a new `Input::Errors`), then forwards the client part. The hub gets a `calls: HashMap<u64, oneshot::Sender<Answer>>` and a counter. `Hub::control(client, operation)` sends `ServerMessage::Control` through the client's sender and returns the receiver. The attached client's connection task routes `ControlAnswer` to `Hub::answer(call, answer)`. The control task awaits each receiver under `tokio::time::timeout(5 s)` and then removes the call, so a late answer finds no sender and is dropped. For `reload`, the server loads first and the clients after, in parallel.

### 5. The chosen client is tracked in the hub
`Hub` keeps, per session, the id of the client that last sent `Key`, `Paste` or `Mouse`, set by the attached connection task, and the order of attachment in `clients` (a `BTreeMap` keyed by increasing id) gives "attached last". The client number shown to users is the connection id, which is unique for the life of the server.

### 6. Server errors become a list
`Inner::error: Option<String>` becomes `errors: Vec<String>`. `report` appends and still broadcasts. `clear_error` empties it on a load without errors. An attaching client still receives only the latest, as today. `Input::Errors` reads the list together with the load number.

### 7. The client remembers where an error came from
`Display::errors` becomes a list of entries holding the text and whether it came from `ServerMessage::ServerError`. `gband.errors()`, the sidebar's `!`, the banner and the `errors.open` list keep reading the text as today. The control answer for `errors` leaves out the server's entries.

### 8. The reload action
`reload` is `ClientAction::Reload` in `crates/core/src/action.rs`, named `reload` in `crates/lua/src/actions.rs` next to `send_prefix`. The client handles it where it handles `ClientAction::SendPrefix`: it calls its loader, applies the result with `Controls::reload`, and sends `ClientMessage::Reload`. The server answers `ServerMessage::Reloaded` after its load and any `ServerError` it caused, so the client can tell when "Forced reload" has ended. The settings window's reopen after a reload from its `reload` line waits for `Reloaded`: `display.settings_line` holds the line, and a new flag marks that it waits for the server. The reload action from a key does not wait for anything.

### 9. The settings window draws its own count
`settings.lua` adds the `reload` line as the last entry of `LABELS`, at index 5 with the modal style and 4 otherwise. Its value comes from `#gband.errors()` each time the window draws. Enter on it calls `gband.action.reload()` and records the line as a save does. When the window is still open after a failed load, `gband.settings.open()` focuses it and draws its lines again, so the count is current.

### 10. Output in `main.rs`
The four subcommands are `Command` variants with `Role::Client`. A small module, `src/control.rs`, holds the conversion between `Value` and `serde_json::Value`, the text and JSON printing, and the exit statuses. `serde_json` is already a workspace dependency. The session comes from `-s`, then `GBAND_SESSION`, then `default`, resolved only for these four.

## Risks / Trade-offs

- [A client that is mid-frame or blocked on its terminal answers late] → the 5-second timeout reports `no answer` instead of hanging the agent, and the exit status is 1.
- [`eval` runs any Lua as the user] → the socket is already `0600` and owned by the user, so anything that can reach it could already open a window running a shell. No new privilege.
- [`prefix !` needs Shift on most layouts] → so does `prefix R`, `N` and `D`. Users can rebind it.
- [A user action named `reload` stops loading] → the proposal marks it breaking. The error names `reload`, as for any built-in name.
- [Reloading all clients from a pane closes every plugin window in them, as any reload does] → this is how a file change behaves today, so it is not new behaviour.
- [The settings window grows by one row] → the screenshot references under `tests/lua/screenshots/settings_spec/` change and must be rewritten and read.

## Migration Plan

The protocol version rises to 12, so a client and a server of different builds refuse each other, as every earlier protocol change did. Restart the server with `gband kill-server` after updating. No configuration file changes.
