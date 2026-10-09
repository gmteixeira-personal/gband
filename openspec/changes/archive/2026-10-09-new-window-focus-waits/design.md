## Context

See proposal.md for the motivation. The pieces this change builds on:

- **Opening a window.** Prefix `n` makes the server add a window and send the client a layout update and an `Opened` message. They arrive separately, and the client moves focus only when `Opened` arrives. Between the two, the screen shows the new tile while the old window still has focus.
- **Waits.** `crates/harness/src/terminal.rs` reads the screen. `tiles` returns each tile's bounds and whether its border is drawn focused. `wait_for_prompt` and `shell_pid` read the focused window, so they read whichever window had focus when they ran.
- **The reference helper.** `second_focused` in `tests/config.rs` already waits for the second tile to be focused after prefix `n`, so the tests built on it have no such race.

## Goals / Non-Goals

**Goals:**
- Every end-to-end test that opens a window with prefix `n` and then reads or types into the focused window first waits until the new window has focus.

**Non-Goals:**
- No change to the client, the server or the protocol. Layout and focus keep arriving in separate frames.
- No change to `crates/harness`. Its helpers stay correct once each caller waits for the focus it means.

## Decisions

### 1. Wait on the focused border, not on the tile count
Each test's wait after prefix `n` requires two tiles and the second one focused (`found.len() == 2 && found[1].focused`). Focus is the state the next step depends on, so the wait reads it directly. It waits for the same state as `second_focused` in `tests/config.rs`, which checks the focused tile's column instead.

*Alternative:* make `wait_for_prompt` or `shell_pid` reject a pid line already seen. Rejected: it hides a wrong wait in the caller and changes a helper every test uses.

### 2. Sort by column where the test already does
`number_follows_the_layout` in `tests/window_names.rs` reads titles in column order through `titles`, which sorts tiles by `left`. Its wait sorts the same way before it checks the second tile's focus, so the tile it checks is the one titled `sh #2`.

### 3. Fix the two tests that have not failed yet
`send_text_runs_a_command_in_another_window` and `number_follows_the_layout` use the same weak wait as the failing helper. Neither has been seen to fail, but each can type into the old window. Fixing them now changes only their wait.

## Risks / Trade-offs

- **A test that waits for focus can time out if the client never focuses a new window.** That would be a real regression, and the timeout names it: "the second of two tiles focused".
- **Other tests may still use a weak wait after opening a window.** This change fixes the three found on `dev` when it was proposed. A later test that copies the old pattern gets the same race.
