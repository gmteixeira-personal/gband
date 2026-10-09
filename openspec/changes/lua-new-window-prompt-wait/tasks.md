## 1. Wait for the new window's prompt

- [ ] 1.1 In `tests/lua/window_names_spec.lua`, make "two shells are numbered in layout order" wait after `g.keys("ctrl+space n enter")`, with `g.wait`, until the screen text holds two `│$` prompts and the `sh #2` title, in place of `g.wait_text("sh #2")`, before `g.settle()` and the `numbered` screenshot. Count the prompts as `prompted_columns` in `tests/lua/keylist_spec.lua` does. Leave the screenshot file unchanged. Verify with `cargo test -p gband --test lua_specs window_names`, then run it 50 times in a loop and check that every run passes

## 2. Gate

- [ ] 2.1 Run `.claude/flow-gate` and check that fmt, clippy and the workspace tests pass
