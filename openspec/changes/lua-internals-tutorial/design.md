## Context

See proposal.md for why. This change builds on `scripting-tutorial`, which must be archived first. That change adds:
- the `tutorials` capability;
- `crates/lua/tests/tutorial.rs`, with the chapter, `lua` block, `screen` block, `api` and namespace checks;
- a way in `crates/lua/tests/common/docs.rs` to test whether a text names a path under the identifier-boundary rule;
- a `tutorial` test in `tests/lua_specs.rs` that discovers chapter directories and runs `gband test` in each.

What gband writes under `defaults/`, from `crates/lua/src/directory.rs` (`prepare`, public as `gband_lua::prepare`) and `crates/lua/src/bundled.rs`:

| path under the configuration directory | source in the repository |
|---|---|
| `defaults/init.lua` | `crates/lua/src/defaults.lua` |
| `defaults/server.lua` | `crates/lua/src/defaults_server.lua` |
| `defaults/keystyle/modal.lua`, `defaults/keystyle/direct.lua` | `crates/lua/src/runtime/gband/keystyle/*.lua` |
| `defaults/lua/gband/<file>` | `crates/lua/src/runtime/gband/<file>`, for every module except the two presets |
| `defaults/colors/<name>.lua` | `crates/lua/src/runtime/gband/colors/<name>.lua` |

That is 30 files, about 3,270 lines with blank lines. Every top-level statement in them starts in the first column. `defaults/colors/default.lua` is empty. Nine palette themes are the same text once their `#rrggbb` colors are masked: `dracula`, `gruvbox`, `kanagawa`, `nord`, `one-dark`, `rose-pine`, `solarized`, `tokyo-night` and `vesper`. The four `catppuccin-*` flavors and the `catppuccin` alias are one line each and differ in the flavor's name. `terminal` is 16 calls to `gband.hl.set`.

`gband.core` exists in the client only. The bundled Lua reaches it through `local core = gband.core`, so a quote holds `core.timer`, never `gband.core.timer`.

## Goals / Non-Goals

**Goals:**
- No line of bundled Lua changes without the tutorial changing in the same commit.
- Quote boundaries follow the explanation, not the file's structure.
- The checks reuse the scripting tutorial's machinery instead of adding a second copy.

**Non-Goals:**
- Checking the prose next to a quote. The checks prove the quotes are current. A reviewer reads the paragraph beside a quote that had to change.
- Showing the repository's source paths in the chapters. Readers have `defaults/`; chapter 00 gives the table above once, for contributors.

## Decisions

### The covered files are what gband writes, read from a scratch directory

The check runs `gband_lua::prepare` on a scratch configuration directory and walks its `defaults/`. The set of covered files and the text a quote is compared with are then exactly what a reader finds on disk. A bundled file added later is covered with no edit to the check.

A table from reader paths to repository paths in the check was rejected. It would drift when a module is added. `bundled_files()` alone would also miss `defaults/init.lua`, `defaults/server.lua` and the presets' real location, `defaults/keystyle/`.

### Quotes may start and end at any line; statements only shape the messages

A quote is any run of consecutive lines. Coverage is counted per line: every non-blank line exactly once, and any line at most once. A long function can therefore be split across several quotes with prose between them. `win.lua`'s `api.open` is 76 lines and `render` is 50.

The top-level statement is used only to report failures. An uncovered line is reported with its number, its text, and the first line of the statement holding it: the nearest line above it, or the line itself, that starts in the first column and does not start with `end`, `}`, `)`, `else`, `elseif` or `until`. That heuristic only affects the message, never the verdict, so an imperfect guess costs nothing.

Alternatives rejected:
- Quotes that must start and end at top-level statement boundaries. The long functions would become 50 to 76 line blocks with no explanation inside them.
- Statement-level coverage, where quoting a statement's first line counts as covering it. An edit to an unquoted body would pass, and the prose explaining it could go stale.

### A quote must match exactly one position

The check finds every position where a quote's lines equal the file's lines. Zero positions means the quote is stale. Two or more means it is ambiguous, as a lone `  end` would be. Both fail. Requiring uniqueness avoids solving a tiling problem across quotes and makes every quote point at one place. An ambiguous quote is fixed by adding a neighbouring line.

### Info strings tell quotes from reader code

In `docs/internals/`:
- `lua <path>` is a quote, checked against `defaults/`;
- `lua` alone is the reader's code, checked against the chapter directory under the scripting tutorial's verbatim rule;
- `screen <path>` is checked under the scripting tutorial's screen rule.

GitHub renders `lua <path>` as Lua, because it uses only the first word. The scripting tutorial's rules for `docs/tutorial/` are unchanged.

### The theme exemption masks colors only

A colorscheme needs no quotes when, with every `#rrggbb` replaced by one placeholder, its text equals a colorscheme whose every non-blank line is quoted. Chapter 01 quotes `nord` in full, so the eight other palette themes need no quotes. It also quotes `terminal` and the five one-line `catppuccin` files, because each differs from the others in more than colors. It names `default.lua`, which is empty. The theme quotes total about 50 lines instead of about 280.

