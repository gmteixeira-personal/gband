# window-names Specification

## Purpose

Defines the name of every window that runs a program: the automatic name the server takes from the program's title or the foreground command, the manual name that overrides it, the number that tells apart windows of one band with the same name, the title drawn on the window's top border, and the rename prompt and Lua call that set the manual name.

## Requirements

### Requirement: Automatic name
Every window that runs a program SHALL have an automatic name, which the server keeps:

1. The window's title, as "Window title" defines, while one is set.
2. Otherwise the window's foreground command, as "Foreground command" defines.

A drawn window, as the session-server capability defines it, SHALL have no name of any kind.

#### Scenario: Title set by the program
- **WHEN** a window runs bash and the user runs `printf '\033]2;build\007'`
- **THEN** the window's automatic name is `build`

#### Scenario: No title set
- **WHEN** a window runs bash, no program in it has set a title, and the user runs `sleep 100`
- **THEN** the window's automatic name is `sleep`

#### Scenario: Prompt title wins over the command
- **WHEN** a window runs a shell that sets the title `user@host:~` at each prompt, and the user runs `sleep 100`
- **THEN** the window's automatic name stays `user@host:~` while `sleep` runs

### Requirement: Window title
The server SHALL keep a title for each window that runs a program, which no title is set at first:

- OSC 0 and OSC 2 SHALL set the title to their text, which SHALL be every parameter after the first, joined with `;`. OSC 1 SHALL leave the title unchanged.
- Control characters SHALL be removed from the text, and leading and trailing whitespace trimmed. A text that is then empty SHALL clear the title, so no title is set.
- `CSI 22 ; 0 t` and `CSI 22 ; 2 t` SHALL push the title, or the absence of one, onto the window's title stack. `CSI 22 t` SHALL do the same. The stack SHALL hold at most 10 entries, and a push onto a full stack SHALL drop the oldest one.
- `CSI 23 ; 0 t`, `CSI 23 ; 2 t` and `CSI 23 t` SHALL pop the latest entry from the stack and make it the title, or clear the title when the entry is the absence of one. A pop from an empty stack SHALL leave the title unchanged.
- `CSI 22 ; 1 t` and `CSI 23 ; 1 t` SHALL change neither the title nor the stack.

The title SHALL be kept when the program that set it exits, until another sequence changes it.

#### Scenario: Title holding a semicolon
- **WHEN** the program in a window writes `\033]0;make; make test\007`
- **THEN** the window's title is `make; make test`

#### Scenario: Empty title clears it
- **WHEN** the window's title is `build`, the program writes `\033]2;\007`, and `sleep 100` runs in the foreground
- **THEN** no title is set and the window's automatic name is `sleep`

#### Scenario: Pushed title is restored
- **WHEN** the window's title is `user@host:~` and a program writes `\033[22;0t`, then `\033]2;VIM - notes\007`, then `\033[23;0t`
- **THEN** the window's title is `user@host:~` again

#### Scenario: Pushed absence clears the title
- **WHEN** no title is set and a program writes `\033[22;2t`, then `\033]2;VIM - notes\007`, then `\033[23;2t`
- **THEN** no title is set

#### Scenario: Control characters removed
- **WHEN** the program writes OSC 2 with the text ` a\tb `
- **THEN** the window's title is `ab`

### Requirement: Foreground command
A window's foreground command SHALL be the base name of the first argument of the foreground process group leader of the window's terminal, with a leading `-` removed, as a login shell's first argument begins with one. On Linux the server SHALL read the argument from `/proc/<pid>/cmdline`, and SHALL use `/proc/<pid>/comm` when the argument is empty. When neither can be read, or on a system without `/proc`, the foreground command SHALL be the base name of the command that started the window's program.

The server SHALL check every window's foreground process group after each output of the window's program and at least once per second, so that the automatic name follows the foreground command within one second.

#### Scenario: Command in the foreground
- **WHEN** a window runs bash and the user runs `/usr/bin/sleep 100`
- **THEN** within one second the window's automatic name is `sleep`
- **AND** once `sleep` exits, within one second the automatic name is `bash`

#### Scenario: Login shell
- **WHEN** a window's program is a login shell whose first argument is `-zsh`
- **THEN** the window's automatic name is `zsh`

#### Scenario: Command that writes nothing
- **WHEN** a window runs bash and the user runs `sleep 100`, which writes no output
- **THEN** within one second the window's automatic name is `sleep`

### Requirement: Manual name
A window that runs a program SHALL have at most one manual name. While it has one, the window's name SHALL be its manual name. Otherwise it SHALL be its automatic name. The server SHALL keep the manual name for as long as the window lives, whatever its title and foreground command do.

