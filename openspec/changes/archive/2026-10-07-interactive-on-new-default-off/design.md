## Context

The setting is read in three places, all Lua under `crates/lua/src/runtime/gband/`, which the docs show as `defaults/`:

- `keystyle/modal.lua` reads it once, as `gband.settings.interactive_on_new() ~= false`, and its `n` enters `root` after `open_window` when that is true.
- `settings.lua`'s `lines()` shows the fourth line as `interactive_on_new() == false and "off" or "on"`.
- `settings.lua`'s `toggle_interactive_on_new()` saves `interactive_on_new() == false and "true" or "false"`.

`gband.settings.interactive_on_new()` returns `true`, `false` or nil, and the internals tutorial says nil lets a caller tell "off" from "never chosen". The internals tutorial quotes all three lines, and `crates/lua/tests/tutorial.rs` requires every quote to match one run of its file and every line of the file to be quoted once.

Many tests open windows with Ctrl+Space then `n` under the default modal preset, which they reach by having no `user/init.lua`. Some of them then type into the new window, which only works while `n` returns to interactive mode.

## Goals / Non-Goals

**Goals:**
- Flip the default without changing what `gband.settings.interactive_on_new()` returns or what a saved file means.
- Keep the settings window's displayed value and its toggle in agreement with the preset's reading, so the first toggle on a fresh configuration turns the setting on.
- Keep every test that is not about the setting independent of it.

**Non-Goals:**
- Writing `user/interactive_on_new.lua` for existing users, or any other migration.

## Decisions

### Read the setting as on only when it is `true`

All three places compare with `true`: the preset reads `interactive_on_new() == true`, the window shows `interactive_on_new() == true and "on" or "off"`, and the toggle saves `interactive_on_new() == true and "false" or "true"`. A nil value, an unknown value and a broken file all read as off, as they read as on before.

Alternatives:
- Make `interactive_on_new()` return `false` when nothing is saved. Rejected: it changes a documented API and loses the difference between "off" and "never chosen" that `sidebar()` and `theme()` keep.
- Make the bundled default configuration save `return false` on first load. Rejected: loading would write to the user's directory, and nothing else does.

### Tests that need interactive mode after `n` press Enter

A test that opens a window with the default preset's `n` and then types into it adds Enter after `n`: `b"\x00n\r"` in a Rust test and `"ctrl+space n enter"` in a Lua spec. Enter is bound in navigation mode to return to interactive mode, and never reaches the window. It is chosen over Escape because a Rust test that writes ESC followed by more bytes risks the client reading them as one Alt key, and `g.keys` would have to wait after it. A screenshot taken after the Enter shows the same `I` in the sidebar as before, so no screenshot reference changes for that reason.

A test whose own `user/init.lua` binds `prefix n`, or that uses the direct key style, does not run the preset's `n` and is left as it is. A test about the setting itself writes the value it needs, `return true` or `return false`, rather than relying on the default.

Alternatives:
- Save `return true` in every affected test's configuration directory. Rejected: it ties tests about other features to this setting, and hides the default the tests otherwise run under.

### Docs describe the default where they describe `n`

`README.md`, `docs/plugins.md` and the internals chapters state the rule the same way the specs do: `n` returns to interactive mode only when `I on new` is on, and it is off until the settings window saves it on. A binding of one's own follows the setting by comparing `gband.settings.interactive_on_new()` with `true`, which the scripting tutorial's options chapter says next to its existing mention of the function.

## Risks / Trade-offs

- [Users who relied on the old default find `n` no longer returns to interactive mode] → The settings window's `I on new` line turns it back on with one key, and the README names the setting where it describes `n`.
- [A test missed by the survey keeps typing after `n` and its keys run navigation bindings] → The full suite runs before the change is ready; such a test fails, since the typed text never reaches the window.
