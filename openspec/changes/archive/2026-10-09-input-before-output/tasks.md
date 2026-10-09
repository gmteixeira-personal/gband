## 1. Server

- [x] 1.1 In `crates/server/src/connection.rs`, make `forward` take a hook that runs after the window entry is found in the session state and before the input is sent to the entry's input channel. A missing window still returns false without running the hook. A failed send still returns false after the hook ran. In `dispatch`, pass `context.taps.notice(&session.name, window, client)` as the hook for `ClientMessage::Key` and `ClientMessage::Paste`, and a hook that does nothing for `ClientMessage::Mouse`. Verify with `cargo build -p gband-server` and the existing `input_notice` test in `cargo test -p gband-server --test scripting`
- [x] 1.2 Add `input_before_its_output` to `crates/server/tests/scripting.rs` beside `input_notice`, for the "Input before the output it causes" scenario: a server config whose `WindowInput` handler counts calls for the window, and whose `WindowOutput` handler keeps a tail of `data` and emits `seen` with the count the first time the tail holds `input`. Wait for the prompt, `type_line("printf 'in%s\\n' put")`, and assert the emitted count is 2. Verify with `cargo test -p gband-server --test scripting input_before_its_output`, then run it 20 times in a loop and check that every run passes

## 2. Agent status spec

- [x] 2.1 In `examples/plugins/agent-status/tests/agent_status_spec.lua`, add `g.wait_text("$")` after `g.start` and before `g.run` in "a prompt marks the window waiting" and "typing clears the waiting state". Verify with `cargo test --test lua_specs example_plugin_agent_status`, and check that the reference under `examples/plugins/agent-status/tests/screenshots/agent_status_spec/` is unchanged

## 3. Documentation

- [x] 3.1 In `docs/plugins.md`, after "A session's events reach the handlers in the order the session applied the changes", add that a key's or paste's `WindowInput` comes before the `WindowOutput` of anything the program writes after reading it, so a handler can clear on input what it set from output. Verify by reading the section, and with `cargo test --workspace`, which runs the documentation's Lua blocks

## 4. Gate

- [x] 4.1 Run `.claude/flow-gate` and check that fmt, clippy and the workspace tests pass
