## Context

See proposal.md for why. The pieces this change builds on:

- `docs/plugins.md` and `docs/testing.md` are the reference. `crates/lua/tests/api_docs.rs` and `crates/harness/tests/api_docs.rs` walk the live `gband` tables with the `WALK` chunk of `crates/lua/tests/common/docs.rs` and fail when the reference misses a field.
- `examples/plugins/{hello,window,agent-status}` each hold `tests/*_spec.lua`, and `tests/lua_specs.rs` runs `gband test` in each through its `gband_test` helper, using the binary `CARGO_BIN_EXE_gband`.
- `gband test` in a directory runs every `_spec.lua` under its `tests/`. `g.start` takes `config`, `server_config`, `files` and `plugins`, the last relative to the test file's directory. The test side's Lua has `io`: a case that runs `io.open("Cargo.toml")` passes under `gband test -`, reading relative to the working directory.
- Screenshot references are text files: a `size ... cursor ...` line, one `N|<row>` line per screen row with trailing spaces trimmed, `--`, then the cell attribute lines.
- `gband_lua::API_VERSION` is the Rust constant behind `gband.api_version`.
- A `user/init.lua` replaces the default configuration rather than adding to it, so a configuration written from an empty file binds only what it asks for.

## Goals / Non-Goals

**Goals:**
- Every chapter's code runs, and the suite proves it on every gated merge.
- A reader can run any chapter's state with `gband test`, with no tool beyond gband.
- The chapters read as plain Markdown on GitHub, with no include syntax.

**Non-Goals:**
- Checking the prose. Only the code blocks, the screens, the API version and namespace coverage are checked; the chapters' own tests back what the prose claims.
- Running the chapters under `bundled_copies`. `every_case_with_bundled_copies` covers gband's own Lua suite; the chapters run once.

## Decisions

### The test suite keeps the tutorial current, not a release step

gband has no CI, and a release is `dev` merged into `main`. The `/ready` gate runs `cargo test` on every change before it merges into `dev`, so a check in `cargo test` holds on every commit a release can be cut from. A release checklist was rejected: nothing runs it, and it would find drift long after the change that caused it.

### The chapter directories are the source; the chapters quote them

Each chapter's code lives as real files in `examples/tutorial/NN-<slug>/`, and the chapter's `lua` blocks must appear in those files verbatim. A reader can copy a directory and run it, and `gband test` runs it with no extraction step. This is the pattern `examples/plugins/` and `api_docs.rs` already follow.

Alternatives rejected:
- Extracting the blocks from the Markdown into a scratch directory and running them. The tests and the files a chapter does not show would have to live in the Markdown too, and a reader could not run a chapter's state without the extractor.
- mdBook-style `{{#include}}` directives. GitHub renders them as literal text.

### Each chapter directory is a complete snapshot

A chapter directory holds every file at the end of that chapter, even when most of it repeats the previous chapter. A reader who starts at chapter 05 copies `examples/tutorial/05-layout/` and has a working state. Patches between chapters were rejected: a reader cannot run a patch, and the block check would need the previous chapter's files.

Layout, mirroring where the files are installed:

```
examples/tutorial/09-plugin/
+-- user/init.lua                   -> ~/.config/gband/user/init.lua
+-- plugins/marks/plugin.lua        -> ~/.local/share/gband/plugins/marks/
+-- plugins/marks/client.lua
+-- plugins/marks/lua/marks/init.lua
+-- tests/plugin_spec.lua           chapter checks, run by gband test here
+-- tests/screenshots/plugin_spec/*.txt
```

### Chapter specs read the chapter's files with `io`

A chapter spec reads `user/init.lua` and its other files with `io.open`, relative to the working directory, and passes them to `g.start` as `config`, `server_config` and `files`. Plugins are passed as `plugins = { "../plugins/marks" }`. `gband test` in the chapter directory, as the suite and the reader both run it, sets that working directory.

A new `g.start` option that copies a directory into the case's configuration directory was rejected for this change. It would change the `plugin-testing` capability and the test API for a need that a few lines of Lua cover. Chapter 11 teaches the pattern, and a later change can add the option.

### The tutorial writes `user/init.lua` from an empty file

Chapter 00 starts with an empty `user/init.lua` and adds the key style, the error list and the sidebar. Copying `defaults/init.lua` was rejected as the starting point. The chapters would hold a snapshot of the default configuration, which would drift from `crates/lua/src/defaults.lua` with no check failing. Chapter 00 mentions the copy as an alternative and links to the README's Configuration section.

