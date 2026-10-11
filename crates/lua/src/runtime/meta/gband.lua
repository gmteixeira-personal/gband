---@meta
---@diagnostic disable: duplicate-set-field

---@alias gband.Side "client"|"server"|"test"
---@alias gband.KeyStyle "modal"|"direct"|"floating"
---@alias gband.Color string|integer
---@alias gband.Edge "left"|"right"|"top"|"bottom"
---@alias gband.Binding fun(event?: gband.MouseEvent): any
---@alias gband.ActionValue fun(target?: gband.Target): any
---@alias gband.Line string|(string|gband.Span)[]

---@class gband.Span
---@field text string
---@field hl? string

---@class gband.Style
---@field fg? gband.Color
---@field bg? gband.Color
---@field bold? boolean
---@field italic? boolean
---@field underline? boolean
---@field reverse? boolean
---@field dim? boolean

---@class gband.StyleSpec: gband.Style
---@field link? string

---@class gband.Target
---@field window? integer
---@field session? string
---@field band? integer
---@field after? integer
---@field floating? boolean
---@field step? number
---@field program? string|string[]
---@field edges? gband.Edge[]

---@class gband.MouseEvent
---@field button? "left"|"middle"|"right"
---@field direction? "up"|"down"|"left"|"right"
---@field col integer
---@field row integer
---@field ctrl boolean
---@field alt boolean
---@field shift boolean
---@field target "window"|"plugin_window"|"ribbon"|"outside"
---@field window? integer
---@field plugin_window? integer
---@field content_col? integer
---@field content_row? integer
---@field box_col? integer
---@field box_row? integer
---@field box_width? integer
---@field box_height? integer
---@field table string

---@class gband.Border
---@field sides? gband.Edge[]
---@field chars? string|string[]

---@class gband.View
---@field band integer
---@field window? integer
---@field floating boolean
---@field plugin_window? integer
---@field table string
---@field cols integer
---@field rows integer
---@field peek? true
---@field multi? true

---@class gband.LayoutWindow
---@field id integer
---@field rows? integer
---@field weight? number
---@field plugin_window? integer
---@field name? string
---@field manual_name? string
---@field last_focus? integer

---@class gband.LayoutColumn
---@field width number
---@field full_width boolean
---@field tabbed boolean
---@field tab_rows? integer
---@field windows gband.LayoutWindow[]

---@class gband.LayoutFloating: gband.LayoutWindow
---@field width number
---@field full_width boolean
---@field rows integer
---@field col integer
---@field row integer
---@field minimized? true

---@class gband.LayoutBand
---@field id integer
---@field columns gband.LayoutColumn[]
---@field floating gband.LayoutFloating[]

---@class gband.Layout
---@field cols integer
---@field rows integer
---@field bands gband.LayoutBand[]

---@class gband.WinOptions
---@field kind? "floating"|"tiled"
---@field lines? gband.Line[]
---@field focus? boolean
---@field cursorline? boolean
---@field keys? table<string, fun(win: integer)>
---@field on_input? fun(win: integer, text: string)
---@field on_close? fun(win: integer)
---@field on_resize? fun(win: integer, cols: integer, rows: integer)
---@field on_mouse? fun(win: integer, event: gband.WinMouseEvent)
---@field hover? boolean
---@field row? integer|"center"
---@field col? integer|"center"
---@field width? integer
---@field height? integer
---@field border? boolean|gband.Border
---@field title? string
---@field band? integer
---@field after? integer
---@field column_width? number

---@class gband.WinMouseEvent
---@field kind "press"|"release"|"drag"|"scroll"|"move"
---@field button? "left"|"middle"|"right"
---@field direction? "up"|"down"|"left"|"right"
---@field content_col? integer
---@field content_row? integer
---@field line? integer
---@field box_col? integer
---@field box_row? integer
---@field box_width? integer
---@field box_height? integer
---@field ctrl boolean
---@field alt boolean
---@field shift boolean

