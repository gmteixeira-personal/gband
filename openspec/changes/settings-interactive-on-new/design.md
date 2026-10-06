## Context

See proposal.md for the motivation. This change starts after settings-themes is archived, which itself waits on key-style and band-sidebar. The state it builds on:

- After key-style, `crates/lua/src/runtime/gband/keystyle/modal.lua` binds `prefix n` to a local function that calls `gband.action.open_window()` and then the preset's `interactive` function, which enters `root`. `direct.lua` binds `n` to the action itself.
- After settings-themes, `crates/lua/src/runtime/gband/settings.lua` holds `open`, `theme`, `sidebar` and `themes`. `theme()` and `sidebar()` share one local reader (`loadfile(path, "t", {})` under `pcall`, then a type check), and every save goes through one local atomic writer shared with `keystyle.lua`. The settings window's `keys` table dispatches Enter, `h`, `l`, Left and Right on the cursor line read with `gband.win.info`. Every save is followed by a reload, and `host.settings_reopen(line)` reopens the window on the saved line.
- `settings.lua` is in `bundled::API`, so `gband.settings` exists before the init file runs, and so before `gband.keystyle.use()` evaluates a preset.

## Goals / Non-Goals

**Goals:**
- The setting follows the sidebar setting's shape exactly: one file, one boolean, one reader, one row.
- A user's own `init.lua` that calls `gband.keystyle.use("modal")` gets the setting with no extra call.

**Non-Goals:**
- A host change. Nothing here needs Rust.

## Decisions

### The modal preset reads the setting once, at load
`modal.lua` calls `gband.settings.interactive_on_new()` while it evaluates and keeps the result in a local. Its `n` function calls `interactive()` after `open_window()` unless that local is `false`. A save reloads the configuration, so the local is always current, the same way the sidebar setting reaches the default configuration.

Alternatives considered:
- *Read the file on each `n`.* A file read per key press, for a value that only changes through a save that reloads anyway.
- *Two `n` bindings chosen at load.* The `n` function stays one function with one description, so the key list and `default_keys_follow_the_spec` see the same entry either way.

### The direct preset is untouched
Under the direct style the prefix table is not a mode, so the key sequence ends after `n` and `root` is active whatever the binding does. Reading the setting there would change nothing.

### The fourth line exists only under the modal style
`open()` builds its lines from the same values it shows. When the `keys` value is `direct`, the line list stops at three, and the height is the line count plus 2, so the window shrinks with no special case. The key dispatch matches on the line's label, not its number, so a line that is absent can never be dispatched to.

Saving the `keys` line from `modal` to `direct` reopens on line 3, which still exists. A reopen line past the last line cannot come from the window's own saves, because the fourth line only saves while it is shown and saving it keeps the style.

### Label and file names
The label `I on new` is 8 cells, so the 9-cell label column and the 31-column width stay. The file is `user/interactive_on_new.lua` and the reader `gband.settings.interactive_on_new()`, named for what the setting does rather than for the label's abbreviation.

### The first-start offer stays as it is
The offer opens when no theme, sidebar or key style is saved. Adding the new file to that condition would only delay the offer for a user who saved this one setting by hand.

## Risks / Trade-offs

- [The row disappears and reappears as the key style changes] → Chosen so the direct style shows no row that does nothing. It sits last, so the other rows keep their line numbers and the reopen line stays valid.
- [A user's own `init.lua` with its own `n` binding ignores the setting] → Documented in the README beside the sidebar and key style notes. `gband.settings.interactive_on_new()` is the hook such a file can read.
- [The settings window screen references change height] → Regenerated in the task that adds the line, and each diff is checked to hold only the new line and the border moving down.
