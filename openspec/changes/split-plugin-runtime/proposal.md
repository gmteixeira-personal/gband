## Why

gband's client can run on the user's machine while the server runs on another host, reached later over SSH, or both can run on one machine. Lua today has no boundary for what runs where: the server evaluates the whole client configuration, plugins included, only to read the column width options, so a plugin's top-level code runs in both processes and nothing can run while no client is attached. Plugins such as agent notifications, status lines and key bindings need one model that behaves the same whether client and server share a machine or not, so that third parties can write them without guessing.

The rule: what must keep working while no client is attached, or must be shared by every attached client, runs in the server. What concerns one user's screen, keyboard or machine runs in the client. The two never share Lua state; they exchange plain data only.

## What Changes

- **Server Lua VM**: the server process runs one Lua state of its own, with `gband.side` `"server"`. It loads `user/server.lua` from the server machine's configuration directory, or the built-in server defaults written to `defaults/server.lua`, then the `server.lua` of every plugin. It reloads when `user/server.lua` changes. The instruction limit, ownership and plugin error isolation apply as in the client.
- **Plugin layout**: a plugin is a directory `plugins/<name>/` holding a manifest `plugin.lua` that returns `{ name, version }`, plus an optional `server.lua` loaded only by the server and an optional `client.lua` loaded only by each client. `lua/` modules stay requirable on each side. The manifest may name a client requirement that the server advertises.
- **BREAKING**: `plugin/*.lua` and `plugin/client/*.lua`, in plugin directories and in `user/`, are no longer sourced. Their code moves to `client.lua`.
- **BREAKING**: the server no longer evaluates `user/init.lua`. The options `default_column_width` and `width_presets` become server options, set in `user/server.lua`. Setting them in `user/init.lua` is reported as an unknown option.
- **Side guard**: each VM keeps a flat `gband` table for its own side. Reading an API of the other side, such as `gband.keymap` in the server, raises an error naming the API and the side, so a misplaced script fails at load.
- **Server API**: `gband.on` with server events (`SessionCreated`, `SessionEnded`, `PaneOpened`, `PaneClosed`, `PaneExited`, `PaneOutput`, `PaneInput`, `ClientAttached`, `ClientDetached`, `ConfigReloaded`); `gband.emit(name, data, opts)` to send an event to clients; `gband.pane_state(session, pane)` for the shared per-pane state table; `gband.cmd.register` for commands clients can call; `gband.action` with explicit targets for the session actions; `gband.sessions()` and `gband.layout(session)` to read structure; `gband.plugin`, `gband.opt` and `print` as on the client.
- **Message contract**: event data, pane state values, command arguments and command results are plain data: nil, booleans, numbers, strings, and tables of these without cycles, up to a nesting depth and an encoded size. Anything else is an error at the line that passed it. The contract has one encoding in the wire protocol, so every transport carries it unchanged.
- **Events to clients**: an emitted event reaches every attached client, or the clients of one session, as the client event `ServerEvent` with `name`, `data`, `queued` and `time`. Events emitted while no target client is attached are queued in the server, up to a limit, and delivered to the next client that attaches.
- **Shared pane state**: the server publishes every change of a pane's state to the clients of its session, and a full copy when a client attaches. The client exposes it through `gband.pane_state(pane)`, the event `PaneStateChanged`, and a `panes` list in the status line's render context.
- **Command RPC**: `gband.rpc(name, args, callback)` in the client calls a server command. The command runs in the server with the arguments and a context naming the caller's session and client, and can focus a pane in that client only. Its result or error returns to the callback.
- **Client local actions**: `gband.notify(text, opts)` raises a desktop notification through the client's terminal, `gband.bell()` rings it, `gband.clipboard(text)` sets the local clipboard through OSC 52, and `gband.open(target)` opens a URL or file with the client machine's opener.
- **Required plugins**: after attach the server sends the client requirements of its loaded plugins, such as `agent-status >= 0.1`. The client compares them with its own manifests and reports each missing or incompatible plugin as a configuration warning.
- **Trust**: the server never sends Lua code. A client loads only Lua from its own machine, so attaching to a server cannot run code locally.
- Server plugin and configuration errors are logged by the server and sent to attached clients, which show them prefixed with `server: `.
- **BREAKING**: protocol version 5. New client message `command`; new server messages `event`, `pane state`, `result`, `requirements` and `server error`.
- `docs/plugins.md` is rewritten around the two sides, and an example plugin `examples/plugins/agent-status` ships a `server.lua` that detects a waiting coding agent and a `client.lua` that notifies, counts waiting agents in the status line and jumps to the next one.

Out of scope: the SSH transport, plugin distribution or a package manager, process survival across a server crash, built-in pane title or working directory tracking, a server reload triggered by plugin directory changes, and running server code on behalf of a single client's view (focus and scroll stay client state).

## Capabilities

### New Capabilities

- `server-runtime`: the server's Lua state, its configuration files and defaults, load order, reload, server events, emitting and queueing events, pane state, server commands, targeted session actions, structure queries, and how its errors reach clients.
- `plugin-bridge`: the plain-data contract, delivery of server events to client handlers, shared pane state on the client, command RPC with results, required plugin advertisement and the client's check, and the rule that no code crosses the connection.
- `local-actions`: the client's notify, bell, clipboard and open functions.

### Modified Capabilities

- `plugins`: the runtimepath on each side, the manifest and side files replacing `plugin/*.lua`, `gband.side` per process, and the side guard.
- `configuration`: `defaults/server.lua`, the server configuration file, the split of options between client and server, server errors shown in the client, and reload of `user/server.lua`.
- `lua-events`: the events `ServerEvent` and `PaneStateChanged`, and patterns for `ServerEvent`.
- `status-line`: the `panes` field of the render context, and server errors in the error item.
- `wire-protocol`: protocol version 5 and the new messages.
- `session-server`: session actions applied from the server's Lua, and pane output and input observed by it without slowing panes.

## Impact

- `crates/lua`: a side parameter through `evaluate` and `install`; manifest reading and side file sourcing in `runtime.rs`; the side guard; a new `server` module with the server API, server events and pane state proxy; a `value` module converting Lua values to and from the protocol's plain-data value; client functions for `rpc`, `pane_state`, notify, bell, clipboard and open; `ServerEvent` and `PaneStateChanged`; an embedded `defaults_server.lua`; `watch` for `user/server.lua`.
- `crates/protocol`: the `Value` type, the new messages and version 5.
- `crates/server`: a scripting thread that owns the server Lua state, fed by the session bus, a pane output tap and input notices; pane state storage and broadcast; the event queue; command dispatch with results; requirements on attach; server error forwarding.
- `crates/client`: handling the new server messages, the requirement check, local actions, and the render context's panes.
- `src/main.rs`: the server loads its own configuration and passes its runtime to the server crate.
- Tests across `crates/lua`, `crates/server`, `crates/client`, `crates/protocol` and the top-level `tests/`. Existing tests that use `plugin/*.lua` or set width options in `user/init.lua` move to the new layout.
- `docs/plugins.md`, `README.md`, `examples/plugins/`.
- No new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- status-line
- key-hints

### Expected Files
- Cargo.lock
- crates/protocol/src/
- crates/protocol/tests/messages.rs
- crates/lua/
- crates/server/Cargo.toml
- crates/server/src/
- crates/server/tests/scripting.rs
- crates/client/src/
- crates/client/tests/
- src/main.rs
- tests/
- examples/plugins/
- docs/plugins.md
- README.md
- openspec/changes/split-plugin-runtime/
