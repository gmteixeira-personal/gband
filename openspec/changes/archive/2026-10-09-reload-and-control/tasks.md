## 1. Protocol

- [x] 1.1 In `crates/protocol/src/message.rs`, raise `PROTOCOL_VERSION` to 12 and add the types of design decision 3: `ClientMessage::Control`, `ClientMessage::Reload`, `ClientMessage::ControlAnswer`, `ServerMessage::Reloaded`, `ServerMessage::Control`, `ServerMessage::ControlResults`, with `Target`, `Operation`, `Answer`, `Process` and `Entry`, exported from `crates/protocol/src/lib.rs`. Add round-trip tests for a control request naming `work`, client 2 and eval `return 1`, and for each new message, to `crates/protocol/tests/messages.rs`, and update the version tests. Verify with `cargo test -p gband-protocol`

## 2. Loaders and load numbers

- [x] 2.1 In `src/main.rs`, build one loader per side from `Locations` or the defaults, and pass it to the client in `Configuration` and to the server in `ServerConfig` whether or not a test channel is joined. Make the watcher and both `TestChannel`s call it, keeping the server's `options_tx` update. Verify with `cargo test --workspace --test config` and `cargo test --test lua_specs`, which cover the test channel's reload
- [x] 2.2 Count loads: the client's `Controls` starts at 1 and adds one in `Controls::reload`, carrying the count across the `*self` replacement; the server's scripting thread starts at 1 and adds one on `Input::Reload`. Add a Rust test in `crates/client/tests/actions.rs` and one in `crates/server/tests/scripting.rs` that a failed load counts. Verify with `cargo test -p gband-client` and `cargo test -p gband-server --test scripting`

## 3. Error lists

- [x] 3.1 In `crates/server/src/hub.rs`, replace `error: Option<String>` by a list that `report` appends to and a load without errors empties, still sending only the latest to an attaching client. Update `latest_error_until_cleared` and add a test that two errors are both kept. Verify with `cargo test -p gband-server`
- [x] 3.2 In `crates/client/src/lib.rs`, keep each error with whether it came from `ServerMessage::ServerError`, with `gband.errors()`, the sidebar, the banner and the error list unchanged. Verify with `cargo test --test errors` and `target/debug/gband test tests/lua/errors_spec.lua`

## 4. Server side of control

- [x] 4.1 Add `Input::Errors` to `crates/server/src/scripting.rs`, answering the server's load number and error list, and make the answer to a reload carry the load number. Verify with `cargo test -p gband-server --test scripting`
- [x] 4.2 In `crates/server/src/hub.rs`, add the pending calls map, `Hub::control` and `Hub::answer`, and track per session the client that last sent a key, paste or mouse message. In `crates/server/src/connection.rs`, route an attached client's `ControlAnswer` to `Hub::answer`, record its input for the chosen client, and handle `ClientMessage::Reload` by loading through the server's loader, waiting on `Input::Barrier` and sending `ServerMessage::Reloaded`. Verify with `cargo test -p gband-server`
- [x] 4.3 Handle the `Control` request in `connection.rs` as design decision 4 describes: no such session, the server's part first, then each addressed client with the 5-second timeout, `ControlResults` in ascending client number, and close. Add `crates/server/tests/control.rs` covering the wire-protocol scenarios "Control request answered by every client", "Control request for an absent session" and "Reload message answered", and the remote-control scenario "Stopped client" with a client stand-in that never answers. Verify with `cargo test -p gband-server --test control`

## 5. Client side

- [x] 5.1 Add `ClientAction::Reload` to `crates/core/src/action.rs` and the action `reload`, `reload the configuration`, to `crates/lua/src/actions.rs`. In `crates/client/src/lib.rs`, handle it by loading through the client's loader, applying it with `Controls::reload` and sending `ClientMessage::Reload`. Update the action-list tests in `crates/lua/tests/config.rs` and `crates/lua/tests/control.rs` for "Every action is named" and "Reload description", and add a dispatch test to `crates/client/tests/actions.rs`. Verify with `cargo test -p gband-lua` and `cargo test -p gband-client --test actions`
- [x] 5.2 Answer `ServerMessage::Control` in the client's main loop: reload as the test channel does, errors from the client's own entries, eval through `Controls::eval`, and command by running the command's function and returning its value, reporting an error as `gband.cmd.run` does, through a new function in `crates/lua/src/commands.rs`. Verify with `cargo test -p gband-client` and `cargo test -p gband-lua --test commands`

## 6. Subcommands