---@class gband.WinInfo
---@field id integer
---@field kind "floating"|"tiled"
---@field focused boolean
---@field window? integer
---@field top integer
---@field cursor integer
---@field line_count integer
---@field cols integer
---@field rows integer
---@field hover boolean
---@field row? integer
---@field col? integer
---@field width? integer
---@field height? integer

---@class gband.BarSpec
---@field id? string
---@field side "left"|"right"
---@field size? integer
---@field order? number
---@field lines? gband.Line[]
---@field hl? string
---@field on_resize? fun(id: string, width: integer, height: integer)

---@class gband.BarInfo
---@field id string
---@field side "left"|"right"
---@field size integer
---@field order number
---@field hl string
---@field plugin? string
---@field shown boolean
---@field col? integer
---@field width? integer
---@field height? integer

---@class gband.Palette
---@field fg? string
---@field bg? string
---@field black? string
---@field red? string
---@field green? string
---@field yellow? string
---@field blue? string
---@field magenta? string
---@field cyan? string
---@field white? string
---@field bright_black? string
---@field bright_red? string
---@field bright_green? string
---@field bright_yellow? string
---@field bright_blue? string
---@field bright_magenta? string
---@field bright_cyan? string
---@field bright_white? string

---@class gband.OptionSpec
---@field type "boolean"|"integer"|"number"|"string"
---@field default boolean|number|string
---@field values? (boolean|number|string)[]
---@field desc? string

---@class gband.CommandContext
---@field session? string
---@field client? integer
---@field focus fun(window: integer)

---@class gband.CoreState
---@field table string
---@field band { number: integer, index: integer, count: integer }
---@field window? integer
---@field multi boolean
---@field width integer
---@field height integer
---@field error? string
---@field ribbon { cols: integer, rows: integer }

---@class gband.Session
---@field name string
---@field bands { band: integer, columns: { width: number, full_width: boolean, tabbed: boolean, windows: { window: integer }[] }[], floating: { window: integer, width: number, full_width: boolean, rows: integer, col: integer, row: integer }[] }[]
---@field clients integer[]

---@class gband
gband = {}

---(client and server) `"client"`, `"server"`, or `"test"` in a test file.
---@type gband.Side
gband.side = "client"

---(client and server) The version of the documented API.
---@type integer
gband.api_version = 2

---(client and server) The configuration directory's path, or nil when the process has none.
---@type string?
gband.config_dir = nil

---(client and server) The directories searched for plugins, modules and colorschemes, in order.
---@type string[]
gband.runtimepath = {}

---(client and server) Every built-in action of the side, and every registered action under its full name.
---@class gband.Action
---@field [string] gband.ActionValue
gband.action = {}

---(client and server) Registers an action and returns its action value.
---@param name string
---@param fn fun()
---@param opts? { desc?: string }
---@return gband.ActionValue
function gband.action.register(name, fn, opts) end

---(client and server) Every built-in and registered action of the side, in byte order of names.
---@return { name: string, desc: string }[]
function gband.action.list() end

---(client) focus the column to the left
function gband.action.focus_column_left() end

---(client) focus the column to the right
function gband.action.focus_column_right() end

---(client) focus the window below
function gband.action.focus_window_down() end

---(client) focus the window above
function gband.action.focus_window_up() end

---(client) view the band below
function gband.action.focus_band_down() end

---(client) view the band above
function gband.action.focus_band_up() end

---(client) center the focused column
function gband.action.center_column() end

---(client) switch focus between floating and tiled windows
function gband.action.switch_focus_floating_tiled() end

---(client) minimize the focused floating window
function gband.action.minimize_window() end

---(client and server) open a window running the user's shell
---@param target? { session?: string, band?: integer, after?: integer, floating?: boolean, program?: string|string[] }
function gband.action.open_window(target) end

---(client and server) close the window
---@param target? { session?: string, window?: integer }
function gband.action.close_window(target) end

---(client and server) consume or expel the window to the left
---@param target? { session?: string, window?: integer }
function gband.action.consume_or_expel_left(target) end

---(client and server) consume or expel the window to the right
---@param target? { session?: string, window?: integer }
function gband.action.consume_or_expel_right(target) end

