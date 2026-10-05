## Context

This change starts after `client-plugin-foundation` is archived, and is written against what that change delivers in `crates/lua`:
- `runtime.rs`, with the runtimepath searcher at `package.searchers[2]` and plugin-file sourcing.
- `owner.rs`, with the owner stack and the failed-plugin set.
- `guard.rs`, with protected runs, the instruction hook and a budget that resets at the outermost run.
- `events.rs`, with `gband.on` and `Runtime::emit`.
- `options.rs`, with built-in option metadata and `gband.opt`.
- The `Runtime` handle, returning `Outcome { dispatched, errors, disabled }`.

In `crates/client`, `Controls` diffs an `Observed` snapshot to emit events, and `Display` holds `terminal: Size`. That size is both what the client reports to the server and the view's viewport. `render::draw_frame` draws the ribbon over the whole buffer, then the error banner on the bottom row.

The client detects no terminal capability today: no truecolor check exists anywhere. Bands have numbers and an order, and no names. The client has no reconnect state: on losing the server it exits.

## Goals / Non-Goals

**Goals:**
- The status line container, highlight groups, colorschemes and bundled segments are Lua shipped inside gband. They use the foundation's owner model, guard and events. Rust adds only host primitives that Lua cannot provide: display width, timers, the client's view state, a guarded call with its own budget, built-in event emission and the drawn line.
- No `render` runs on a frame. Frames draw a line that Rust already holds.
- A change of placement goes through the same code as a terminal resize.

**Non-Goals:**
- Mouse input, several lines of components, and per-component rows. Rows after the first are fill.
- A 16-color fallback. Without truecolor, the client assumes 256 colors.
- Exposing the host primitives as public API.

## Decisions

### Bundled Lua lives in `crates/lua/src/runtime/`
The Lua sources sit under `crates/lua/src/runtime/gband/`: `hl.lua`, `colorscheme.lua`, `statusline.lua` and `statusline/{band,mode,position,clock}.lua`. They also hold `colors/default.lua`. A `bundled.rs` embeds them with `include_str!` into a name-to-source table. Every chunk is named `@gband/<path>`, so errors read `gband/statusline/band.lua:12: …` and need no new parsing.

Two kinds of bundled files exist:
- **API chunks** (`hl.lua`, `colorscheme.lua`, `statusline.lua`) are run by Rust while it installs the `gband` table, before the init file. Each receives the private `host` table as its chunk argument (`...`) and fills part of `gband`. They belong to no plugin.
- **Modules** (`gband.statusline.*`) are found by a searcher inserted right after the runtimepath searcher, so a runtimepath entry can shadow them. They are set up through `gband.plugin` like any plugin.

Bundled colorschemes are not modules. `colorscheme.lua` looks for `colors/<name>.lua` on the runtimepath with `io.open`, and then in the bundled table.
- Alternative: a bundled runtimepath directory written to disk like `defaults/`. Rejected because the foundation rejected an auto-sourced bundled layer, and because files on disk drift from the binary.
- Alternative: the container in Rust with a Lua API. Rejected by the user: the status line is built on the plugin foundation and written in Lua, so that built-in segments and third-party segments use one API.

### The private host table
`host` is a Rust-made table that is passed only to the API chunks and never stored in `gband`, so user code cannot reach it:

| primitive | purpose |
|---|---|
| `host.owner()` | the current owner's plugin name, or nil |
| `host.loading()` | whether the configuration is loading |
| `host.call(owner, label, fn, ...)` | an isolated guarded call; returns `true, results…` or `false, message` |
| `host.emit(event, payload)` | deliver a built-in event (`HighlightChanged`, `ColorschemeChanged`) through `Runtime::emit`'s path |
| `host.state()` | the latest view state pushed by the client |
| `host.present(line)` | store the laid-out line for the client to take |
| `host.timer(ms, fn)`, `host.cancel(id)` | repeating timers driven by the client loop |
| `host.after_event(fn)` | register the function `Runtime::emit` calls after an event's handlers |
| `host.warn(text)` | log a warning |

