## 1. Starting point

- [x] 1.1 Confirm `status-line` and `key-hints` have archive records on `origin/<base>`, and that `crates/lua/src/bundled.rs`, `crates/lua/src/runtime/gband/statusline.lua` and the `host.state()` primitive exist. Verify with `git ls-tree --name-only origin/<base> openspec/changes/archive/` and `rg -n "fn state|host.state" crates/lua/src`.

## 2. Plain data values

- [x] 2.1 Add `Value` and `Key` to `crates/protocol/src/value.rs`, exported from `lib.rs`, with postcard serialisation and a decode-side depth check of 32. Verify with `crates/protocol/tests/messages.rs` cases for "Value round trip" and a 33-deep value refused.
- [x] 2.2 Add `gband-protocol` to `crates/lua/Cargo.toml`, and `crates/lua/src/value.rs`: Lua to `Value` with the offending path in errors, cycle detection, metatables ignored, depth 32 and the 1 MiB size limit; `Value` to Lua building new tables. Verify with unit tests for "Nested data round trip", "Function inside a table" (path `data.cb.run`), "Cycle", "Binary string" and integer and string keys kept apart.

## 3. Sides, manifests and the guard in `crates/lua`

- [x] 3.1 Add `Side`, and pass it through `evaluate`, `install`, `load`, `load_defaults`, `defaults` and `watch`. Set `gband.side` from it, pick `user/init.lua` or `user/server.lua`, and pick `defaults.lua` or a new `defaults_server.lua`, which sets the two width options to their defaults. Verify with `crates/lua/tests/plugins.rs` cases for "Side and version" and "Server side", and that every existing client test passes with `Side::Client`.
- [x] 3.2 Add `crates/lua/src/version.rs` for versions and requirements. Verify with unit tests for `0.1` meeting `>= 0.1`, `0.1.4` failing `>= 0.2, < 1`, missing parts read as 0, and malformed input refused.
- [x] 3.3 Replace `plugin_files` in `runtime.rs` with manifest evaluation in an empty environment and one side file per plugin; report a side file without a manifest; add `gband.plugins()`. Verify with tests for "Valid manifest", "Name mismatch", "Manifest cannot reach gband", "Load order", "One file per side", "Client-only plugin", "Legacy plugin files are ignored", "Server plugin files are ignored", "Init file adds an entry", "Error in a plugin file" and "Default configuration with a plugin".
- [x] 3.4 Split the API install into shared and per-side parts, and add the `gband` and `gband.action` metatables of the side guard. Verify with tests for "Client API in the server", "Server API in the client" and "View action in the server".
- [x] 3.5 Give each built-in option a side in `options.rs`: move the widths to the server, add `notify_style` to the client, and report an option of the other side naming that side. Verify with `crates/lua/tests/options.rs` cases for "Server option in the client file", "Partial update" and "Invalid value falls back to the default" on the server side, and "Invalid status line height" unchanged.
- [x] 3.6 Write `defaults/server.lua` in `directory.rs::prepare` beside `defaults/init.lua`. Verify with unit tests for "First run" and "Edited defaults are restored".

## 4. Server API in `crates/lua`

- [x] 4.1 Add the server events to `events.rs` with their payloads, refuse `User` and client events on the server side and server events on the client side, and allow `pattern` for `ServerEvent`. Verify with tests for "User event refused", "Server event name in the client" and the payload of each server event built from a test `Event`.
- [x] 4.2 Add `crates/lua/src/server.rs` with host traits the server crate implements: `gband.emit` with `opts.session`, `gband.pane_state(session, pane)` as a proxy table with `__index`, `__newindex` and `__pairs`, `gband.sessions()`, `gband.session(name)`, and `gband.action` with target tables, each producing `Dispatch` effects or calling the host. Verify with unit tests against a fake host: emit validation errors at the call line, "Function refused", the 64 KiB state limit, equal assignment changing nothing, and an `open_pane` target with a program list.
- [x] 4.3 Give server commands the `(args, ctx)` signature with `session`, `client` and `focus`, and turn a non-plain result into a failure. Verify with tests for "Local run" and a non-plain result returning `false`.

## 5. Protocol version 6

- [x] 5.1 Add `ClientMessage::Command` and `ServerMessage::{Event, PaneState, Result, Requirements, ServerError}`, and set `PROTOCOL_VERSION` to 6, `plugin-windows` having taken 5. Verify with `crates/protocol/tests/messages.rs` cases for "Command round trip", "Result round trip" and "Version 5 client meets a version 6 server".

## 6. Server scripting thread and hub