Masking string literals or identifiers too was rejected. A theme that renamed or added a palette field would still match and pass. Quoting all sixteen files was rejected as 230 lines of hex values that teach nothing new.

### Naming is checked outside fenced blocks

The file-naming check and the `gband.core` check search the chapters' text with fenced blocks removed, under the identifier-boundary rule from `common/docs.rs`. A quote that happens to contain a path or a primitive's name never counts as explaining it. The `gband.core` paths come from the `WALK` chunk of `common/docs.rs`, run on a client loaded through `common::Scratch` with the root `gband.core`. List entries such as those of `gband.core.events` have integer keys, so the walk skips them.

### One check file, generalized over two tutorials

`crates/lua/tests/tutorial.rs` gains a tutorial description: its docs directory, its examples directory, and whether every chapter needs a directory. The scripting tutorial requires a directory for every chapter. The internals tutorial requires one only for a chapter that has a `lua` block with no path, or a `screen` block. The chapter-to-directory check, the `lua` block check and the `screen` block check take this description. The quote, coverage, theme, naming and `gband.core` checks are internals-only tests in the same file. The `tutorial` test in `tests/lua_specs.rs` discovers directories under `examples/internals/` as well as `examples/tutorial/`.

A second check file was rejected. Each file under `crates/lua/tests/` is its own crate, so the fence parser and the screen comparison would have to move into `common/` or be copied.

### Which chapter quotes which file

| chapter | quotes | lines |
|---|---|---|
| 00-boundary | `defaults/lua/gband/prelude.lua` | 8 |
| 01-themes | `defaults/colors/nord.lua`, `terminal.lua`, the four `catppuccin-*.lua` and `catppuccin.lua`; names `default.lua` and the eight other palette themes | 50 |
| 02-highlights | `defaults/lua/gband/palette.lua`, `hl.lua` | 311 |
| 03-colorschemes | `colorscheme.lua`, `theme.lua`, `theme/catppuccin.lua` | 211 |
| 04-bars | `bar.lua` | 374 |
| 05-plugin-windows | `win.lua` from the top through `api.list`: state, line handling, placing, rendering, option checks and the API | about 585 |
| 06-window-provider | `win.lua` from `LINE_STEPS` to the end: keys, mouse, the hooks, `core.provide("windows", hooks)` and `gband.win = api` | about 245 |
| 07-settings | `settings.lua` | 379 |
| 08-key-styles | `keystyle.lua`, `defaults/keystyle/modal.lua`, `defaults/keystyle/direct.lua` | 170 |
| 09-sidebar-and-errors | `sidebar.lua`, `errors.lua` | 293 |
| 10-keylist-and-prompt | `keyform.lua`, `keylist.lua`, `prompt.lua` | 382 |
| 11-default-configs | `defaults/init.lua`, `defaults/server.lua` | 30 |
| 12-own-prelude | none; the reader's own files in `examples/internals/12-own-prelude/` | 0 |

The coverage check does not enforce this table; quotes may move between chapters. The table spreads the explanation of each `gband.core` primitive across the chapters whose code uses it, and chapter 00 introduces them all.

### The last chapter's prelude

The reader copies `defaults/lua/gband/prelude.lua` to `user/lua/gband/prelude.lua`. The copy keeps the bundled requires in their order, then requires a module of the reader's own, `user/lua/mine/clock.lua`. That module adds a bar segment showing the time through `gband.bar`. The chapter's spec freezes the time, starts gband with the chapter's `user/` files, and shows the segment in a screenshot. That proves the reader's prelude ran in place of the bundled one, before `user/init.lua`.

The prelude must keep `gband.settings`: `gband.colorscheme`'s `start` and the key style presets read it. If a bar segment cannot be added while the prelude loads, the module instead adds a `Clock` highlight group and an action. The spec then checks them with `g.client`. Either way, the chapter's `lua` blocks are the files in the directory.

## Risks / Trade-offs

- [Every edit to bundled Lua now also edits the tutorial] -> This is the decision the user made. The failure names the file, the line, its text and the statement holding it, so the quote to change is found at once. The prose beside it is then the only thing to reread.
- [A change in flight that edits bundled Lua meets this change at `/ready`] -> Merging `dev` into that change brings the coverage check, which then fails on its edits. Its implementer revises the quotes. Changes proposed after this one is archived should list `docs/internals/` in Expected Files when they touch bundled Lua.
- [A quote updated mechanically can leave stale prose] -> The checks cannot read prose. The failing quote's paragraph is the one to reread, and review catches the rest.
- [Uniqueness can force an awkward boundary] -> Add a neighbouring line to the quote. Lines that repeat in a file, such as `end`, are only ever ambiguous on their own.
- [Thirteen chapters, about 3,000 quoted lines] -> The tasks split the writing per chapter. The quote check runs from the first task, so each chapter's quotes are checked as they are written. The coverage and naming checks pass only once the last chapter is in, so they are verified in the final task.