### One running example: the `marks` plugin

The chapters grow one feature instead of unrelated snippets. The feature marks windows with a letter and jumps back to them:

| chapters | what the reader adds |
|---|---|
| 00-01 | a working `user/init.lua`; bindings, including one that marks the focused window |
| 02-05 | marking as an action and a command; an option for the mark letters; reacting to focus; finding a marked window in the layout and focusing it |
| 06-08 | a highlight group for marks, a plugin window listing them, a bar segment showing the current window's mark |
| 09 | the code moves to `plugins/marks/`, with a manifest, `setup` options and `api = 1` |
| 10 | a server half: marks kept in window state so every client sees them, a server command listing them, and an event when a marked window's program exits, which the client turns into a notification |
| 11 | `plugins/marks/tests/marks_spec.lua` with cases and screenshots |

Which chapter names each top-level field, so the namespace check passes:

| chapter | fields |
|---|---|
| 00-setup | `config_dir`, `errors`, `clear_errors` |
| 01-keys | `keymap`, `bind`, `unbind`, `keystyle` |
| 02-actions | `action`, `cmd`, `spawn`, `notify`, `bell`, `clipboard`, `open` |
| 03-options | `opt`, `set`, `settings` |
| 04-events | `on`, `augroup`, `emit` |
| 05-layout | `layout`, `view`, `window`, `band` |
| 06-colors | `hl`, `colorscheme`, `palette` |
| 07-plugin-windows | `win`, `ui` |
| 08-bars | `bar` |
| 09-plugin | `plugin`, `plugins`, `runtimepath`, `side`, `api_version` |
| 10-server | `window_state`, `sessions`, `session`, `rpc` |

The check walks the live tables, so a field added later fails the suite until a chapter names it.

### Screen blocks name their reference in the info string

A screen block opens with `` ```screen <path> ``, the path relative to the chapter directory, such as `tests/screenshots/bars_spec/shows-the-bar--two-windows.txt`. GitHub renders an unknown language as plain text, so the reader sees only the rows. The prose before the block says what the reader sees. The block holds the reference's rows with the `N|` prefix removed. Colors and the cursor are left out because a Markdown code block cannot show them. Rendering references to images was rejected: it needs a renderer and committed image files.

Chapter screens use small sizes, such as `40x8`, and the `terminal` theme that `g.start` saves by default, so a block stays short.

### Where the checks live

- `crates/lua/tests/tutorial.rs` holds the static checks:
  - the one-to-one match between chapters and chapter directories;
  - the `lua` blocks;
  - the `screen` blocks;
  - the `api` fields;
  - namespace coverage.
  It uses `common::Scratch` to load an empty client and server configuration, as `api_docs.rs` does, and walks only the top level of each `gband` table.
- Namespace matching reuses the identifier-boundary rule of `crates/lua/tests/common/docs.rs`, so `gband.window_state` does not count as naming `gband.window`, while `gband.window.focus` does. `docs.rs` gains a way to search a text that is not one file, here every chapter joined.
- `tests/lua_specs.rs` gains a `tutorial` test. It discovers every directory under `examples/tutorial/` and every `plugins/<name>/` in them that holds `tests/`, and runs `gband_test` in each. Discovery means a new chapter runs with no Rust edit.
- A fence is a line starting with three backticks. The info string is the rest of that line. The block's text is the lines up to the closing fence, joined with newlines. The `api` check matches `api = <integer>` in every `.lua` file under `examples/tutorial/`.

## Risks / Trade-offs

- [The suite gets slower: twelve chapter directories plus the plugin's own tests start real clients and servers] -> Keep each chapter to one to three cases with small terminal sizes, and run the directories concurrently in the discovery test.
- [A visual change to gband forces tutorial edits: a changed screenshot means a changed `screen` block] -> The failure names the chapter and the reference. `gband test --update` in the chapter directory rewrites the reference, and the block is then copied from it. Few `screen` blocks per chapter keep this cheap.
- [A very short `lua` block, such as `end`, matches almost any file] -> Write blocks of at least one complete statement. Reviewers catch fragments.
- [The prose can still drift] -> Chapters link to the reference instead of restating it. Each claim the prose makes about behavior is a case in the chapter's spec.
- [Duplicate files across snapshots make a cross-chapter edit touch many directories] -> This is accepted as the price of runnable chapters. The block check finds every chapter that still shows the old code.
