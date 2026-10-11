## Why

The Lua prompt, opened by `prefix :`, and the rename prompt, opened by `prefix N`, keep their cursor at the end of the line. Left and Right are discarded, so fixing a typo in the middle of a line means erasing everything after it with Backspace and typing it again.

## What Changes

- The Lua prompt and the rename prompt keep a cursor position inside their line, from before the first character to after the last.
- Left moves the cursor one character back and Right one character forward. Neither moves past an end of the line.
- A typed character and a paste insert at the cursor, and the cursor moves past the inserted text.
- Backspace removes the character before the cursor. On an empty line it still closes the prompt. With the cursor at the start of a line that holds text, it does nothing.
- Ctrl+U still clears the whole line, and puts the cursor at its start.
- A prompt opens with its cursor after the last character, so the rename prompt's starting manual name is edited from its end.
- The cursor cell draws the character under the cursor in `PromptCursor`, or one space in `PromptCursor` when the cursor is after the last character.
- A line wider than the content area leaves out the fewest leading characters that keep the cursor cell shown, and cuts what does not fit after it.
- The README, `docs/plugins.md` and `docs/internals/10-keylist-and-prompt.md` describe Left, Right and editing at the cursor.

## Capabilities

### New Capabilities

### Modified Capabilities
- `lua-prompt`: "Prompt plugin window" draws the cursor cell at the cursor and fits the line around it, and "Editing the line" adds Left and Right and makes typing, pasting and Backspace act at the cursor.
- `window-names`: "Rename prompt" places the cursor cell as the Lua prompt does, opens with the cursor after the manual name, and keeps the cursor when focused again.

## Impact

- `crates/lua/src/runtime/gband/prompt.lua`: a cursor position per prompt kind, `left` and `right` keys, insertion and Backspace at the cursor, and line fitting around the cursor.
- `tests/lua/prompt_spec.lua` and new screenshots under `tests/lua/screenshots/prompt_spec/`.
- `tests/lua/window_names_spec.lua`: one case editing the middle of a manual name, with its screenshot under `tests/lua/screenshots/window_names_spec/`.
- `README.md`, `docs/plugins.md` and `docs/internals/10-keylist-and-prompt.md`.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- crates/lua/src/runtime/gband/prompt.lua
- crates/lua/tests/prompt.rs
- tests/lua/prompt_spec.lua
- tests/lua/screenshots/prompt_spec/
- tests/lua/window_names_spec.lua
- tests/lua/screenshots/window_names_spec/
- README.md
- docs/plugins.md
- docs/internals/10-keylist-and-prompt.md
- openspec/changes/prompt-cursor-movement/
