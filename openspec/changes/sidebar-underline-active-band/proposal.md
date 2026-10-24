## Why

The sidebar marks the viewed band and the focused agent with a brighter colour and bold. Under themes such as `tokyo-night`, the bright label reads as a highlight on one row of a muted column. An underline marks the same label while keeping the column one colour.

## What Changes

- The viewed band's label draws in the colour and attributes of the other band labels, plus an underline. The brighter colour and the bold are dropped.
- A band holding agents keeps its most urgent state's colour when viewed, and gains an underline. The bright variant and the bold are dropped.
- The focused agent's label, below the divider, keeps its state's colour and gains an underline. The bright variant and the bold are dropped.
- `SidebarBandActive` defaults to `{ link = "SidebarBand", underline = true }`.
- `SidebarAgentBlockedActive`, `SidebarAgentDoneActive`, `SidebarAgentWorkingActive` and `SidebarAgentIdleActive` default to `{ link = "SidebarAgent<State>", underline = true }`. `SidebarAgentUnknownActive` keeps its link to `SidebarBandActive`.
- Every bundled theme sets `SidebarBandActive` to `{ link = "SidebarBand", underline = true }`, so the underline shows under every theme.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `sidebar`: the defaults of `SidebarBandActive` and four `SidebarAgent<State>Active` groups, and the scenario that pins the focused agent in a bright bold colour.
- `colorschemes`: bundled themes give `SidebarBandActive` the underlined link to `SidebarBand`.

## Impact

- `crates/lua/src/runtime/gband/sidebar.lua`: five `gband.hl.default` calls. The drawing code already picks the `Active` group for the viewed band and the focused agent, and stays as it is.
- `crates/lua/src/runtime/gband/theme.lua` and `crates/lua/src/runtime/gband/colors/terminal.lua`: the `SidebarBandActive` line. The other bundled themes set the group through `theme.apply`.
- `examples/plugins/window/colors/dusk.lua`: the example colorscheme's `SidebarBandActive` line.
- Rust tests in `crates/lua/tests/` and `crates/client/tests/sidebar.rs` that assert bold or bright styles on the viewed band and the focused agent.
- Lua screenshot references under `tests/lua/screenshots/` and the example plugins' and tutorials' `tests/screenshots/`, regenerated with `gband test --update`.
- `docs/plugins.md`, `docs/internals/03-colorschemes.md` and `docs/internals/09-sidebar-and-errors.md`: the group tables and quoted code.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- crates/lua/src/runtime/gband/sidebar.lua
- crates/lua/src/runtime/gband/theme.lua
- crates/lua/src/runtime/gband/colors/terminal.lua
- crates/lua/tests/sidebar.rs
- crates/lua/tests/themes.rs
- crates/lua/tests/colorschemes.rs
- crates/client/tests/sidebar.rs
- tests/lua/sidebar_spec.lua
- tests/lua/agents_spec.lua
- tests/lua/screenshots/
- examples/plugins/window/colors/dusk.lua
- examples/internals/12-own-prelude/tests/screenshots/
- examples/plugins/agent-status/tests/screenshots/
- examples/plugins/hello/tests/screenshots/
- examples/plugins/window/tests/screenshots/
- examples/tutorial/00-setup/tests/screenshots/
- examples/tutorial/05-layout/tests/screenshots/
- examples/tutorial/06-colors/tests/screenshots/
- examples/tutorial/07-plugin-windows/tests/screenshots/
- examples/tutorial/08-bars/tests/screenshots/
- examples/tutorial/11-testing/tests/screenshots/
- examples/tutorial/11-testing/plugins/marks/tests/screenshots/
- docs/plugins.md
- docs/internals/03-colorschemes.md
- docs/internals/09-sidebar-and-errors.md
- openspec/changes/sidebar-underline-active-band/
