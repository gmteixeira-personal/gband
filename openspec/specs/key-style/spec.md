# key-style Specification

## Purpose

Defines gband's two key styles, modal and direct: the bundled preset files that make each style's bindings, how a configuration uses one, where the user's choice is saved, the chooser that saves it, and the offer of that chooser on the first start.

## Requirements

### Requirement: Key style presets
gband SHALL bundle two key style presets, the modules `gband.keystyle.modal` and `gband.keystyle.direct`, which `require` finds as the plugins capability defines for bundled modules. Each preset SHALL be a file of top-level calls that uses only the `gband` API a configuration file can use. Requiring a preset while the configuration loads SHALL make its bindings in that configuration. A preset SHALL set no option and SHALL bind nothing in `root`.

Before its bindings, each preset SHALL set up, with `gband.plugin` and no options, each bundled plugin that registers an action it binds: the key list plugin `gband.keylist`, then the Lua prompt plugin `gband.prompt`. It SHALL set up no other plugin, so the error list plugin `gband.errors`, whose action no preset binds, the status line plugin `gband.statusline` and its segment plugins stay with the default configuration, as the configuration capability defines. A binding to an action SHALL take the description of the action it binds, as the actions capability lists them or as `gband.action.list()` gives it for a registered action. A binding to a Lua function SHALL take the description that the client-attach capability's default table gives its key.

Each action that a preset binds to `h`, `j`, `k` or `l`, alone or with Ctrl, SHALL also be bound to Left, Down, Up or Right, with the same modifier.

#### Scenario: Preset required by name
- **WHEN** `user/init.lua` holds only `require("gband.keystyle.direct")`, and the user presses Ctrl+Space then `h` with the second of two columns focused
- **THEN** loading succeeds and the first column is focused

#### Scenario: Preset shadowed by a user module
- **WHEN** `user/lua/gband/keystyle/direct.lua` binds only `prefix x`, and `user/init.lua` calls `gband.keystyle.use("direct")`
- **THEN** `gband.keymap.list("prefix")` holds only the entry for `x`

#### Scenario: Arrow keys beside hjkl
- **WHEN** either preset is in use and `gband.keymap.list("prefix")` is read
- **THEN** `left` binds the action `h` binds, and `ctrl+up` binds the action `ctrl+k` binds

#### Scenario: Bundled plugins set up by the preset
- **WHEN** `user/init.lua` calls only `gband.keystyle.use("modal")`, and the user presses Ctrl+Space then `?`
- **THEN** loading succeeds and the key list opens
- **AND** `gband.action.list()` holds `keylist.open` and `prompt.open`, and no `errors.open`
- **AND** no status line is drawn

### Requirement: Modal preset
`gband.keystyle.modal` SHALL declare `prefix` a mode with the label `navigation`, with `gband.keymap.mode`. It SHALL make the bindings of the modal key style that the client-attach capability's default table gives, in that order.

#### Scenario: Navigation mode
- **WHEN** `user/init.lua` calls `gband.keystyle.use("modal")`, the first of three columns is focused, and the user presses Ctrl+Space, `l`, then `l`
- **THEN** the third column is focused and no window receives a key
- **AND** `gband.keymap.label("prefix")` returns `navigation`

#### Scenario: Escape returns to typing
- **WHEN** the modal preset is in use and the user presses Ctrl+Space, `h`, Escape, then types `echo left` and Enter, with the second of two columns focused
- **THEN** `left` appears in the first tile only

### Requirement: Direct preset
`gband.keystyle.direct` SHALL declare no mode. It SHALL make the bindings of the direct key style that the client-attach capability defines, in that order. `prefix` is then not a mode, so each key after the prefix key SHALL end the key sequence, and the keys that follow SHALL go to the focused plugin window or window.

#### Scenario: One key per action
- **WHEN** the direct preset is in use, the second of two columns is focused, and the user presses Ctrl+Space, `h`, then types `echo left` and Enter
- **THEN** the first column is focused and `left` appears in the first tile only

