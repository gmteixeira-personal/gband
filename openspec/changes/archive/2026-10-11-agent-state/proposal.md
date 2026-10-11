## Why

A gband user runs coding agents in several windows and cannot see which agent waits for an answer, which one works and which one has finished without visiting each window. herdr solves this by detecting each agent's state from its process and its screen, and by listing the agents in its sidebar with a color for each state. The only detection gband has is the `agent-status` example plugin, which matches 4 prompt strings in the last 512 bytes a window printed and knows one state, waiting.

## What Changes

- The server detects coding agents. A window is an **agent window** while the leader of its terminal's foreground process group is a known agent. This change knows one agent, Claude Code, named `claude`.
- The server gives each agent window one of five states, decided by rules for that agent over the window's title and the bottom lines of its screen:
  - `blocked`: the agent shows a permission or choice form.
  - `working`: the agent runs a turn.
  - `done`: the agent became idle after working or blocked, and no client has focused the window since.
  - `idle`: the agent shows its prompt.
  - `unknown`: the agent runs, and no rule matches its screen.
- The server publishes the state in the window state key `agent_state`, as `{ agent = "claude", state = "working" }`. The existing window state messages carry it, and `WindowStateChanged` reports each change. Server Lua can read the key and cannot assign it.
- The client reports the window it focuses to the server in a new **focus** message, so the server can turn `done` into `idle` when a client focuses the window. The protocol version rises from 13 to 14. **BREAKING** for a mix of builds: a version 13 client and a version 14 server refuse each other at the handshake.
- A bundled client module `gband.agents` lists the agent windows of the client's layout in layout order, focuses the agent at a position, and steps to the previous or next agent.
- The sidebar splits in two halves, as herdr's does: the band labels in the top half, a divider `─` on the middle row, and the **agent section** in the bottom half, one label per agent window. A label is the agent's position, `1` to `9` then `a` to `z`, as band labels are. The label's color shows the agent's state, with no icon.
- A band label that holds agent windows takes the color of its most urgent agent: blocked, then done, working, idle and unknown. A band with no agent keeps today's groups.
- Each state has two highlight groups, such as `SidebarAgentWorking` and `SidebarAgentWorkingActive`. The brighter `Active` group draws the viewed band's label and the label of the agent this client focuses.
- A left click on an agent label focuses that agent's window, viewing its band.
- Two new key modes. In the modal and the direct key styles:
  - `a` after the prefix key enters **agents mode**, labelled `agents`. A digit `1` to `9` focuses that agent and returns to interactive mode. `k` or Up and `j` or Down focus the previous and the next agent, and the mode stays active.
  - `b` after the prefix key enters **bands mode**, labelled `bands`. A digit `1` to `9` views that band. `k` or Up and `j` or Down view the band above and the band below. Only Escape and Enter leave the mode, so the user can look through band after band. `b` in agents mode switches to bands mode, and `a` in bands mode switches back.
  - Escape and Enter leave either mode. The sidebar's mode letter shows `A` or `B`.
- The `agent-status` example moves its jump binding from `prefix a` to `prefix W`, because `prefix a` now enters agents mode.

## Capabilities

### New Capabilities
- `agent-state`: agent windows and their identification, the five states and Claude Code's screen rules, the change from `done` to `idle`, timing, the `agent_state` window state key, and the client module `gband.agents`.
- `jump-modes`: agents mode and bands mode, their key tables, digits, `j`, `k`, Up and Down, and how each leaves.

### Modified Capabilities
- `sidebar`: "Sidebar rows" splits the sidebar at the divider and adds the agent section; "Band labels" stops the bands at the divider; "Viewed band emphasis" colors band labels by agent state; "Band click" leaves agent labels to the new "Agent click"; "Sidebar groups" adds the ten state groups and `SidebarDivider`. New requirements "Agent labels", "Agent colors" and "Agent click".
- `client-attach`: "Key bindings" gains the `a` and `b` rows in the modal and direct tables. New requirement "Report the focused window".
- `wire-protocol`: "Handshake" raises the protocol version to 14; "Client messages" lists the focus message.
- `server-runtime`: "Shared window state" refuses a server Lua assignment to `agent_state`.

## Impact

- Server: a new detector in `crates/server/src/agent.rs`, the full argument list of the foreground process in `crates/server/src/process.rs`, detection after output and on a tick in `crates/server/src/window.rs` and `crates/server/src/session.rs`, each client's focused window in `crates/server/src/connection.rs`, and publishing through `Hub::set_state` in `crates/server/src/hub.rs`.
- Emulator: `crates/emulator/src/lib.rs` gives the bottom non-empty lines of the screen as text.
- Wire format: `ClientMessage::Focused` and `PROTOCOL_VERSION` 14 in `crates/protocol/src/message.rs`.
- Client: `crates/client/src/lib.rs` sends the focus message after the first layout and on each change of focus.
- Lua: the refusal of `agent_state` in `crates/lua/src/server.rs`; new bundled modules `gband.agents` and `gband.jump` registered in `crates/lua/src/bundled.rs`; the agent section, colors and click in `crates/lua/src/runtime/gband/sidebar.lua`; `a` and `b` in `keystyle/modal.lua` and `keystyle/direct.lua`; the new functions in the type definition file `crates/lua/src/runtime/meta/gband.lua`, which lua-type-definitions adds.
- Example: `examples/plugins/agent-status` binds `prefix W`, and `docs/plugins.md` quotes the new binding.
- Docs: README key table and an agents section, `docs/plugins.md` for `agent_state` and `gband.agents`, and the quoted presets in `docs/internals/08-key-styles.md`.
- Tests: detector rules over fixed screens, protocol round trip and handshake, the focus report, the Lua refusal, keystyle lists, sidebar rows and colors, and a new screen spec that runs a stand-in Claude Code process.
- No new crate dependency. The rules match text without the `regex` crate.

