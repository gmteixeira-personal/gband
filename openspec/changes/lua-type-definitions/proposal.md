## Why

lua-language-server reports `undefined global gband` on every line of the files gband writes to `defaults/`, such as `defaults/keystyle/modal.lua`, and on every user configuration and plugin. The executable creates the `gband` table at run time (`crates/lua/src/runtime.rs`, `crates/lua/src/sides.rs`), and gband ships no type definition file and no `.luarc.json`. Editors therefore offer no definition, completion or type check for `gband.keymap`, `gband.action`, `gband.core` and the rest of the documented API.

## What Changes

- gband bundles a lua-language-server definition file, `---@meta`, that declares the `gband` global: every field of the client's and the server's `gband` table, with each function's parameters and return values as `docs/plugins.md` gives them. `gband.action` also accepts any string key, so plugin actions such as `action["keylist.open"]` resolve.
- Each process writes the definition file to `defaults/meta/gband.lua` of the configuration directory, restored on every start like the other files under `defaults`. Loading never runs it, and `defaults/meta` is not a runtimepath entry.
- Each process writes `.luarc.json` at the top of the configuration directory when no file of that name exists, and never replaces one. It sets the runtime to Lua 5.4. Its presence makes editors that look for a root marker, such as Neovim's `lua_ls` setup, open the configuration directory as the workspace, so the definition file under `defaults/meta` is indexed.
- A test fails when the definition file misses a path of the client's or the server's `gband` table, or declares a path neither table holds. Changing the API without updating the definitions fails the build.
- The repository gains a `.luarc.json` that points lua-language-server at the bundled definition file, so the bundled Lua under `crates/lua/src/runtime/` and the example plugins check clean too.
- `docs/plugins.md` gains an "Editor support" section: what gband writes, and the `workspace.library` entry a plugin outside the configuration directory adds.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `configuration`: "Configuration directory" adds `defaults/meta/gband.lua`, written and restored like the other defaults, and `.luarc.json` at the top of the directory, written only when absent.
- `lua-api`: adds "Type definitions": the bundled definition file declares exactly the paths of the client's and the server's `gband` table, is never loaded, and is described in the API documentation.

## Impact

- Code: `crates/lua/src/directory.rs` writes the two files; `crates/lua/src/bundled.rs` embeds the definition file from `crates/lua/src/runtime/meta/gband.lua`.
- Tests: `crates/lua/tests/type_definitions.rs` compares the declared paths with the walked tables; the directory tests in `crates/lua/src/directory.rs` cover the new files.
- Docs: `docs/plugins.md` gains "Editor support".
- Users: no behavior of the multiplexer changes. An existing `.luarc.json` in the configuration directory is left as it is, so its owner adds nothing and keeps their settings; the definition file still sits in the workspace.
- `gband.api_version` stays 2: the change adds a file for editors and changes no documented function.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- .luarc.json
- crates/lua/src/bundled.rs
- crates/lua/src/directory.rs
- crates/lua/src/runtime/meta/gband.lua
- crates/lua/tests/type_definitions.rs
- crates/lua/tests/common/docs.rs
- docs/plugins.md
- openspec/changes/lua-type-definitions/
