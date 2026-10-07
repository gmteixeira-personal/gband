## Why

Every bundled Lua file uses only the public API, and each process writes a copy of it under `defaults/` so users can read, copy and replace it. The files hold no comments, by the project's coding rule, so nothing explains why gband's own Lua is built the way it is. The scripting tutorial, which this change depends on, teaches the API from the outside. Nothing teaches how gband builds its API modules, bundled plugins, key styles and themes on `gband.core`. A walk-through written by hand would drift with every edit to the bundled Lua. The suite must therefore check it, as `scripting-tutorial` already does for the scripting tutorial.

## What Changes

- **Internals tutorial**: a new `docs/internals/` holds an index, `docs/internals/README.md`, and thirteen chapters, `00-boundary.md` to `12-own-prelude.md`. They walk through the files gband writes under `defaults/`, in the order the prelude loads the API:
  1. the boundary between the executable and Lua;
  2. a colorscheme, through the `gruvbox` theme;
  3. the palette and highlight groups;
  4. colorschemes;
  5. bars;
  6. plugin windows and their provider;
  7. settings;
  8. key styles;
  9. the bundled plugins;
  10. the default configurations.
  The last chapter has the reader write their own `gband.prelude`. The tutorial explains the real files as they are. It does not ask the reader to write replacements.
- **Quotes of the bundled files**: a chapter quotes a bundled file in a fenced block whose info string is `lua` followed by the file's path under the configuration directory, such as `` ```lua defaults/lua/gband/bar.lua ``. Each quote is a run of the file's lines, verbatim. Quotes may appear in any order, so a chapter can explain a function before the helpers it calls.
- **Whole-file coverage**: every non-blank line of every covered file is quoted exactly once across the chapters. The covered files are every file under `defaults/` except the colorschemes other than `defaults/colors/gruvbox.lua`, and except `defaults/lua/gband/theme/catppuccin.lua`, the color tables that only the catppuccin colorschemes read. One colorscheme shows how all of them are built, and the others differ from it in data only. Every covered file is named by its path in some chapter. Any edit to a covered file therefore fails the suite until the tutorial is revised in the same change.
- **Every primitive taught**: every field of the client's `gband.core`, at every depth, is named as `gband.core.<field>` in some chapter.
- **Chapter directories**: a chapter that shows code of its own, such as the reader's prelude in the last chapter, has a runnable directory under `examples/internals/`. Its `lua` blocks and `screen` blocks are checked as the scripting tutorial's are, and the suite runs `gband test` in it.
- **Links**: the README's Scripting section, the section "The prelude and the API modules" of `docs/plugins.md`, and the scripting tutorial's index link to the internals tutorial's index.
- **Licensing**: every file under `docs/` is licensed under the MIT License, the internals tutorial included. `LICENSE-MIT` lists `docs/` in place of `docs/tutorial/`, and the README's License section says so.

Out of scope:
- Rewriting or reordering bundled Lua for teaching. The tutorial follows the files as they are.
- The colorschemes other than `gruvbox`, and the catppuccin color tables in `defaults/lua/gband/theme/catppuccin.lua`.
- `gband.test`, the test side's module. It is not written under `defaults/`, and `docs/testing.md` and the scripting tutorial cover it.
- A `gband test` mode that runs gband's own Lua spec suite against a user's copies of the bundled files.
- Copies of the tutorial written under `defaults/` of the configuration directory.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tutorials`, created by `scripting-tutorial`: adds the internals tutorial, the rules for quoting bundled files, whole-file coverage of `defaults/`, coverage of `gband.core`, and the internals chapter directories.

## Impact

- `docs/internals/`: new, the index and thirteen chapters, about 2,900 quoted lines of bundled Lua with the explanation between them.
- `examples/internals/`: new, a directory for each chapter that shows code of its own, starting with `12-own-prelude/`.
- `crates/lua/tests/tutorial.rs`: the checks for quotes, whole-file coverage of the covered files, file naming and `gband.core` coverage. The chapter, `lua` block and `screen` block checks are generalized to cover `docs/internals/` and `examples/internals/`.
- `tests/lua_specs.rs`: the `tutorial` test also discovers `examples/internals/`.
- `README.md`, `docs/plugins.md` and `docs/tutorial/README.md`: links to the internals tutorial.
- `LICENSE-MIT` and the README's License section: `docs/` as a whole is MIT.
- No change to the executable, the bundled Lua, the Lua API, the protocol or the harness, and no new dependency.
- Every later change that edits a covered file, from `crates/lua/src/runtime/gband/`, `crates/lua/src/defaults.lua` or `crates/lua/src/defaults_server.lua`, must revise the internals tutorial in the same change. Its Expected Files lists only the chapter files that quote the files it edits, never the directory `docs/internals/`, so that such changes can still run side by side.

## Coordination

### Author
- gmteixeira

### Depends On
- scripting-tutorial

### Expected Files
- docs/internals/
- examples/internals/
- crates/lua/tests/tutorial.rs
- tests/lua_specs.rs
- README.md
- docs/plugins.md
- docs/tutorial/README.md
- openspec/changes/lua-internals-tutorial/
