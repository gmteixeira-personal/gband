## MODIFIED Requirements

### Requirement: Read the view
`gband.view()` SHALL return a new table holding:
- `band`: the viewed band's number.
- `window`: the focused window's number, or nil when no window is focused.
- `floating`: `true` when the floating layer of the viewed band is active, and `false` otherwise.
- `plugin_window`: the focused plugin window's number, as the plugin-windows capability defines it, or nil.
- `peek`: `true` while the focused window is a window this client peeks, as the layout-view capability's "Peek a window" defines, and absent otherwise.
- `multi`: `true` while this client's multi mode is on, as the multi-mode capability defines, and absent otherwise.
- `table`: the active key table.
- `cols` and `rows`: the size of the ribbon area.

Calling it while the configuration loads SHALL be an error at the line of the call.

#### Scenario: Focused window
- **WHEN** the client views band 1 with window 2 focused, its ribbon area is 80×23, and a binding in `prefix` calls `gband.view()`
- **THEN** the result is `{ band = 1, window = 2, floating = false, table = "prefix", cols = 80, rows = 23 }`

#### Scenario: Empty band
- **WHEN** the client views the empty band and a binding function calls `gband.view()`
- **THEN** the result's `window` is nil

#### Scenario: Floating focus
- **WHEN** the client focuses floating window 3 and a binding function calls `gband.view()`
- **THEN** the result's `window` is 3 and its `floating` is `true`

#### Scenario: Peeked focus
- **WHEN** band 1 holds window 1 in a column and floating window 3, window 1 is focused, a binding function calls `gband.window.focus(3, { peek = true })`, and a later binding function calls `gband.view()`
- **THEN** the result's `window` is 3, its `floating` is `true` and its `peek` is `true`
- **AND** after a binding function calls `gband.window.focus(1)`, `gband.view()` holds `window` 1 and no `peek`


#### Scenario: Multi mode on
- **WHEN** a binding function dispatches `gband.action.toggle_multi()`, and a later binding function calls `gband.view()`
- **THEN** the result's `multi` is `true`
- **AND** after a binding function dispatches `gband.action.toggle_multi()` again, `gband.view()` holds no `multi`