## Coordination

### Author
- gmteixeira

### Depends On
- lua-type-definitions

### Expected Files
- openspec/changes/agent-state/
- README.md
- crates/client/src/bindings.rs
- crates/client/src/lib.rs
- crates/client/tests/events.rs
- crates/client/tests/sidebar.rs
- crates/client/tests/snapshots/sidebar__default_sidebar_snapshot.snap
- crates/client/tests/snapshots/sidebar__error_marker_snapshot.snap
- crates/emulator/src/lib.rs
- crates/emulator/tests/contract.rs
- crates/lua/src/bundled.rs
- crates/lua/src/server.rs
- crates/lua/src/sides.rs
- crates/lua/src/runtime/gband/agents.lua
- crates/lua/src/runtime/gband/jump.lua
- crates/lua/src/runtime/gband/prelude.lua
- crates/lua/src/runtime/gband/sidebar.lua
- crates/lua/src/runtime/gband/keystyle/direct.lua
- crates/lua/src/runtime/gband/keystyle/modal.lua
- crates/lua/src/runtime/meta/gband.lua
- crates/lua/tests/agents.rs
- crates/lua/tests/config.rs
- crates/lua/tests/keystyle.rs
- crates/lua/tests/plugins.rs
- crates/lua/tests/server.rs
- crates/lua/tests/sidebar.rs
- crates/protocol/src/message.rs
- crates/protocol/tests/messages.rs
- crates/server/src/agent.rs
- crates/server/src/connection.rs
- crates/server/src/event.rs
- crates/server/src/lib.rs
- crates/server/src/process.rs
- crates/server/src/scripting.rs
- crates/server/src/session.rs
- crates/server/src/window.rs
- crates/server/tests/agent.rs
- crates/server/tests/scripting.rs
- crates/server/tests/sync.rs
- crates/server/tests/fixtures/claude/
- docs/plugins.md
- docs/internals/00-boundary.md
- docs/internals/08-key-styles.md
- docs/internals/09-sidebar-and-errors.md
- docs/internals/12-own-prelude.md
- docs/tutorial/00-setup.md
- docs/tutorial/05-layout.md
- docs/tutorial/06-colors.md
- docs/tutorial/07-plugin-windows.md
- docs/tutorial/08-bars.md
- docs/tutorial/11-testing.md
- examples/internals/12-own-prelude/tests/screenshots/own_prelude_spec/the-own-prelude-adds-the-clock--the-clock.txt
- examples/plugins/agent-status/README.md
- examples/plugins/agent-status/client.lua
- examples/plugins/agent-status/tests/agent_status_spec.lua
- examples/plugins/agent-status/tests/screenshots/agent_status_spec/a-prompt-marks-the-window-waiting.txt
- examples/tutorial/00-setup/tests/screenshots/setup_spec/
- examples/tutorial/05-layout/tests/screenshots/layout_spec/a-mark-names-the-window.txt
- examples/tutorial/06-colors/tests/screenshots/colors_spec/dusk-sets-mark-and-the-palette.txt
- examples/tutorial/07-plugin-windows/tests/screenshots/plugin_windows_spec/the-list-jumps-to-the-line-under-the-cursor--two-marks.txt
- examples/tutorial/08-bars/tests/screenshots/bars_spec/shows-the-bar--two-windows.txt
- examples/tutorial/11-testing/plugins/marks/tests/screenshots/marks_spec/
- examples/tutorial/11-testing/tests/screenshots/testing_spec/the-configuration-sets-the-plugin-up--two-windows.txt
- tests/keystyle.rs
- tests/lua_specs.rs
- tests/plugin_runtime.rs
- tests/sidebar.rs
- tests/lua/agents_spec.lua
- tests/lua/settings_spec.lua
- tests/lua/sidebar_spec.lua
- tests/lua/screenshots/agents_spec/
- tests/lua/screenshots/bundled_copies_spec/copied-sidebar--error.txt
- tests/lua/screenshots/errors_spec/
- tests/lua/screenshots/floating_spec/list-of-two-windows-its-buttons-and-a-minimized-entry--list.txt
- tests/lua/screenshots/floating_spec/list-of-two-windows-its-buttons-and-a-minimized-entry--minimized-entry.txt
- tests/lua/screenshots/floating_spec/list-of-two-windows-its-buttons-and-a-minimized-entry--tall.txt
- tests/lua/screenshots/floating_spec/windows-in-two-bands--two-bands.txt
- tests/lua/screenshots/floating_target_spec/float-twice-in-one-callback--floated.txt
- tests/lua/screenshots/keylist_spec/
- tests/lua/screenshots/keystyle_spec/the-direct-style-s-mode-letter-and-key-list-title--direct.txt
- tests/lua/screenshots/prompt_spec/
- tests/lua/screenshots/settings_spec/
- tests/lua/screenshots/sidebar_spec/
- tests/lua/screenshots/window_names_spec/