`gband.ui.width` and `gband.ui.truncate` are public, use `unicode-width`, and strip control characters first. ratatui measures with the same crate, so the Lua layout and the drawn cells agree.

### Isolated guarded calls
The foundation's guard shares one budget across nested runs and keeps raising after exhaustion until the outermost run returns. A render called inside the status line's own run would then kill that run and every later component. `guard::isolated(owner, f)` saves the current budget and exhaustion state, runs `f` as if it were outermost, and restores the saved state after it. Its error becomes a `ConfigError` carrying the owner. When the limit stopped the call, the owner is marked failed, as the foundation does for a stopped callback. `host.call` wraps it. The `label` argument replaces the plugin name in the error's prefix, so a colorscheme reports as `colors/<name>`. A labelled call never marks a plugin failed.

### Highlight store in Lua
`hl.lua` keeps `groups[name] = { explicit = spec?, default = spec? }`. `set` and `default` validate the spec, normalise colors (hex to lowercase, names kept for `get`), and run the cycle check: they walk the definition chain from the group, with the new spec in place, and raise `error(msg, 2)` so the error points at the caller's line. `resolve(name)` walks the links with a visited set. When it meets a group a second time, it stops at that link and calls `host.warn`, once per distinct cycle per load. For drawing, a resolved style maps named colors to their index, so Rust receives only `"#rrggbb"` strings or integers. Change events go through `host.emit` when `host.loading()` is false and no colorscheme is loading.
- Alternative: the store in Rust. Rejected: it would add a second validation path beside the Lua one, and the status line, the only reader, is Lua.

### Colorscheme loading
`gband.colorscheme(name)` snapshots every `explicit` field, clears them all, then runs the file through `host.call(nil, "colors/" .. name, chunk)`. On failure it restores the snapshot. On success it records the active name, and then emits `ColorschemeChanged` when not loading. While a colorscheme runs, `hl.set` emits nothing. Rust installs the API chunks and then calls `gband.colorscheme("default")` before the init file.

### Line layout in Lua, drawing in Rust
`statusline.lua` holds the components in a list with their cached outputs. After any render or removal, and when the error or the width changes, it lays out the line as the spec defines. The steps are: normalise spans, measure, drop by priority, truncate the last one, and place the regions. It then calls `host.present({ height, base = style, spans = { { col, text, style } } })` with every style already merged over `StatusLine`. Rust converts the styles once, in `present`, into a `StatusLine` value that `Runtime::take_line()` hands to `Display`. Each frame, `draw_frame` fills the status rows with the base style and writes the spans. No Lua runs during a frame.

### Triggers and ordering
Rust drives the status line through the runtime, never per frame:
- `Runtime::emit` delivers an event's handlers, then calls the `after_event` function. That function renders the components whose `redraw_on` names the event, and every component on `TerminalResized`, `HighlightChanged` and `ColorschemeChanged`. So a component renders after the user's handlers, as the spec requires.
- `Runtime::refresh_statusline()` renders every component. The client calls it after `Attached`, after `ConfigReloaded`, and when the status line becomes drawn.
- Timers: `host.timer` stores `{ period, next, RegistryKey }` in app data. `Runtime::next_timer()` gives the earliest deadline, and the client's `select!` sleeps until the sooner of it and the next animation frame. `Runtime::fire_timers(now)` runs the due timers through the guard and reschedules each from its deadline, skipping missed periods. The status line creates a component's timer only while the status line is drawn, and cancels it on removal or disable.

### View state pushed by the client
`Controls` already snapshots `Observed` around every change. The snapshot gains the band index and count, the focused column index and count, the active table, the terminal width, whether the status line is drawn, and the latest error text. `Runtime::set_state` stores them before every emit, refresh and timer run. `host.state()` returns a fresh table, and the status line builds each context from it. `Observed` also keeps a layout fingerprint (bands, columns, panes, widths), and a difference after the first layout emits `LayoutChanged` after any `PaneOpened` and `PaneClosed`.

