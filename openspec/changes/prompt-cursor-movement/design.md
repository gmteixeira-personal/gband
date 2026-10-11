## Context

`crates/lua/src/runtime/gband/prompt.lua` holds both prompts as two entries of `kinds`, `lua` and `rename`. Each entry keeps `line`, `width`, `win` and `target`. Typing and pasting reach `on_input`, which appends to `line`. Enter, Backspace and Ctrl+U are entries of the window's `keys` table. `fitted` drops leading characters of `marker .. line` until the text plus one cursor cell fits `width`, and `show` draws that text followed by a space in `PromptCursor`.

The `left` and `right` key names already exist (README line 520). The plugin window's default key handling in `crates/lua/src/runtime/gband/win.lua` scrolls on `up`, `down`, `pageup`, `pagedown`, `home` and `end`, and discards `left` and `right`. A `keys` entry for `left` or `right` therefore takes the key with no default to override.

`docs/internals/10-keylist-and-prompt.md` quotes `prompt.lua` in blocks tagged `lua defaults/lua/gband/prompt.lua`. `crates/lua/tests/tutorial.rs` fails when a tagged block no longer appears in the bundled file, so every quoted block that changes is quoted again from the new source.

## Goals / Non-Goals

**Goals:**
- One cursor model shared by the Lua prompt and the rename prompt.
- Drawing that keeps the cursor cell on screen for any cursor position and any line length.

**Non-Goals:**
- Home, End, Delete, Ctrl+A, Ctrl+E and word motion. Home and End keep the plugin window's scrolling default, which leaves a one-line prompt unchanged.
- Command history on Up and Down.
- A scroll offset that persists between keys, as Neovim's command line keeps one.

## Decisions

### The cursor is a character count

Each kind keeps `cursor`, the number of characters before the cursor, from 0 to `utf8.len(line)`. Byte offsets come from `utf8.offset(line, cursor + 1)` at the moment of an edit.

A byte offset stored directly would need care on every move to land on a character boundary. A count matches the spec's unit, one Unicode scalar value, and `utf8.offset` converts it in one call.

### Edits split the line at the cursor

Insertion builds `before .. text .. after` and adds `utf8.len(text)` to `cursor`. Backspace removes the one character before the cursor and subtracts 1. Ctrl+U sets `line = ""` and `cursor = 0`. `open` sets `cursor = utf8.len(line)`, which puts the rename prompt's cursor after the manual name. `on_close` resets `cursor` to 0 with `line`.

Backspace keeps its current check first: an empty line closes the prompt. A cursor of 0 on a line with text then returns without a change.

### Fitting keeps the cursor cell and cuts both ends

`fitted` takes the shown text `marker .. line` and the cursor's index in it, which is `utf8.len(marker) + cursor`. It returns three pieces: the text before the cursor cell, the cursor cell, and the text after it. The cursor cell is the character at the cursor, or a space when the cursor is at the end.

Leading characters are dropped until the text before the cursor cell plus the cursor cell fits `width`. Trailing characters are then taken from the text after the cursor cell while they fit the remaining cells. With the cursor at the end, this draws exactly what the current `fitted` draws, so the existing `long` screenshot stays valid.

`show` draws the pieces as three spans: `PluginWindow` text, `PromptCursor` cell, `PluginWindow` text.

A scroll offset carried between keys, as Neovim keeps, would avoid the line jumping when the cursor moves back into a line wider than the box. It adds a fourth field per kind and a rule for every edit. The stateless rule draws the same thing for the same line and cursor, which makes each screenshot depend on the line and the cursor alone.

### A wide character under the cursor fills the cursor cell

A double-width character such as `界` under the cursor is drawn whole in `PromptCursor`, so both of its cells are reversed. Fitting counts its width with `gband.ui.width`, as `fitted` does today.

## Risks / Trade-offs

- [The line jumps when the cursor moves left in a line wider than the box] → Each jump keeps the cursor cell shown, which is what the spec requires. A persistent scroll offset can come later without changing the spec.
- [A wide character as the cursor cell may not fit in a box with one free cell] → Fitting drops leading characters until it fits. A box narrower than 2 columns cannot show it, and `kind.width` already clamps to 1, so that case draws the cell cut by the window, as a wide character at the box's edge is drawn today.
- [The internals chapter's quoted blocks go stale] → Task 3.3 quotes them again, and `cargo test -p gband-lua --test tutorial` checks them.
