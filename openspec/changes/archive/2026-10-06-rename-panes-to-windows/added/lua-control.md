### Requirement: Removed pane names
The Lua names that gband 0.1.0 gave windows SHALL no longer work, in the client or the server. Using one SHALL be an error at the line of the use, whose message names the replacement, such as "`gband.pane` is now `gband.window`". The uses that SHALL raise it are:

- reading `gband.pane` or `gband.pane_state`
- reading `gband.action.open_pane`, `close_pane`, `focus_pane_down`, `focus_pane_up`, `move_pane_down`, `move_pane_up`, `toggle_pane_floating`, `grow_pane_height`, `shrink_pane_height` or `reset_pane_height`, which are now `open_window`, `close_window`, `focus_window_down`, `focus_window_up`, `move_window_down`, `move_window_up`, `toggle_window_floating`, `grow_window_height`, `shrink_window_height` and `reset_window_height`
- naming `PaneOpened`, `PaneClosed`, `PaneExited`, `PaneOutput`, `PaneInput` or `PaneStateChanged` to `gband.on` or in a status line component's `redraw_on`, which are now `WindowOpened`, `WindowClosed`, `WindowExited`, `WindowOutput`, `WindowInput` and `WindowStateChanged`
- an action target holding the field `pane`, which is now `window`
- `gband.win.open` with `kind = "pane"` or `kind = "float"`, which are now `kind = "tiled"` and `kind = "floating"`
- naming an old action in the hints segment's `labels`

Fields that gband fills in, such as `gband.view().pane`, the layout's `panes` and an event payload's `pane`, SHALL be absent, so reading one gives nil. The environment variable `GBAND_PANE` SHALL no longer be set; `GBAND_WINDOW` SHALL be set in its place.

#### Scenario: Old action name
- **WHEN** line 3 of `user/init.lua` calls `gband.bind("prefix x", gband.action.close_pane)`
- **THEN** the configuration fails to load with an error at line 3 whose message names `close_window`

#### Scenario: Old event name
- **WHEN** a plugin's `client.lua` calls `gband.on("PaneOpened", fn)` on line 5
- **THEN** the plugin fails with an error at line 5 whose message names `WindowOpened`

#### Scenario: Old target field
- **WHEN** a binding function calls `gband.action.close_window({ pane = 1 })`
- **THEN** the call raises an error whose message names the field `window`, and nothing is dispatched

#### Scenario: Old plugin window kind
- **WHEN** a binding function calls `gband.win.open({ kind = "pane" })`
- **THEN** the call raises an error whose message names `"tiled"`, and no plugin window opens

#### Scenario: Old floating kind
- **WHEN** a binding function calls `gband.win.open({ kind = "float" })`
- **THEN** the call raises an error whose message names `"floating"`, and no plugin window opens

#### Scenario: Old environment variable
- **WHEN** a window runs `echo "[$GBAND_PANE] $GBAND_WINDOW"` as window 2
- **THEN** it prints `[] 2`
