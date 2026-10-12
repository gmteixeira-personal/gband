# lua-prompt Specification

## Purpose

Defines the Lua prompt: a bundled client plugin that opens a one-line floating plugin window, in the manner of Neovim's command line, takes one line of Lua from the user, and runs it as code of the configuration file would run.

## Requirements

### Requirement: Prompt plugin
gband SHALL bundle the client plugin module `gband.prompt`, whose plugin name is `prompt`. Its `setup` SHALL take no options, and an options table holding any field SHALL make it raise an error naming the field. `setup` SHALL register the action `prompt.open` with the description `run Lua`.

Dispatching `prompt.open` SHALL open the prompt, as "Prompt plugin window" defines, with an empty line. It SHALL also make `root` the active table, as `gband.keymap.enter("root")` does, so the keys that follow reach the prompt rather than a mode. Dispatching it while the prompt is open SHALL focus that prompt, as `gband.win.focus` does, keep its line, and open no second one.

#### Scenario: Action registered
- **WHEN** a configuration calls `gband.plugin("gband.prompt")` and reads `gband.action.list()`
- **THEN** it holds an entry named `prompt.open` with the description `run Lua`

#### Scenario: Unknown option
- **WHEN** a configuration calls `gband.plugin("gband.prompt", { title = "eval" })`
- **THEN** `gband.plugin` returns `false` and a plugin error of `prompt` names `title`

#### Scenario: Opened from navigation mode
- **WHEN** the default configuration is in use, two columns are open with the second focused, and the user presses Ctrl+Space, `:`, then `h`
- **THEN** `root` is the active table and the prompt shows `:h`
- **AND** the second column is still focused

#### Scenario: Open again while open
- **WHEN** the prompt is open and shows `:ab`, and the user presses Ctrl+Space, `l`, which focuses another column and leaves the prompt unfocused, then `:`
- **THEN** exactly one prompt is drawn, it has focus, and it shows `:ab`

### Requirement: Prompt plugin window
Opening the prompt SHALL open a floating plugin window, as the plugin-windows capability defines, that belongs to the plugin `prompt` and takes focus. It SHALL have a border, the title `lua`, a height of 3 and the width of the ribbon area, and SHALL sit on the ribbon area's last three rows, from its first column. The ribbon area's size SHALL be read from `gband.view()` when the prompt opens.

The prompt SHALL keep a cursor position in its line: a count of characters before the cursor, from 0 to the line's length. The prompt SHALL open with the cursor after the last character of its starting line.

The floating plugin window SHALL hold one line: `:` followed by the typed text, in the group `PluginWindow`, with the cursor cell in the group `PromptCursor`. The cursor cell SHALL be the character after the cursor, or one space appended after the text when the cursor is after the last character.

The prompt SHALL keep a scroll position: the number of leading characters of the line, `:` included, that are left out of the drawing. It SHALL be 0 when the prompt opens. After each change to the line, to the cursor or to the content area's width, the scroll position SHALL become the one before the change, lowered to at most the number of typed characters before the cursor and to at most the fewest left out that let the rest of the line and one more cell fit in the content area, and then raised to at least the fewest left out that let the text up to and including the cursor cell fit. The characters after the cursor cell that do not fit SHALL be left out. The cursor cell therefore stays shown, and a line that fits is drawn whole.

The plugin SHALL give the group `PromptCursor` the default `{ reverse = true }`, as the highlights capability defines defaults.

#### Scenario: Prompt on the bottom rows
- **WHEN** the ribbon area is 79×24, as the default sidebar leaves it on an 80×24 terminal, and a binding dispatches `prompt.open`
- **THEN** a focused floating plugin window titled `lua` spans columns 0 to 78 and rows 21 to 23 of the ribbon area
- **AND** its content row reads `:` followed by one cell drawn reversed

#### Scenario: Typed text and the cursor
- **WHEN** the prompt is open and the user types `abc`
- **THEN** the content row reads `:abc` and the cell after `c` is drawn in `PromptCursor`'s resolved style

#### Scenario: Cursor on a character
- **WHEN** the prompt shows `:abc` with the cursor after `c`, and the user presses Left twice
- **THEN** the content row reads `:abc`, the cell holding `b` is drawn in `PromptCursor`'s resolved style, and no cell after `c` is