`gband.window.rename(window, name)` SHALL send a rename of the named window to the server, which SHALL set the window's manual name to `name` with leading and trailing whitespace trimmed. A `name` that is nil, or empty once trimmed, SHALL clear the manual name, so the window's name is its automatic name again. A window number not in the client's layout, a drawn window, a `name` that is neither a string nor nil, or a `name` holding a control character SHALL be an error at the line of the call.

#### Scenario: Manual name overrides the title
- **WHEN** window 1's title is `build` and a binding function calls `gband.window.rename(1, "logs")`, and the program then sets the title `test`
- **THEN** window 1's name is `logs`

#### Scenario: Clear the manual name
- **WHEN** window 1's manual name is `logs`, its title is `test`, and a binding function calls `gband.window.rename(1, "")`
- **THEN** window 1's name is `test`

#### Scenario: Control character rejected
- **WHEN** a binding function calls `gband.window.rename(1, "a\nb")`
- **THEN** the call raises an error and window 1's name is unchanged

#### Scenario: Unknown window
- **WHEN** the client's layout holds no window 9 and a binding function calls `gband.window.rename(9, "x")`
- **THEN** the call raises an error naming window 9

### Requirement: Names shared by every client
The server SHALL send each attached client a window name message, as the wire-protocol capability defines it, whenever a window's automatic name or manual name changes, and SHALL send none when neither changed. Every client SHALL show the same name for a window. A client that attaches SHALL receive each window's names after the snapshots, as the wire-protocol capability defines the attach order.

#### Scenario: Rename seen by another client
- **WHEN** two clients are attached to one session and the first renames window 1 `logs`
- **THEN** the second client draws window 1 with the title `logs`

#### Scenario: Name kept across detach
- **WHEN** window 1 is renamed `logs`, its client detaches, and a client attaches to the session again
- **THEN** that client draws window 1 with the title `logs`

### Requirement: Shown name
A window's shown name SHALL be its name, followed by ` #` and a number when another window of the same band that runs a program has the same name, compared as exact text. The windows of one band sharing a name SHALL be numbered from 1 in layout order: the band's columns left to right, each column's windows top to bottom, then the band's floating windows in their floating list order. A name no other window of the band has SHALL be shown with no number. The client SHALL work out the shown names again from each layout and each window name message, so a window's number follows the layout.

#### Scenario: Two shells
- **WHEN** a band holds columns A and B, each with one window running bash, and no title is set in either
- **THEN** A's shown name is `bash #1` and B's is `bash #2`

#### Scenario: Number follows the layout
- **WHEN** a band holds columns A and B, both shown `bash #1` and `bash #2`, and the user moves B's column to the left of A's
- **THEN** B's shown name is `bash #1` and A's is `bash #2`

#### Scenario: Floating windows numbered last
- **WHEN** a band holds a tiled window and a floating window, both named `bash`
- **THEN** the tiled window is shown `bash #1` and the floating window `bash #2`

#### Scenario: Unique name has no number
- **WHEN** a band holds two windows running bash and one running `vim`
- **THEN** the `vim` window's shown name is `vim`

#### Scenario: Other bands do not count
- **WHEN** band 1 holds one window running bash and band 2 holds another
- **THEN** each is shown `bash`

#### Scenario: Manual names numbered too
- **WHEN** a band holds two windows, both renamed `logs`
- **THEN** they are shown `logs #1` and `logs #2`

### Requirement: Border title
While titles are on, as "Titles turned off" defines, the client SHALL draw the shown name of every tiled and floating window that runs a program on the window's top border, after drawing the border. It SHALL start at the border's second column. It SHALL be cut, leaving out whole characters, as `gband.ui.width` measures them, to a room of:
- the border's width less 2 cells, when the client draws no decorations on that border;
- the border's width less 4 cells and less the width of the decorations, when the client draws them, as the window-decorations capability defines.

So at least one cell of the border SHALL stay between the title and the decorations. A room of 0 cells SHALL draw no title. The title SHALL be drawn in the window's border style, as the client-attach capability defines it, distinct for the focused window. When the window's border definition, as the borders capability defines it, does not draw the top side, the client SHALL draw no title. A drawn window SHALL have no title drawn.

#### Scenario: Title on a tile
- **WHEN** the client's 80×24 terminal sets the screen area, the client has no bar, the only window sits in a column of width 1/2, and its name is `vim`
- **THEN** the tile's top row shows `vim` from column 1 to column 3, with the border's top side on either side

#### Scenario: Long title cut
- **WHEN** a tile is 10 columns wide and its shown name is `cargo test --workspace`
- **THEN** its top row shows `cargo te` from column 1 to column 8

