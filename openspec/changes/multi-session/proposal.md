## Why

A gband server hosts exactly one session, so keeping two unrelated pieces of work apart means running two servers. A user with several projects wants each one in its own session, with its own workspaces and panes, behind one server that they attach to by name. The option letter is already reserved: `-s` selects a session, while server and socket selection are separate options.

## What Changes

- A server hosts any number of named sessions. Each session owns its own layout, panes, screen area and attached clients, as layout-core defines a session.
- A global option `-s <name>` selects a session. It is accepted before or after the subcommand. Without it, the session is named `default`.
- `gband server` starts with one session, the one `-s` names.
- `gband attach` attaches to the named session in the server the client selects. When that server has no session of that name, the server creates it, with its first pane in the client's working directory.
- A session ends when its last pane's program exits. Only that session's clients are told it ended. The server exits when its last session ends.
- New subcommand `gband list-sessions`: prints each session's name, pane count and attached-client count.
- New subcommand `gband kill-session`: ends the session `-s` names, hanging up every pane in it.
- `gband kill-server` and SIGTERM stop every session.
- Panes get `GBAND_SESSION`, set to their session's name, in their environment.
- **BREAKING**: the protocol version becomes 3. After the server's info, a client sends one request: attach to a named session, list sessions, or kill a named session. The hello and its answer keep their encoding.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `session-server`: a server hosts several named sessions. Covers session creation on start and on attach, per-session layout, screen area and input, a session ending without ending the server, the pane environment, SIGTERM and `kill-server` across sessions, and the new `kill-session` and `list-sessions` subcommands.
- `client-attach`: the client names a session when it attaches, and the server it starts creates that session.
- `wire-protocol`: protocol version 3, with the request a client sends after the server's info, and the server's answers to listing and killing sessions.
- `command-line`: the global `-s` option, its default and validation, and the `list-sessions` and `kill-session` subcommands.

## Impact

- Code: the `-s` option and two subcommands in `src/main.rs`. Request messages and session answers in `crates/protocol/src/message.rs`. A session map in `crates/server/src/`, where layout-core's session task becomes one per session and the server ends with the last one. The attach request and the two new commands in `crates/client/src/`.
- Tests: server tests for two sessions, per-session input, screen area and ending, listing and killing. End-to-end tests in `tests/` for `-s`, `list-sessions` and `kill-session`.
- Dependencies: none added.
- Compatibility: a version 2 client and a version 3 server reject each other through the existing handshake. A debug client replaces a version 2 server, which ends that server's one session.
- Out of scope:
  - Selecting the server or the socket. A separate change defines that. This change uses the server the client selects.
  - Renaming a session, and switching sessions from inside an attached client.
  - Moving panes between sessions.
  - Commands run inside a pane defaulting to that pane's own session.
  - A session-specific working directory or environment beyond the first pane's working directory.

## Coordination

### Author
- gmteixeira

### Depends On
- layout-core

### Expected Files
- crates/protocol/src/
- crates/protocol/tests/messages.rs
- crates/server/src/
- crates/server/tests/common/mod.rs
- crates/server/tests/lifecycle.rs
- crates/server/tests/sessions.rs
- crates/client/src/
- crates/client/tests/connect.rs
- src/main.rs
- tests/common/mod.rs
- tests/subcommands.rs
- tests/kill_server.rs
- tests/sessions.rs
- openspec/changes/multi-session/
