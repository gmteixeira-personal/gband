## MODIFIED Requirements

### Requirement: Configuration errors
A configuration error SHALL be reported as the file's path, a colon, the line, a colon and a message. The line SHALL be the line of a syntax error, the line where a runtime error was raised, or the line of the call or assignment that received the invalid value. A plugin error SHALL be reported the same way, preceded by the plugin's name, a colon and a space. Each process SHALL record every configuration error and plugin error of its own Lua in its log. The client SHALL also report the errors of the server's Lua that the server sends it, as the server-runtime capability defines, preceded by `server: `, and the plugin requirement errors of the plugin-bridge capability. The client SHALL keep its error list: every error it has reported since the list was last emptied, oldest first. A load with none of its own SHALL empty the list, and so SHALL clearing the errors. `gband.errors()` SHALL return a new list of the error list's texts, oldest first. The client SHALL show the latest error until the list is emptied. While the status line is drawn, the client SHALL show the status line's error item, as the status-line capability defines, and the error's text only through the error list. While no status line is drawn, the client SHALL show it as a banner on the bottom row of the ribbon area, over the ribbon and cut to the ribbon area's width. Neither SHALL change the size the client reports.

`gband.clear_errors()` SHALL clear the client's errors. It SHALL be callable wherever an action value is, and calling it while the configuration loads SHALL be an error at the line of the call. It SHALL empty the error list at once, so `gband.errors()` returns an empty list in the same callback. Once that callback returns, the client SHALL show no error until it reports another. An error raised while that callback runs, before or after the call, SHALL be reported after the clear.

Clearing SHALL change nothing else:
- It SHALL NOT reload the configuration.
- It SHALL NOT enable a status line component or a callback that an error disabled, and SHALL NOT clear a plugin's failed mark. These SHALL wait for the next load, as before.
- It SHALL NOT remove an error from any log.
- It SHALL clear only the calling client's errors. An error of the server's Lua SHALL leave only this client's error list. Other clients SHALL keep it, and the server SHALL still send its latest error to each client that attaches, as the server-runtime capability defines.

#### Scenario: Syntax error
- **WHEN** `XDG_CONFIG_HOME` is unset, the home directory is `/home/u`, line 12 of `user/init.lua` holds a syntax error, and a client attaches
- **THEN** the default configuration's status line shows `error` on row 0
- **AND** `gband.errors()` holds the error, beginning `/home/u/.config/gband/user/init.lua:12:`
- **AND** the client log records it

#### Scenario: Plugin error in the status line
- **WHEN** `XDG_DATA_HOME` is `/tmp/data`, line 3 of `/tmp/data/gband/plugins/hello/client.lua` raises `boom`, and a client attaches
- **THEN** the status line's error item shows `error`
- **AND** `gband.errors()` holds `hello: /tmp/data/gband/plugins/hello/client.lua:3: boom`

#### Scenario: Server error in the client
- **WHEN** line 2 of `user/server.lua` holds a syntax error and a client starts a server
- **THEN** the client shows an error beginning `server: ` and naming `user/server.lua:2:`
- **AND** the server log records it

#### Scenario: Plugin error on the banner
- **WHEN** `XDG_DATA_HOME` is `/tmp/data`, `user/init.lua` does not set up `gband.statusline`, line 3 of `/tmp/data/gband/plugins/hello/client.lua` raises `boom`, and a client attaches
- **THEN** the client's bottom row shows `hello: /tmp/data/gband/plugins/hello/client.lua:3: boom`

#### Scenario: Banner without a status line
- **WHEN** `user/init.lua` does not set up `gband.statusline`, line 3 of a plugin's `client.lua` raises `boom`, and a client attaches
- **THEN** the client's bottom row shows the error over the ribbon

#### Scenario: Banner cleared by a good load
- **WHEN** the client shows a configuration error and the user fixes `user/init.lua`
- **THEN** the error disappears once the configuration reloads

#### Scenario: Errors kept in order
- **WHEN** two plugins raise errors while the configuration loads, `alpha` first and then `beta`
- **THEN** `gband.errors()` returns the error of `alpha` and then the error of `beta`

#### Scenario: Cleared by hand
- **WHEN** `gband.errors()` holds two errors and a binding function calls `gband.clear_errors()` and then reads `gband.errors()`
- **THEN** the binding function reads an empty list
- **AND** once it returns, the status line shows no error item
- **AND** the client log still records both errors

#### Scenario: Banner cleared by hand
- **WHEN** `user/init.lua` does not set up `gband.statusline`, the client's bottom row shows a plugin error over a tile, and a binding function calls `gband.clear_errors()`
- **THEN** the bottom row shows the tile again

#### Scenario: Error after a clear
- **WHEN** a binding function has called `gband.clear_errors()`, and a status line component's `render` then raises `boom`
- **THEN** `gband.errors()` holds only that error
- **AND** the status line's error item shows `error`

#### Scenario: Error in the clearing callback
- **WHEN** a binding function calls `gband.clear_errors()` and then raises an error on line 9 of `user/init.lua`
- **THEN** `gband.errors()` holds only the error at `user/init.lua` line 9

#### Scenario: Clear while loading
- **WHEN** line 4 of `user/init.lua` calls `gband.clear_errors()` at the top level
- **THEN** loading fails with an error at `user/init.lua` line 4

#### Scenario: Clearing keeps a failed plugin failed
- **WHEN** the plugin `broken` registers the action `broken.go`, binds `alt+b` to it, then raises an error in `setup`, and a binding function calls `gband.clear_errors()`
- **THEN** `gband.errors()` returns an empty list and no `ConfigReloaded` event is emitted
- **AND** Alt+B still does nothing

#### Scenario: Server error cleared in one client
- **WHEN** line 2 of `user/server.lua` holds a syntax error, two clients are attached and both show an error beginning `server: `, and a binding function in the first calls `gband.clear_errors()`
- **THEN** the first client shows no error
- **AND** the second client still shows the error
- **AND** a third client that attaches then shows the error
