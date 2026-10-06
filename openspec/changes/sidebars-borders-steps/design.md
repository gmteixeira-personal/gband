## Context

See proposal.md for the motivation. The code today:

- **Status line.** `gband/statusline.lua` already owns the component API, render scheduling and layout. It hands the client a positioned line through `host.present`, kept as `ui::StatusLine` in `crates/lua/src/ui.rs`. The client splits the terminal into ribbon and status rows in `crates/client/src/placement.rs`, from the `statusline_*` options in `crates/lua/src/options.rs`, and reports the ribbon's size to the server. `render::draw_status` draws the spans.
- **Floating plugin windows.** `gband/win.lua` validates plugin windows and presents frames through `host.present_window`, and `plugin_windows::ribbon_resized` replaces floating plugin windows after a ribbon change. Bars follow the same split.
- **Borders.** Every tile border is `Block::bordered()` with the `FOCUSED_BORDER` or `UNFOCUSED_BORDER` style in `crates/client/src/render.rs`. Floating plugin windows use `Block::bordered()` with `PluginWindowBorder`. The window's terminal size is always its tile less 2 by 2, in `gband_core::geometry`.
- **Steps.** `SessionCommand::StepWidth(Step)` and `StepHeight(Step)` carry only grow or shrink. `Column::step_width` adds `1/10` through `Proportion::step`, and `Column::step_height` computes `(rows + 5) / 10` rows.
- **Dependencies.** This change depends on `floating-windows`, `loop-bands` and `navigation-mode`. Its deltas are written against their versions of the requirements they share: floating-windows' "Present the ribbon", "Action targets", "Session actions resolve against the view", "Client messages", "Handshake" and the new `floating-windows` spec; navigation-mode's status-line "Layout" and "Bundled segment plugins", key-hints "Hints of the active table", "Hint labels" and "Default setup", client-attach "Key bindings", and configuration "Defaults use the public API"; and loop-bands' configuration "Options", which gains `loop_bands`. floating-windows' "Client messages" predates the `command` message from split-plugin-runtime, so this change's delta restores that row and its scenario. The side bar makes scenarios of requirements this change would otherwise leave alone false, so it modifies those requirements to restate them: client-attach "Send input", where "Height excludes the status line" prints 22 beside a left bar; "Key bindings", where "Grow the window's height" starts from an 80×24 terminal and "Center the column", "Float the focused window" and "Move a floating window" name a client with no bar; key-hints "Hint labels", where "Registered action" asks for a segment wide enough for every hint; and plugin-testing "Case environment", where "Configuration of the case" sets the status line up on the right. Later changes that copy these requirements start from this text.

## Goals / Non-Goals

**Goals:**
- Bars change only where the ribbon is drawn. The reported size, the screen area and every PTY size stay independent of bars and borders, so clients with different bars and borders never disturb each other or the windows' programs.
- The status line is ordinary Lua on the public bar API, with no status-line code left in Rust.
- One placement function serves both the Lua API and the client's frame layout.

**Non-Goals:**
- Top and bottom bars, and any bar that reduces the reported size.
- Per-window border overrides and border highlight groups.
- Live refresh of an open error list.

## Decisions

### Bar placement lives in Rust, the registry in Lua
`gband/bar.lua` keeps the bars and validates calls, as `win.lua` does for plugin windows. After any change it presents the ordered list of `{ id, side, size, order, seq, hl, lines }` through `host.present_bars`.

A pure function, `bars::place(bars, terminal) -> (Vec<Option<Rect>>, Rect)`, in a new `crates/lua/src/bars.rs`, gives each bar's rectangle and the ribbon. Lua reaches it as `host.place_bars`, so `gband.bar.info` answers synchronously. The client calls it on every terminal resize and every presentation, so the ribbon is known before the next frame without a Lua round trip.

After a placement changes, the client calls a `bars_resized` hook. `bar.lua` then runs each changed bar's `on_resize`, the way `plugin_windows::ribbon_resized` serves floating plugin windows.

Alternative: place bars in Lua only. Rejected, because a terminal resize would then need a Lua call before the client could size its ribbon and draw.

### The reported size is the terminal size
`Display::set_size` stops deriving the reported size from the ribbon. It keeps `terminal` for the server and `ribbon` for drawing.
- `View` already sizes its camera from the size it is given. The display hands it the ribbon's size and offsets drawing by the ribbon's left column.
- Tile geometry stays computed for the layout's screen area, which follows the terminal.
- A bar change takes the path of a resize except for the message: `presentation.snap()` and `View::sync`, with no `Resize` sent.

Alternative: keep the reported size equal to the ribbon. Rejected because the user asked that bars never change window sizes.

### The status line is the `gband.statusline` plugin
`require("gband.statusline")` already resolves to the bundled `gband/statusline.lua`. The module keeps installing `gband.ui.statusline` when the runtime loads it, and also returns a plugin table, `{ name = "statusline", setup = ... }`. `setup` validates `side`, `min_width`, `max_width` and `order`, and adds the bar `statusline` with `hl = "StatusLine"`.

