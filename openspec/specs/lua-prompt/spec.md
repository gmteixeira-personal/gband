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

The floating plugin window SHALL hold one line: `:` followed by the typed text, in the group `PluginWindow`, then one space in the group `PromptCursor`, which marks the cursor. When that line is wider than the content area, the fewest leading characters SHALL be left out that make the rest, with the cursor, fit in the content area, so the cursor stays shown at the end. The line SHALL be fitted again each time the content area's width changes.

The plugin SHALL give the group `PromptCursor` the default `{ reverse = true }`, as the highlights capability defines defaults.

#### Scenario: Prompt on the bottom rows
- **WHEN** the ribbon area is 60×24, as the default status line leaves it on an 80×24 terminal, and a binding dispatches `prompt.open`
- **THEN** a focused floating plugin window titled `lua` spans columns 0 to 59 and rows 21 to 23 of the ribbon area
- **AND** its content row reads `:` followed by one cell drawn reversed

#### Scenario: Typed text and the cursor
- **WHEN** the prompt is open and the user types `abc`
- **THEN** the content row reads `:abc` and the cell after `c` is drawn in `PromptCursor`'s resolved style

#### Scenario: Long line shows its end
- **WHEN** the prompt's content area is 10 columns wide and the user types `abcdefghijkl`
- **THEN** the content row reads `defghijkl` followed by the cursor cell

#### Scenario: Theme sets the cursor
- **WHEN** `user/init.lua` calls `gband.hl.set("PromptCursor", { bg = "#ff0000" })` and the prompt is drawn on a truecolor terminal
- **THEN** the cursor cell has background `#ff0000`

### Requirement: Editing the line
While the prompt is focused, it SHALL edit its line as follows:

| input | effect |
|---|---|
| a key that types a character, as the plugin-windows capability defines it | append the character, `j`, `k`, `q` and space included |
| a paste | append the pasted text, with each `\r\n` pair and every other control character turned into one space |
| Backspace | remove the last character; on an empty line, close the prompt and run nothing |
| Ctrl+U | clear the line |
| Escape | close the prompt and run nothing |
| Enter | run the line, as "Running the line" defines |

A character SHALL be one Unicode scalar value. Every other key SHALL take the plugin-windows capability's default, or be discarded, and SHALL leave the line unchanged. Nothing typed into the prompt SHALL reach the focused window.

#### Scenario: Letters that scroll elsewhere
- **WHEN** the prompt is open and the user types `jkq`
- **THEN** the prompt stays open and shows `:jkq`

#### Scenario: Backspace
- **WHEN** the prompt shows `:ab` and the user presses Backspace
- **THEN** the prompt shows `:a`

#### Scenario: Backspace on an empty line
- **WHEN** the prompt shows `:` and the user presses Backspace
- **THEN** the prompt closes and nothing runs

#### Scenario: Clear the line
- **WHEN** the prompt shows `:abc` and the user presses Ctrl+U
- **THEN** the prompt shows `:`

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
- **THEN** navigation mode is active and the status line's mode segment shows `navigation`

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
The default configuration SHALL set up `gband.prompt` with `gband.plugin` and no options, after `gband.keylist` and before it makes its key bindings. It SHALL bind `prefix :` to `prompt.open`, as the client-attach capability's default table lists it. It SHALL bind no key in `root`, so `:` typed in interactive mode SHALL reach the focused window.

#### Scenario: Colon in interactive mode
- **WHEN** the default configuration is in use, `root` is active, and the user types `echo a:b` and Enter at a shell prompt
- **THEN** the focused window prints `a:b` and no prompt opens

#### Scenario: Hint for the prompt
- **WHEN** the default configuration is in use, the client's terminal is 80×60, and the user presses Ctrl+Space
- **THEN** the hints segment shows the hint `: run Lua` right after the hint `? list the keys` and right before the hint `D detach`

#### Scenario: Prompt in the key list
- **WHEN** the default configuration is in use and the key list opens
- **THEN** a line shows `:` and `run Lua` in `PluginWindow`, between the lines for `?` and `D`
