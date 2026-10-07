## MODIFIED Requirements

### Requirement: Running the line
Enter SHALL close the prompt and then run its line. An empty line SHALL run nothing. The prompt SHALL run the line by calling `gband.eval(line, "prompt")`, as the plugins capability's "Evaluate source" defines, inside the callback of the prompt's Enter, and SHALL discard what it returns. So the line SHALL be compiled as Lua source text, never as a binary chunk, and SHALL run protected.

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
