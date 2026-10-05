## Purpose

Defines the bundled error list: a client plugin whose action and command open a window listing the configuration and plugin errors the client has reported, as a float by default or as a plugin pane.

## ADDED Requirements

### Requirement: Error list plugin
gband SHALL bundle the client plugin module `gband.errors`, whose plugin name is `errors`. Its `setup` SHALL take one option, `kind`, which is `"float"` or `"pane"` and `"float"` by default. Any other field, or a `kind` of another value, SHALL make `setup` raise an error naming the field. `setup` SHALL register:
- the action `errors.open`, with the description `list the errors`, which opens the error list with the setup's `kind`;
- the command `errors.open`, which takes one optional argument, `kind`, and opens the error list with that `kind`, or with the setup's `kind` when the argument is absent.

A `kind` argument of another value SHALL make the command raise an error naming it.

#### Scenario: Action registered
- **WHEN** `user/init.lua` calls `gband.plugin("gband.errors")` and reads `gband.action.list()`
- **THEN** it holds `errors.open` with the description `list the errors`

#### Scenario: Pane by option
- **WHEN** `user/init.lua` calls `gband.plugin("gband.errors", { kind = "pane" })` and a binding dispatches `errors.open`
- **THEN** the error list opens as a plugin pane

#### Scenario: Kind from the command
- **WHEN** the plugin is set up with no options and a binding function calls `gband.cmd.run("errors.open", "pane")`
- **THEN** the error list opens as a plugin pane

#### Scenario: Unknown option
- **WHEN** `user/init.lua` calls `gband.plugin("gband.errors", { width = 40 })`
- **THEN** `gband.plugin` returns `false` and a plugin error names `width`

### Requirement: Error list window
Opening the error list SHALL open a window, as the plugin-windows capability defines, that belongs to the plugin `errors` and takes focus. It SHALL also make `root` the active table, as `gband.keymap.enter("root")` does, so the keys that follow reach the window.
- A float SHALL have a border, the title `errors`, and a width and height of three quarters of the ribbon area's width and half of its height, each rounded down and at least 1, centered in the ribbon area.
- A pane SHALL be opened as a window of kind `"pane"` with its defaults.

The window's lines SHALL be the texts `gband.errors()` returns when it opens, oldest first. Each text SHALL be wrapped into lines no wider than the content area's width, breaking at the last character that fits, by display width. Two texts SHALL be separated by one empty line. When the list is empty, the window SHALL hold the one line `no errors`. Each time the content area's width changes, the texts SHALL be wrapped again. Opening the error list while its window is open SHALL focus that window and open no other.

#### Scenario: Float of the defaults
- **WHEN** the ribbon area is 60×24, the error list holds two errors, and a binding dispatches `errors.open`
- **THEN** a focused float of width 45 and height 12 titled `errors` shows the first error's lines, an empty line, and the second error's lines

#### Scenario: Long error wrapped
- **WHEN** the error list holds one error 100 cells wide and the float's content area is 43 columns wide
- **THEN** the float shows it on three lines of 43, 43 and 14 cells

#### Scenario: No errors
- **WHEN** the error list is empty and a binding dispatches `errors.open`
- **THEN** the window shows `no errors`

#### Scenario: Opened from navigation mode
- **WHEN** the default configuration is in use, navigation mode is active, and a binding in it dispatches `errors.open`
- **THEN** `root` is active and the next key reaches the error list's window

### Requirement: Closing the error list
In either kind, `q` SHALL close the window, as `gband.win.close` does. In a float, Escape SHALL close it, as the plugin-windows capability's default for Escape does.

#### Scenario: Close with q
- **WHEN** the error list is open as a plugin pane and the user presses `q`
- **THEN** the window closes and its plugin pane leaves the layout
