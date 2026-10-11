## Context

The server owns everything detection needs, and server Lua reaches none of it:

- `crates/emulator/src/lib.rs` wraps `vt100` 0.16.2. It gives the screen (`screen()`, with `rows()`), the title from OSC 0 and OSC 2 (`title()`) and `take_title_changed()`.
- `crates/server/src/window.rs` keeps each window's `Terminal { grid, master, names }` behind a mutex. `Window::process` feeds output to the grid on the window's reader thread and calls `Terminal::update_name`, which rereads the foreground command when the foreground process group changes.
- `crates/server/src/process.rs::foreground_command` reads only the first argument from `/proc/<pgid>/cmdline` and drops the rest.
- `crates/server/src/session.rs::drive` runs the session loop. It holds a 1 s `NAME_CHECK` interval that calls `refresh_names`, and it keeps `shown: HashMap<u64, HashSet<WindowId>>`, each client's shown windows.
- `Hub::set_state` in `crates/server/src/hub.rs` stores a window state key and sends `ServerMessage::WindowState` to the session's clients. Attach replays every state. The client emits `WindowStateChanged` for each later change.
- The session event bus (`crates/server/src/event.rs`) reaches the consumer in `crates/server/src/scripting.rs`, which already calls `self.hub.window_closed` for a closed window.
- Server Lua has no timer and no `Host` method for the screen, the title or the process (`crates/lua/src/server.rs`). The `Host` trait has `sessions`, `session`, `window_state`, `set_window_state` and `emit`.

The client side is Lua:

- The sidebar is the bundled plugin `crates/lua/src/runtime/gband/sidebar.lua`. `lines_for` builds one span per row, `draw` calls `gband.bar.set_lines`, and `core.on_state(draw)` redraws on a change of the key table, the bands, the viewed band, multi mode or the error.
- `on_state` does not fire for a window state change or for a window that opens inside an existing band.
- `clicked` maps a press on column 0 to a band through `position_at`.
- The presets `keystyle/modal.lua` and `keystyle/direct.lua` bind `prefix`. Neither binds `a`, `b` or a digit.
- The `agent-status` example binds `prefix a` and writes the state key `agent`.

## Goals / Non-Goals

**Goals:**
- Detection in Rust, beside the data it reads, at no cost for windows that run no agent.
- One published key per agent window, carried by the existing window state messages, so the only wire change is the client's focus report.
- Display, click and keys in Lua, built on `gband.window_state`, `gband.layout()` and the existing focus functions.
- Rules for one agent written so a second agent is one new function and one row in the identity table.

**Non-Goals:**
- Agents other than Claude Code. Codex, Gemini CLI and OpenCode need screens captured from the real programs before their rules can be written and tested.
- Rules defined in Lua or loaded from files. A rule language and its validation are a separate change.
- Notifications, sounds or toasts when an agent becomes blocked or done. The `agent-status` example shows how a plugin raises one.
- Agent hooks that report state over a socket, as herdr's integrations do. The screen rules work without installing anything into the agent.
- Agent names, titles or a wider agent panel in the sidebar. The sidebar stays 1 column wide.
- Bindings in the floating key style. Its window list already reaches windows directly.
- `j` and `k` in the new modes. Up and Down are the requested keys.

## Decisions

### Detection lives in the server, in Rust

A new module `crates/server/src/agent.rs` holds:
- `Agent`, with one variant `Claude`, and its name `claude`.
- `identify(args: &[String]) -> Option<Agent>`, from the identity table in the spec.
- `ScreenState { Blocked, Working, Idle, Unknown }` and `screen_state(agent, title, lines) -> ScreenState`, with `claude_rules` as an ordered list of checks.
- `Tracker`, one per agent window, which turns screen states into published states.

Rules compare with `str::to_ascii_lowercase` and `contains`, `starts_with` and a character range test. No `regex` dependency is added.

Alternative: four new `Host` methods (screen tail, title, foreground arguments, a timer) and detection in a bundled server Lua plugin. Rejected for this change. Every agent window would copy its bottom lines across to the `gband-lua` thread several times a second. The "Handlers never slow a session" requirement keeps that thread free for user handlers, and a slow handler would delay every state. Rules in Rust also avoid writing a regex engine's worth of Lua patterns.

### Identity from the full argument list

`process.rs` gains `foreground_args(group) -> Option<Vec<String>>`, which splits `/proc/<pgid>/cmdline` on NUL. `Terminal::update_name` already rereads on a change of the foreground group, so the same reread calls `agent::identify` and creates or drops the window's `Tracker`. The 1 s `refresh_names` tick catches an agent that starts without printing. Without `/proc`, `identify` falls back to the foreground command.

### Evaluation on a 100 ms agent tick

`Window::process` sets an `agent_dirty` flag when the window has a `Tracker` and the output changed the screen or the title. `drive` gains a 100 ms `AGENT_CHECK` interval that calls `Session::refresh_agents`. For each window whose flag is set, or whose `Tracker` waits on the 500 ms idle hold, it locks the terminal once, reads the title and the bottom 20 non-empty rows, computes the screen state, and passes it to the `Tracker`. Windows without a `Tracker` cost one flag read.

Alternative: evaluate inside `Window::process`. Rejected: it runs on every output chunk, and a working agent prints many chunks a second. The tick also gives the idle hold a clock without a timer per window.

