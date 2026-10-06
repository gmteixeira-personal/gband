## MODIFIED Requirements

### Requirement: Running any binding
Enter SHALL run the binding on the cursor line, unless it binds `keylist.open`, as `gband.keymap.run("prefix", key)` runs it with the line's key. A binding to an action SHALL dispatch that action with no target, as calling its value in `gband.action` does. A binding to a function SHALL run the function. An action resolved against the view SHALL act on the focused window behind the key list, as its description says, except close window, which closes the key list, as the actions capability defines. Nothing SHALL reach the focused window. Enter on the binding of `keylist.open` SHALL do nothing.

The key list SHALL stay open, except as this paragraph defines. As the plugin-windows capability defines for a floating plugin window's own key, it SHALL stay focused through any change of the focused window or the viewed band that the binding causes. When the binding opens a floating plugin window with focus, or focuses one that was already open, that floating plugin window SHALL take focus, as the plugin-windows capability defines, and the key list SHALL close once the binding returns.

#### Scenario: Run a focus action
- **WHEN** two columns are open with the second focused, the key list is open, and the user presses Enter on the line for `h`
- **THEN** the first column is focused
- **AND** the key list is still drawn and focused, so a further Down moves its cursor line

#### Scenario: Run a resize from the list
- **WHEN** a column holds two windows with automatic heights, the top one focused, the key list is open, and the user presses Enter on the line for `+`
- **THEN** the top window grows and the key list stays open and focused

#### Scenario: Run close from the list
- **WHEN** the key list is open and the user moves to the line for `q` and presses Enter
- **THEN** the key list closes and every window stays open

#### Scenario: Run an action that opens a floating plugin window
- **WHEN** a plugin registers an action that opens a focused floating plugin window, `prefix e` binds it, the key list is open, and the user presses Enter on the line for `e`
- **THEN** the new floating plugin window has focus and the key list is no longer drawn

#### Scenario: Open the settings from the list
- **WHEN** the default configuration is in use, the key list is open, and the user moves to the line for `s` and presses Enter
- **THEN** the key list closes and the settings window is open and focused

#### Scenario: Open the settings by the line's key
- **WHEN** the default configuration is in use, the key list is open, and the user presses `s`
- **THEN** the key list closes and the settings window is open and focused

#### Scenario: Run a function binding
- **WHEN** the default configuration is in use, one window is open, the key list is open, and the user moves to the line for `n` and presses Enter
- **THEN** a second window opens
- **AND** the key list stays open and focused, and the focused window receives nothing

#### Scenario: The key list's own line does nothing
- **WHEN** the cursor line is on the binding of `keylist.open` and the user presses Enter
- **THEN** nothing is dispatched and the key list stays open
