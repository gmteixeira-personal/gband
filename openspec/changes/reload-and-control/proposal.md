## Why

gband reloads its configuration only when a `.lua` file under `user/` changes. A change to a plugin's files reloads nothing, and nothing outside the client can learn whether a load worked: the error shows only in the client's sidebar, banner and `gband.errors()`. A coding agent running in a gband window can edit the configuration but can neither reload it on demand nor check the result, and a person has no key that reloads. Programs in a window already find their server through `GBAND`, so a command line that talks to that server, in the manner of `zellij action`, `tmux` and `niri msg`, closes the gap for people and agents alike.

## What Changes

- **Reload action.** A new client action, `reload` (`reload the configuration`), loads this client's configuration again and asks the server to load its own, each as a file change would, reading the plugins' files as they are now. It always ends with `root` active.
- **Reload key.** `prefix !` binds `reload` in the modal, direct and floating presets. `prefix R` keeps resetting the window's height. With the floating style, `!` in the window list keeps the preview and runs `prefix !`, as the list's other `prefix` shortcuts do.
- **Reload line in the settings window.** A last line, `reload`, shows `ok` when `gband.errors()` is empty and the number of errors otherwise. Enter on it dispatches `reload`, and the settings window opens again on that line once both loads have ended, with the new count. The settings window grows by one line in every key style.
- **Load counter.** The client and the server each number their loads from 1 at start. Every load, by a file change, the reload action, a control request or the test channel, adds one, whether it succeeds or fails.
- **Server error list.** The server keeps every error of its Lua since its last load without one, instead of only the last.
- **Control subcommands.** Four new `gband` subcommands talk to the running server over its socket. They select the server as `attach` does, so from a gband window they reach that window's server, and the session from `-s`, else `GBAND_SESSION`, else `default`. Each prints text by default and JSON with `--json`, and exits with 1 when the request failed:
  - `gband reload` forces a reload of the server and of every client attached to the session, waits for each, and prints one result per process.
  - `gband errors` prints the error list of the server and of every client of the session, each with its load number.
  - `gband eval [--on-server | --client <n>] <source> [<arg>...]` runs a Lua chunk in one client of the session, or in the server, and prints its return values.
  - `gband cmd [--on-server | --client <n>] <name> [<args>]` runs a command registered with `gband.cmd`, or a server command, with JSON arguments, and prints its result.
- **Wire protocol 12.** A new request carries a control operation for a session. New messages let the server forward an operation to an attached client and collect its answer, and let an attached client ask the server to reload and learn when it has.
- **BREAKING** for a configuration that registers its own action named `reload` with `gband.action.register`: the name now belongs to a built-in action, so loading fails as for any built-in name. `gband.api_version` stays 2, since the documented API only grows.

## Capabilities

### New Capabilities
- `remote-control`: the `reload`, `errors`, `eval` and `cmd` subcommands, how they choose the session and the client, their text and JSON output, exit statuses, the answer timeout, and the load counter they report.

### Modified Capabilities
- `actions`: "Kinds of action" and "Lua names of actions" add the client action `reload`.
- `configuration`: a new requirement, "Forced reload", for the reload action: both sides reload, the plugins' files are read again, and `root` is active after it.
- `client-attach`: "Key bindings" adds `!` to the modal table, which the direct table follows.
- `floating-key-style`: "Floating bindings" adds `prefix !`.
- `settings`: "Settings window" adds the `reload` line and its value, with the new sizes; "Settings window keys" defines Enter on it; "Reopen after a save" reopens on that line after a reload from it.
- `command-line`: "Subcommands run the session", "Help and version", "Server selection" and "Session option" name the new subcommands and their session default.
- `logging`: the new subcommands log to the client series, as `list-sessions` does.
- `shell-completions`: the completion script offers the new subcommands.
- `wire-protocol`: "Handshake" raises the protocol version to 12; "Client messages", "Server messages" and "Requests" add the control request, its answers and the reload message.

## Impact

- Rust: `src/main.rs` (subcommands, the loaders that both the watcher and a forced reload use, output), `crates/protocol/src/message.rs` (messages, version 12), `crates/server/src/connection.rs`, `crates/server/src/hub.rs`, `crates/server/src/scripting.rs`, `crates/server/src/channel.rs` and `crates/server/src/lib.rs` (control requests, forwarding, error list, load counter), `crates/client/src/lib.rs`, `crates/client/src/connect.rs` and `crates/client/src/channel.rs` (answers, the reload action, the error origin), `crates/core/src/action.rs` and `crates/lua/src/actions.rs` (the action), `crates/lua/src/commands.rs` (a command's value), `crates/lua/src/runtime/gband/keystyle/*.lua` and `crates/lua/src/runtime/gband/settings.lua`.
- Tests: Rust integration tests for the subcommands and the protocol, Lua specs and screenshots for the key and the settings window.
- Docs: `README.md`, `docs/plugins.md` and the key style and settings chapters that list the keys and the settings window.
- No new dependency: `serde_json` is already in the workspace.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- openspec/changes/reload-and-control/
- Cargo.lock
- Cargo.toml
- README.md
- crates/client/src/bindings.rs
- crates/client/src/channel.rs
- crates/client/src/lib.rs
- crates/client/src/requests.rs
- crates/client/tests/actions.rs
- crates/core/src/action.rs
- crates/lua/src/actions.rs
- crates/lua/src/commands.rs
- crates/lua/src/runtime.rs
- crates/lua/src/runtime/gband/keystyle/direct.lua
- crates/lua/src/runtime/gband/keystyle/floating.lua
- crates/lua/src/runtime/gband/keystyle/modal.lua
- crates/lua/src/runtime/gband/settings.lua
- crates/lua/src/watch.rs
- crates/lua/tests/commands.rs
- crates/lua/tests/config.rs
- crates/lua/tests/control.rs
- crates/lua/tests/keystyle.rs
- crates/lua/tests/plugins.rs
- crates/lua/tests/settings.rs
- crates/protocol/src/lib.rs
- crates/protocol/src/message.rs
- crates/protocol/tests/messages.rs
- crates/server/src/channel.rs
- crates/server/src/connection.rs
- crates/server/src/hub.rs
- crates/server/src/lib.rs
- crates/server/src/scripting.rs
- crates/server/tests/control.rs
- crates/server/tests/scripting.rs
- crates/server/tests/sync.rs
- crates/test-support/src/lib.rs
- docs/internals/07-settings.md
- docs/internals/08-key-styles.md
- docs/plugins.md
- docs/tutorial/03-options.md
- src/control.rs
- src/main.rs
- tests/completions.rs
- tests/config.rs
- tests/control.rs
- tests/keystyle.rs
- tests/lua/floating_spec.lua
- tests/lua/keylist_spec.lua
- tests/lua/keystyle_spec.lua
- tests/lua/prompt_spec.lua
- tests/lua/screenshots/keylist_spec/default-list--last-page.txt
- tests/lua/screenshots/prompt_spec/the-key-list-holds-the-prompt-between-the-key-list-and-detach--hint.txt
- tests/lua/screenshots/settings_spec/the-settings-window-beside-the-default-sidebar-on-the-first-start--first-start.txt
- tests/lua/screenshots/settings_spec/the-settings-window-with-the-direct-key-style--direct.txt
- tests/lua/screenshots/settings_spec/the-settings-window-with-the-floating-key-style--floating.txt
- tests/lua/screenshots/settings_spec/the-theme-list-beside-the-default-sidebar--theme-list.txt
- tests/lua/settings_spec.lua
- tests/settings.rs
- tests/subcommands.rs