---(client and server) move the column or floating window to the left
---@param target? { session?: string, window?: integer }
function gband.action.move_column_left(target) end

---(client and server) move the column or floating window to the right
---@param target? { session?: string, window?: integer }
function gband.action.move_column_right(target) end

---(client and server) move the window down
---@param target? { session?: string, window?: integer }
function gband.action.move_window_down(target) end

---(client and server) move the window up
---@param target? { session?: string, window?: integer }
function gband.action.move_window_up(target) end

---(client and server) float or tile the window
---@param target? { session?: string, window: integer, after?: integer, floating?: boolean }
function gband.action.toggle_window_floating(target) end

---(client and server) cycle the width of the window's column
---@param target? { session?: string, window?: integer }
function gband.action.cycle_column_width(target) end

---(client and server) toggle full width of the window's column
---@param target? { session?: string, window?: integer }
function gband.action.toggle_full_width(target) end

---(client and server) toggle tabbed display of the window's column
---@param target? { session?: string, window?: integer }
function gband.action.toggle_column_tabbed(target) end

---(client and server) grow the width of the window's column
---@param target? { session?: string, window?: integer, step?: number }
function gband.action.grow_column_width(target) end

---(client and server) shrink the width of the window's column
---@param target? { session?: string, window?: integer, step?: number }
function gband.action.shrink_column_width(target) end

---(client and server) grow the height of the window
---@param target? { session?: string, window?: integer, step?: number }
function gband.action.grow_window_height(target) end

---(client and server) shrink the height of the window
---@param target? { session?: string, window?: integer, step?: number }
function gband.action.shrink_window_height(target) end

---(client and server) reset the height of the window
---@param target? { session?: string, window?: integer }
function gband.action.reset_window_height(target) end

---(client) detach
function gband.action.detach() end

---(client) send the prefix key to the focused window
---@param target? { window?: integer }
function gband.action.send_prefix(target) end

---(client) reload the configuration
function gband.action.reload() end

---(client) toggle multi mode
function gband.action.toggle_multi() end

---(client) move the window with the mouse
function gband.action.drag_window() end

---(client) resize the window with the mouse
---@param target? { edges: gband.Edge[] }
function gband.action.drag_resize_window(target) end

---(client) slide the band or switch bands with the mouse
function gband.action.drag_band() end

---(client and server) Named functions that take a table of arguments.
---@class gband.Cmd
gband.cmd = {}

---(client and server) Registers a command and returns its full name. A server command's function also receives a context.
---@param name string
---@param fn fun(args: table, ctx?: gband.CommandContext): any
---@param opts? { desc?: string, args?: string[] }
---@return string
function gband.cmd.register(name, fn, opts) end

---(client and server) Calls the command; `true` when its function returns, `false` when it fails or is disabled.
---@param name string
---@param args? table
---@return boolean
function gband.cmd.run(name, args) end

---(client and server) Every command, in byte order of names.
---@return { name: string, desc?: string, args: string[] }[]
function gband.cmd.list() end

---(client and server) Reads and sets an option by name.
---@class gband.Opt
---@field [string] any
---@field declare fun(name: string, spec: gband.OptionSpec): string
---@field list fun(): { name: string, type: string, default: any, value: any, desc?: string }[]
gband.opt = {}

---(client and server) Sets several options; raises an error on an invalid value.
---@param options table<string, any>
function gband.set(options) end

---(client and server) Runs `fn` with a payload table each time `event` happens.
---@param event string
---@param fn fun(event: table)
---@param opts? { group?: integer|string, once?: boolean, pattern?: string }
function gband.on(event, fn, opts) end

---(client and server) Returns the id of the group `name`, first removing its handlers unless `opts.clear` is `false`.
---@param name string
---@param opts? { clear?: boolean }
---@return integer
function gband.augroup(name, opts) end

---(client and server) In a client, emits a `User` event. In the server, sends an event to clients, to `opts.session` only when given.
---@param name string
---@param data? any
---@param opts? { session?: string }
function gband.emit(name, data, opts) end