#### Scenario: A second key needs the prefix again
- **WHEN** the direct preset is in use, the first of three columns is focused, and the user presses Ctrl+Space, `l`, then `l`
- **THEN** the second column is focused
- **AND** the focused window receives the second `l`

#### Scenario: Open a window
- **WHEN** the direct preset is in use, one window is focused, and the user presses Ctrl+Space then `n`
- **THEN** a second window opens right of the first and is focused
- **AND** `root` is active, so `echo $GBAND_WINDOW` runs in the new window

#### Scenario: Escape after the prefix
- **WHEN** the direct preset is in use and the user presses Ctrl+Space then Escape
- **THEN** the focused window receives nothing and `root` is active

#### Scenario: Hints and labels of the direct style
- **WHEN** the default configuration is in use with the direct key style saved and `root` is active
- **THEN** the hints segment shows `C-space prefix`
- **AND** after Ctrl+Space then `?`, the key list's title is `prefix keys`

### Requirement: Use a key style
`gband.keystyle.use(style)` SHALL make the bindings of one preset by requiring the module `gband.keystyle.<style>`, and SHALL return the style's name. `style` SHALL be `"modal"`, `"direct"` or nil. With nil, it SHALL use the saved key style, as "Saved key style" defines, or `"modal"` when none is saved. A configuration MAY bind and unbind keys after the call, as with any other binding.

`gband.keystyle.use` SHALL be callable only while the configuration loads, and only once in one load. Any other `style`, a second call in the same load, and a call after loading SHALL be an error at the line of the call.

#### Scenario: Explicit style
- **WHEN** `user/init.lua` calls `gband.keystyle.use("direct")`
- **THEN** the call returns `direct`
- **AND** `gband.keymap.label("prefix")` returns `prefix`

#### Scenario: Saved style
- **WHEN** `user/keystyle.lua` holds `return "direct"` and `user/init.lua` calls `gband.keystyle.use()`
- **THEN** the call returns `direct` and the bindings are those of the direct preset

#### Scenario: Nothing saved
- **WHEN** no `user/keystyle.lua` exists and `user/init.lua` calls `gband.keystyle.use()`
- **THEN** the call returns `modal` and `prefix` is the mode `navigation`

#### Scenario: Unknown style
- **WHEN** line 3 of `user/init.lua` calls `gband.keystyle.use("vi")`
- **THEN** loading fails with an error at `user/init.lua` line 3 naming `vi`

#### Scenario: Used twice
- **WHEN** line 2 of `user/init.lua` calls `gband.keystyle.use()` and line 4 calls `gband.keystyle.use("direct")`
- **THEN** loading fails with an error at `user/init.lua` line 4

#### Scenario: Used after the load
- **WHEN** a binding function calls `gband.keystyle.use("modal")` on line 6 of `user/init.lua`
- **THEN** the client shows an error at `user/init.lua` line 6

#### Scenario: Change a preset binding
- **WHEN** `user/init.lua` calls `gband.keystyle.use()` and then binds `prefix q` to `gband.action.detach`
- **THEN** Ctrl+Space then `q` detaches, and no binding closes the window

### Requirement: Saved key style
The saved key style SHALL live in the file `user/keystyle.lua` of the configuration directory. Saving a style SHALL write the text `return "<style>"` followed by a newline to that file. It SHALL create the file or replace it in one step, so no process reads it half written. Saving SHALL change no other file in `user`, and SHALL never write `user/init.lua`.

`gband.keystyle.saved()` SHALL evaluate `user/keystyle.lua` as a Lua chunk in text mode with an empty environment, and SHALL return the chunk's value when it is `"modal"` or `"direct"`. It SHALL return nil when `gband.config_dir` is nil, and when the file is missing, cannot be read, does not compile, raises an error or returns any other value. It SHALL raise no error and report none. It SHALL be callable while the configuration loads and in any callback.

Loading SHALL evaluate `user/keystyle.lua` only through `gband.keystyle.saved()`, never as a configuration file, a plugin file or a module. Saving it SHALL reload the configuration, as the configuration capability's "Reload on change" defines for a file whose name ends in `.lua`. The reloaded configuration SHALL apply the saved style where it calls `gband.keystyle.use()` with no argument.

