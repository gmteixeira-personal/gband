## MODIFIED Requirements

### Requirement: Providers
`gband.core.provide(kind, implementation)` SHALL register the implementation the client calls for `kind`. A later call for the same kind SHALL replace the earlier implementation. A reload SHALL clear every registration before the new configuration loads. The kinds and the functions the client calls on them are:

| kind | implementation | the client calls |
|---|---|---|
| `"windows"` | a table | `key(id, name, text)`, `mouse(id, event)`, `set_box(id, col, row, width, height)`, `raise(id)`, `unfocus()`, `paste(id, text)`, `close_focused()`, `release()`, `opened(id, window)`, `window_resized(id, cols, rows)`, `window_closed(id)`, `ribbon_resized()`, `focused()`, `plugin_window_of(window)` and `flush()`, when the plugin window input, layout or drawing events they name happen |
| `"bars"` | a table | `flush()`, before the client draws |
| `"settings"` | a table | `open(line)`, after a load that followed `reopen_settings(line)` succeeds |
| `"styles"` | a function | the function, before the client draws. It returns `border`, `border_focused` and `banner`: the drawn styles of window borders, focused borders and the error banner |
| `"decorations"` | a function | the function, with an info table, when the client draws a window's top border. It returns nil or a list of spans, which the client draws at the right end of that border, as the window-decorations capability defines |

The API documentation SHALL give each function's arguments, return value and the moment the client calls it. A kind with no registration SHALL make the client do what it does without that feature: draw no plugin windows, no bars, no reopened settings window, default styles, or no decorations. An unknown kind, or an implementation of the wrong type, SHALL raise an error at the line of the call.

#### Scenario: Bundled plugin windows
- **WHEN** the default configuration is in use
- **THEN** the windows provider is the bundled `gband.win` module's, and `gband.win.open({ lines = { "hi" } })` draws a plugin window

#### Scenario: Replaced bar implementation
- **WHEN** `user/lua/gband/bar.lua` is a copy of the bundled module whose `flush` also records each call, and the default configuration is in use
- **THEN** the sidebar is drawn as with the bundled module, and the record grows with each frame

#### Scenario: Unknown kind
- **WHEN** line 2 of `user/init.lua` calls `gband.core.provide("menus", {})`
- **THEN** loading fails with an error at `user/init.lua` line 2

#### Scenario: Decorations need a function
- **WHEN** line 2 of `user/init.lua` calls `gband.core.provide("decorations", {})`
- **THEN** loading fails with an error at `user/init.lua` line 2 that names a function

#### Scenario: No decorations by default
- **WHEN** the default configuration is in use and two windows are open
- **THEN** no decorations provider is registered, and each tile's top border shows only the border and the title