---(client and server) Requires the module `name` and calls its `setup(opts)`; `false` on a plugin error.
---@param name string
---@param opts? table
---@return boolean
function gband.plugin(name, opts) end

---(client and server) One table per plugin whose manifest the load ran, in runtimepath order.
---@return { name: string, version: string, client?: string, failed: boolean }[]
function gband.plugins() end

---(client and server) In a client, a copy of the window's latest state, or nil. In the server, a table that reads and writes the state of `window` in `session`, or nil.
---@param window integer
---@return table<string, any>?
---@overload fun(session: string, window: integer): table<string, any>?
function gband.window_state(window) end

---(server) The session names in ascending byte order.
---@return string[]
function gband.sessions() end

---(server) The session's bands and attached clients, or nil for an unknown session.
---@param name string
---@return gband.Session?
function gband.session(name) end

---(client) Binds `key` in `root`, or in `prefix` for `"prefix <key>"`.
---@param key string
---@param action gband.Binding
function gband.bind(key, action) end

---(client) Removes a binding made as `gband.bind` makes it.
---@param key string
function gband.unbind(key) end

---(client) Opens a window running `cmd`, a shell command line or a list of a program and its arguments, or the user's shell.
---@param request? { cmd?: string|string[], band?: integer, after?: integer }
function gband.spawn(request) end

---(client) A new list of the client's errors, oldest first.
---@return string[]
function gband.errors() end

---(client) Empties the client's error list.
function gband.clear_errors() end

---(client) A new table describing the layout the client holds.
---@return gband.Layout
function gband.layout() end

---(client) The viewed band, focus, active key table and ribbon size.
---@return gband.View
function gband.view() end

---(client) Calls the server command `name`; `callback` receives `true` and the result, or `false` and a message.
---@param name string
---@param args? table
---@param callback? fun(ok: boolean, result: any)
function gband.rpc(name, args, callback) end

---(client) Asks the terminal for a desktop notification.
---@param text string
---@param opts? { title?: string }
function gband.notify(text, opts) end

---(client) Rings the terminal's bell.
function gband.bell() end

---(client) Sets the terminal's clipboard.
---@param text string
function gband.clipboard(text) end

---(client) Opens a URL or a path with the machine's opener; `true` when the opener started.
---@param target string
---@return boolean
function gband.open(target) end

---(client) Runs the colorscheme `name` and returns `true` on success, or returns the active colorscheme's name with no argument.
---@param name? string
---@return boolean|string
function gband.colorscheme(name) end

---(client) Key tables and their bindings.
---@class gband.Keymap
gband.keymap = {}

---(client) Binds `key` in `table` to an action value or a function.
---@param table string
---@param key string
---@param action gband.Binding
---@param opts? { desc?: string }
function gband.keymap.set(table, key, action, opts) end

---(client) Removes a binding.
---@param table string
---@param key string
function gband.keymap.del(table, key) end

---(client) The bindings of `table`, in the order they were made.
---@param table string
---@return { key: string, desc?: string, action?: string }[]
function gband.keymap.list(table) end

---(client) The active key table.
---@return string
function gband.keymap.current_table() end

---(client) Makes `table` active for the next key.
---@param table string
function gband.keymap.enter(table) end

---(client) Makes `table` a mode, with an optional label.
---@param table string
---@param opts? { label?: string }
function gband.keymap.mode(table, opts) end

---(client) The label of a mode, or `table` itself.
---@param table string
---@return string
function gband.keymap.label(table) end

---(client) Runs the binding of `key` in `table`; `false` when the key is unbound there.
---@param table string
---@param key string
---@return boolean
function gband.keymap.run(table, key) end

---(client) The key style presets.
---@class gband.Keystyle
gband.keystyle = {}

---(client) Makes the bindings of a key style and returns its name; the saved style, or `"modal"`, with no argument.
---@param style? gband.KeyStyle
---@return gband.KeyStyle
function gband.keystyle.use(style) end

