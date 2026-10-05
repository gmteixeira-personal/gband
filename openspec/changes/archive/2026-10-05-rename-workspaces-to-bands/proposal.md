## Why

gband calls the vertically stacked groups of strips "workspaces", a term borrowed from niri. gband's own term for them is "band". Today the old word appears in the specs, the README, the Lua action names and the Rust types, and Cargo's own "workspace" sits beside it in the same tree. Renaming now, before any release, means no user configuration needs a migration.

## What Changes

- The specs call every workspace a band. Three requirements take a new name: "Workspace switch" becomes "Band switch", "Dynamic workspaces" becomes "Dynamic bands", and "Switch workspace" becomes "Switch band".
- **BREAKING** (unreleased): the Lua actions `focus_workspace_down` and `focus_workspace_up` become `focus_band_down` and `focus_band_up`. The old names are not kept as aliases. The default configuration binds the new names.
- The layout events "workspace added" and "workspace removed" become "band added" and "band removed".
- The Rust types, fields, functions and tests follow the term: `Workspace` becomes `Band`, `WorkspaceId` becomes `BandId`, `ViewAction::WorkspaceDown` becomes `ViewAction::BandDown`, and so on.
- The client's existing drawing type `Band`, one workspace's frame during a switch animation, becomes `DrawnBand`, so the domain term is free.
- The README, `summary.txt`, the OpenSpec project context in `openspec/config.yaml` and the Purpose sections of the animations, layout and layout-view specs use "band".
- Cargo's `[workspace]`, `*.workspace = true` and `cargo ... --workspace` keep their names. They are Cargo's term, not gband's.
- The wire encoding does not change. Postcard encodes fields and variants by position, so the renamed identifiers produce the same bytes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `actions`: the view action examples and the Lua action names name bands.
- `animations`: the switch animation is the band switch, and the region each band is drawn in is no longer called a band.
- `client-attach`: the ribbon, the key binding table and the shown-panes report name bands.
- `layout`: the layout is a list of bands, and dynamic bands replace dynamic workspaces.
- `layout-view`: the view shows one band and keeps a camera per band.
- `session-events`: the layout events name bands, including band added and band removed.
- `session-server`: session actions and pane resizes name bands.
- `wire-protocol`: client and server messages carry band identifiers.

## Impact

- `crates/core`: `layout.rs`, `view.rs`, `event.rs`, `geometry.rs` and their tests.
- `crates/client`: `animation.rs`, `render.rs`, `bindings.rs`, `lib.rs` and their tests.
- `crates/lua`: the action name table, the default configuration and its tests.
- `crates/server`: `session.rs` and its tests.
- `crates/protocol/tests`, `crates/test-support`, `tests/attach.rs`.
- `README.md`, `summary.txt`, `openspec/config.yaml`, and the Purpose sections of three main specs.
- A user configuration written against the unreleased lua-config names `focus_workspace_*` fails to load with an error. No release has shipped those names.
- The change builds on config-directory and ctrl-space-prefix, which rewrite the default configuration and two of the requirements this change modifies.

## Coordination

### Author
- gmteixeira

### Depends On
- lua-config
- config-directory
- ctrl-space-prefix

### Expected Files
- README.md
- crates/client/src/animation.rs
- crates/client/src/bindings.rs
- crates/client/src/lib.rs
- crates/client/src/render.rs
- crates/client/tests/actions.rs
- crates/client/tests/animation.rs
- crates/client/tests/render.rs
- crates/client/tests/snapshots/render__band_switch_mid_slide.snap
- crates/client/tests/snapshots/render__empty_band_is_blank_without_a_cursor.snap
- crates/core/src/event.rs
- crates/core/src/geometry.rs
- crates/core/src/layout.rs
- crates/core/src/view.rs
- crates/core/tests/events.rs
- crates/core/tests/geometry.rs
- crates/core/tests/layout.rs
- crates/core/tests/view.rs
- crates/lua/src/api.rs
- crates/lua/src/defaults.lua
- crates/lua/tests/config.rs
- crates/protocol/tests/messages.rs
- crates/server/src/session.rs
- crates/server/tests/events.rs
- crates/server/tests/panes.rs
- crates/server/tests/programs.rs
- crates/server/tests/resize.rs
- crates/test-support/src/lib.rs
- openspec/config.yaml
- openspec/specs/actions/spec.md
- openspec/specs/animations/spec.md
- openspec/specs/client-attach/spec.md
- openspec/specs/layout-view/spec.md
- openspec/specs/layout/spec.md
- openspec/specs/session-events/spec.md
- summary.txt
- tests/attach.rs
- openspec/changes/rename-workspaces-to-bands/