#### Scenario: Long line shows its end
- **WHEN** the prompt's content area is 10 columns wide and the user types `abcdefghijkl`
- **THEN** the content row reads `defghijkl` followed by the cursor cell

#### Scenario: Long line follows the cursor
- **WHEN** the prompt's content area is 10 columns wide, the user types `abcdefghijkl`, then presses Left 12 times
- **THEN** the content row reads `:abcdefghi`, and the cell holding `a` is the cursor cell

#### Scenario: Long line keeps its place
- **WHEN** the prompt's content area is 10 columns wide, the user types `abcdefghijkl`, then presses Left 3 times
- **THEN** the content row reads `defghijkl`, and the cell holding `j` is the cursor cell

#### Scenario: Long line scrolls at its left edge
- **WHEN** the prompt's content area is 10 columns wide, the user types `abcdefghijkl`, then presses Left 9 times
- **THEN** the content row reads `cdefghijkl`, and the cell holding `d` is the cursor cell

#### Scenario: End of a scrolled line
- **WHEN** the prompt's content area is 10 columns wide, the user types `abcdefghijkl`, presses Home, then End
- **THEN** the content row reads `defghijkl` followed by the cursor cell

#### Scenario: Theme sets the cursor
- **WHEN** `user/init.lua` calls `gband.hl.set("PromptCursor", { bg = "#ff0000" })` and the prompt is drawn on a truecolor terminal
- **THEN** the cursor cell has background `#ff0000`

### Requirement: Editing the line
While the prompt is focused, it SHALL edit its line as follows:

| input | effect |
|---|---|
| a key that types a character, as the plugin-windows capability defines it | insert the character at the cursor, `j`, `k`, `q` and space included, and move the cursor after it |
| a paste | insert the pasted text at the cursor, with each `\r\n` pair and every other control character turned into one space, and move the cursor after it |
| Left | move the cursor one character back; at the start of the line, do nothing |
| Right | move the cursor one character forward; after the last character, do nothing |
| Home | move the cursor before the first character |
| End | move the cursor after the last character |
| Backspace | remove the character before the cursor; on an empty line, close the prompt and run nothing; with the cursor at the start of a line that holds text, do nothing |
| Delete | remove the character after the cursor; after the last character, do nothing and keep the prompt open |
| Ctrl+U | clear the line and put the cursor at its start |
| Escape | close the prompt and run nothing |
| Enter | run the line, as "Running the line" defines, wherever the cursor is |

A character SHALL be one Unicode scalar value. Every other key SHALL take the plugin-windows capability's default, or be discarded, and SHALL leave the line and the cursor unchanged. Nothing typed into the prompt SHALL reach the focused window.

The prompt SHALL take mouse events through `on_mouse`, as the plugin-windows capability defines, whether or not it is focused. A left press on its content row SHALL move the cursor as follows, after the press has focused the prompt:

- on a cell drawn with a character of the typed text, either cell of a double-width one included: before that character
- on the cell drawn with `:`: before the first character
- on any other cell of the content row: after the last character

Every other mouse event, a press with another button, a release, a drag and a wheel step included, SHALL leave the line and the cursor unchanged.

#### Scenario: Click a character
- **WHEN** the prompt shows `:abc` with the cursor after `c`, and the user left-clicks the cell holding `b`, then types `x`
- **THEN** the prompt shows `:axbc`

#### Scenario: Click the colon
- **WHEN** the prompt shows `:abc` with the cursor after `c`, and the user left-clicks the cell holding `:`, then types `x`
- **THEN** the prompt shows `:xabc`

#### Scenario: Click past the text
- **WHEN** the prompt shows `:abc` with the cursor before `a`, and the user left-clicks the fifth cell after `c`, then types `x`
- **THEN** the prompt shows `:abcx`

#### Scenario: Click a wide character
- **WHEN** the prompt shows `:a界b` with the cursor after `b`, and the user left-clicks the second cell of `界`, then types `x`
- **THEN** the prompt shows `:ax界b`

#### Scenario: Click a scrolled line
- **WHEN** the prompt's content area is 10 columns wide, the user types `abcdefghijkl`, then left-clicks the cell holding `f`
- **THEN** the content row reads `defghijkl`, and the cell holding `f` is the cursor cell

#### Scenario: Click an unfocused prompt
- **WHEN** the prompt shows `:ab`, the user presses Ctrl+Space, `l` and Ctrl+Space, which focus another column and leave navigation mode, then left-clicks the cell holding `b` and types `x`
- **THEN** the prompt has focus and shows `:axb`

