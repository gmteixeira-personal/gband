## Context

See proposal.md, "Why". Three facts shape the approach:

- `directory::prepare` (`crates/lua/src/directory.rs`) already writes every bundled file under `defaults/` through `write_defaults`, which skips identical content and replaces a changed file through a temporary file and a rename.
- `crates/lua/tests/api_docs.rs` already walks the client's and the server's `gband` tables with `common::docs::WALK` and checks each path against `docs/plugins.md`. The same walk can check a definition file.
- lua-language-server indexes every `---@meta` file inside its workspace. Neovim's `lua_ls` setup picks the workspace from root markers, `.luarc.json` first, and falls back to single-file mode when none is found. In single-file mode no other file is indexed, so a definition file alone does not help a user who opens `~/.config/gband/defaults/keystyle/modal.lua` today.

## Goals / Non-Goals

**Goals:**
- No `undefined-global` or `undefined-field` diagnostic for `gband` in the files gband writes, in `user/`, or in the repository's bundled Lua.
- Completion and hover text for the documented API.
- A build that fails when the definitions and the runtime tables disagree.

**Non-Goals:**
- Types for the exports of `require("gband.hl")` and the other modules. The sources sit under `defaults/lua/gband/`, so lua-language-server resolves `require` from them with its default `runtime.path` and `runtime.pathStrict` false.
- Types for the test side, `require("gband.test")`. The `gband` global the definition file declares already removes the undefined-global diagnostic in test files.
- Generating the definitions from Rust. The API is defined partly by Rust and partly by bundled Lua, so a generator would need both and would still lack the parameter types that only `docs/plugins.md` holds.
- Other language servers such as EmmyLua. The `---@meta` annotations are lua-language-server's format.

## Decisions

### One hand-written definition file, checked by a test

The file lives at `crates/lua/src/runtime/meta/gband.lua`, outside `runtime/gband/`, so no `require` and no `bundled::files()` listing reaches it. `bundled.rs` embeds it with `include_str!` as its own constant, and `prepare` writes it with `write_defaults` to `defaults/meta/gband.lua`.

Each table is a `---@class` (for example `gband.Keymap`, `gband.Core`). Each path is declared by an assignment or a function statement on its full path, `function gband.keymap.set(table, key, action, opts) end` or `gband.api_version = 2`, with `---@param`, `---@return` and a description line that names the side: `(client)`, `(server)` or `(client and server)`. Declaring by full path keeps the checker a line scan: the test reads every `function <path>(` and `<path> =` at the start of a line whose path starts with `gband.`, and normalises `["name"]` to `.name`.

`gband.action` is a class with `---@field [string] fun(...): any` for plugin actions, plus one function statement per built-in action.

Alternative: generate the file from the walk at build time. Rejected: the walk knows names and Lua types, not parameters, returns or descriptions, so the output would be little better than `diagnostics.globals = {"gband"}`.

### The test walks client, plugins and server

`crates/lua/tests/type_definitions.rs` reuses `common::docs::WALK`. It loads a client whose `user/init.lua` calls `gband.plugin` for each bundled plugin (`gband.sidebar`, `gband.keylist`, `gband.errors`, `gband.prompt`, `gband.desktop`) and a server with an empty `user/server.lua`, walks both `gband` tables, and takes the union. It asserts both directions: every walked path is declared, and every declared path was walked, apart from the string index of `gband.action`. The plugins' own actions, such as `gband.action.keylist.open` in walk notation, are covered by that string index and skipped by the first assertion.

If `WALK` needs a shared helper to build the roots, it moves into `common/docs.rs` rather than being copied.

### `.luarc.json` at the top of the configuration directory, written once

The file is what makes `~/.config/gband` the workspace root. Its built-in content is:

```json
{
  "$schema": "https://raw.githubusercontent.com/LuaLS/vscode-lua/master/setting/schema.json",
  "runtime.version": "Lua 5.4"
}
```

Lua 5.4 matches the `lua54` feature of `mlua` in the workspace `Cargo.toml`. No `workspace.library` entry is needed, because `defaults/meta/gband.lua` is inside the workspace; listing it as a library as well would index it twice.

It is written only when absent: a temporary file in the same directory, then `fs::hard_link` to `.luarc.json`, which fails with `AlreadyExists` instead of replacing a file another process or the user created in between. `AlreadyExists` counts as success. The temporary file is removed in every case. The file sits outside `user/`, so the "leave `user` unchanged" rule is untouched; the user owns it from the moment it exists.

Alternative: rewrite it on every start like `defaults/`. Rejected: users add their own settings, such as `diagnostics.globals` for another tool, and a restored file would drop them on the next start.

Alternative: put it in `defaults/.luarc.json`. Rejected: that makes `defaults/` the root only for files under it, and `user/init.lua` keeps the diagnostic.

### Repository `.luarc.json`

The repository root gains a `.luarc.json` with `runtime.version` Lua 5.4 and `workspace.library` set to `crates/lua/src/runtime/meta`. The bundled modules, the presets and `examples/plugins/` then check against the same definitions users get. It changes no build.

## Risks / Trade-offs

- [The definitions drift from the documentation in parameter types, which the test cannot see] → The test pins names only. `docs/plugins.md` stays the reference; tasks write each annotation from the matching docs section.
- [lua-language-server reports `inject-field` or `duplicate-set-field` where the bundled modules assign `gband.win = M` and the definition file declares `gband.win`] → The verification task runs `lua-language-server --check` on a prepared configuration directory and on the repository; any such diagnostic is resolved in the definition file, by declaring the field on the class instead of by assignment.
- [A user who already has `~/.config/gband/.luarc.json` without the runtime version gets Lua 5.4 features flagged] → Their file wins by design; "Editor support" in `docs/plugins.md` names the setting.
- [A user who opens `user/` alone as the workspace sees no definitions] → "Editor support" says to open the configuration directory, or to add `defaults/meta` to `workspace.library`.

## Migration Plan

None. The next start of any gband process writes both files. Removing the change leaves a harmless `defaults/meta/gband.lua` and `.luarc.json` behind.
