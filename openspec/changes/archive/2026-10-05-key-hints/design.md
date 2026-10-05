## Context

This change starts after `status-line` is archived, and is written against what that change delivers in `crates/lua`:
- `src/runtime/gband/statusline.lua`, the container: `add`, `remove`, `list`, the render triggers through `after_event`, `refresh_statusline` and timers, the layout, and `host.present`.
- `bundled.rs`, which embeds `src/runtime/gband/**` and serves `gband.*` modules after the runtimepath.
- The segment modules `statusline/{band,mode,position,clock}.lua`, each returning `{ name, api = 1, setup }`.
- `hl.lua` with `gband.hl.default`, and the built-in `StatusLine*` groups.

The foundation already gives everything the hints read: `gband.keymap.list(table)` returns `key` as written, `desc` and the action's full name; `gband.action.list()` returns each action's name and description; `gband.opt.prefix` returns the prefix key name; the context's `table` names the active table; and `KeyTableChanged` fires on every table change. Bindings are fixed once the configuration has loaded, so a render only needs the active table to be current.

The default configuration binds nothing in `root` and nineteen keys in `prefix`, each with its action's long description.

## Goals / Non-Goals

**Goals:**
- The hints segment is a bundled Lua module using only the public component API, like the other segments.
- The segment never overflows the line by its own output: it fits itself to the width the status line leaves it, so priority dropping never removes it whole on a normal line.
- No Rust host primitive is added.

**Non-Goals:**
- A second status line row, grouping, paging or scrolling of hints.
- Changing the default bindings' descriptions to short labels.

## Decisions

### A `fill` field in the status line, not a fixed render order
The hints must fit `ctx.width`, which counts the other components' latest outputs. With the status line as `status-line` specifies it, a render can see a stale width in two ways: on `KeyTableChanged` the mode segment may render after the hints, and on `FocusChanged` the position segment changes width without the hints rendering at all. A stale width either wastes space or overflows the line, and overflow drops the lowest priority component, the hints, whole.

`fill = true` fixes both. In `statusline.lua`, a trigger's render pass sorts its components so that fill components come last, in descending priority, then in insertion order. After the pass, the container lays the line out once, computes each enabled fill component's available width as the context does, and renders again each one whose width differs from the `width` it last received. It then lays out and presents. The second pass never triggers a third, so the cost is bounded at one extra render per fill component per trigger.
- Alternative: give the hints `redraw_on` every event the other default segments use. Rejected: any user component with another trigger still leaves the hints stale, and the order of renders within a trigger stays undefined.
- Alternative: let the layout cut the hints instead of dropping them. Rejected: cutting mid-hint shows half a key, and the status line's truncation applies only to the last component left.

The component stores the `width` of its latest context, which the container already builds, so the check needs no new state beyond that number.

### Hint building in the module
`statusline/hints.lua` builds the hints on each render from `ctx.table`:
- In `root`, it prepends `{ key = gband.opt.prefix, label = "prefix" }` when `gband.keymap.list("prefix")` is not empty, and returns nil when `opts.root` is false.
- For each binding, it resolves the label by the precedence the spec gives. `gband.action.list()` is read once per render into a name-to-description map. The short labels are a constant table in the module.
- It formats each key, then emits spans: the key in `KeyHintKey`, then `" " .. label` in `KeyHintLabel`, with `"  "` between hints.
- It fits: it walks the hints, summing `gband.ui.width`, and keeps the longest run whose width, plus 2 for ` …` when hints remain, is at most `ctx.width`.

### Key form by parsing the written key
`keymap.list` returns the key as the user wrote it, so the module parses it: split on `+`, treating a trailing `+` as the key; lowercase the modifiers and collect `ctrl`, `alt` and `shift`; lowercase a key longer than one character and map `escape` to `esc`; turn `shift` plus a lowercase letter into the uppercase letter. The prefix's hint and a `prefix` key both go through the same function with `gband.opt.prefix`. The key was validated when it was bound, so the parser handles only valid names.
- Alternative: expose the parsed key from Rust in `keymap.list`. Rejected: it adds a host field for display only, and the parse is a few lines.

### Groups declared when the module loads
The module calls `gband.hl.default` for `KeyHintKey` and `KeyHintLabel` at its top level, as the sample `pane` plugin does, so a colorscheme loaded before `setup` can still override them. The bundled `default` colorscheme does not set them: their links follow whatever it sets for `StatusLineAccent` and `StatusLineSegment`.

### Options
`setup` validates `align` against the three regions, `priority` and `order` as numbers, `labels` as a table whose keys are strings and whose values are strings or `false`, and `root` as a boolean, raising `error(msg, 2)` like the other segments.

## Risks / Trade-offs

- [`status-line`'s spec may change before it is archived, and the MODIFIED blocks here copy its delta] → Task 1.1 diffs the archived `status-line` and `configuration` specs against these deltas and reconciles them before any code.
- [Two fill components see each other's width from before the extra pass] → The spec defines one extra pass. Only the hints segment uses `fill` by default.
- [Every default line now shows `C-space prefix`, which changes snapshots and row assertions] → Task 4 updates them.
- [A user who copied `defaults/init.lua` before this change has no hints] → `prepare` rewrites only `defaults/init.lua`. The guide shows the one `gband.plugin` line to add.

## Migration Plan

`prepare` rewrites `defaults/init.lua` on the first start of the new build, which adds the hints. A user with their own `user/init.lua` adds `gband.plugin("gband.statusline.hints")` to get them. Rollback is reverting the change.
