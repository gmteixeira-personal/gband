## Why

The Lua spec case "two shells are numbered in layout order" in `tests/lua/window_names_spec.lua` failed once in the gate run of `/ready` for `new-window-focus-waits`, and passed when rerun alone. Its `numbered` screenshot showed the cursor at `1:41` instead of `1:43`, with the second tile's first row blank. The case opens a window with prefix `n`, waits for the `sh #2` title and calls `g.settle()`, then takes the screenshot. `g.settle()` waits only until the client and the server stop sending messages. The new shell is a separate process and can draw its `$` prompt after that, so the screenshot can be taken before the second tile shows its prompt.

Two specs already wait for the prompts before they act: `prompted_columns` in `tests/lua/keylist_spec.lua` and the first case of `examples/plugins/agent-status/tests/agent_status_spec.lua` count the `│$` prompts on the screen. The window names case does not.

## What Changes

- "two shells are numbered in layout order" in `tests/lua/window_names_spec.lua` waits, after it opens the window, until the screen shows two `│$` prompts and the `sh #2` title, before it settles and takes the screenshot.
- `in_window` in `tests/control.rs` types its marker as `echo "done-"<file>`. The gate for this change failed on `from_a_window_and_the_window_session`: in a worktree with a long path, the typed command wrapped in the 37-column tile and left `done-reload.json` alone on a row, so the wait ended before the command wrote its JSON file. The quotes keep the typed text from ever matching the output row.
- Test code only: no behaviour, spec, screenshot or documentation changes.

Out of scope:
- Changing `g.settle()` in `crates/harness` to wait for window output. It cannot know which output a test expects, and the specs that need a prompt already wait for it.
- The other specs that open a window with prefix `n`. `tests/lua/keylist_spec.lua` and `examples/plugins/agent-status/tests/agent_status_spec.lua` already wait for the prompts. `tests/lua/loop_bands_spec.lua` waits for the output of a command it types into the new window.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. The change edits tests only, so `.openspec.yaml` sets `skip_specs: true`.

## Impact

- `tests/lua/window_names_spec.lua`: a stronger wait after opening a window.
- `tests/control.rs`: a done marker that the echoed command line cannot match.
- No product code, harness, Lua API, screenshot or documentation change.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- openspec/changes/lua-new-window-prompt-wait/
- tests/control.rs
- tests/lua/window_names_spec.lua
