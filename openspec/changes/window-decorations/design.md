## Context

See proposal.md for the motivation. The state this change starts from:

- `crates/client/src/render.rs` builds one `Layer` per drawn tile, lifted tile and floating window in `layers()`, each with its `RegionKind`, drawn size, `Placement` (position and clip) and border. `draw_tile` draws each one into a scratch buffer: `draw_border`, then the title through `draw_title`, which cuts it to the width less 2, only when `border.sides.top` holds and `clip.top == 0`. Floating plugin windows are drawn later by `draw_float` and never go through `layers()`.
- `Display` (`crates/client/src/lib.rs`) owns everything drawn, and `Display::draw(frame, now)` has no access to Lua. `Controls` owns the `Runtime`. The client loop calls `draw(terminal, &mut display, now)` once per iteration, and `Controls::react` runs after every input and message and ends with `display.take_look(&self.runtime)`.
- Providers are registered in `provide` in `crates/lua/src/ui.rs`. They report failures in two ways. The `windows` and `bars` functions run inside `Runtime::within_callback`, so an error lands in the error list. The `styles` function runs in `take_client_styles` after each `react`, and an error there is only logged as a warning.
- Highlight groups exist only in Lua (`gband.hl`, `hl.lua`). Frames and bars hand Rust resolved styles, which `require("gband.hl").drawn(name)` produces. Rust's `Style` holds plain booleans, so it cannot tell an unset attribute from `false`.
- `guard::isolated(lua, owner, label, f)` gives a call its own instruction limit and labels its error, and `guard::push` reports an error without marking a plugin failed.
- The sibling change `mouse-binding-fallthrough` adds `box_col`, `box_row`, `box_width` and `box_height` to the mouse fields: the press's cell in the box, border included and counted from 0 at the top-left cell, and the box as drawn in the frame the event was resolved against. It also raises `gband.api_version` to 2.

## Goals / Non-Goals

**Goals:**
- One generic mechanism that a Lua key style can use for title bar buttons, with no knowledge of that key style in the executable.
- Nothing drawn today changes while no provider is registered, so no screen reference is regenerated.
- A provider that is called at every frame costs one Lua call per window only when something it can see has changed.
- Lua can work out which span a press hit from documented numbers alone.

**Non-Goals:**
- Merging the decorations of several plugins. One function serves all windows. A bundled module can add merging later on top of the provider.
- Hit-testing in the executable, such as a `span` field in the mouse payload.
- Decorations anywhere but at the right end of the top border, and on floating plugin windows, which draw their own titles.

## Decisions

### A provider function, called by the client
The client calls `fn(info)` when it draws a window's top border, and draws what the function returns.

Alternatives considered:
- *A push primitive, `gband.core.set_decorations(window, spans)` or `gband.window.set_decorations`.* Lua would have to follow every window's opening, closing, focus, floating state and width through events, and set the spans again for each change. It could not adapt to the drawn width during an animation, and stale spans would outlive closed windows unless the client also dropped them. The pull model gives the function every fact it needs in the info table, at the moment the client needs the answer.
- *A bundled API module on top, such as `gband.decorations.add(fn)`.* It would let several plugins contribute spans, but it needs a new module in the prelude and a new public table, for one known consumer. The provider is the smallest primitive, and a module can be built on it later without changing it.

The `styles` provider already sets the pattern of a function the client calls for data to draw, so this needs no new concept.

### Spans carry `style`, not `hl`
A span is a string or `{ text = ..., style = ... }`, where `style` is a resolved style as frames and bars take it.

The executable knows no highlight groups: they live in `gband.hl`, a Lua module that a user can replace. Accepting `hl = "Group"` would make Rust call into that module to resolve a name, which reverses the boundary that `stable-lua-api` drew. Frames and bars already take resolved styles for this reason. A provider gets a group's style with `require("gband.hl").drawn(name)`, a documented export, as `gband.bar` and `gband.win` do.

