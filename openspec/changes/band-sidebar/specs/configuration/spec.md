## MODIFIED Requirements

### Requirement: Configuration file
The client configuration file SHALL be `user/init.lua` in the configuration directory. Each client SHALL load its configuration when it starts, after preparing the configuration directory. Loading SHALL use one new Lua state, starting from the declared defaults of the client options and no key bindings, and SHALL evaluate the init file and then the `client.lua` files the plugins capability sources from the runtimepath. The init file SHALL be the configuration file when it exists, and the default client configuration otherwise. Before the init file, loading SHALL install only the client's `gband` API and the bundled `default` colorscheme, as the colorschemes capability defines. Nothing of the default configuration SHALL be evaluated before the init file: the configuration file replaces the default configuration rather than adding to it. A missing configuration file SHALL NOT be an error. The client SHALL use the prefix, the key tables, the camera policy, the notification style, the highlight groups, the bars and the callbacks. The server SHALL NOT evaluate the client configuration; it loads the server configuration as the server-runtime capability defines.

#### Scenario: No configuration file
- **WHEN** no `user/init.lua` exists, no plugin directory exists, and a client attaches
- **THEN** no error is reported
- **AND** the key bindings, options and behaviour are those of the default configuration

#### Scenario: XDG_CONFIG_HOME is honoured
- **WHEN** `XDG_CONFIG_HOME` is `/tmp/cfg`, `/tmp/cfg/gband/user/server.lua` sets `width_presets` to `{ 1/4, 1/2 }`, `/tmp/cfg/gband/user/init.lua` binds `prefix r` to `gband.action.cycle_column_width`, and a client starts a server and cycles a column of width 1/2
- **THEN** the column's width is 1/4

#### Scenario: User file replaces the defaults
- **WHEN** `user/init.lua` holds only `gband.bind("alt+h", gband.action.focus_column_left)` and a client attaches
- **THEN** Alt+H focuses the column to the left
- **AND** Ctrl+Space then `q` reaches the focused window as `\x00` and then `q`

#### Scenario: Plugin files see the init file's options
- **WHEN** `user/init.lua` sets `gband.opt.prefix` to `"ctrl+b"` and a plugin's `client.lua` reads `gband.opt.prefix`
- **THEN** `client.lua` reads `"ctrl+b"`

#### Scenario: Plugin sets a width option
- **WHEN** a plugin's `server.lua` sets `gband.opt.default_column_width` to `1/3`, and a client starts a server and opens a window
- **THEN** the new window's column has width 1/3

### Requirement: Defaults use the public API
The default configuration SHALL use only the `gband` API that the configuration file can use. It SHALL set the options to their declared defaults. It SHALL make its key bindings only by calling `gband.keystyle.use()` with no argument, as the key-style capability defines, so the saved key style, or the modal key style when none is saved, makes them. It SHALL make no other binding and declare no mode. It SHALL set up the key list plugin `gband.keylist` and the Lua prompt plugin `gband.prompt` only through that call. It SHALL then set up, with `gband.plugin` and no options, the bundled error list plugin `gband.errors`, then the bundled sidebar plugin `gband.sidebar`, in that order, and register the offer of the key-style capability's "Offer on the first start". Evaluated alone, with no configuration directory, it SHALL produce the declared defaults of the options and the default key bindings of the client-attach capability for the modal key style.

The default key bindings and the declaration of the `navigation` mode SHALL exist only in the key style presets. The setup of the error list and sidebar plugins and the offer of the chooser SHALL exist only in the default configuration.

#### Scenario: Defaults reproduce the built-in behaviour
- **WHEN** the default configuration is evaluated alone
- **THEN** the options equal the defaults in the "Options" table
- **AND** the bindings equal the client-attach capability's default table for the modal key style, entry for entry
- **AND** `gband.bar.list()` holds exactly one bar, `sidebar`, on the side `left`, 1 column wide on an 80×24 terminal

#### Scenario: Every default binding is described
- **WHEN** the default configuration is evaluated with either key style saved and `gband.keymap.list("prefix")` is read
- **THEN** every entry with an action has the description `gband.action.list()` gives its action
- **AND** every entry without an action has the description the client-attach capability's default table gives its key

#### Scenario: Navigation mode declared by the defaults
- **WHEN** the default configuration is evaluated alone and `gband.keymap.label("prefix")` is read
- **THEN** it returns `navigation`

#### Scenario: Direct style from the saved choice
- **WHEN** `user/keystyle.lua` holds `return "direct"`, no `user/init.lua` exists, and a client attaches
- **THEN** `gband.keymap.label("prefix")` returns `prefix`
- **AND** the bindings equal the client-attach capability's bindings for the direct key style, entry for entry

#### Scenario: Key list set up by the defaults
- **WHEN** the default configuration is evaluated alone and `gband.keymap.list("prefix")` is read
- **THEN** the entry for `?` has the action `keylist.open`
- **AND** it comes after the entry for `R` and before the entry for `D`

