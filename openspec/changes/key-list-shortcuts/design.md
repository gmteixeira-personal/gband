## Context

See proposal.md for the motivation. This change starts after navigation-mode, so it reads the key list as navigation-mode leaves it:

- `crates/lua/src/runtime/gband/keylist.lua` opens the key list with `gband.win.open`, enters `root`, and gives the plugin window one `keys` entry, `enter`, that runs the cursor line's binding through `gband.keymap.run("prefix", key)` unless it is the line of `keylist.open`.
- `crates/lua/src/runtime/gband/win.lua` checks a focused plugin window's `keys` entries before its defaults, in `hooks.key`. A `keys` entry for `j`, `k`, Up or Down therefore replaces that default. Every `keys` name is canonicalised through `host.parse_key` when the plugin window opens, and an invalid name raises an error.
- `gband.keymap.list("prefix")` returns the binding of the prefix key with the key `prefix`, which `host.parse_key` does not accept as a key name.
- With navigation-mode's defaults, the `prefix` table binds `h`, `j`, `k`, `l`, the four arrows, Enter and Escape, so every movement default of a plugin window except PageUp, PageDown, Home and End is also a line's key.

## Goals / Non-Goals

**Goals:**
- Pressing a line's key behaves exactly as moving to the line and pressing Enter, through the same handler.
- The change stays inside the key list plugin and uses only the public `gband.win` and `gband.keymap` API.

**Non-Goals:**
- A general accelerator option for plugin windows. No other plugin window needs one yet.
- Any key that moves the cursor line in place of a bound `j` or `k`. The user chose to have none.

## Decisions

### One `keys` entry per line, built when the list opens

`open` builds the `keys` table from the list's entries: for each line whose key is neither `prefix` nor an own key, an entry under the binding's key that calls `gband.win.set_cursor(win, index)` and then the same run function Enter calls. The Enter handler and the line handlers share that one function, so the two paths cannot drift apart. The entries are built from the same `gband.keymap.list("prefix")` result as the lines, so a line and its key always agree.

Alternatives:
- Give plugin windows an `on_key` fallback and look the key up in the list there. Rejected. It needs a plugin-windows API change for one plugin, and `keys` already matches keys the way bindings do.
- Run the binding through `gband.keymap.run` without moving the cursor line. Rejected. The user asked for the press to select the line too, and the cursor line then shows what ran.

### The own keys: Up, Down, PageUp, PageDown, Home, End, Enter and Escape

The key list gets no `keys` entry for these, so they keep the plugin-window defaults, and Enter keeps its own entry. Every other key a line shows takes an entry, `j` and `k` included, even though that overrides their plugin-window default. Left and Right are not own keys: the list has no sideways movement, so they run their lines.

Alternatives:
- Keep `j` and `k` as movement keys. Rejected by the user: bindings win.
- Add Ctrl+J and Ctrl+K, or Ctrl+Down and Ctrl+Up, as movement keys. Rejected by the user: the arrows are the list's movement keys and nothing replaces `j` and `k`. This is a deliberate exception to the project rule that `j` and `k` work beside the arrows for every directional action.

### The prefix key's line takes no entry

The binding of the prefix key is listed with the key `prefix`, which is not a key name, and the prefix key is handled by the `root` binding before any plugin window sees it. The line is skipped when building `keys`, so `gband.win.open` raises no error, and it stays runnable through Enter.

### Matching follows the bindings' rules

A `keys` name is canonicalised as a binding's key is, so `R` and `shift+r`, or `+`, match exactly what the binding matches in the `prefix` table. A key bound in `root` never reaches the list, as the plugin-windows capability defines, so such a line runs only through Enter. With the defaults `root` binds only the prefix key.

## Risks / Trade-offs

- [`j` and `k` no longer move the cursor line with the defaults] → Accepted by the user. Up and Down move it, and the README and plugin docs say so.
- [The MODIFIED blocks are written against `openspec/specs/key-list/spec.md` as key-list was archived, because strict validation compares them with the current main spec, and navigation-mode rewrites the same two requirements before this change starts] → Task 0.1 folds navigation-mode's archived wording, such as the `navigation keys` title, entering `root` and running function bindings, into the MODIFIED blocks, keeping only this change's own differences.
- [navigation-mode lands with wording or tests that differ from what these deltas assume, such as its end-to-end key list case moving the cursor line with `j`] → Task 0.1 folds navigation-mode's archived wording into the MODIFIED blocks, and task 2.4 finds every test that moves the key list's cursor line with `j` or `k`.
- [A user binds `prefix` keys that are also own keys, such as `prefix home`] → Their lines run only through Enter, as the spec states. The own keys are few and are the plugin-window movement keys.
- [The cursor moves to a line whose binding then closes the list] → Harmless. The list is gone, and it opens on its first line next time.

## Migration Plan

Nothing to migrate. A user who moved through the list with `j` and `k` uses Up and Down. A rollback is a revert of the change's merge.
