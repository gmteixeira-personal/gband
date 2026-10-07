## 1. Checks

- [x] 1.1 In `crates/lua/tests/tutorial.rs`, describe a tutorial by its docs directory, its examples directory and whether every chapter needs a directory. Run the existing chapter, `lua` block and `screen` block checks over the scripting tutorial through that description, with no change in behavior. Verify `cargo test -p gband-lua --test tutorial` passes as before
- [x] 1.2 Add the internals tutorial description: `docs/internals/` and `examples/internals/`, where a directory is required only for a chapter holding a `lua` block with no path or a `screen` block, and every directory needs a chapter. Plain `lua` and `screen` blocks use the scripting tutorial's rules. Verify that a scratch chapter with a plain `lua` block and no directory fails and names the chapter, then remove the scratch file
- [x] 1.3 Add the quote check: run `gband_lua::prepare` on a scratch configuration directory, collect every file under its `defaults/`, and for each internals block whose info string is `lua <path>`, require the path to be a collected file and the block's lines to match exactly one run of the file's lines. Failures name the chapter, the path and the quote's first line. Verify with scratch quotes that are stale, ambiguous (`  end` of `win.lua`) and of an unknown path
- [x] 1.4 Add the coverage check: count, per collected file, how many quotes hold each line. Any line of any file may be in at most one quote. Every non-blank line of a covered file needs exactly one. The covered files are the collected files except those under `defaults/colors/` other than `gruvbox.lua`, and except `defaults/lua/gband/theme/catppuccin.lua`. An uncovered line is reported with the file, its number, its text and the first line of the top-level statement holding it. A twice-quoted line is reported with the file, its number and both chapters. Verify on a scratch chapter set that quotes all of `palette.lua` but one line, that quotes one line twice, and that leaves `nord.lua` and `theme/catppuccin.lua` unquoted with no failure for them
- [x] 1.5 Add the naming check: every covered file's path appears, outside fenced blocks and under the identifier-boundary rule of `common/docs.rs`, in some internals chapter. Verify that a scratch chapter set that omits `defaults/colors/gruvbox.lua` fails and names it, and that one which never names `defaults/colors/nord.lua` passes for it
- [x] 1.6 Add the `gband.core` check: load an empty client configuration with `common::Scratch`, walk `gband.core` with the `WALK` chunk under the root name `gband.core`, and require every collected path to appear outside fenced blocks in some internals chapter. Failures list every missing path. Verify that a scratch chapter naming `gband.core.timer` only inside a quote fails and names `gband.core.timer`
- [x] 1.7 Extend the `tutorial` test in `tests/lua_specs.rs` so that it also discovers every directory under `examples/internals/` and every `plugins/<name>/` there that holds `tests/`, and runs `gband test` in each. Verify with `cargo test --test lua_specs tutorial` once `examples/internals/12-own-prelude/` exists

## 2. Index and chapters 00 to 03

Each chapter task: write `docs/internals/NN-<slug>.md`, quoting the files that design.md's table gives the chapter in `lua defaults/<path>` blocks. Explain in the prose between quotes why the code is written as it is, and name each `gband.core` primitive the quoted code uses as `gband.core.<path>`, linking to its section of `docs/plugins.md`. Verify each with `cargo test -p gband-lua --test tutorial`: the chapter's quotes pass, and the coverage report no longer lists the chapter's files.

- [x] 2.1 Write `docs/internals/README.md`, the index: what the tutorial explains, that it follows the files under `defaults/` as they are, how to read a quote's path, and links to all thirteen chapters in order. Verify every link resolves once the chapters exist
- [x] 2.2 Write `00-boundary`: the executable and Lua, every `gband.core` primitive by group as `docs/plugins.md` orders them, the providers `windows`, `bars`, `settings` and `styles`, the prelude quoted in full, `require` and the runtimepath, the copies under `defaults/`, replacing a bundled file through `user/lua/gband/` or `user/colors/`, and the table of repository source paths for contributors
- [x] 2.3 Write `01-themes`: quote `defaults/colors/gruvbox.lua` in full and explain how a colorscheme hands a palette and UI colors to `gband.theme`. Say that the other bundled colorschemes are built the same way and that the tutorial does not walk through them
- [x] 2.4 Write `02-highlights`: `palette.lua` and `hl.lua` in full, including the styles provider
- [x] 2.5 Write `03-colorschemes`: `colorscheme.lua` and `theme.lua` in full, including how `start` reads the saved theme

## 3. Chapters 04 to 08

- [x] 3.1 Write `04-bars`: `bar.lua` in full, including the bars provider, `gband.core.place_bars`, `gband.core.present_bars` and `gband.core.error_marker`
- [x] 3.2 Write `05-plugin-windows`: `win.lua` from the top through `api.list`: state, line handling, placing, rendering, option checks and the API
- [x] 3.3 Write `06-window-provider`: `win.lua` from `LINE_STEPS` to the end: keys, mouse, every hook of the windows provider, the `after_event` function and `gband.win = api`. Verify that `win.lua` has no uncovered or twice-quoted line
- [x] 3.4 Write `07-settings`: `settings.lua` in full, including the settings provider and `gband.core.reopen_settings`
- [x] 3.5 Write `08-key-styles`: `defaults/lua/gband/keystyle.lua`, `defaults/keystyle/modal.lua` and `defaults/keystyle/direct.lua` in full, and why the presets are written to `defaults/keystyle/` and not under `defaults/lua/gband/`

## 4. Chapters 09 to 12

- [x] 4.1 Write `09-sidebar-and-errors`: `sidebar.lua` and `errors.lua` in full, as bundled plugins built on the public API
- [x] 4.2 Write `10-keylist-and-prompt`: `keyform.lua`, `keylist.lua` and `prompt.lua` in full
- [x] 4.3 Write `11-default-configs`: `defaults/init.lua` and `defaults/server.lua` in full, and how they use the modules of the earlier chapters
- [x] 4.4 Write `12-own-prelude` and `examples/internals/12-own-prelude/`. The directory holds `user/lua/gband/prelude.lua`, a copy of the bundled prelude that also requires `user/lua/mine/clock.lua`; the module itself; the chapter's `user/init.lua`; and `tests/own_prelude_spec.lua`, which reads the files with `io.open`, freezes the time and shows the reader's module at work, as design.md describes. The chapter's plain `lua` blocks are those files. Verify `gband test` passes in the directory and `cargo test -p gband-lua --test tutorial` passes for the chapter

## 5. Links and the whole suite

- [x] 5.1 Link `docs/internals/README.md` from the Scripting section of `README.md`, from the section "The prelude and the API modules" of `docs/plugins.md`, and from `docs/tutorial/README.md`. Verify all three links resolve
- [x] 5.2 Run `cargo test -p gband-lua --test tutorial` and confirm the quote, coverage, naming and `gband.core` checks all pass, with no uncovered line in any covered file
- [x] 5.3 Run `cargo test --test lua_specs tutorial`, then the full `cargo test`, and confirm every check passes and `examples/internals/12-own-prelude/` ran
- [x] 5.4 In `LICENSE-MIT`, list `docs/` in place of `docs/tutorial/`, and say in the README's License section that the documentation under `docs/` is MIT