---(client) The saved key style, or nil.
---@return gband.KeyStyle?
function gband.keystyle.saved() end

---(client) The style `use` picked in this load, or nil.
---@return gband.KeyStyle?
function gband.keystyle.current() end

---@alias gband.AgentState "blocked"|"working"|"done"|"idle"|"unknown"

---An agent window of the client's layout.
---@class gband.AgentEntry
---@field window integer
---@field band integer
---@field agent string
---@field state gband.AgentState

---(client) The agent windows of the client's layout.
---@class gband.Agents
gband.agents = {}

---(client) A new list with one entry per agent window, in layout order.
---@return gband.AgentEntry[]
function gband.agents.list() end

---(client) Focuses the window of entry `n` of `list()` and returns true, or returns false when there is no entry `n`.
---@param n integer
---@return boolean
function gband.agents.focus(n) end

---(client) Focuses the agent after the focused one with 1, or before it with -1, wrapping; returns the focused window, or nil when there is no agent.
---@param delta 1|-1
---@return integer?
function gband.agents.step(delta) end

---(client) The agents and bands key modes.
---@class gband.Jump
gband.jump = {}

---(client) Declares the `agents` and `bands` modes with their bindings, and binds `a` and `b` in `prefix` to enter them.
function gband.jump.setup() end

---(client) The settings window and the settings it saves.
---@class gband.Settings
gband.settings = {}

---(client) Enters `root` and opens the settings window, or focuses it.
function gband.settings.open() end

---(client) The saved theme, or nil.
---@return string?
function gband.settings.theme() end

---(client) The saved sidebar setting, or nil.
---@return boolean?
function gband.settings.sidebar() end

---(client) The saved `I on new` setting, or nil.
---@return boolean?
function gband.settings.interactive_on_new() end

---(client) A new list of the names the theme list shows.
---@return string[]
function gband.settings.themes() end

---(client) Text measurement as plugin windows and bars draw it.
---@class gband.Ui
gband.ui = {}

---(client) The cells `text` takes.
---@param text string
---@return integer
function gband.ui.width(text) end

---(client) `text` cut to `width` cells, ending in `…` when cut.
---@param text string
---@param width integer
---@return string
function gband.ui.truncate(text, width) end

---(client) Highlight groups.
---@class gband.Hl
gband.hl = {}

---(client) Replaces the group's explicit setting; nil removes it.
---@param name string
---@param spec gband.StyleSpec?
function gband.hl.set(name, spec) end

---(client) Replaces the group's default setting.
---@param name string
---@param spec gband.StyleSpec
function gband.hl.default(name, spec) end

---(client) A copy of the group's definition, or its resolved style with `{ resolve = true }`.
---@param name string
---@param opts? { resolve?: boolean }
---@return gband.StyleSpec?
function gband.hl.get(name, opts) end

---(client) The terminal palette.
---@class gband.PaletteApi
gband.palette = {}

---(client) Replaces the whole palette with `spec`.
---@param spec gband.Palette
function gband.palette.set(spec) end

---(client) A new table of the fields the palette sets.
---@return gband.Palette
function gband.palette.get() end

---(client) Windows of the layout.
---@class gband.Window
gband.window = {}

---(client) Views the window's band and focuses the window, or peeks it with `{ peek = true }`.
---@param window integer
---@param opts? { peek?: boolean }
function gband.window.focus(window, opts) end

---(client) Hides a floating window from this client until it is focused again.
---@param window integer
function gband.window.minimize(window) end

---(client) Sets the width of the window's column, or of its box when it floats.
---@param window integer
---@param width number
function gband.window.set_width(window, width) end

---(client) Gives a fixed height in rows or an automatic height of a weight.
---@param window integer
---@param height { rows?: integer, weight?: number }
function gband.window.set_height(window, height) end

---(client) Places a floating window's box.
---@param window integer
---@param position { col: integer, row: integer }
function gband.window.set_position(window, position) end

---(client) Sends a key name, or a list of them, as key presses.
---@param window integer
---@param keys string|string[]
function gband.window.send_keys(window, keys) end

