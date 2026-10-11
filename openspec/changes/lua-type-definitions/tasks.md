## 1. Definition file

- [x] 1.1 Write `crates/lua/src/runtime/meta/gband.lua`: first line `---@meta`, a `---@class` per table of the client's and the server's `gband` table, and one `function <path>(...) end` or `<path> = <value>` statement per path, each with `---@param`, `---@return` and a side marker `(client)`, `(server)` or `(client and server)` taken from the matching section of `docs/plugins.md`. Give `gband.action` a `---@field [string] fun(...): any`. Verify the file compiles through `mlua` in a unit test
- [x] 1.2 Embed the file in `crates/lua/src/bundled.rs` with `include_str!` as its own constant, outside `MODULES` and `bundled::files()`. Verify `cargo test -p gband-lua` passes and no `require` reaches it: `user/init.lua` holding `require("meta.gband")` fails to load

## 2. Writing the files

- [x] 2.1 In `directory::prepare`, create `defaults/meta` and write the definition file to `defaults/meta/gband.lua` with `write_defaults`. Extend the first-run and restore tests in `crates/lua/src/directory.rs` to cover it. Verify with `cargo test -p gband-lua directory`
- [x] 2.2 Add a write-if-absent helper to `crates/lua/src/directory.rs` that writes a temporary file and hard-links it to the target, treats `AlreadyExists` as success and always removes the temporary file. Use it in `prepare` to write the built-in `.luarc.json` at the top of the configuration directory. Add tests for "First run", "Existing editor settings are kept" and "Removed editor settings are written again". Verify with `cargo test -p gband-lua directory`
- [x] 2.3 Add a test that appends `error("boom")` to `defaults/meta/gband.lua` of a prepared directory and reloads, expecting success and no reported error. Verify with `cargo test -p gband-lua`

## 3. Coverage test

- [x] 3.1 Add `crates/lua/tests/type_definitions.rs`: walk the client's `gband` table with every bundled plugin loaded through `gband.plugin` and the server's `gband` table using `common::docs::WALK`, collect the declared paths from the definition file by line scan, and assert both directions, skipping plugin actions under `gband.action`. Move any shared root-building code into `crates/lua/tests/common/docs.rs`. Verify with `cargo test -p gband-lua --test type_definitions`
- [x] 3.2 Add a test that declares nothing for a probe field set by `user/init.lua`, as `an_undocumented_field_is_reported` does in `api_docs.rs`, and asserts the first direction names it. Verify with `cargo test -p gband-lua --test type_definitions`

## 4. Editor support

- [x] 4.1 Add the repository `.luarc.json` with `runtime.version` `Lua 5.4` and `workspace.library` `["crates/lua/src/runtime/meta"]`. Verify `lua-language-server --check . --checklevel=Warning` from the repository root reports no `undefined-global` or `undefined-field` for `gband`, and resolve any `inject-field` or `duplicate-set-field` it reports on `gband` in the definition file
- [x] 4.2 Add an "Editor support" section to `docs/plugins.md`: the two files gband writes, that `.luarc.json` is written only when absent, that the configuration directory should be the editor's workspace, and the `workspace.library` entry naming `~/.config/gband/defaults/meta` and `~/.config/gband/defaults/lua` for a plugin directory elsewhere. Verify `cargo test -p gband-lua --test api_docs` passes
- [x] 4.3 Run a client against `XDG_CONFIG_HOME` set to a scratch directory, then run `lua-language-server --check <scratch>/gband --checklevel=Warning`. Verify it reports no undefined global `gband` and no undefined field of `gband` in `defaults/keystyle/modal.lua`, `defaults/init.lua` or `defaults/server.lua`

## 5. Gate

- [x] 5.1 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`. Verify all three exit with status 0