- [x] 6.1 Add `crates/server/src/hub.rs` with pane states, the attached-client sets, the event queue of 256, the latest server error, the requirements and the delivery broadcast, and the attach copy taken under the same lock as the subscription. Verify with unit tests for "Queued while detached", "Queue limit", "One session" and state removal on `PaneClosed`.
- [x] 6.2 Add `gband-lua` to `crates/server/Cargo.toml`, and `crates/server/src/scripting.rs`: the `gband-lua` thread, the `Input` channel, the bus forwarder, effects routed to the registry and the hub, reloads swapping the `Config`, and server errors sent to the hub. Verify with `crates/server/tests/scripting.rs` cases for "Exit then close", "Handler replaced", "Kept across reload" and "Handler error shown".
- [x] 6.3 Add the output tap to `pane.rs` with the 4 MiB byte budget and the `AtomicBool` that skips it, and the input notice in `connection.rs::forward`. Verify with tests for "Agent prompt detected", "Input notice" and "Slow handler", the last checking that the pane's screen follows its output and the log records dropped bytes.
- [x] 6.4 Send `SessionCreated` and `SessionEnded` from the registry, and route server session actions to the session's `drive` task without a focus reply. Verify with tests for "Close from a handler", "Open without stealing focus" and "Action from the server's Lua".
- [x] 6.5 Handle `Command` in `connection.rs`: forward the call with a reply channel and the focus sender, and send `Result`. Verify with tests for "Command focuses for its caller", "Unknown command" and "Failing command".
- [x] 6.6 After the layout and snapshots of an attach, send pane states, requirements, the latest server error and the queued events, then deliveries from the broadcast filtered by session. Verify with tests for "Attach order with plugin state", "Broadcast" and "State at attach".

## 7. Server process wiring

- [x] 7.1 In `src/main.rs`, load the server side, send `options.layout` through the `watch` channel on each successful load, start the scripting thread with the `Config`, and watch `user/` for reloads with `Side::Server`. Remove the client load from the server path. Verify with `tests/config.rs` cases for "Width set for the server", "No server configuration file", "Broken server file at start", "Client file not evaluated" and "New default width".

## 8. Client side of the bridge

- [x] 8.1 Store pane states in `Display`, prune them with each layout, emit `PaneStateChanged` after the attach batch, and expose them to `gband.pane_state(pane)` and the render context's `panes` through `host.state()`. Verify with `crates/client/tests/events.rs` cases for "State at attach", "Change event", "Read-only copy", "State change" and "Nothing at attach", and a status line test for "Waiting agents counted".
- [x] 8.2 Deliver `ServerEvent` with `queued` and `time`, and match its `pattern`. Verify with tests for "Notify on a waiting agent", "Pattern filters" and "Server event pattern".
- [x] 8.3 Add `gband.rpc`, the pending call map and `Runtime::call_with`, and focus on a command's focus message. Verify with a client test for "Jump to the next waiting agent" against a scripted server stream.
- [x] 8.4 Check `Requirements` against the loaded manifests and report failures, and show `ServerError` with the `server: ` prefix in the error item or banner. Verify with tests for "Missing client side", "Old client side", "Met", "Server error in the client" and "Server plugin error".
- [x] 8.5 Add `gband.notify`, `gband.bell`, `gband.clipboard` and `gband.open` as dispatches the client loop writes after the current frame or spawns. Verify with `crates/client/tests/local_actions.rs` cases for "Default style", "Title with osc777", "Escape removed", "Ring", "Copy", "Open a URL" with a fake opener on `PATH`, and "Opener missing".

## 9. Migration of tests, examples and docs

- [x] 9.1 Move every test fixture writing `plugin/*.lua` or `plugin/client/*.lua` to a manifest and `client.lua`, and every width option set in `user/init.lua` to `user/server.lua`, across `crates/lua/tests/`, `crates/client/tests/` and `tests/`. Verify with `cargo test --workspace`.
- [x] 9.2 Move any `plugin/` file under `examples/plugins/` to a manifest and `client.lua`, and add `examples/plugins/agent-status/` with `plugin.lua` (`client = ">= 0.1"`), a `server.lua` that keeps a short tail per pane, sets `agent = "waiting"` and emits `agent.waiting` on a prompt, clears the state on `PaneInput`, and registers `agent-status.next_waiting`, and a `client.lua` that notifies on `agent.waiting`, adds a status line component counting waiting panes, and binds `prefix a` to the command. Verify with a `tests/` integration test that installs the example, prints the prompt in a pane, and sees the state, the count and the focus jump.
- [x] 9.3 Rewrite `docs/plugins.md` around the two sides: the guiding rule, the plugin directory layout and manifest, load order per side, the side guard, the plain data contract, the server API and events, pane state, `gband.emit` and the queue, commands and `gband.rpc`, local actions, requirements, and the trust rule. Update the configuration section of `README.md` for `user/server.lua` and the moved options. Verify that every API in the server-runtime, plugin-bridge, local-actions and plugins specs appears in the guide.

## 10. Gate

- [x] 10.1 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`, and confirm they pass.
- [x] 10.2 Manual check: install `examples/plugins/agent-status` on one machine, attach, run `printf 'Do you want to proceed?\n'` in a pane, and confirm a desktop notification, the status line count and `prefix a` focusing that pane. Then run `sleep 5; printf 'Do you want to proceed?\n'` in a pane, detach before it prints, wait, attach again, and confirm the queued event notifies once.