---(client) Sends each character as a key press.
---@param window integer
---@param text string
function gband.window.send_text(window, text) end

---(client) Sends `text` as a paste.
---@param window integer
---@param text string
function gband.window.paste(window, text) end

---(client) Sets the window's manual name; nil or empty clears it.
---@param window integer
---@param name string?
function gband.window.rename(window, name) end

---(client) Bands of the layout.
---@class gband.Band
gband.band = {}

---(client) Views a band, focusing the window last focused there.
---@param band integer
function gband.band.view(band) end

---(client) Plugin windows.
---@class gband.Win
gband.win = {}

---(client) Opens a plugin window and returns its number.
---@param opts gband.WinOptions
---@return integer
function gband.win.open(opts) end

---(client) Replaces the lines.
---@param win integer
---@param lines gband.Line[]
function gband.win.set_lines(win, lines) end

---(client) Moves the first shown line.
---@param win integer
---@param count integer
function gband.win.scroll(win, count) end

---(client) Moves the cursor line.
---@param win integer
---@param line integer
function gband.win.set_cursor(win, line) end

---(client) Focuses a floating plugin window, or the window of a tiled one.
---@param win integer
function gband.win.focus(win) end

---(client) Changes a floating plugin window's `row`, `col`, `width`, `height`, `border` or `title`.
---@param win integer
---@param config { row?: integer|"center", col?: integer|"center", width?: integer, height?: integer, border?: boolean|gband.Border, title?: string }
function gband.win.set_config(win, config) end

---(client) Closes a plugin window, and for a tiled one its window too.
---@param win integer
function gband.win.close(win) end

---(client) The plugin window's state.
---@param win integer
---@return gband.WinInfo
function gband.win.info(win) end

---(client) The open plugin windows' numbers in ascending order.
---@return integer[]
function gband.win.list() end

---(client) Side bars.
---@class gband.Bar
gband.bar = {}

---(client) Adds a bar and returns its full id.
---@param spec gband.BarSpec
---@return string
function gband.bar.add(spec) end

---(client) Replaces the bar's lines.
---@param id string
---@param lines gband.Line[]
function gband.bar.set_lines(id, lines) end

---(client) Changes `side`, `size`, `order` and `hl`.
---@param id string
---@param config { side?: "left"|"right", size?: integer, order?: number, hl?: string }
function gband.bar.set_config(id, config) end

---(client) Removes the bar; `false` when no bar has that id.
---@param id string
---@return boolean
function gband.bar.remove(id) end

---(client) The bar's placement and settings.
---@param id string
---@return gband.BarInfo
function gband.bar.info(id) end

---(client) Every bar, in byte order of ids.
---@return gband.BarInfo[]
function gband.bar.list() end

---(client) The primitives the Lua API modules are built from.
---@class gband.Core
---@field register fun(name: string, timeout: number?, fn: fun(g: table)) (test) Adds a test case.
---@field wrap fun(wrapper: fun(raw: function): function) (test) Sets the function the runner passes each handle function through.
gband.core = {}

---(client) The name of the plugin whose code is running, or nil.
---@return string?
function gband.core.owner() end

---(client) `true` while the configuration loads.
---@return boolean
function gband.core.loading() end

---(client) `true` while a callback runs.
---@return boolean
function gband.core.dispatching() end

---(client) `true` when the plugin is failed.
---@param plugin string?
---@return boolean
function gband.core.failed(plugin) end

---(client) Runs `fn` as code of the plugin `owner`; `true` and its values, or `false` and the message.
---@param owner string?
---@param label string?
---@param fn function
---@param ... any
---@return boolean ok
---@return any ...
function gband.core.call(owner, label, fn, ...) end

---(client) Adds an error to the error list and the log.
---@param label string?
---@param message string
---@param level? integer
function gband.core.report(label, message, level) end

---(client) Writes `text` to the client log as a warning.
---@param text string
function gband.core.warn(text) end

---(client) Compiles the file at `path` without running it.
---@param path string
---@return function
function gband.core.load(path) end

