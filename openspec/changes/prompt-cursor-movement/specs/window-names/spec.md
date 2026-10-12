## MODIFIED Requirements

### Requirement: Rename prompt
The `setup` of the bundled plugin `gband.prompt`, as the lua-prompt capability defines it, SHALL also register the action `prompt.rename` with the description `rename the window`.

Dispatching `prompt.rename` while `gband.view()` names a focused window that runs a program SHALL open the rename prompt for that window and make `root` the active table, as `gband.keymap.enter("root")` does. With no such window, it SHALL do nothing. The rename prompt SHALL be a floating plugin window that belongs to the plugin `prompt`, takes focus, and has the border, height, width and place of the Lua prompt, as the lua-prompt capability's "Prompt plugin window" defines, with the title `rename`. It SHALL hold one line: the typed text in the group `PluginWindow`, with no leading `:`, with the cursor cell in the group `PromptCursor`, placed and fitted to the content area as the Lua prompt's line is. Its line SHALL start as the window's manual name, or empty when it has none, with the cursor after the last character.

The rename prompt SHALL edit its line as the lua-prompt capability's "Editing the line" defines, except that Enter SHALL close it and rename the window it was opened for, as `gband.window.rename` does with the line, and SHALL do nothing more when that window is no longer in the layout.

At most one of the Lua prompt and the rename prompt SHALL be open. Opening either SHALL first close the other, dropping its line. Dispatching `prompt.rename` while the rename prompt is open SHALL focus it and keep its line, its cursor and its window.

#### Scenario: Action registered
- **WHEN** a configuration calls `gband.plugin("gband.prompt")` and reads `gband.action.list()`
- **THEN** it holds an entry named `prompt.rename` with the description `rename the window`

#### Scenario: Rename the focused window
- **WHEN** the focused window runs bash with no manual name, the user presses Ctrl+Space then Shift+N, types `logs` and presses Enter
- **THEN** the rename prompt closes and the window's top border shows `logs`

#### Scenario: Prompt starts with the manual name
- **WHEN** the focused window's manual name is `logs` and the user presses Ctrl+Space then Shift+N
- **THEN** the rename prompt's content row reads `logs` followed by the cursor cell

#### Scenario: Edit the middle of the manual name
- **WHEN** the focused window's manual name is `logs`, and the user presses Ctrl+Space, Shift+N, Left twice, types `-x`, then presses Enter
- **THEN** the window's top border shows `lo-xgs`

#### Scenario: Clear by renaming to nothing
- **WHEN** the focused window's manual name is `logs`, no title is set, and the user presses Ctrl+Space, Shift+N, Ctrl+U, then Enter
- **THEN** the window's top border shows `bash`

#### Scenario: Escape keeps the name
- **WHEN** the rename prompt is open for a window named `logs`, the user types `x` and presses Escape
- **THEN** the prompt closes and the window's name stays `logs`

#### Scenario: Empty band
- **WHEN** the viewed band holds no window and a binding dispatches `prompt.rename`
- **THEN** no floating plugin window opens

#### Scenario: Window closed while renaming
- **WHEN** the rename prompt is open for window 2, window 2 closes, and the user presses Enter
- **THEN** the prompt closes and nothing else happens

#### Scenario: Lua prompt replaces the rename prompt
- **WHEN** the rename prompt is open and the user presses Ctrl+Space then `:`
- **THEN** only the Lua prompt is drawn, and it has focus