#### Scenario: Saved file
- **WHEN** the chooser saves the direct style
- **THEN** `user/keystyle.lua` holds `return "direct"` and a newline

#### Scenario: Written by hand
- **WHEN** `user/keystyle.lua` holds `return 'modal'` and a callback calls `gband.keystyle.saved()`
- **THEN** the call returns `modal`

#### Scenario: Unknown saved value
- **WHEN** `user/keystyle.lua` holds `return "vi"`, no `user/init.lua` exists, and a client attaches
- **THEN** no error is reported, the modal style is in use, and the chooser opens

#### Scenario: Broken file is not a configuration error
- **WHEN** `user/keystyle.lua` holds `error("boom")` and `user/init.lua` calls `gband.keystyle.use()`
- **THEN** loading succeeds with no error and the modal style is in use

#### Scenario: Saving reloads the configuration
- **WHEN** a client is attached with no `user/init.lua`, the modal style is in use, and the chooser saves the direct style
- **THEN** within a second Ctrl+Space then `h` focuses the column to the left and the keys that follow reach the focused window

### Requirement: Choose a key style
`gband.keystyle.choose()` SHALL make `root` the active table, as `gband.keymap.enter("root")` does, so keys reach the chooser even when it runs in navigation mode. It SHALL then open the key style chooser: a floating plugin window, as the plugin-windows capability defines, that takes focus. It SHALL be callable wherever an action value is, as the configuration capability defines. Calling it while the configuration loads SHALL be an error at the line of the call. Calling it while the chooser is open SHALL focus that chooser, as `gband.win.focus` does, and SHALL open no second one.

The chooser SHALL have a border, the title `key style  enter choose  esc later`, and its cursor line on. It SHALL hold one line per style, in this order:

| line | style | text |
|---|---|---|
| 1 | `modal` | `modal`, padded with spaces to 8 cells, then `<prefix> enters a mode, keys repeat until Escape` |
| 2 | `direct` | `direct`, padded with spaces to 8 cells, then `<prefix> then one key per action, back to typing` |

`<prefix>` SHALL be the key the `prefix` option names, in the form the key-hints capability's "Key form" defines, such as `C-space`. The cursor line SHALL start on the line of the saved style, or on the first line when none is saved. The chooser's width SHALL be the smaller of its longest line plus 2 for the border and the ribbon area's width. Its height SHALL be the smaller of 4 and the ribbon area's height. It SHALL be centered in the ribbon area. A line wider than the content area SHALL be cut at the content area's edge, as the plugin-windows capability defines, so each line still starts with its style's name.

j, k and the arrow keys SHALL move the cursor line, as the plugin-windows capability's defaults do. Enter SHALL save the style of the cursor line, as "Saved key style" defines, and close the chooser. Escape and `q` SHALL close the chooser and save nothing, as the plugin-windows capability defines for a floating plugin window with no `keys` entry for them. The style in use SHALL then stay until the configuration next loads.

When `gband.config_dir` is nil, or the file cannot be written, Enter SHALL close the chooser and save nothing. It SHALL then raise an error naming `user/keystyle.lua` and the reason, which the client reports as the configuration capability defines for an error in a callback. The style in use SHALL stay.

#### Scenario: Chooser opens
- **WHEN** no style is saved and a binding function calls `gband.keystyle.choose()`
- **THEN** a focused floating plugin window titled `key style  enter choose  esc later` shows `modal   C-space enters a mode, keys repeat until Escape` on its first line, with the cursor line there
- **AND** its second line shows `direct  C-space then one key per action, back to typing`

#### Scenario: Chooser beside the default status line
- **WHEN** no style is saved, the default configuration is in use on an 80×24 terminal, and the chooser opens
- **THEN** the chooser is 57 columns wide and 4 rows high, and spans columns 1 to 57 and rows 10 to 13 of the 60-column ribbon area
- **AND** neither line is cut

#### Scenario: Chooser in a narrow ribbon area
- **WHEN** the ribbon area is 40 columns wide and the chooser opens
- **THEN** the chooser is 40 columns wide
- **AND** its lines are cut to 38 cells, starting with `modal` and `direct`

