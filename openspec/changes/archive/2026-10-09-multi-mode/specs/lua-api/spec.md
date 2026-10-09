## MODIFIED Requirements

### Requirement: Core primitives
The client SHALL provide the table `gband.core`, holding the primitives below. Each primitive SHALL check its arguments and raise an error at the line of the call when one has the wrong type or value. "Callback" means code that runs where an action can be dispatched, as the configuration capability defines.

| primitive | effect |
|---|---|
| `owner()` | returns the name of the plugin whose code is running, or nil for code of no plugin |
| `loading()` | returns `true` while the configuration loads |
| `dispatching()` | returns `true` while a callback runs |
| `failed(plugin)` | returns `true` when the plugin named `plugin` is marked failed, as the plugins capability defines, and `false` for nil |
| `call(owner, label, fn, ...)` | runs `fn` with the arguments as code of the plugin `owner`, or of no plugin when `owner` is nil, with its own instruction budget. It returns `true` and `fn`'s values, or reports the failure as the plugins capability defines, under `label` when given, and returns `false` and the message |
| `report(label, message, level)` | reports an error with `message`, preceded by `label` and a colon when `label` is given, located at the file and line of the stack level `level` when given |
| `warn(text)` | records `text` in the client log as a warning |
| `load(path)` | compiles the file at `path` as Lua source text and returns the chunk, or raises the compile error |
| `bundled(name)` | returns the bundled colorscheme `name` as a chunk, or nil |
| `colorschemes()` | returns a new list of the names of every `colors/<name>.lua` on the runtimepath, sorted by byte value |
| `bundled_themes` | the list of the bundled themes' names, in the colorschemes capability's order |
| `events` | the list of the client's built-in event names |
| `emit(name, payload)` | runs the handlers of the built-in event `name` with `payload`, as the lua-events capability defines, then every `after_event` function |
| `after_event(fn)` | adds `fn`, which runs with the event's name after each built-in event's handlers |
| `on_state(fn)` | adds `fn`, which runs when the active key table, the viewed band's position, the number of bands, this client's multi mode or the error list changes |
| `state()` | returns a new table: `table`, the active key table; `band`, with the viewed band's `number`, its 1-based `index` and the band `count`; `window`, the focused window or nil; `multi`, `true` while this client's multi mode is on, as the multi-mode capability defines, and `false` otherwise; `width` and `height`, the terminal's size; `error`, the latest error or nil; and `ribbon`, with the ribbon area's `cols` and `rows` |
| `timer(ms, fn)` | runs `fn` as a callback every `ms` milliseconds, at least 1, until cancelled, and returns its id |
| `cancel(id)` | cancels the timer `id` |
| `palette.set(spec)`, `palette.get()` | set and read the terminal palette, as the colorschemes capability defines |
| `next_window()` | returns a plugin window number never returned before in this client |
| `present_window(id, frame)`, `forget_window(id)` | set, or remove, what the client draws for the plugin window `id` |
| `request(entry)` | asks the server to open or close a tiled plugin window's drawn window |
| `focus_window(window)` | focuses the window `window` as `gband.window.focus` does, without checking that the client's layout holds it yet |
| `parse_key(name)` | returns the canonical form of the key name `name`, or nil when it is not a key name |
| `border(spec)` | returns a border spec in its canonical form, or nil and a message |
| `width(value)` | returns a column width as a numerator and a denominator, or nil and a message |
| `open_target(band, after)` | returns `true` and the band and window a new tile goes to, or `false` and a message |
| `removed(name)` | returns the message that names the replacement of a removed API name, or nil |
| `place_bars(slots, cols)` | places bars against `cols` columns, as the bars capability defines, and returns each slot's columns, or `false` when not shown, then the ribbon area's columns |
| `present_bars(list)` | sets what the client draws for its bars |
| `error_marker(shown)` | records whether a bar draws the error marker, so the client draws no error banner while it does |
| `reopen_settings(line)` | remembers the settings window line to reopen after the next load succeeds, or forgets it for nil |
| `provide(kind, implementation)` | registers an implementation, as "Providers" defines |

The frame, entry, slot and bar tables SHALL have the fields the API documentation gives. The documentation SHALL describe every field, every return value and every error of each primitive. A primitive SHALL do exactly what gband did through the private table that this change removes.

#### Scenario: Owner inside a plugin
- **WHEN** the plugin `hello`'s `setup` calls `gband.core.owner()`
- **THEN** it returns `hello`

#### Scenario: Isolated call
- **WHEN** a binding function calls `gband.core.call(nil, nil, function() while true do end end)` and then focuses the column to the left
- **THEN** the call returns `false` and a message naming the instruction limit, and the column to the left is focused

#### Scenario: Wrong argument
- **WHEN** line 4 of `user/init.lua` calls `gband.core.timer("soon", print)`
- **THEN** loading fails with an error at `user/init.lua` line 4

#### Scenario: Timer
- **WHEN** a binding function starts `gband.core.timer(100, fn)` and cancels it after `fn` ran twice
- **THEN** `fn` runs no more

#### Scenario: Focus before the layout holds the window
- **WHEN** the client reports that a tiled plugin window's drawn window is window 5, the layout the client last gave Lua does not hold window 5, and a binding function calls `gband.win.focus` on the plugin window
- **THEN** the client focuses window 5 and reports no error

#### Scenario: Several state functions
- **WHEN** two plugins each call `gband.core.on_state` and the viewed band changes
- **THEN** both functions run

#### Scenario: State follows multi mode
- **WHEN** a plugin calls `gband.core.on_state(fn)` and a binding function dispatches `gband.action.toggle_multi()`
- **THEN** `fn` runs, and `gband.core.state().multi` is `true`
