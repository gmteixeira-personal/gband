## Context

See proposal.md for the motivation. The relevant current state:

- `crates/lua/src/runtime/gband/win.lua` holds every plugin window in Lua. A focused float gets each unbound key through `hooks.key`, which runs a `keys` entry, moves the cursor for the default keys, and closes a float on Escape with `close(window, true, true)`, which also runs `on_close`.
- `crates/client/src/lib.rs` resolves a session action against the view and sends it, and does nothing when no pane is focused. Close pane is a session action, so today it always reaches the server.
- `crates/lua/src/runtime/gband/statusline/hints.lua` has `key_form`, the "Key form" of the key-hints capability, as a local function, and a table of short labels for the built-in actions.
- `StatusLineAccent` and `StatusLineMuted` get their defaults in `gband/statusline.lua`, which loads only when a status line segment is set up.
- `crates/client/src/bindings.rs` has two tests that read the default configuration: `default_keys_follow_the_spec` and `every_default_binding_names_an_action`. Both assume every default binding names a built-in action.
- floating-windows has been archived, so the deltas here are built on its versions of "Lua names of actions", "Session actions resolve against the view" and "Key bindings".
- center-column modifies "Lua names of actions" and "Key bindings" too. navigation-mode and sidebars-borders-steps modify this change's requirements and depend on it.

## Goals / Non-Goals

**Goals:**
- The key list is a plain bundled plugin that uses only the public API: `gband.action.register`, `gband.keymap.list`, `gband.win` and `gband.hl`.
- Ctrl+Space then `q` and Enter on the list's `q` line close the list through the same path that `gband.win.close` uses.

**Non-Goals:**
- A focus-lost callback for plugin windows. A list left unfocused is focused again by opening it.
- Renaming any identifier, which rename-panes-to-windows does.

## Decisions

### Run after center-column

`Depends On` names center-column. Both changes replace the whole "Lua names of actions" and "Key bindings" requirements, so whichever archives second must carry the other's rows. center-column is small and can start now, so it goes first, and the first task here folds its `center_column` row and its `c` binding into these deltas. navigation-mode and sidebars-borders-steps already depend on this change, so the order is fixed for all four.

Alternatives:
- No dependency, and a race. Rejected. The second archive would silently drop the first change's rows.

### Close pane checks for a focused float before the view

The client resolves close pane by first asking the runtime whether a float is focused. When one is, the runtime closes it as `gband.win.close` does, which runs `on_close`, and the client sends nothing. This happens before the client checks for a focused pane, so it also works on a band with no pane. A new runtime entry point, `close_focused_float`, returns whether it closed one, and the client sends close pane only when it did not.

Alternatives:
- Bind `prefix q` to a function that checks for a float. Rejected. Every user configuration that binds `close_pane` itself would miss the behaviour.
- Make every action act on the focused float. Rejected. The descriptions say window, and a float is a plugin window, not a window. Only close needs to reach what has focus.

### One key form for both segments

`key_form` moves out of `hints.lua` into a bundled module, `gband.keyform`, that both the hints segment and the key list require. The key list pads every key to the widest one plus two cells, so `C-right` and its description never touch.

### Size the float to its lines

The list's width is its longest line plus the border, so every default description fits on an 80-column terminal. The longest line is `C-right` padded to 9 cells plus "move the column or floating window to the right", 56 cells. The ribbon area caps the width, and 15 rows cap the height.

### Give the status line groups their defaults

The key list links its groups to `StatusLineAccent` and `StatusLineMuted`, so a colorscheme that styles the status line styles the list. The plugin also calls `gband.hl.default` for those two groups with the status line's own defaults. A default never overrides a group that is already set, so loading order does not matter.

### Enter runs actions against the pane behind

An action from the list is dispatched with no target, exactly as its binding would be, so it acts on the focused pane behind the list. `send_prefix` sends the prefix key to that pane, and `+` grows it. This is what their descriptions say once a pane is called a window. Only close pane is different, because the focused float is what it closes.

The list stays focused through the focus changes its own Enter causes, by the rule plugin-windows already has for a float's own keys. A float that the action opens with focus takes it, as any newly opened float does.

## Risks / Trade-offs

- [center-column changes its wording after this proposal] → Task 1.1 diffs each MODIFIED block against the then-current main spec and folds in every difference.
- [navigation-mode's deltas were drafted against an earlier version of this change] → navigation-mode depends on this change, so its own first task folds in this change's final wording, such as the width rule and the key form.
- [User configurations copied from the old defaults show long hints] → The README tells the user to copy `defaults/init.lua` again or update the descriptions.
- [Enter on `send_prefix` sends NUL to the pane behind the list] → Accepted. It is what the action does from any other binding.

## Migration Plan

Nothing to migrate: the default configuration gains the key list, and a user configuration opts in by setting up `gband.keylist` and binding `keylist.open`. A rollback is a revert of the change's merge.
