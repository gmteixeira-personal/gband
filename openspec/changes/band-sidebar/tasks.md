## 0. Specs

- [ ] 0.1 For each MODIFIED requirement in this change's delta specs, compare the block with the current `openspec/specs/<capability>/spec.md` block of the same requirement, as sidebars-borders-steps, clear-errors, lua-prompt, key-style, mouse and the changes archived before this one left it. Fold in every requirement text and scenario this change did not write, and keep every scenario name the current block has. Do the same for each REMOVED requirement of `status-line` and `key-hints`, so that every requirement those specs hold is removed. Verify that the only differences left are this change's own, and that `openspec validate band-sidebar --strict` passes with no INFO line about a missing target spec

## 1. Sidebar plugin

- [ ] 1.1 Add `crates/lua/src/runtime/gband/sidebar.lua`, the plugin `sidebar`, and list it in `crates/lua/src/bundled.rs`. Its `setup` checks `side` and `order`, rejects any other option, sets the defaults of `SidebarMode`, `SidebarBand`, `SidebarBandActive` and `SidebarError`, and adds a bar of size 1 with `gband.bar.add`. Verify in a new `crates/lua/tests/sidebar.rs`: "Default sidebar", "Sidebar on the right", "Unknown option", "Wrong side" and "Defaults without a colorscheme".
- [ ] 1.2 Trim the view state that `crates/client/src/lib.rs` pushes and `crates/lua/src/ui.rs` stores to the band index, the band count, the active table and the errors. Keep the `on_state` hook and `host.error_item`. Have `sidebar.lua` build its lines in `on_state` as design.md's "Rows", "Band labels" and "Mode letter" give them, set them with `gband.bar.set_lines`, and report `host.error_item(shown)`. Verify in a new `crates/client/tests/sidebar.rs`, which replaces the sidebar-related cases of `crates/client/tests/statusline.rs`:
  - "Rows of a new session" and "Short terminal";
  - "Navigation mode", "Back to interactive mode", "Prefix without a mode" and "User mode";
  - "Window opened in the empty band", "Band removed", "Tenth band" and "More bands than rows";
  - "Viewed band" and "View moves down", reading each label's style;
  - "Error reported", "Error cleared" and "Sidebar hidden";
  - "Theme the viewed band".
- [ ] 1.3 Add `tests/lua/sidebar_spec.lua`, with references under `tests/lua/screenshots/sidebar_spec/`, and a test in `tests/lua_specs.rs` that runs it, as the gband-test skill describes. Cover "Rows of a new session", "Navigation mode", "Window opened in the empty band" and "View moves down" with a screenshot each. Read each reference before committing it, and verify with `cargo test --test lua_specs`.

## 2. Defaults and colours

- [ ] 2.1 In `crates/lua/src/defaults.lua`, set up `gband.errors` and then `gband.sidebar`, in place of `gband.statusline` and its segment plugins. Verify "Defaults reproduce the built-in behaviour" and the other default configuration scenarios in `crates/lua/tests/config.rs`.
- [ ] 2.2 In `crates/lua/src/runtime/gband/colors/default.lua`, remove the `StatusLine*` groups and set the sidebar groups and `KeyListKey` and `KeyListMuted` to the colours in design.md. In `crates/lua/src/runtime/gband/keylist.lua`, replace the status line group defaults and links with `KeyListKey` as `{ bold = true }` and `KeyListMuted` as `{ dim = true }`. Verify the colorschemes scenarios "Default sets the sidebar groups", "Explicit settings replaced", "User settings after the colorscheme" and "Failing colorscheme" in `crates/lua/tests/colorschemes.rs`, "Change in a callback" in `crates/lua/tests/highlights.rs`, and "Muted without a status line" in `tests/lua/keylist_spec.lua`.

## 3. Removing the status line and the key hints

- [ ] 3.1 Delete `crates/lua/src/runtime/gband/statusline.lua` and `crates/lua/src/runtime/gband/statusline/`, and drop their entries from `crates/lua/src/bundled.rs`. Remove the status line component registry, its layout and `gband.ui.statusline` from `crates/lua/src/ui.rs`, `crates/lua/src/runtime.rs` and `crates/lua/src/lib.rs`, keeping `gband.ui.width` and `gband.ui.truncate`. Verify that `rg -n 'statusline|StatusLine|KeyHint' crates/lua/src crates/client/src` prints nothing, and that `cargo build --workspace` passes.
- [ ] 3.2 Move the scenarios that outlive the status line to their new capabilities:
  - "Width" and "Truncate at a wide character" from `crates/lua/tests/statusline.rs` to `crates/lua/tests/plugin_windows.rs`, calling `gband.ui.width` and `gband.ui.truncate` directly;
  - the key form cases from `crates/lua/tests/key_hints.rs` to `crates/lua/tests/plugins.rs`, through `require("gband.keyform")`, and "Prefix key in the key list" to `tests/lua/keylist_spec.lua`;
  - the configuration error, banner and clear cases from `crates/client/tests/statusline.rs` to `crates/client/tests/sidebar.rs`, reading the error marker in place of the error item.

  Then delete `crates/lua/tests/statusline.rs`, `crates/lua/tests/key_hints.rs`, `crates/client/tests/statusline.rs` with its `statusline__*` snapshots, and `tests/lua/statusline_spec.lua` with `tests/lua/screenshots/statusline_spec/` and its entry in `tests/lua_specs.rs`. Remove `old_event_in_redraw_on` and `old_action_in_hints_labels` from `crates/lua/tests/removed.rs`, and verify "Removed pane names" with the cases left there. Verify with `cargo test -p gband-lua -p gband-client`.
