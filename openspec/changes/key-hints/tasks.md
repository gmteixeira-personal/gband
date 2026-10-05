## 1. Preconditions

- [ ] 1.1 Confirm that `status-line` is archived on `<base>`. Diff its archived `status-line` and `configuration` specs against the MODIFIED blocks of this change, and reconcile every requirement that change altered after this one copied it. Verify with `openspec validate key-hints --strict` reporting no error and no "target spec does not exist" notice.

## 2. The `fill` field

- [ ] 2.1 In `crates/lua/src/runtime/gband/statusline.lua`, accept and validate `fill`, report it in `list()`, and keep the `width` of each component's latest context. Verify with `crates/lua/tests/statusline.rs` tests for "Fill of the wrong type" and a `list()` entry holding `fill`.
- [ ] 2.2 Order each trigger's renders with fill components last, in descending priority then insertion order. After the pass, lay out, render again each enabled fill component whose available width changed, then lay out and present. Verify with `crates/lua/tests/statusline.rs` tests for "Fill renders after the others", "Fill follows another component's width" and "Fill width unchanged", counting render calls, and that the extra pass causes no third render.

## 3. Hints segment

- [ ] 3.1 Write `crates/lua/src/runtime/gband/statusline/hints.lua` with `setup` and option validation, the `KeyHintKey` and `KeyHintLabel` defaults, and the component settings. Register it in `crates/lua/src/bundled.rs` if the bundle lists files by name. Verify with `crates/lua/tests/key_hints.rs` tests for "Component entry", "Invalid option", "Linked defaults" and "Theme override".
- [ ] 3.2 Implement the key form. Verify with `crates/lua/tests/key_hints.rs` tests for every "Key form" scenario, plus `escape` shown as `esc` and `ctrl+alt+x` shown as `C-A-x`.
- [ ] 3.3 Implement the hints of the active table and their labels, including the prefix hint in `root`, the `root` option, and bindings left out. Verify with `crates/lua/tests/key_hints.rs` tests, rendering with a given context, for every "Hint labels" scenario and for "Root with the defaults", "Prefix table with the defaults", "Named table", "No prefix bindings", "Root hints turned off" and "Groups".
- [ ] 3.4 Implement fitting to `ctx.width`. Verify with `crates/lua/tests/key_hints.rs` tests for "Some hints left out", "All hints fit" and "Nothing fits", and a hint holding a wide character.

## 4. Defaults, client and integration

- [ ] 4.1 Add `gband.plugin("gband.statusline.hints")` to `crates/lua/src/defaults.lua` between `mode` and `position`. Verify with the updated "Defaults reproduce the built-in behaviour" test and "Copied defaults load unchanged".
- [ ] 4.2 Update the client's default status line snapshots and add one after Ctrl+Space showing the prefix table's hints cut with `…`. Verify with `cargo insta test -p gband-client` accepting only the intended changes, and with a `crates/client/tests/events.rs` test for "Back to root".
- [ ] 4.3 Update `tests/statusline.rs` for the default line, and add an integration test for "Hints in the default line". Verify with `cargo test --test statusline`.

## 5. Docs

- [ ] 5.1 Extend `docs/plugins.md`: the `fill` field and its render order in the status line section, and a hints segment section with its options, label precedence, short labels and groups. Verify that every option and group in the key-hints spec appears in the guide.
- [ ] 5.2 Update `README.md` to list the hints segment among the default segments and show its `gband.plugin` line. Verify by reading it against the configuration and key-hints specs.

## 6. Gate

- [ ] 6.1 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`, and confirm they pass.
- [ ] 6.2 Manual check: attach with no `user/init.lua` and confirm the bottom row shows `C-space prefix` after `band 1`. Press Ctrl+Space and confirm the prefix hints replace it, ending with `…` on an 80-column terminal, and return after the next key. Narrow the terminal and focus columns until the position grows, and confirm the hints shrink without the line overflowing.
