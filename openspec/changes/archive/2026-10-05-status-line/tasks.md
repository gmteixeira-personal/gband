## 1. Preconditions

- [x] 1.1 Confirm that `client-plugin-foundation` is archived on `<base>`. Diff its archived `configuration`, `client-attach`, `lua-events` and `plugins` specs against the MODIFIED blocks of this change, and reconcile every requirement the foundation changed after this change copied it. Verify with `openspec validate status-line --strict` reporting no error and no "target spec does not exist" notice.
- [x] 1.2 Add `unicode-width` to `[workspace.dependencies]` at the version ratatui already uses, and to `crates/lua`. Verify with `cargo tree -d -i unicode-width` showing one version.

## 2. Host primitives in `crates/lua`

- [x] 2.1 Add `guard::isolated(owner, label, f)`. It saves the budget and exhaustion state, runs `f` with a fresh budget, restores the saved state, marks the owner failed when the limit stopped an unlabelled call, and uses `label` in place of the plugin name in the error. Verify with unit tests: an isolated `while true do end` inside an outer run stops only the inner call, and the outer run then completes. A labelled stop marks nothing failed.
- [x] 2.2 Add `bundled.rs`, embedding `src/runtime/gband/**` by name, and a searcher placed after the runtimepath searcher with chunks named `@gband/<path>`. Verify with `crates/lua/tests/plugins.rs` tests for "Bundled module" and "Bundled module shadowed".
- [x] 2.3 Add `ui.rs` with the private `host` table (`owner`, `loading`, `call`, `emit`, `state`, `present`, `timer`, `cancel`, `after_event`, `warn`) and the public `gband.ui.width` and `gband.ui.truncate`. Verify with unit tests for the "Width" and "Truncate at a wide character" scenarios, control-character stripping, and `truncate` with a width of 0 and of 1.
- [x] 2.4 Add `Runtime::set_state`, `refresh_statusline`, `take_line`, `next_timer` and `fire_timers`. Make `Runtime::emit` call the `after_event` hook after the handlers. Make `host.emit` deliver `HighlightChanged` and `ColorschemeChanged` like the other built-in events. Verify with unit tests: a timer of 100 ms fires 3 times when `fire_timers` is called at 350 ms, and `after_event` runs after a user handler of the same event.
- [x] 2.5 Add the built-in options `statusline_position`, `statusline_height` and `statusline_separator` to `options.rs`, with validation and `gband.opt` reads. Verify with tests for "Invalid status line height" and reads of each option's default.
- [x] 2.6 Change the load sequence: install the `gband` table, run the API chunks with `host`, then call `gband.colorscheme("default")`, then the init file and the plugin files. Verify that every existing `crates/lua` test still passes.

## 3. Highlights and colorschemes in Lua

- [x] 3.1 Write `src/runtime/gband/hl.lua`: name and spec validation, color normalisation, the explicit and default layers, `set(name, nil)`, the cycle check at the caller's line, resolution with the cycle warning, `get` with and without `resolve`, and change events outside the load. Verify with `crates/lua/tests/highlights.rs` covering every highlights scenario except "Drawn colors".
- [x] 3.2 Write `src/runtime/gband/colorscheme.lua` and the bundled `colors/default.lua`, which sets every built-in group with hex colors. Implement the lookup order, name validation, snapshot and restore on failure, `colors/<name>` errors, the active name and `ColorschemeChanged`. Verify with `crates/lua/tests/colorschemes.rs` covering every colorschemes scenario.

## 4. Status line container in Lua

- [x] 4.1 Write `src/runtime/gband/statusline.lua` with `add`, `remove` and `list`: spec validation, namespaced ids, the id omitted inside a plugin, duplicates, and the built-in group defaults. Verify with `crates/lua/tests/statusline.rs` covering every "Components" scenario and "Defaults without a colorscheme setting".
- [x] 4.2 Implement output normalisation and the context from `host.state()`. Verify with tests for "Plain string", "Escape sequence removed", "Hidden component", "Context values" and "Available width".
- [x] 4.3 Implement the layout: regions, `order`, separators, gaps, dropping by priority, truncating the last component, centering, and the error item first with the top priority. Then `host.present` with merged styles. Verify with tests for every "Layout" scenario that needs no client, plus the error item being kept when the line overflows.
- [x] 4.4 Implement the triggers through `after_event`, `refresh_statusline`, adds after the load, and timers created only while drawn. Run every render through `host.call`. Verify with tests that count render calls: "Redraw on an event", "Interval", "Width change", no render when the status line is not drawn, and no render from `take_line` called many times.
- [x] 4.5 Implement render errors: disable on error, invalid return or limit, hide failed plugins' components, and report the error through the run's `Outcome`. Verify with tests for "Failing component" and "Looping render", the second with a small budget, checking that the other components still render in the same refresh.