- [ ] 3.3 Replace `tests/statusline.rs` with `tests/sidebar.rs`, covering the client-attach scenarios "Height excludes the status line", "Default sidebar", "Turning the status line off" and "Moving the status line" end to end. Verify with `cargo test --test sidebar`.

## 4. Scenarios that move to the one-column sidebar

- [ ] 4.1 Client: update `crates/client/tests/render.rs`, `crates/client/tests/bars.rs`, `crates/client/tests/events.rs`, `crates/client/tests/plugin_windows.rs` and `crates/client/tests/actions.rs`, and their snapshots under `crates/client/tests/snapshots/`, for "Present the ribbon": "Tile taller than the ribbon area", "Ribbon below a top status line" and the banner drawn only while no error marker is drawn. Review each changed snapshot with `cargo insta review` before accepting it, and verify with `cargo test -p gband-client`.
- [ ] 4.2 Configuration: update `crates/lua/tests/config.rs`, `tests/config.rs` and `tests/errors.rs` for "Configuration file", "Configuration errors" ("Syntax error", "Plugin error in the status line", "Banner without a status line", "Cleared by hand", "Error after a clear") and "Server error cleared in one client". Verify with `cargo test -p gband-lua --test config`, `cargo test --test config` and `cargo test --test errors`.
- [ ] 4.3 Modes: update `tests/navigation.rs` for "Repeated focus moves", `tests/keystyle.rs`, `crates/lua/tests/keystyle.rs`, `tests/lua/keystyle_spec.lua` and `tests/lua/screenshots/keystyle_spec/` for "Bundled plugins set up by the preset", "Hints and labels of the direct style", "Chooser beside the default status line" and "First start". Verify with `cargo test --test navigation --test keystyle --test lua_specs` and `cargo test -p gband-lua --test keystyle`.
- [ ] 4.4 Prompt and error list: update `tests/prompt.rs`, `crates/lua/tests/prompt.rs`, `tests/lua/prompt_spec.lua` and `tests/lua/screenshots/prompt_spec/` for "Prompt on the bottom rows", "Enter a mode" and "Hint for the prompt". Update `crates/lua/tests/errors.rs`, `tests/lua/errors_spec.lua` and `tests/lua/screenshots/errors_spec/` for "Floating plugin window of the defaults" and "Clear with c". Verify with `cargo test --test prompt --test lua_specs` and `cargo test -p gband-lua --test prompt --test errors`.
- [ ] 4.5 Plugin testing and the test channel: update `tests/plugin_testing.rs` for "Configuration of the case", "Plugin under test is installed", "Reload after editing a plugin", "Status line colour", "Server handler effect is drawn", "Clock segment frozen" and "Time moved", and `crates/lua/tests/plugins.rs` for "Bundled module" and "Bundled module shadowed". Verify with `cargo test --test plugin_testing` and `cargo test -p gband-lua --test plugins`.
- [ ] 4.6 Update every other screen test and screenshot that assumed the 20-column status line: `tests/floating.rs`, `tests/attach.rs`, `tests/plugin_windows.rs`, `tests/plugin_runtime.rs`, `tests/mouse.rs`, `tests/lua/keylist_spec.lua`, `tests/lua/loop_bands_spec.lua`, `tests/lua/mouse_spec.lua` and their directories under `tests/lua/screenshots/`, plus the borders scenario "Tiles without side borders" where sidebars-borders-steps tests it. Read each regenerated screenshot before committing it. Verify that `rg -n -i 'statusline|status line' tests crates/*/tests` prints only scenario names the specs keep, and that `cargo test --workspace` passes.

## 5. Examples

- [ ] 5.1 Rewrite `examples/plugins/window/lua/window/init.lua` to add a right bar 12 columns wide showing `window <n>`, redrawn on `FocusChanged`, with `WindowSegment` linked to `SidebarMode`. Make `examples/plugins/window/colors/dusk.lua` set the sidebar groups in place of the `StatusLine*` groups. Update `examples/plugins/window/README.md`, `examples/plugins/window/tests/window_spec.lua` and its screenshots, and read each screenshot. Verify with `gband test` run in the example's directory, as `docs/testing.md` describes.
- [ ] 5.2 Rewrite `examples/plugins/agent-status/client.lua` to keep the set of waiting windows from `WindowStateChanged` and `WindowClosed` and show its size in a right bar 3 columns wide. Update `examples/plugins/agent-status/README.md`, `examples/plugins/agent-status/tests/agent_status_spec.lua` and its screenshot. Regenerate `examples/plugins/hello/tests/screenshots/hello_spec/greet-opens-a-window-that-prints-the-greeting.txt`. Read each screenshot, and verify both examples with `gband test`.

## 6. Docs

- [ ] 6.1 Update `README.md`, `docs/plugins.md`, `docs/testing.md` and `.claude/skills/gband-test/SKILL.md`:
  - replace the status line sections with the sidebar: its rows, band labels, the viewed band's groups, the error marker, and the `side` and `order` options;
  - remove the component API, the segment plugins, the hints segment and its `labels`, and the `StatusLine*` and `KeyHint*` groups;
  - move `gband.ui.width` and `gband.ui.truncate` and the key form next to plugin windows and the key list;
  - show a plugin bar with `gband.bar.add` where the docs showed a status line component;
  - list the sidebar and key list groups under highlight groups and colorschemes.

  Verify that `rg -n -i 'statusline|status line|StatusLine|hints segment' README.md docs .claude/skills/gband-test/SKILL.md` prints nothing.

## 7. Gate

- [ ] 7.1 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`, and verify all pass.