#### Scenario: Other buttons
- **WHEN** the prompt shows `:abc` with the cursor after `c`, and the user right-clicks the cell holding `a`, then types `x`
- **THEN** the prompt shows `:abcx`

#### Scenario: Home
- **WHEN** the prompt shows `:abc` with the cursor after `c`, and the user presses Home, then types `x`
- **THEN** the prompt shows `:xabc`

#### Scenario: End
- **WHEN** the prompt shows `:abc` with the cursor after `c`, and the user presses Home, then End, then types `x`
- **THEN** the prompt shows `:abcx`

#### Scenario: Letters that scroll elsewhere
- **WHEN** the prompt is open and the user types `jkq`
- **THEN** the prompt stays open and shows `:jkq`

#### Scenario: Insert in the middle
- **WHEN** the prompt shows `:ac` with the cursor after `c`, and the user presses Left, then types `b`
- **THEN** the prompt shows `:abc` and the cell holding `c` is the cursor cell

#### Scenario: Paste in the middle
- **WHEN** the prompt shows `:ad` with the cursor after `a`, and the user pastes `b\nc`
- **THEN** the prompt shows `:ab cd` and the cell holding `d` is the cursor cell

#### Scenario: Left at the start
- **WHEN** the prompt shows `:ab` with the cursor before `a`, and the user presses Left, then types `x`
- **THEN** the prompt shows `:xab`

#### Scenario: Right at the end
- **WHEN** the prompt shows `:ab` with the cursor after `b`, and the user presses Right, then types `x`
- **THEN** the prompt shows `:abx`

#### Scenario: Right after Left
- **WHEN** the prompt shows `:abc` with the cursor after `c`, and the user presses Left twice, Right once, then types `x`
- **THEN** the prompt shows `:abxc`

#### Scenario: Backspace
- **WHEN** the prompt shows `:ab` and the user presses Backspace
- **THEN** the prompt shows `:a`

#### Scenario: Backspace in the middle
- **WHEN** the prompt shows `:abc` with the cursor after `c`, and the user presses Left, then Backspace
- **THEN** the prompt shows `:ac` and the cell holding `c` is the cursor cell

#### Scenario: Backspace at the start of a line with text
- **WHEN** the prompt shows `:ab` with the cursor before `a`, and the user presses Backspace
- **THEN** the prompt stays open and shows `:ab`

#### Scenario: Backspace on an empty line
- **WHEN** the prompt shows `:` and the user presses Backspace
- **THEN** the prompt closes and nothing runs

#### Scenario: Delete in the middle
- **WHEN** the prompt shows `:abc` with the cursor after `c`, and the user presses Left twice, then Delete
- **THEN** the prompt shows `:ac` and the cell holding `c` is the cursor cell

#### Scenario: Delete at the end
- **WHEN** the prompt shows `:ab` with the cursor after `b`, and the user presses Delete
- **THEN** the prompt stays open and shows `:ab`

#### Scenario: Delete on an empty line
- **WHEN** the prompt shows `:` and the user presses Delete
- **THEN** the prompt stays open and shows `:`

#### Scenario: Clear the line
- **WHEN** the prompt shows `:abc` with the cursor before `b`, and the user presses Ctrl+U, then types `x`
- **THEN** the prompt shows `:x`

#### Scenario: Run with the cursor in the middle
- **WHEN** two columns are open with the second focused, the user opens the prompt, types `gband.action.focus_column_left()`, presses Left 5 times, then presses Enter
- **THEN** the first column is focused and the prompt is closed

#### Scenario: Wide character under the cursor
- **WHEN** the prompt shows `:a界b` with the cursor after `b`, and the user presses Left twice
- **THEN** both cells of `界` are drawn in `PromptCursor`'s resolved style

#### Scenario: Escape
- **WHEN** the prompt shows `:gband.action.detach()` and the user presses Escape
- **THEN** the prompt closes, the client stays attached, and the focused window receives nothing

#### Scenario: Paste with line breaks
- **WHEN** the prompt is open and the user pastes `a = 1\nb = 2\r\n`
- **THEN** the prompt shows `:a = 1 b = 2 `

### Requirement: Running the line
Enter SHALL close the prompt and then run its line. An empty line SHALL run nothing. The line SHALL be compiled as Lua source text, never as a binary chunk, and SHALL run protected, inside the callback of the prompt's Enter. Its return values SHALL be discarded.