### Ribbon area and placement
`crates/client/src/placement.rs` holds `Placement { position, height }` built from the options, and `split(terminal) -> (ribbon: Rect, status: Option<Rect>)`. `Display` keeps `terminal` and adds `ribbon: Rect`. `Scene::viewport` becomes the ribbon size, and `render` draws into the ribbon rectangle: `Placement` gains the rectangle's `y` offset, and the rectangle's height clips rows. A status-line-aware `Display::set_size(terminal, placement)` replaces `resize`. A terminal resize and a reload both call it. When the ribbon size changes, it snaps the presentation, syncs the view and returns the new reported size, which the caller sends as `ClientMessage::Resize`. The handshake in `connect.rs` takes the placement from the client configuration, and sends the ribbon size in `Hello`.

The banner moves to the ribbon's bottom row and is drawn only when `status` is `None`.

### Errors into the status line
`Display` keeps the latest error text, set from `Configuration::error`, `Config::errors` and every `Outcome::errors`, as the banner text is set today. Render errors come back from `host.call` inside the status line. The status line reports them through the `Outcome` of the run that triggered the render, so they reach the log and the same latest-error slot. The error text travels to Lua in the pushed state. The error item is laid out from it, with a priority above every component.

### Color support in the client
`crates/client/src/color.rs` holds `ColorSupport::{TrueColor, Indexed}`, detected once at startup from the client's `COLORTERM`. It also holds the conversion of a resolved style to a ratatui `Style`, and `nearest_index(rgb) -> u8` over indexes 16 to 255, with the lower index winning a tie. `ColorSupport` is passed to `Display` and used in `present`.

### Bundled segment plugins
Each module returns `{ name = <plugin>, api = 1, setup = function(opts) … end }`. Its `setup` validates `align`, `priority`, `order` and `hl` (and, for `clock`, `format` and `interval`) and calls `gband.ui.statusline.add` without `id`. `band` renders `"band " .. ctx.band.index`. `mode` returns nil for `root`. `position` returns nil when `ctx.column` is nil. `clock` returns `os.date(format)`.

### Sample plugin
`examples/plugins/pane/` has `lua/pane/init.lua` and `colors/dusk.lua`. The module declares `gband.hl.default("PaneSegment", { link = "StatusLineAccent" })` and adds a right-aligned component, with `redraw_on = { "FocusChanged" }`, that shows `pane <n>`. `colors/dusk.lua` sets the built-in groups with hex colors, and sets `PaneSegment`. The README installs the plugin by symlink and shows `gband.plugin("pane")` with `gband.colorscheme("dusk")`.

## Risks / Trade-offs

- [The foundation's spec text may change before it is archived, and the MODIFIED blocks here copy it] → Task 1.1 diffs the archived specs against these deltas and reconciles them before any code.
- [Every user with a `user/init.lua` loses one ribbon row and sees an empty status line] → The proposal marks this as breaking. The guide and the README show the three `gband.plugin` lines and `statusline_position = "off"`.
- [Integration tests that measure rows assume the full terminal] → Task 7.3 updates them to the ribbon area, or turns the status line off where the scenario says so.
- [A component's `ctx.width` uses the other components' latest outputs, so it can be stale until they render again] → The spec defines it that way. The guide calls it a hint, not a promise.
- [A colorscheme or segment file runs in the server too, because the server evaluates the full load] → Nothing renders there and no timer fires. The cost is one file run per load.
- [Lua's `%c` misses C1 controls] → Stripping removes U+0080 to U+009F by their UTF-8 bytes, and Rust strips again when it converts the line.

## Migration Plan

`prepare` rewrites `defaults/init.lua` with the three `gband.plugin` calls on the first start of the new build. A user with their own `user/init.lua` adds those calls to get the segments, or sets `statusline_position = "off"` to keep the old screen. Rollback is reverting the change. No state persists beyond the mirror file.