- [x] 6.1 Add `reload`, `errors`, `eval` and `cmd` to `Command` in `src/main.rs`, with `--json`, `--on-server` and `--client <n>` where the remote-control spec gives them, `-s` then `GBAND_SESSION` then `default` for the session, the client log series, and the invalid invocations with status 2. Put the JSON conversion, the text printing and the exit statuses in `src/control.rs`, with unit tests for every scenario of "JSON values". Add a client entry point in `crates/client/src/connect.rs` that sends one control request and reads `ControlResults` or `NoSuchSession`. Verify with `cargo test --bin gband`
- [x] 6.2 Add `tests/control.rs` with real servers and clients for "No server", "No such session", "From a window", "Without a terminal", "Load after a file change", every scenario of "Reload subcommand", "Errors subcommand", "Eval subcommand", "Cmd subcommand", "Chosen client" and "JSON output", and the command-line scenarios "Run reload", "Session of the window", "Option over the window's session" and "Outside a window". Verify with `cargo test --test control`
- [x] 6.3 Update `tests/subcommands.rs`, `tests/completions.rs` and the logging tests for the help listing, the eleven completed subcommands and "Reload logs to the client series". Verify with `cargo test --test subcommands` and `cargo test --test completions`

## 7. Key bindings

- [x] 7.1 Bind `prefix !` to `action.reload` with the description `reload the configuration` after `prefix s` in `crates/lua/src/runtime/gband/keystyle/modal.lua`, `direct.lua` and `floating.lua`. Update `crates/lua/tests/keystyle.rs`, `crates/lua/tests/config.rs` and `tests/keystyle.rs` for the binding lists and "Prefix bindings in order", and add "Reload the configuration", "Reload with the direct key style", "Reset height keeps its key" and "Reload from the window list" to `tests/keystyle.rs`. Verify with `cargo test --test keystyle`, `cargo test -p gband-lua --test keystyle` and `cargo test -p gband-lua --test config`
- [x] 7.2 Add the configuration scenarios "Plugin file changed", "Server plugin changed", "Broken configuration", "Back to typing" and "Other clients keep theirs" to `tests/config.rs` or a Lua spec under `tests/lua/`, whichever drives two clients and a plugin directory already. Verify with `cargo test --test config` or `target/debug/gband test` on that spec

## 8. Settings window

- [x] 8.1 In `crates/lua/src/runtime/gband/settings.lua`, add the `reload` line last, its value from `#gband.errors()` (`ok`, `1 error`, `<n> errors`), Enter dispatching `gband.action.reload()` and recording the line, and `h`, `l`, Left and Right doing nothing on it. Make `gband.settings.open()` draw the lines again when the window is open. In `crates/client/src/lib.rs`, hold the reopen after a reload from that line until `ServerMessage::Reloaded`, and reopen after a failed load too. Verify with `cargo test -p gband-lua --test settings` and `cargo test --test settings`
- [x] 8.2 Update `tests/lua/settings_spec.lua` and `tests/settings.rs` for every changed and new scenario of the settings delta: the sizes 7 and 6 rows, `reload   ok`, the counts, Enter and `l` on the line, "Back on the reload line", "Failed reload counted" and "Server error counted". Rewrite the references under `tests/lua/screenshots/settings_spec/` with `target/debug/gband test --update tests/lua/settings_spec.lua`, read each one against its scenario, then run without `--update`. Verify with `cargo test --test lua_specs settings_and_themes`

## 9. Documentation

- [x] 9.1 Update `README.md`: the key tables gain `prefix !`, the settings window gains its `reload` line, and a new section on controlling a running gband from a shell or an agent with `gband reload`, `gband errors`, `gband eval` and `gband cmd`, their session from `GBAND_SESSION`, `--json` and exit statuses. Verify with `rg -n "gband reload|prefix !" README.md`
- [x] 9.2 Update `docs/plugins.md` (the `reload` action in the action list, `prefix !` in the key tables, the settings window's line) and `docs/tutorial/03-options.md` where it names the settings window's lines. Verify with `cargo test -p gband-lua --test api_docs` and `cargo test --test lua_specs tutorial`
- [x] 9.3 Update the quotes and prose of `docs/internals/07-settings.md` for `settings.lua` and of `docs/internals/08-key-styles.md` for the three key style files. Verify with `cargo test -p gband-lua --test tutorial`

## 10. Gates

- [x] 10.1 Search the main specs with `rg -n "protocol version SHALL be 11|five subcommands|seven subcommands|holds three lines|6 rows high|5 rows high" openspec/specs` and confirm that each hit is in a requirement this change's deltas modify. Run `openspec validate reload-and-control --strict`
- [x] 10.2 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace`, and verify that all pass
