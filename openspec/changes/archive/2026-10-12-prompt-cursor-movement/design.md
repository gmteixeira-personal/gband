## Context

`crates/lua/src/runtime/gband/prompt.lua` holds both prompts as two entries of `kinds`, `lua` and `rename`. Each entry keeps `line`, `width`, `win` and `target`. Typing and pasting reach `on_input`, which appends to `line`. Enter, Backspace and Ctrl+U are entries of the window's `keys` table. `fitted` drops leading characters of `marker .. line` until the text plus one cursor cell fits `width`, and `show` draws that text followed by a space in `PromptCursor`.

The `left`, `right`, `home` and `end` key names already exist (README line 520). The plugin window's key hook in `crates/lua/src/runtime/gband/win.lua` tries a `keys` entry first, then `on_input`, then the default keys. The defaults scroll on `up`, `down`, `pageup`, `pagedown`, `home` and `end`, and discard `left` and `right`. A `keys` entry for any of the four therefore takes the key before a default can.

The prompt opens with no `on_mouse`, so a left press takes the plugin window default, which moves a `cursorline` the prompt does not have. `on_mouse` receives `kind`, `button` and `content_col` and `content_row` counted from 0, and nil on the border. A press focuses the plugin window before `on_mouse` runs. Setting `on_mouse` removes the wheel default, which scrolls nothing in a one-line window.

`docs/internals/10-keylist-and-prompt.md` quotes `prompt.lua` in blocks tagged `lua defaults/lua/gband/prompt.lua`. `crates/lua/tests/tutorial.rs` fails when a tagged block no longer appears in the bundled file, so every quoted block that changes is quoted again from the new source.

## Goals / Non-Goals

**Goals:**
- One cursor model shared by the Lua prompt and the rename prompt.
- Drawing that keeps the cursor cell on screen for any cursor position and any line length.
- A click that lands the cursor on the character drawn under the pointer, with the text staying where it was drawn.

**Non-Goals:**
- Ctrl+A, Ctrl+E and word motion.
- Command history on Up and Down.
- Selection by drag, and any action on the right or middle button.

## Decisions

### The cursor is a character count

Each kind keeps `cursor`, the number of characters before the cursor, from 0 to `utf8.len(line)`. Byte offsets come from `utf8.offset(line, cursor + 1)` at the moment of an edit.

A byte offset stored directly would need care on every move to land on a character boundary. A count matches the spec's unit, one Unicode scalar value, and `utf8.offset` converts it in one call.

### Edits split the line at the cursor

Insertion builds `before .. text .. after` and adds `utf8.len(text)` to `cursor`. Backspace removes the one character before the cursor and subtracts 1. Ctrl+U sets `line = ""` and `cursor = 0`. Home sets `cursor = 0`, and End sets `cursor = utf8.len(line)`. `open` sets `cursor = utf8.len(line)`, which puts the rename prompt's cursor after the manual name. `on_close` resets `cursor` to 0 with `line`.

Backspace keeps its current check first: an empty line closes the prompt. A cursor of 0 on a line with text then returns without a change.

Backspace and Delete share `cut`, which removes the character at a given count. Backspace lowers `cursor` by 1 and cuts there. Delete cuts at `cursor` and leaves `cursor` alone. With the cursor after the last character, Delete returns without a change. It never closes the prompt, because Delete on an empty line is more likely a stray key than a request to leave.

### The scroll position is kept between keys

Each kind keeps `first`, the number of leading characters of `marker .. line` left out of the drawing. `open` and `on_close` set it to 0. Every call of `show` recomputes it from its previous value, with `c` the cursor's index in the shown text, `utf8.len(marker) + cursor`:

1. `low`: the fewest leading characters to leave out so the shown text through index `c`, the cursor cell, fits `width`.
2. `fill`: the fewest leading characters to leave out so the whole shown text plus one cell fits `width`.
3. `first = max(low, min(first, cursor, fill))`.

`fitted` then returns three pieces from index `first`: the text before the cursor cell, the cursor cell, and the text after it, cut where it stops fitting. The cursor cell is the character at index `c`, or a space when the cursor is at the end. `show` draws them as `PluginWindow`, `PromptCursor` and `PluginWindow` spans.

The `min` with `cursor` keeps one character left of the cursor cell when the Lua prompt scrolls left, and shows the `:` again once the cursor reaches the start. The `min` with `fill` brings hidden leading characters back when Backspace or a resize leaves free cells at the end. The `max` with `low` is applied last, so the cursor cell wins in a box too narrow for anything else. Typing at the end gives `low = fill`, so a long line draws exactly what the current `fitted` draws, and the existing `long` screenshot stays valid.

A stateless rule, recomputing the fewest dropped characters on every key, was the first draft. It needs no field, and its drawing depends on the line and the cursor alone. It moves the text under the pointer: on a 10-column box showing `defghijkl`, a click on `f` would redraw the line as `:abcdefghi` with `f` 4 cells to the right of where it was clicked.

### A click maps a content cell to a character

`on_mouse` acts only on a `press` with `button == "left"` and `content_row == 0`. It walks the drawn characters from index `first`, summing `gband.ui.width` of each, and finds the one whose cells hold `content_col`. The marker's character gives `cursor = 0`. A line character at shown index `i` gives `cursor = i - utf8.len(marker)`. A cell past the drawn text gives `cursor = utf8.len(line)`. `show` then runs, and the `first` it keeps leaves the text in place, since the clicked character is already inside the drawn run.

The walk reads the `first` stored by the last `show`, which is the drawing the user saw. Recomputing it from the line would map the click against a drawing that may differ after a resize that has not been drawn yet.

### A wide character under the cursor fills the cursor cell

A double-width character such as `界` under the cursor is drawn whole in `PromptCursor`, so both of its cells are reversed. Fitting counts its width with `gband.ui.width`, as `fitted` does today. A click on either of its cells maps to it.

## Risks / Trade-offs

- [The scroll position makes each drawing depend on the keys pressed before it] → Each spec scenario names its key sequence, and the tests replay the same sequence. `first` resets on open and close, so no state carries across prompts.
- [A wide character as the cursor cell may not fit in a box with one free cell] → Fitting drops leading characters until it fits. A box narrower than 2 columns cannot show it, and `kind.width` already clamps to 1, so that case draws the cell cut by the window, as a wide character at the box's edge is drawn today.
- [`on_mouse` removes the plugin window's wheel default] → The prompt holds one line, so the default scrolled nothing.
- [The internals chapter's quoted blocks go stale] → Task 3.3 quotes them again, and `cargo test -p gband-lua --test tutorial` checks them.