The `floating-key-style` change must therefore build its spans with `style`, not `hl`.

### The style is applied over the border style in Rust
Rust reads the span's style table as a patch: `fg` and `bg` when present, and each boolean only when the table holds it. It applies the patch over the window's border style, focused or not, so a field the span sets wins and every other field stays the border's. This matches how `gband.win` and `gband.bar` lay a group over a base in Lua with `over`. The patch type keeps an absent boolean apart from `false`, which `Style`'s plain booleans cannot.

A span without `style` is drawn exactly as the border, so a title bar of plain `[_][□][X]` follows `WindowBorder` and `WindowBorderFocused` with no work in the provider.

### Placement: right-aligned, two cells from the corner, all or nothing
For a box of width `w` and spans of total width `d`, the spans start at column `w − 2 − d` and end at column `w − 3`. The corner cell and the cell beside it stay border on both ends. The floating key style treats those two cells along each side as a resize corner, and the sibling change `drag-resize-edges` lets it resize from them.

When `d > w − 4`, the client draws no span at all. Cutting from the left would show half a button, such as `_][□][X]`, and Lua would have to repeat the cut to hit-test it. Leaving out whole spans from the left would keep `[X]` visible longer, but it is a second rule for Lua to repeat. The provider receives `width`, so it can return fewer spans for a narrow box when it wants to.

The title gets `w − 4 − d` cells while the spans are drawn, which leaves at least one border cell between them, and keeps `w − 2` otherwise. In a 40-column box with `[_]`, `[□]` and `[X]`:

```
col  0   1 … 27  28  29 … 31  32 … 34  35 … 37  38  39
     ╭   title    ─   [_]      [□]      [X]      ─   ╮
```

### Hit-testing stays in Lua
A mouse binding finds the span under a press from the documented rule and the `box_*` fields of `mouse-binding-fallthrough`. It calls its own provider function again with an info table built from the event, and walks the spans:

```lua
local function span_at(spans, box_width, box_col)
  local widths, total = {}, 0
  for index, span in ipairs(spans) do
    widths[index] = gband.ui.width(type(span) == "string" and span or span.text)
    total = total + widths[index]
  end
  if total == 0 or total > box_width - 4 then
    return nil
  end
  local first = box_width - 2 - total
  for index, width in ipairs(widths) do
    if box_col >= first and box_col < first + width then
      return index
    end
    first = first + width
  end
end
```

With `box_width = 40` and the three spans above, `first` is 29, so `box_col` 29 to 31 is span 1, 32 to 34 span 2, and 35 to 37 span 3, `[X]`. A press on `box_row` 0 at `box_col` 36 closes the window.

This relies on the function returning the same spans for the same state and info. The client redraws after every callback, so the spans a press lands on are the ones the function returns when the binding runs. A provider whose spans depend on `focused` must pass the focus as drawn, which is the focus before the binding acts.

### Called from the client loop before each draw
`Display::draw` cannot reach Lua, and the function must see the drawn width, which only the frame's layers know. So drawing is split in two:

1. `render::tops(ribbon, area)` lists the top borders a frame will draw, from the same `layers()` that `draw_tile` uses: each layer of kind tile, lifted tile or floating window whose border draws the top side and whose `clip.top` is 0, with its window, floating flag, focus and drawn width.
2. `Controls::decorate(&mut display, now)` asks `Display` for those tops, calls the provider for each one it holds no fresh result for, and stores the spans in `Display`. The client loop calls it right before `draw`, with the same `now`, so the drawn values match.
3. `Display::ribbon` hands the stored spans to `Ribbon`, and `draw_tile` draws them and cuts the title by them.

Tests that call `Display::draw` without `decorate` draw no decorations, so every existing client test keeps its output.