Layout becomes vertical:
1. Arrange the regions top, center and bottom.
2. Drop the lowest priority until the rows fit.
3. Measure the widest line of the error item and the non-fill components, and clamp it between `min_width` and `max_width`.
4. Render the fill components with `width` and `height` from that layout.
5. Emit the rows as bar lines, with empty lines for gaps, cut with `gband.ui.truncate`.

The bar's size changes through `gband.bar.set_config` only when the clamped width changes. The bar's `on_resize` drives the triggers for terminal height and width.

`ui::StatusLine`, `host.present`, `take_line`, `StatusArea`, `draw_status`, `Placement`'s status rows, and the three `statusline_*` options are deleted.

Alternative: keep a Rust status line beside bars. Rejected because the user asked for a status line made entirely of scripts.

### Component output may hold several lines
`{ lines = { ... } }` is the only new output shape. A string or a span list stays one line, so no existing component changes meaning. The hints segment returns its wrapped lines this way.

### The banner and the error item
The status line no longer prints the message. `statusline.lua` reports through `host.error_item(shown)` whether its error item is drawn. The client draws the banner when an error is reported and no error item is drawn. This keeps the banner's existing behaviour when the status line is absent or hidden, with no Rust knowledge of which bar is the status line.

### The error list is kept by the client, read through `gband.errors()`
The client's `Configuration` already tracks the latest error and when it clears. It gains an ordered `Vec<String>` with the same lifetime: pushed on every reported error, and emptied by a load with none of the client's own. It reaches Lua through the state that `host.state()` pushes, and `gband.errors()` copies it.

`gband/errors.lua` is a bundled plugin that uses only public API: `gband.action.register`, `gband.cmd.register`, `gband.win`, `gband.keymap.enter("root")` and `gband.errors()`. It wraps each text by display width with `gband.ui.width` per character, and wraps again in `on_resize`.

### Borders are drawn by one helper, independent of ratatui's `Block`
A new `draw_border(buffer, rect, sides, chars, style)` in `render.rs` follows the borders spec cell by cell: sides, corners, a corner on one side, and blank cells. The interior is always the rectangle inset by one. The same helper serves tiled windows, floating windows and floating plugin windows, so all three kinds draw a border the same way.

The client options become a `Border { sides: Sides, chars: BorderChars }` per window kind in `Options`. `Ribbon` gains `tile_border` and `floating_border`, and `FloatingFrame::border` becomes `Option<Border>`.

Alternative: keep `Block` with `Borders` flags and `border_set`. Rejected because `Block::inner` shrinks only by the drawn sides, and its corner handling on a missing side is not the spec's.

### Steps travel with the request
- `SessionCommand::StepWidth` and `StepHeight` become `StepWidth { step, by: Proportion }` and `StepHeight { step, by: Proportion }`. `SessionAction` carries the same fields.
- `Column::step_width` adds or subtracts `by`. `Column::step_height` uses `round_half_up(rows × by).max(1)` rows.
- `bindings.rs` fills `by` from `width_step` or `height_step` when it resolves a binding.
- `control.rs` fills it from the client target's `step`, or the same options. `actions.rs` fills it from the server target's `step`, which the server's targets of the four grow and shrink actions now accept, with the client's ranges. The server's Lua uses `1/10` when no step is given.
- `geometry::height_step` and `width_step` stay the fixed tenth. Floating moves, and the height of a new floating box, keep using them, so `crates/core/src/geometry.rs` does not change. The requested step is turned into rows in `layout.rs`.
- The floating-window sizing that floating-windows adds receives the same `by`.

Alternative: server options for the steps. Rejected because the user wants each client to have its own settings, while the resulting sizes stay shared.

### Protocol version
The grow and shrink messages change shape. floating-windows takes the next version after dev's 6, so this change writes 8. When `/ready` merges dev, the version SHALL be one above dev's at that moment. The spec and the `Handshake` scenario are adjusted then if the order of merges differs.

## Risks / Trade-offs

- [A 20-column status line narrows an 80-column ribbon to 60 columns, and full-width columns are cut] → `min_width` and `max_width` are setup options, `side` moves it, and leaving the plugin out removes it. The README shows each.
- [The status line's width follows its content, so the ribbon can shift when a segment grows] → A default `min_width` of 20 covers the bundled segments, so the default width stays put. Fill components never widen the bar.
- [Clients with different borders draw the same grid differently] → Window sizes never depend on borders, so only the drawn characters differ.
- [The `floating-windows` delta modifies a spec that exists only after floating-windows archives] → floating-windows is a declared dependency, so archive order is enforced.
- [Removing `statusline_*` breaks user files] → The error names the option, loading continues with the default status line, and the README gives the plugin options that replace them.
- [navigation-mode and floating-windows edit the same segment files and `render.rs`] → Both are dependencies, so this change starts from their merged code.

## Migration Plan

- A user file that set `statusline_position = "off"` now leaves out `gband.plugin("gband.statusline")`. One that set `"top"` or `"bottom"` sets `side` instead.
- A plugin that used `align = "left"` or `"right"` moves to `"top"` or `"bottom"`.
- Old clients and servers refuse each other at the handshake, as every protocol bump does.