## 5. Bundled segments and defaults

- [x] 5.1 Write `src/runtime/gband/statusline/{band,mode,position,clock}.lua` with their options and validation. Verify with tests that set each one up, check its `list()` entry and output for a given state, and check that an invalid `align` fails `setup`.
- [x] 5.2 Add the three `gband.plugin` calls to `crates/lua/src/defaults.lua`. Verify with the updated "Defaults reproduce the built-in behaviour" test and with "Copied defaults load unchanged".

## 6. Client

- [x] 6.1 Add `crates/client/src/color.rs`: `ColorSupport` detected from `COLORTERM`, style conversion, and `nearest_index`. Verify with unit tests for every "Drawn colors" scenario, a tie, and named colors kept as indexes.
- [x] 6.2 Add `crates/client/src/placement.rs` and the ribbon rectangle in `Display`. Make the viewport the ribbon size. `set_size` replaces `resize` and returns the reported size when it changes. Send that size in `Hello` and in `Resize`. Verify with `crates/client` tests for the ribbon size under each placement, "Terminal too short", and the view's shown panes using the ribbon height.
- [x] 6.3 Draw into the ribbon rectangle with the row offset, draw the status rows from `StatusLine`, and draw the banner only without a status line. Verify that every existing render snapshot is unchanged with the status line off, and add insta snapshots for the default status line, the top placement, a narrow width with a dropped segment and a cut segment, and the error item.
- [x] 6.4 Extend `Observed` with the band, column, table, width, drawn flag, error text and layout fingerprint. Push the state before every emit, refresh and timer. Emit `LayoutChanged`. Call `refresh_statusline` after `Attached`, after `ConfigReloaded` and when the status line becomes drawn. Take the line after every runtime call. Verify with `crates/client/tests/events.rs` for "Layout change without a pane change" and the updated "Nothing at attach", and with a test showing the mode segment after Ctrl+Space.
- [x] 6.5 Add the timer deadline to the `select!` in `attach`, beside the animation frame. On reload, apply the new placement through `set_size` and send `Resize` when the reported size changes. Verify with a client test in which a reload from `bottom` to `off` sends one resize of the full terminal size, and a reload from `bottom` to `top` sends none.
- [x] 6.6 Route `Configuration::error`, `Config::errors`, `Outcome::errors` and render errors to the latest-error slot. Verify with client tests for "Plugin error at start", "Cleared by a good load" and "Plugin error on the banner".

## 7. Integration

- [x] 7.1 Add `tests/statusline.rs`, running real processes, with tests for "Height excludes the status line", "Turning the status line off" and "Moving the status line". Verify with `cargo test --test statusline`.
- [x] 7.2 Add an integration test with a looping component in a plugin directory and the real budget. The client keeps handling keys, and the status line shows the error. Verify with `cargo test --test statusline`.
- [x] 7.3 Run `cargo test --workspace`. Update each integration test whose expected rows assumed the full terminal: expect the ribbon area, or turn the status line off where the spec scenario says it is off. Verify that the full suite passes.

## 8. Sample plugin and docs

- [x] 8.1 Add `examples/plugins/pane/` with `lua/pane/init.lua`, `colors/dusk.lua` and `README.md`, as the design describes. Verify with a `crates/lua` test that loads an `init.lua` calling `gband.colorscheme("dusk")` and `gband.plugin("pane")` with the example on the runtimepath. It checks that `PaneSegment` resolves to the colorscheme's style and that the component renders `pane 3` for focused pane 3.
- [x] 8.2 Extend `docs/plugins.md` with a status line section: the component spec, the context, the triggers, the layout, errors and the bundled segments with their options. Add a highlight groups section and a colorscheme section. Replace "planned for later" for the status line and highlight groups. Verify that every API and option in the highlights, colorschemes and status-line specs appears in the guide.
- [x] 8.3 Update `README.md`: the status line, its placement options, and the three `gband.plugin` lines a `user/init.lua` needs for the default segments. Verify by reading it against the configuration and status-line specs.

## 9. Gate

- [x] 9.1 Run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`, and confirm they pass.
- [x] 9.2 Manual check: attach with no `user/init.lua` and confirm that the bottom row shows `band 1` and the position, and `prefix` while Ctrl+Space is held in sequence. Set `statusline_position = "top"`, then `"off"`, in `user/init.lua`, and confirm that each reload moves or removes the line and that `tput lines` follows. Install `examples/plugins/pane`, load `dusk` under `COLORTERM=truecolor` and again with `COLORTERM` unset, and confirm that the colors degrade. Then add a component that raises an error, and confirm that the error item names its plugin.
