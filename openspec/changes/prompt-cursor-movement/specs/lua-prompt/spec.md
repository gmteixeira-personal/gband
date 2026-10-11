## MODIFIED Requirements

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
- **WHEN** the prompt shows `:ab`, the user presses Ctrl+Space then `l`, which focuses another column, then left-clicks the cell holding `b` and types `x`
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
