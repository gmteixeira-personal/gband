## Context

See proposal.md for the motivation. This change starts after rename-panes-to-windows, so it reads the specs and the code with their window names: `close_window`, `gband.window`, floating windows, and floating and tiled plugin windows. The relevant state then:

- `crates/lua/src/runtime/gband/win.lua` holds every plugin window in Lua. A focused floating plugin window gets each unbound key through `hooks.key`, which runs a `keys` entry, moves the cursor for the default keys, and closes a floating plugin window on Escape, running its `on_close`.
- `crates/client/src/lib.rs` resolves a session action against the view and sends it, and does nothing when no window is focused. Close window is a session action, so today it always reaches the server.
- `crates/lua/src/runtime/gband/statusline/hints.lua` has `key_form`, the "Key form" of the key-hints capability, as a local function.
- `StatusLineAccent` and `StatusLineMuted` get their defaults in `gband/statusline.lua`, which loads only when a status line segment is set up.
- `crates/client/src/bindings.rs` has two tests that read the default configuration, `default_keys_follow_the_spec` and `every_default_binding_names_an_action`. Both assume every default binding names a built-in action.
- center-column replaces the whole "Key bindings" requirement too. navigation-mode and sidebars-borders-steps modify this change's requirements and depend on it.

## Goals / Non-Goals

**Goals:**
- The key list is a plain bundled plugin that uses only the public API: `gband.action.register`, `gband.keymap.list`, `gband.win` and `gband.hl`.
- Ctrl+Space then `q` and Enter on the list's `q` line close the list through the same path that `gband.win.close` uses.

**Non-Goals:**
- A focus-lost callback for plugin windows. A list left unfocused is focused again by opening it.

## Decisions

### Run after rename-panes-to-windows and center-column

`Depends On` names both. The deltas here are written against the specs as the rename leaves them, with its requirement names and scenario titles, so they validate only once the rename is archived. center-column replaces the whole "Key bindings" requirement too, so it goes first and the first task here folds its `c` binding into this change's copy. navigation-mode and sidebars-borders-steps already depend on this change, so the order is fixed for all of them.

Alternatives:
- Run before the rename. Rejected. The specs and the code would still say pane and `close_pane`.

### Close window checks for a focused floating plugin window first

The client resolves close window by first asking the runtime whether a floating plugin window is focused. When one is, the runtime closes it as `gband.win.close` does, which runs `on_close`, and the client sends nothing. This happens before the client checks for a focused window, so it also works on a band with no window. A new runtime entry point returns whether it closed one, and the client sends close window only when it did not.

Alternatives:
- Bind `prefix q` to a function that checks for a floating plugin window. Rejected. Every user configuration that binds `close_window` itself would miss the behaviour.
- Make every action act on the focused floating plugin window. Rejected. The actions' descriptions name a window, and a plugin window is not a window. Only close needs to reach what has focus.

### One key form for both segments

`key_form` moves out of `hints.lua` into a bundled module, `gband.keyform`, that both the hints segment and the key list require. The key list pads every key to the widest one plus two cells, so `C-right` and its description never touch.

### Size the list to its lines

The list's width is its longest line plus the border, so every default description fits on an 80-column terminal. The longest line is `C-right` padded to 9 cells plus "move the column or floating window to the right", 56 cells. The ribbon area caps the width, and 15 rows cap the height.

### Give the status line groups their defaults

The key list links its groups to `StatusLineAccent` and `StatusLineMuted`, so a colorscheme that styles the status line styles the list. The plugin also calls `gband.hl.default` for those two groups with the status line's own defaults. A default never overrides a group that is already set, so loading order does not matter.

### Enter runs actions against the window behind

An action from the list is dispatched with no target, exactly as its binding would be, so it acts on the focused window behind the list. `send_prefix` sends the prefix key to that window, and `+` grows it, as their descriptions say. Only close window is different, because the focused floating plugin window is what it closes.

The list stays focused through the focus changes its own Enter causes, by the rule plugin-windows already has for a floating plugin window's own keys. A floating plugin window that the action opens with focus takes it, as any newly opened one does.

## Risks / Trade-offs

- [center-column or the rename changes a requirement this change replaces after this proposal] → Task 1.1 diffs each MODIFIED block against the then-current main spec and folds in every difference.
- [navigation-mode's deltas were drafted against an earlier version of this change] → navigation-mode depends on this change, so its first task folds in this change's final wording, such as the width rule and the key form.
- [Enter on `send_prefix` sends the prefix key to the window behind the list] → Accepted. It is what the action does from any other binding.

## Migration Plan

Nothing to migrate: the default configuration gains the key list, and a user configuration opts in by setting up `gband.keylist` and binding `keylist.open`. A rollback is a revert of the change's merge.
