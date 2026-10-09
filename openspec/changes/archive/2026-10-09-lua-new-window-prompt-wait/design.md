## Context

See proposal.md for the motivation. The pieces this change builds on:

- **Settling.** `g.settle()` in a Lua spec runs `Case::settle` in `crates/harness/src/case.rs`. It waits until the client has read the input written before it, and until the client and the server stop exchanging messages. It does not watch a window's shell, which is a separate process that draws its prompt whenever it is ready.
- **Prompt waits in other specs.** `prompted_columns` in `tests/lua/keylist_spec.lua` waits with `g.wait` until the screen text holds one `│$` per column. A shell prompt sits against the tile's left border, so `│$` marks a tile whose shell has drawn its prompt.
- **Line waits in the control tests.** `wait_for_line` in `crates/harness/src/terminal.rs` ends when any row of the focused window equals the expected text exactly. `in_window` in `tests/control.rs` types a command that ends in `echo done-<file>` and waits for the row `done-<file>`.

## Goals / Non-Goals

**Goals:**
- The `numbered` screenshot in "two shells are numbered in layout order" is taken only after both shells have drawn their prompts.
- `in_window` waits until its command has finished, however the typed command line wraps.

**Non-Goals:**
- No change to `crates/harness`. `g.settle()` and `wait_for_line` keep their meaning.
- No change to the `numbered` screenshot file. It already shows both prompts, which is the state the stronger wait now guarantees.

## Decisions

### 1. Wait for two prompts and the title together
The case replaces `g.wait_text("sh #2")` with one `g.wait` that requires two `│$` matches and the `sh #2` title in the same screen. The title proves the server numbered the new window. The prompts prove both shells have drawn. One predicate checks both, so neither can be satisfied by a screen the other has not reached.

*Alternative:* call `prompted_columns` from `keylist_spec.lua`. Rejected: each spec file defines its own helpers, and that helper also queries the layout through `g.client`, which this case does not need.

### 2. Count prompts on the screen, not in the layout
The predicate counts `│$` in `screen.text()`. The race is between the shell and the screenshot, and only the screen shows what the shell has drawn.

### 3. Quote inside the echoed marker
`in_window` types `echo "done-"<file>`. The shell prints `done-<file>`, but the typed command line now holds `"done-"<file>`, so no wrap of the command line can leave a row equal to the marker. The wait still matches the output row exactly.

*Alternative:* match the marker as a prefix or substring. Rejected: that makes the echoed command line match sooner, which is the race being fixed.

## Risks / Trade-offs

- **A prompt that changes shape breaks the wait.** If the test shell's prompt stopped starting with `$`, the case would time out instead of passing. `keylist_spec.lua` and the agent status spec depend on the same shape, so the change adds no new assumption.
- **Other control tests may type a marker the command line can match.** This change fixes `in_window`, the one seen to fail. A test that copies the unquoted pattern gets the same race when its path is long enough to wrap.