### Cache: same info, no Lua run since
`Runtime` counts its Lua runs: `within_callback` adds one at each start, and every callback, timer, state function, event, windows and bars function and test chunk goes through it. `Display` keeps, per window, the info of the last call, the count at that call, and the spans. `decorate` reuses them while both match. So a still screen calls nothing, a key press calls each window once, and an animation that changes a box's width calls that window once per frame. The `styles` function is the one provider call outside `within_callback`, but it only runs at the end of `react`, after `within_callback` has already counted a run. A reload builds a new `Runtime`, and `Controls::reload` drops every kept result.

Alternatives considered:
- *No cache.* A sidebar clock, an animation and every frame would each call every window's function, and a function that allocates tables would run at frame rate for nothing.
- *Cache by info only.* A maximize state kept in Lua would never show, because the info does not change when it does.

### Calls run outside a callback, with their own limit, labelled `decorations`
`Runtime::decorations(info)` calls the function through `guard::isolated(lua, None, Some("decorations"), …)`, outside `within_callback`. The function therefore gets the instruction limit of one call, `gband.core.dispatching()` is `false` in it, and calling an action is an error, as drawing must not change the layout. An error reads `decorations: user/init.lua:3: boom`, as a labelled `gband.core.call` failure does.

### A failure is reported once, then the provider is stopped
The two existing behaviours do not fit a function called at every frame. Reporting at each call, as the `windows` and `bars` functions are, would add an error per frame during an animation. Logging only, as for `styles`, would hide a bug in a user's function. So a failure is reported to the error list and the log, as `windows` and `bars` failures are, and then the function is stopped: the client draws no decorations and calls it no more until `provide("decorations", …)` registers a function or a load succeeds.

The report goes through `Controls::report`, and `decorate` then runs `react` with no events, so state functions such as the sidebar's error marker see the new error. That `react` counts a Lua run, but the stopped function is not called again, so a failure cannot loop. The stopped flag lives beside the registration in `crates/lua/src/decorations.rs`, and `provide` clears it.

A wrong return value is a failure too, named by the first wrong entry, such as `decorations: span 2: expected a string or a table with a string text, found number`.

### Every drawn window, including drawn windows
The function is called for every tile, lifted tile and floating window, a tiled plugin window's drawn window included. The title skips drawn windows because they have no name, but decorations need none, and the function can tell a plugin window apart through `gband.layout()`'s `plugin_window`. Floating plugin windows and the drop outline are not windows and are skipped.

### Independent of window titles
`window_titles` and `GBAND_WINDOW_TITLES` turn off names, not decorations. Tests start with titles off to keep references free of names, and a case about decorations should not have to turn titles on to see them.

### `gband.api_version` does not rise
The change adds a provider kind and documents its info table and return value. It removes nothing and changes no documented argument, return value, error or effect: `provide("decorations", fn)` was an error for an unknown kind, and an addition that makes a former error valid is what the API version rule calls adding. The "unknown kind" message lists one more kind, which is not documented text. So this change keeps whatever value the integration branch holds, which is 2 once `mouse-binding-fallthrough` lands.

## Risks / Trade-offs

- [A provider that is slow, or whose spans change with the width, runs at frame rate during an animation] → One call per animating window per frame, with the instruction limit as a bound. The docs advise a cheap function and say when it is called.
- [The `floating-key-style` draft may expect `hl` in spans] → The decision and the reason are stated above. That change resolves its groups with `require("gband.hl").drawn`.
- [A stopped provider leaves the title bar without buttons until a reload] → The failure is in the error list and the log, and saving the fixed file reloads the configuration.
- [Hit-testing depends on the function returning the same spans for the same state] → The docs state it, and the worked example uses the same function for drawing and for the press.
- [Siblings edit the same files: `docs/plugins.md`, `README.md`, `crates/client/src/lib.rs`, `crates/client/src/render.rs`] → This change touches only the providers section, its new decorations section, the window names and borders paragraphs, and the draw path. The first task rebases the deltas on the main specs on `dev`.

## Migration Plan

No protocol, server or saved file changes. A client of this build draws exactly what it drew before until a configuration registers a decorations function.