#### Scenario: Title cut before decorations
- **WHEN** a tile is 20 columns wide, its shown name is `cargo test --workspace`, and its decorations are the spans `[_]`, `[□]` and `[X]`
- **THEN** its top row shows `cargo t` from column 1 to column 7, `─` on column 8, and `[_][□][X]` from column 9 to column 17

#### Scenario: Short title beside decorations
- **WHEN** a tile is 40 columns wide, its shown name is `vim`, and its decorations are the spans `[_]`, `[□]` and `[X]`
- **THEN** its top row shows `vim` from column 1 to column 3, `─` from column 4 to column 28, and `[_][□][X]` from column 29 to column 37

#### Scenario: Focused title styled as the focused border
- **WHEN** two tiles are drawn with the first focused
- **THEN** the first tile's title is drawn in `WindowBorderFocused`'s resolved style and the second's in `WindowBorder`'s

#### Scenario: No top side
- **WHEN** `user/init.lua` sets `tile_border_sides` to `{ "left", "right" }`
- **THEN** no tile's top row shows its name

#### Scenario: Title follows the program
- **WHEN** a tile shows the title `bash` and the user runs `vim` in it
- **THEN** within one second the tile's top border shows `vim`

### Requirement: Rename prompt
The `setup` of the bundled plugin `gband.prompt`, as the lua-prompt capability defines it, SHALL also register the action `prompt.rename` with the description `rename the window`.

Dispatching `prompt.rename` while `gband.view()` names a focused window that runs a program SHALL open the rename prompt for that window and make `root` the active table, as `gband.keymap.enter("root")` does. With no such window, it SHALL do nothing. The rename prompt SHALL be a floating plugin window that belongs to the plugin `prompt`, takes focus, and has the border, height, width and place of the Lua prompt, as the lua-prompt capability's "Prompt plugin window" defines, with the title `rename`. It SHALL hold one line: the typed text in the group `PluginWindow`, with no leading `:`, then one space in the group `PromptCursor`, fitted to the content area as the Lua prompt's line is. Its line SHALL start as the window's manual name, or empty when it has none.

The rename prompt SHALL edit its line as the lua-prompt capability's "Editing the line" defines, except that Enter SHALL close it and rename the window it was opened for, as `gband.window.rename` does with the line, and SHALL do nothing more when that window is no longer in the layout.

At most one of the Lua prompt and the rename prompt SHALL be open. Opening either SHALL first close the other, dropping its line. Dispatching `prompt.rename` while the rename prompt is open SHALL focus it and keep its line and its window.

#### Scenario: Action registered
- **WHEN** a configuration calls `gband.plugin("gband.prompt")` and reads `gband.action.list()`
- **THEN** it holds an entry named `prompt.rename` with the description `rename the window`

#### Scenario: Rename the focused window
- **WHEN** the focused window runs bash with no manual name, the user presses Ctrl+Space then Shift+N, types `logs` and presses Enter
- **THEN** the rename prompt closes and the window's top border shows `logs`

#### Scenario: Prompt starts with the manual name
- **WHEN** the focused window's manual name is `logs` and the user presses Ctrl+Space then Shift+N
- **THEN** the rename prompt's content row reads `logs` followed by the cursor cell

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

### Requirement: Titles turned off
The client SHALL read the environment variable `GBAND_WINDOW_TITLES` when it starts. When its value is `off`, the client SHALL draw no border title, whatever the `window_titles` option holds. When it is unset or `on`, the client SHALL draw border titles, as "Border title" defines, while the client option `window_titles` is `true`, and SHALL draw none while it is `false`. Any other value SHALL be logged as a warning in the client's log and read as unset.

Turning titles off SHALL change only drawing. The server SHALL keep and send names, `gband.layout()` SHALL hold `name` and `manual_name`, and the rename prompt SHALL rename, as with titles on. A reload that changes `window_titles` SHALL take effect from the next frame the client draws.

#### Scenario: Option off
- **WHEN** `user/init.lua` calls `gband.set({ window_titles = false })` and the only window runs bash
- **THEN** the tile's top border shows `─` across its width
- **AND** `gband.layout()` gives the window the `name` `bash`

#### Scenario: Environment turns titles off
- **WHEN** a client starts with `GBAND_WINDOW_TITLES=off` and `user/init.lua` sets `window_titles` to `true`
- **THEN** no tile's top border shows its name

#### Scenario: Unknown value
- **WHEN** a client starts with `GBAND_WINDOW_TITLES=no`
- **THEN** the client's log holds a warning naming `GBAND_WINDOW_TITLES` value `no`
- **AND** tiles show their names on their top borders

#### Scenario: Turned back on by a reload
- **WHEN** `user/init.lua` sets `window_titles` to `false`, the user changes it to `true`, and the configuration reloads
- **THEN** the next frame shows each tile's name on its top border