Alternative: treat recent output as `working`, as herdr does. Rejected: the echo of typing into Claude Code's prompt would read as working. The title spinner and `esc to interrupt` mark a turn without that error.

### The tracker owns `done` and the idle hold

`Tracker` keeps `published: State`, `idle_since: Option<Instant>` and `busy: bool`, which is true once the agent was `working` or `blocked` since it was last seen. `observe(screen, now, focused) -> Option<State>` returns a state only when the published one changes:

- `Idle` while `published` is `Working` or `Blocked` starts `idle_since`, and publishes after 500 ms of unbroken `Idle`.
- An idle result is `Done` when `busy` and no client focuses the window, and `Idle` otherwise. Publishing `Idle` clears `busy`.
- Every other screen state publishes at once and clears `idle_since`.

`seen()` turns `Done` into `Idle`. The session calls it when a client's focus message names the window, and on `Command::Key`, `Command::Paste` and `Command::Mouse` for that window.

### Focus report as a new client message

`ClientMessage::Focused(Option<WindowId>)` joins `crates/protocol/src/message.rs`, and `PROTOCOL_VERSION` becomes 14. The client sends it beside `Shown`, from the same place that compares the shown set after a layout, a focus message, a view action or a peek. The connection forwards it as `Command::Focused { client, window }`. The session keeps `focused: HashMap<u64, WindowId>` next to `shown`, removes a client's entry on detach, and calls `Tracker::seen` for the window.

Alternative: treat a shown window as seen. Rejected: a wide terminal shows several windows at once, so an agent finishing beside the focused one would never read as done.

Alternative: keep `done` per client in the sidebar's Lua. Rejected: an agent that finishes while no client is attached would read as idle at the next attach, which loses the case `done` exists for.

### Publishing through the bus

`SessionEvent::AgentState { window, value: Option<Value> }` joins `crates/server/src/event.rs`. `refresh_agents` sends it when a `Tracker` publishes, and when a window's `Tracker` is dropped. The bus consumer in `scripting.rs` calls `self.hub.set_state(&session, window, "agent_state", value)` and runs no Lua event, as it does for `WindowClosed`. The value is `{ agent = "claude", state = "<state>" }` as a plain data table.

The server Lua proxy in `crates/lua/src/server.rs` refuses an assignment to `agent_state` with an error naming the key, before it reaches `set_window_state`.

### Key name `agent_state`

The `agent-status` example already writes `agent`, and four spec scenarios name `agent` as a sample key. A separate name lets both run, and leaves those scenarios true.

### Client module `gband.agents`

`crates/lua/src/runtime/gband/agents.lua` builds the list from `gband.layout()` and `gband.window_state(id).agent_state` on each call: bands in order, each column's `windows` in order, then `floating`. `focus(n)` calls `gband.window.focus`. `step(delta)` finds the focused window's index from `gband.view().window`, and wraps. The module keeps no cache, so it cannot disagree with the client's state.

### Sidebar drawing

`lines_for` appends the agent section after the band labels, with one blank row and then one span per entry. The span's group comes from a table that maps each state to its group name and `Active` name. `band_group(band)` takes the most urgent state of that band's entries by the order `blocked, done, working, idle, unknown`, and falls back to `SidebarBand` or `SidebarBandActive`. `clicked` resolves a row below the band labels to an agent position, and calls `gband.window.focus`.

`core.on_state` does not report window state changes or focus moves within a band. The sidebar also registers handlers, in its own handler group so a reload replaces them, for `WindowStateChanged` with key `agent_state`, `LayoutChanged`, `FocusChanged` and `Attached`. Each calls `draw`.

### Jump modes in their own module

`crates/lua/src/runtime/gband/jump.lua` exports `setup()`. It declares the modes `agents` and `bands`, binds the digits, Up, Down, Escape and Enter, and binds `prefix a` and `prefix b` to functions that call `gband.keymap.enter`. Both presets call `require("gband.jump").setup()` after their `i` binding, so the two presets bind the same names in the same order, which the key-style capability's "Preset parity" needs. The digit bindings close over their position, and call `gband.agents.focus(p)` or `gband.band.view(gband.layout().bands[p].id)`. A digit enters `root` only when that call focused or viewed something.

### The `agent-status` example moves to `prefix W`

`prefix a` now enters agents mode, and a plugin binding made after the preset would replace it. `W` is free in the modal and the direct presets. The example's `client.lua`, README, test case name and the snippet in `docs/plugins.md` change together.

## Risks / Trade-offs

- [Claude Code changes its screen text] → The rules follow the signals herdr's published rules use for Claude Code. A task captures the real screens of Claude Code in a gband window before the rules are final, and corrects the spec's table in this change when a capture disagrees. A Claude Code release can still break a rule until gband follows.
- [Title spinner without a turn] → herdr ranks the title spinner first, and this design follows it. A spinner shown during a permission form would read as working until the title changes.
- [Bus consumer on the Lua thread] → If `scripting.rs` hands bus events to the `gband-lua` thread before the `match`, a slow Lua handler delays agent states. The implementation checks where the match runs, and calls `set_state` before queueing Lua work when it runs on that thread.
- [Protocol 14 so soon after 13] → Every mixed pair of builds refuses at the handshake and names `gband kill-server`, as for every earlier bump.
- [Digits past 9] → Agents and bands at positions 10 to 35 show a letter in the sidebar, and only Up, Down and a click reach them. Letters in the modes would shadow future bindings, so they stay out.