The line SHALL belong to no plugin, as code of the configuration file does, although a callback of the plugin `prompt` runs it. It SHALL be able to call everything a binding function can call, as the configuration capability defines. The actions and other dispatches it makes SHALL take effect when the callback returns, in order, as a binding function's do. A name it registers SHALL be used as given, as the plugins capability defines for code that belongs to no plugin.

The line SHALL run with its own instruction budget, of the size the plugins capability defines. A line that exceeds it SHALL stop only the line, and the code that ran it SHALL continue. The stop SHALL NOT mark the plugin `prompt` failed, and the prompt SHALL open again as before.

A syntax error, an error raised while the line runs, and a stop by the instruction limit SHALL each be reported as a configuration error, as the configuration capability defines, with `prompt` in place of the file's path and 1 as the line. The actions the line dispatched before an error SHALL stand.

#### Scenario: Run an action
- **WHEN** two columns are open with the second focused, and the user presses Ctrl+Space, `:`, types `gband.action.focus_column_left()` and presses Enter
- **THEN** the first column is focused and the prompt is closed
- **AND** `root` is active and no window received a key

#### Scenario: Enter a mode
- **WHEN** the default configuration is in use and the user runs `gband.keymap.enter("prefix")` from the prompt
- **THEN** navigation mode is active and the sidebar's mode row shows `N`

#### Scenario: Open a plugin window
- **WHEN** the user runs `gband.win.open({ lines = { "hi" } })` from the prompt
- **THEN** a focused floating plugin window shows `hi`, and the prompt is closed

#### Scenario: Names are not namespaced
- **WHEN** the user runs `gband.action.register("greet", function() end)` from the prompt
- **THEN** `gband.action.greet` holds the action and `gband.action["prompt.greet"]` is nil

#### Scenario: Runtime error
- **WHEN** the user runs `error("boom")` from the prompt
- **THEN** the client reports the error `prompt:1: boom`

#### Scenario: Syntax error
- **WHEN** the user runs `gband.(` from the prompt
- **THEN** the client reports an error beginning `prompt:1:`, and nothing is dispatched

#### Scenario: Endless loop
- **WHEN** the user runs `while true do end` from the prompt
- **THEN** the client keeps running and reports the error `prompt:1: instruction limit exceeded`
- **AND** pressing Ctrl+Space then `:` opens the prompt again, and a line run from it takes effect

#### Scenario: Empty line
- **WHEN** the prompt shows `:` and the user presses Enter
- **THEN** the prompt closes, nothing is dispatched, and no error is reported

### Requirement: Default setup
Each key style preset of the key-style capability SHALL set up `gband.prompt` with `gband.plugin` and no options, after `gband.keylist` and before it makes its key bindings. It SHALL bind `prefix :` to `prompt.open`, as the client-attach capability's default table lists it for the modal and direct key styles, and the floating-key-style capability's "Floating bindings" for the floating key style. The default configuration SHALL set up `gband.prompt` only through `gband.keystyle.use()`. Neither the presets nor the default configuration SHALL bind a key in `root`, so `:` typed in interactive mode SHALL reach the focused window.

#### Scenario: Colon in interactive mode
- **WHEN** the default configuration is in use, `root` is active, and the user types `echo a:b` and Enter at a shell prompt
- **THEN** the focused window prints `a:b` and no prompt opens

#### Scenario: Hint for the prompt
- **WHEN** the default configuration is in use with the modal key style saved, and the user presses Ctrl+Space and then `?`
- **THEN** the key list holds the line for `:` right after the line for `?` and right before the line for `D`

#### Scenario: Prompt in the key list
- **WHEN** the default configuration is in use and the key list opens
- **THEN** a line shows `:` and `run Lua` in `PluginWindow`, between the lines for `?` and `D`

#### Scenario: Prompt with the direct key style
- **WHEN** the direct key style is saved, no `user/init.lua` exists, and the user presses Ctrl+Space then `:`
- **THEN** the Lua prompt opens and has focus, and `root` is active

#### Scenario: Prompt with the floating key style
- **WHEN** the floating key style is saved, no `user/init.lua` exists, and the user presses Ctrl+Space then `:`
- **THEN** the window list closes, the Lua prompt opens and has focus, and `root` is active
