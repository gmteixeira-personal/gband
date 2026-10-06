## Why

A full-width column, or a full-width floating window, takes the whole terminal width, not the space left beside the bars. With the default 1-column sidebar on an 80-column terminal it is 80 columns wide in a 79-column ribbon, so its right border is always cut off, and a wider bar cuts off more. The same holds for every width: two columns of 1/2 do not fit side by side beside a sidebar. The cause is a deliberate rule, "Bars keep window sizes": the client reports its whole terminal to the server and only narrows its own viewport. niri measures column widths against the output's working area, the space left after exclusive panels, and gband should do the same with its bars.

## What Changes

- **BREAKING** The client's reported size becomes its ribbon area's size: the terminal less the columns its shown bars take. The handshake's hello carries it, and every change of it is reported as a resize, including a change the bars cause.
- `gband server` accepts `--size <cols>x<rows>`, the screen area its first session starts at, 80×24 without it. A client that starts a server passes its reported size, so the first window starts at the right size and is not resized right after launch.
- Adding, removing or resizing a bar, or a bar that stops being shown, now resizes the session's screen area. Every column width, full width included, and every floating box are measured against the space the bars leave. Windows' PTYs follow after the server settles, as for a terminal resize.
- Moving a bar to the other side keeps the ribbon's size, so nothing is reported.
- Several bar changes from one callback or one load are reported as at most one resize.
- The requirement "Bars keep window sizes" is renamed "Bars narrow the screen area" and rewritten to match.
- The session's screen area and a new session's starting area are worded as the client's reported size, not its terminal size.
- Every scenario that assumes an 80-column screen area beside the default sidebar is restated for the 79-column area, or for a client with no bar.
- A band's strip loops when its width less its widest column is at least the terminal's width less one, the exact bound at which no window is drawn twice. The old bound, the terminal's full width, stops three half-width columns from looping on any odd area. That would include the default 79-column area beside the sidebar, which the old area, one column wider than the ribbon, never hit.

## Capabilities

### New Capabilities

### Modified Capabilities
- `bars`: bars narrow the client's reported size and so the session's screen area, in place of keeping window sizes.
- `client-attach`: the reported size is the ribbon area's size, the hello carries it, bar changes that change it are reported, a server the client starts gets it with `--size`, and the sidebar scenarios follow the 79-column area.
- `command-line`: `gband server` accepts `--size <cols>x<rows>`.
- `session-server`: the screen area and a new session's starting area follow the client's reported size, and the session created on start begins at `--size`.
- `layout-view`: the Camera requirement's loop threshold becomes the terminal's width less one.

## Impact

- Client: `crates/client/src/lib.rs` (reported size, bar changes reported as resizes, bars placed before the handshake) and `crates/client/src/connect.rs` (the hello's size).
- No wire protocol change: the hello and resize messages keep their fields and only carry a different size.
- Server start: `src/main.rs` (`server --size`), `crates/server/src/lib.rs` (`ServerConfig::area`), `crates/client/src/transport.rs` (`Transport::open` takes the size, and the started server gets `--size`), `crates/test-support/src/lib.rs`, and `Cargo.toml` (gband-core becomes a dependency of the binary).
- Tests: client bar and sidebar tests, end-to-end tests that measure tiles beside the default sidebar, and the Lua screenshots that show a tile beside the sidebar.
- Docs: `docs/plugins.md` states that bars never change a window's size.
- Core: `crates/core/src/geometry.rs` (`loop_width`), with tests in `crates/core/tests/geometry.rs` and `crates/core/tests/view.rs`.
- The client-attach "Key bindings" requirement holds two scenarios with numbers from the 80-column area, "Repeated resize" and "Grow the column". Three open changes also rewrite that requirement, so the delta for it is written during implementation, against the latest `origin/dev`.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- openspec/changes/ribbon-screen-area/
- Cargo.toml
- src/main.rs
- crates/client/src/connect.rs
- crates/client/src/lib.rs
- crates/client/src/requests.rs
- crates/client/src/transport.rs
- crates/client/tests/actions.rs
- crates/client/tests/bars.rs
- crates/client/tests/connect.rs
- crates/client/tests/sidebar.rs
- crates/client/tests/snapshots/sidebar__default_sidebar_snapshot.snap
- crates/client/tests/snapshots/sidebar__error_marker_snapshot.snap
- crates/client/tests/transport.rs
- crates/core/src/geometry.rs
- crates/core/tests/geometry.rs
- crates/core/tests/view.rs
- crates/server/src/lib.rs
- crates/test-support/src/lib.rs
- docs/plugins.md
- examples/plugins/agent-status/tests/screenshots/agent_status_spec/a-prompt-marks-the-window-waiting.txt
- examples/plugins/window/tests/screenshots/window_spec/
- tests/attach.rs
- tests/config.rs
- tests/lua/keylist_spec.lua
- tests/lua/screenshots/
- tests/navigation.rs
- tests/plugin_runtime.rs
- tests/plugin_testing.rs
- tests/sessions.rs
- tests/sidebar.rs
- tests/subcommands.rs
