## MODIFIED Requirements

### Requirement: Configuration file
The client configuration file SHALL be `user/init.lua` in the configuration directory. Each client SHALL load its configuration when it starts, after preparing the configuration directory. Loading SHALL use one new Lua state, starting from the declared defaults of the client options and no key bindings, and SHALL evaluate the init file and then the `client.lua` files the plugins capability sources from the runtimepath. The init file SHALL be the configuration file when it exists, and the default client configuration otherwise. Before the init file, loading SHALL install only the client's `gband` API and load the start theme: the saved theme, or the bundled `gruvbox` theme, as the colorschemes capability defines. Nothing of the default configuration SHALL be evaluated before the init file: the configuration file replaces the default configuration rather than adding to it. A missing configuration file SHALL NOT be an error. The client SHALL use the prefix, the key tables, the camera policy, the notification style, the highlight groups, the bars and the callbacks. The server SHALL NOT evaluate the client configuration; it loads the server configuration as the server-runtime capability defines.

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
The default configuration SHALL use only the `gband` API that the configuration file can use. It SHALL set the options to their declared defaults. It SHALL make its key bindings only by calling `gband.keystyle.use()` with no argument, as the key-style capability defines, so the saved key style, or the modal key style when none is saved, makes them. It SHALL make no other binding and declare no mode. It SHALL set up the key list plugin `gband.keylist` and the Lua prompt plugin `gband.prompt` only through that call. It SHALL then set up, with `gband.plugin` and no options, the bundled error list plugin `gband.errors`, then the bundled sidebar plugin `gband.sidebar` unless `gband.settings.sidebar()` returns `false`, in that order, and register the offer of the settings capability's "Offer on the first start". Evaluated alone, with no configuration directory, it SHALL produce the declared defaults of the options and the default key bindings of the client-attach capability for the modal key style.

The default key bindings and the declaration of the `navigation` mode SHALL exist only in the key style presets. The setup of the error list and sidebar plugins and the offer of the settings window SHALL exist only in the default configuration.

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

#### Scenario: Settings bound by the defaults
- **WHEN** the default configuration is evaluated alone and `gband.keymap.list("prefix")` is read
- **THEN** the entry for `s` has the description `settings`
- **AND** it comes right after the entry for `:` and before the entry for `D`

#### Scenario: Sidebar left out by the saved setting
- **WHEN** `user/sidebar.lua` holds `return false`, no `user/init.lua` exists, and a client attaches
- **THEN** `gband.bar.list()` holds no bar
- **AND** an error is shown as the banner on the ribbon area's bottom row

#### Scenario: Saved theme with a user file
- **WHEN** `user/theme.lua` holds `return "nord"` and `user/init.lua` binds only `alt+h`
- **THEN** `gband.colorscheme()` returns `nord` once the file has loaded

#### Scenario: No binding in the defaults file
- **WHEN** `defaults/init.lua` is read
- **THEN** it calls `gband.keystyle.use()` once
- **AND** it calls none of `gband.keymap.set`, `gband.bind` and `gband.keymap.mode`

#### Scenario: Copied defaults load unchanged
- **WHEN** `user/init.lua` is a copy of `defaults/init.lua`
- **THEN** loading succeeds
- **AND** the options, bindings and modes equal those of the default configuration with the same saved key style
