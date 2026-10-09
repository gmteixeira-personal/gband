## 1. Wait for the new window's prompt

- [x] 1.1 In `tests/lua/window_names_spec.lua`, make "two shells are numbered in layout order" wait after `g.keys("ctrl+space n enter")`, with `g.wait`, until the screen text holds two `│$` prompts and the `sh #2` title, in place of `g.wait_text("sh #2")`, before `g.settle()` and the `numbered` screenshot. Count the prompts as `prompted_columns` in `tests/lua/keylist_spec.lua` does. Leave the screenshot file unchanged. Verify with `cargo test -p gband --test lua_specs window_names`, then run it 50 times in a loop and check that every run passes

## 2. Unambiguous done marker in the control tests

- [x] 2.1 In `tests/control.rs`, make `in_window` type its marker as `echo "done-"<file>` in place of `echo done-<file>`, so the echoed command line can never leave a row equal to `done-<file>` when it wraps in the 37-column tile. A long worktree path wrapped `done-reload.json` onto a row of its own, and the wait ended before the command wrote the file. Verify by running `cargo test -p gband --test control from_a_window_and_the_window_session` 20 times and checking that every run passes

## 3. Gate

- [x] 3.1 Run `.claude/flow-gate` and check that fmt, clippy and the workspace tests pass
