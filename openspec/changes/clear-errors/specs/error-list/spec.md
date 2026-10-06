## MODIFIED Requirements

### Requirement: Error list plugin
gband SHALL bundle the client plugin module `gband.errors`, whose plugin name is `errors`. Its `setup` SHALL take one option, `kind`, which is `"floating"` or `"tiled"` and `"floating"` by default. Any other field, or a `kind` of another value, SHALL make `setup` raise an error naming the field. `setup` SHALL register:
- the action `errors.open`, with the description `list the errors`, which opens the error list with the setup's `kind`;
- the command `errors.open`, which takes one optional argument, `kind`, and opens the error list with that `kind`, or with the setup's `kind` when the argument is absent;
- the action `errors.clear`, with the description `clear the errors`, which clears the errors as "Clearing the errors" defines;
- the command `errors.clear`, which takes no argument and clears the errors as the action does.

A `kind` argument of another value SHALL make the command raise an error naming it.

#### Scenario: Action registered
- **WHEN** `user/init.lua` calls `gband.plugin("gband.errors")` and reads `gband.action.list()`
- **THEN** it holds `errors.open` with the description `list the errors`

#### Scenario: Clear action and command registered
- **WHEN** `user/init.lua` calls `gband.plugin("gband.errors")` and reads `gband.action.list()` and `gband.cmd.list()`
- **THEN** the actions hold `errors.clear` with the description `clear the errors`
- **AND** the commands hold `errors.clear`

#### Scenario: Tiled by option
- **WHEN** `user/init.lua` calls `gband.plugin("gband.errors", { kind = "tiled" })` and a binding dispatches `errors.open`
- **THEN** the error list opens as a tiled plugin window

#### Scenario: Kind from the command
- **WHEN** the plugin is set up with no options and a binding function calls `gband.cmd.run("errors.open", "tiled")`
- **THEN** the error list opens as a tiled plugin window

#### Scenario: Unknown option
- **WHEN** `user/init.lua` calls `gband.plugin("gband.errors", { width = 40 })`
- **THEN** `gband.plugin` returns `false` and a plugin error names `width`

### Requirement: Error list plugin window
Opening the error list SHALL open a plugin window, as the plugin-windows capability defines, that belongs to the plugin `errors` and takes focus. It SHALL also make `root` the active table, as `gband.keymap.enter("root")` does, so the keys that follow reach the plugin window.
- A floating plugin window SHALL have a border, and a width and height of three quarters of the ribbon area's width and half of its height, each rounded down and at least 1, centered in the ribbon area. Its title SHALL be `errors` while it holds no text. While it holds a text, its title SHALL be `errors`, two spaces, and `c clear`.
- A tiled plugin window SHALL be opened with kind `"tiled"` and its defaults.

The plugin window SHALL hold the texts `gband.errors()` returns when it opens, oldest first, until the errors are cleared as "Clearing the errors" defines. Its lines SHALL be the texts it holds. Each text SHALL be wrapped into lines no wider than the content area's width, breaking at the last character that fits, by display width. Two texts SHALL be separated by one empty line. When it holds no text, the plugin window SHALL hold the one line `no errors`. Each time the content area's width changes, the texts it holds SHALL be wrapped again. Opening the error list while its plugin window is open SHALL focus that plugin window and open no other.

#### Scenario: Floating plugin window of the defaults
- **WHEN** the ribbon area is 60×24, the error list holds two errors, and a binding dispatches `errors.open`
- **THEN** a focused floating plugin window of width 45 and height 12 titled `errors  c clear` shows the first error's lines, an empty line, and the second error's lines

#### Scenario: Long error wrapped
- **WHEN** the error list holds one error 100 cells wide and the floating plugin window's content area is 43 columns wide
- **THEN** the floating plugin window shows it on three lines of 43, 43 and 14 cells

#### Scenario: No errors
- **WHEN** the error list is empty and a binding dispatches `errors.open`
- **THEN** the plugin window shows `no errors`
- **AND** a floating plugin window is titled `errors`

#### Scenario: Opened from navigation mode
- **WHEN** the default configuration is in use, navigation mode is active, and a binding in it dispatches `errors.open`
- **THEN** `root` is active and the next key reaches the error list's plugin window

## ADDED Requirements

### Requirement: Clearing the errors
Clearing the errors through the plugin SHALL call `gband.clear_errors()`, as the configuration capability defines. When the error list's plugin window is open, it SHALL then hold no text, so it shows `no errors`, and a floating one SHALL take the title `errors`. The plugin window SHALL stay open, and SHALL keep its focus when it has it.

In either kind, `c` in the error list's plugin window SHALL clear the errors, as a `keys` entry of the plugin window does. A binding SHALL still win over `c`, as the plugin-windows capability defines, so Ctrl+Space then `c` SHALL NOT clear the errors.

#### Scenario: Clear with c
- **WHEN** the error list is open as a floating plugin window holding two errors, the status line shows its error item, and the user presses `c`
- **THEN** the plugin window shows `no errors` and is titled `errors`, and it stays focused
- **AND** `gband.errors()` returns an empty list, and the status line no longer shows `error`

#### Scenario: Clear a tiled error list
- **WHEN** the error list is open as a tiled plugin window holding one error and the user presses `c`
- **THEN** the plugin window shows `no errors` and stays open and focused

#### Scenario: Cleared list wrapped again
- **WHEN** the user has cleared the errors with `c` in a floating error list, and the ribbon area's width then changes
- **THEN** the plugin window still shows `no errors`

#### Scenario: Clear command with the list open
- **WHEN** the error list is open holding one error and a binding function calls `gband.cmd.run("errors.clear")`
- **THEN** `gband.cmd.run` returns `true`
- **AND** the error list shows `no errors`

#### Scenario: Clear action without the list
- **WHEN** `gband.errors()` holds one error, no error list is open, and a binding dispatches `errors.clear`
- **THEN** `gband.errors()` returns an empty list and no plugin window opens

#### Scenario: Prefix c is not the list's c
- **WHEN** the default configuration is in use, the error list is focused and holds one error, and the user presses Ctrl+Space then `c`
- **THEN** the error list still holds the error and `gband.errors()` still returns it