---(client) Runs `fn` every `ms` milliseconds until cancelled, and returns the timer's id.
---@param ms integer
---@param fn fun()
---@return integer
function gband.core.timer(ms, fn) end

---(client) Stops the timer `id`.
---@param id integer
function gband.core.cancel(id) end

---(client) The client's built-in event names.
---@type string[]
gband.core.events = {}

---(client) Runs the handlers of the built-in event `name`, then every `after_event` function.
---@param name string
---@param payload table
function gband.core.emit(name, payload) end

---(client) Adds `fn`, run after the handlers of each built-in event, and with nil after a load or a plugin refresh.
---@param fn fun(name: string?)
function gband.core.after_event(fn) end

---(client) Adds `fn`, run when the key table, the viewed band, the bands, multi mode or the error list change.
---@param fn fun()
function gband.core.on_state(fn) end

---(client) A new table of the client's state.
---@return gband.CoreState
function gband.core.state() end

---(client) Records whether a bar draws the error marker.
---@param shown boolean
function gband.core.error_marker(shown) end

---(client) The terminal palette primitives.
---@class gband.CorePalette
gband.core.palette = {}

---(client) Replaces the terminal palette with `spec`.
---@param spec gband.Palette
function gband.core.palette.set(spec) end

---(client) A new table of the palette's fields that are set.
---@return gband.Palette
function gband.core.palette.get() end

---(client) The bundled colorscheme `name` as a function that applies it, or nil.
---@param name string
---@return function?
function gband.core.bundled(name) end

---(client) A new list of the names of every `colors/<name>.lua` on the runtimepath.
---@return string[]
function gband.core.colorschemes() end

---(client) The bundled themes' names, in order, without the alias `catppuccin`.
---@type string[]
gband.core.bundled_themes = {}

---(client) A plugin window number this client never returned before.
---@return integer
function gband.core.next_window() end

---(client) Sets what the client draws for the plugin window `id`.
---@param id integer
---@param frame table
function gband.core.present_window(id, frame) end

---(client) Stops drawing the plugin window `id`.
---@param id integer
function gband.core.forget_window(id) end

---(client) Asks the server to open or close the drawn window of a tiled plugin window.
---@param entry { id: integer, op: "open"|"close", band?: integer, after?: integer, num?: integer, den?: integer, focus?: boolean }
function gband.core.request(entry) end

---(client) Focuses the window `window` without checking the client's layout.
---@param window integer
function gband.core.focus_window(window) end

---(client) The canonical form of a key name, or nil.
---@param name string
---@return string?
function gband.core.parse_key(name) end

---(client) A border table in canonical form, or nil and a message.
---@param spec boolean|gband.Border
---@return gband.Border?
---@return string? message
function gband.core.border(spec) end

---(client) A column width as a numerator and a denominator, or nil and a message.
---@param value number
---@return integer? numerator
---@return integer|string denominator
function gband.core.width(value) end

---(client) Checks a target for a new tile; `true`, the band and the window to open after, or `false` and a message.
---@param band integer?
---@param after integer?
---@return boolean ok
---@return integer|string|nil band
---@return integer? after
function gband.core.open_target(band, after) end

---(client) The message naming the replacement of a removed API name, or nil.
---@param name string
---@return string?
function gband.core.removed(name) end

---(client) Places bar slots against `cols` columns; each slot's `{ col, width }` or `false`, then the ribbon's.
---@param slots { side: "left"|"right", size: integer, order: number, seq: integer }[]
---@param cols integer
---@return ({ col: integer, width: integer }|false)[]
function gband.core.place_bars(slots, cols) end

---(client) Sets what the client draws for its bars.
---@param list table[]
function gband.core.present_bars(list) end

---(client) Remembers the settings window line to reopen after the next load, or forgets it for nil.
---@param line integer?
function gband.core.reopen_settings(line) end

---(client) Registers the implementation the client calls for `kind`.
---@param kind "windows"|"bars"|"settings"|"styles"|"decorations"
---@param implementation table|function
function gband.core.provide(kind, implementation) end