#### Scenario: Prompt set up by the defaults
- **WHEN** the default configuration is evaluated alone and `gband.keymap.list("prefix")` is read
- **THEN** the entry for `:` has the action `prompt.open` and the description `run Lua`
- **AND** it comes right after the entry for `?` and before the entry for `D`
- **AND** `gband.keymap.list("root")` holds no entry

#### Scenario: No binding in the defaults file
- **WHEN** `defaults/init.lua` is read
- **THEN** it calls `gband.keystyle.use()` once
- **AND** it calls none of `gband.keymap.set`, `gband.bind` and `gband.keymap.mode`

#### Scenario: Copied defaults load unchanged
- **WHEN** `user/init.lua` is a copy of `defaults/init.lua`
- **THEN** loading succeeds
- **AND** the options, bindings and modes equal those of the default configuration with the same saved key style

### Requirement: Configuration errors
A configuration error SHALL be reported as the file's path, a colon, the line, a colon and a message. The line SHALL be the line of a syntax error, the line where a runtime error was raised, or the line of the call or assignment that received the invalid value. A plugin error SHALL be reported the same way, preceded by the plugin's name, a colon and a space. Each process SHALL record every configuration error and plugin error of its own Lua in its log. The client SHALL also report the errors of the server's Lua that the server sends it, as the server-runtime capability defines, preceded by `server: `, and the plugin requirement errors of the plugin-bridge capability. The client SHALL keep its error list: every error it has reported since the list was last emptied, oldest first. A load with none of its own SHALL empty the list, and so SHALL clearing the errors. `gband.errors()` SHALL return a new list of the error list's texts, oldest first. The client SHALL show the latest error until the list is emptied. While the sidebar's error marker is drawn, as the sidebar capability defines, the client SHALL show the error's text only through the error list. While no error marker is drawn, the client SHALL show it as a banner on the bottom row of the ribbon area, over the ribbon and cut to the ribbon area's width. Neither SHALL change the size the client reports.

`gband.clear_errors()` SHALL clear the client's errors. It SHALL be callable wherever an action value is, and calling it while the configuration loads SHALL be an error at the line of the call. It SHALL empty the error list at once, so `gband.errors()` returns an empty list in the same callback. Once that callback returns, the client SHALL show no error until it reports another. An error raised while that callback runs, before or after the call, SHALL be reported after the clear.

Clearing SHALL change nothing else:
- It SHALL NOT reload the configuration.
- It SHALL NOT enable a callback that an error disabled, and SHALL NOT clear a plugin's failed mark. These SHALL wait for the next load, as before.
- It SHALL NOT remove an error from any log.
- It SHALL clear only the calling client's errors. An error of the server's Lua SHALL leave only this client's error list. Other clients SHALL keep it, and the server SHALL still send its latest error to each client that attaches, as the server-runtime capability defines.

#### Scenario: Syntax error
- **WHEN** `XDG_CONFIG_HOME` is unset, the home directory is `/home/u`, line 12 of `user/init.lua` holds a syntax error, and a client attaches
- **THEN** the default configuration's sidebar shows `!` on its bottom row
- **AND** `gband.errors()` holds the error, beginning `/home/u/.config/gband/user/init.lua:12:`
- **AND** the client log records it

#### Scenario: Plugin error in the status line
- **WHEN** `XDG_DATA_HOME` is `/tmp/data`, line 3 of `/tmp/data/gband/plugins/hello/client.lua` raises `boom`, and a client attaches
- **THEN** the sidebar shows `!` on its bottom row
- **AND** `gband.errors()` holds `hello: /tmp/data/gband/plugins/hello/client.lua:3: boom`

#### Scenario: Server error in the client
- **WHEN** line 2 of `user/server.lua` holds a syntax error and a client starts a server
- **THEN** the client shows an error beginning `server: ` and naming `user/server.lua:2:`
- **AND** the server log records it

#### Scenario: Plugin error on the banner
- **WHEN** `XDG_DATA_HOME` is `/tmp/data`, `user/init.lua` does not set up `gband.sidebar`, line 3 of `/tmp/data/gband/plugins/hello/client.lua` raises `boom`, and a client attaches
- **THEN** the client's bottom row shows `hello: /tmp/data/gband/plugins/hello/client.lua:3: boom`

#### Scenario: Banner without a status line
- **WHEN** `user/init.lua` does not set up `gband.sidebar`, line 3 of a plugin's `client.lua` raises `boom`, and a client attaches
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
- **AND** once it returns, the sidebar shows no error marker
- **AND** the client log still records both errors

#### Scenario: Banner cleared by hand
- **WHEN** `user/init.lua` does not set up `gband.sidebar`, the client's bottom row shows a plugin error over a tile, and a binding function calls `gband.clear_errors()`
- **THEN** the bottom row shows the tile again

#### Scenario: Error after a clear
- **WHEN** a binding function has called `gband.clear_errors()`, and another binding function then raises `boom`
- **THEN** `gband.errors()` holds only that error
- **AND** the sidebar shows `!` on its bottom row

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