#### Scenario: Cursor on the saved style
- **WHEN** `user/keystyle.lua` holds `return "direct"` and the chooser opens
- **THEN** the cursor line is on the second line

#### Scenario: Pick the direct style
- **WHEN** the chooser is open on its first line and the user presses `j` then Enter
- **THEN** the chooser closes and `user/keystyle.lua` holds `return "direct"`
- **AND** within a second the configuration reloads with the direct style's bindings

#### Scenario: Move with the arrow keys
- **WHEN** the chooser is open on its first line and the user presses Down, Up, then Down
- **THEN** the cursor line is on the second line

#### Scenario: Dismiss the chooser
- **WHEN** no style is saved, the chooser is open, and the user presses Escape
- **THEN** the chooser closes and `user/keystyle.lua` does not exist
- **AND** Ctrl+Space, `l`, `l` still moves focus twice, so the modal style stays in use

#### Scenario: Chosen from navigation mode
- **WHEN** the modal style is in use, `user/init.lua` binds `prefix y` to a function that calls `gband.keystyle.choose()`, and the user presses Ctrl+Space, `y`, then `j`
- **THEN** `root` is active and the chooser's cursor line is on the second line

#### Scenario: Chosen from the Lua prompt
- **WHEN** the user opens the Lua prompt with Ctrl+Space then `:`, types `gband.keystyle.choose()` and presses Enter
- **THEN** the chooser is open and has focus

#### Scenario: User file untouched
- **WHEN** `user/init.lua` calls `gband.keystyle.use()` and binds `alt+h`, and the user picks the direct style in the chooser
- **THEN** `user/init.lua` keeps its content
- **AND** after the reload Alt+H still focuses the column to the left and the direct style's bindings apply

#### Scenario: Cannot save
- **WHEN** the `user` directory cannot be written, the modal style is in use, and the user presses Enter on the chooser's second line
- **THEN** the chooser closes and the client shows an error naming `user/keystyle.lua`
- **AND** Ctrl+Space, `l`, `l` still moves focus twice

#### Scenario: Open again while open
- **WHEN** the chooser is open, a binding moves focus to another column, and a binding function calls `gband.keystyle.choose()` again
- **THEN** exactly one chooser is drawn and it has focus

#### Scenario: Choose while loading
- **WHEN** line 5 of `user/init.lua` calls `gband.keystyle.choose()` at the top level
- **THEN** loading fails with an error at `user/init.lua` line 5

### Requirement: Offer on the first start
The default configuration SHALL register a handler of `Attached`, as the lua-events capability defines. The handler SHALL call `gband.keystyle.choose()` when `gband.config_dir` is not nil and `gband.keystyle.saved()` returns nil. A client emits `Attached` once, after it attaches, and never for a reload, so the chooser SHALL open at most once per start of a client. Until a style is saved, the default configuration SHALL use the modal style, and each start SHALL offer the chooser again.

#### Scenario: First start
- **WHEN** neither `user/init.lua` nor `user/keystyle.lua` exists and a client attaches
- **THEN** the chooser is open and focused, with the cursor line on its first line
- **AND** the hints segment shows `C-space navigation`, so the modal style is in use

#### Scenario: Style already saved
- **WHEN** `user/keystyle.lua` holds `return "modal"`, no `user/init.lua` exists, and a client attaches
- **THEN** no floating plugin window is open

#### Scenario: Offered again on the next start
- **WHEN** the user dismisses the chooser with Escape, detaches, and attaches again
- **THEN** the chooser is open again

#### Scenario: No offer after a reload
- **WHEN** the user dismisses the chooser with Escape and then saves `user/lua/extra.lua`
- **THEN** the configuration reloads and no chooser opens

#### Scenario: No configuration directory
- **WHEN** the default configuration is evaluated with no configuration directory and `Attached` is emitted
- **THEN** no floating plugin window opens

#### Scenario: Own configuration
- **WHEN** `user/init.lua` binds only `alt+h`, no style is saved, and a client attaches
- **THEN** no floating plugin window is open
