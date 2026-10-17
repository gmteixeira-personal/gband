# window-sixel Specification

## Purpose

Defines how gband carries a window program's sixel images to the attached clients: the emulator collecting the program's sixel DCS sequences and recording each one's cursor position while leaving the window's screen unchanged, the server forwarding each sequence with its position to the clients whose terminal supports sixel and advertising Primary Device Attributes attribute `4` to the program by them, the client drawing each forwarded sixel at its position, reporting its terminal's cell size in pixels so the program sizes images to the cells, startup detection of the outer terminal's support, and the `window_sixel` option that turns it off.

## Requirements

### Requirement: Sixel sequences of a window program
The server SHALL recognise a sixel sequence in the output of a window's program: a device control string `ESC P <params> q <payload> ESC \`, where `<params>` holds only decimal digits and `;`, `q` is the sixel final byte, and `<payload>` is the sixel data. It SHALL recognise a sequence whose bytes arrive across several writes, ending it at the first `ESC \`. It SHALL record the position of the window's cursor, 1-based from the top-left of the window's screen, at the point the sequence begins, as the sequence's placement. Recognising a sequence SHALL change nothing on the window's screen, because the emulator drops the device control string.

The server SHALL ignore a device control string that is not a sixel, including one whose bytes before the final byte hold an intermediate such as `$`, and SHALL stop buffering and ignore a sixel sequence that has not ended within 16 MiB, so a program cannot grow the server's memory without bound.

#### Scenario: A sixel sequence is collected with its position
- **WHEN** the program in window 1 moves its cursor to row 3, column 5 and writes `\x1bPq#0;2;0;0;0#0~~\x1b\\`
- **THEN** the server collects one sixel sequence of window 1 holding those bytes, with the position row 3, column 5
- **AND** window 1's screen is unchanged

#### Scenario: A sequence split across writes
- **WHEN** the program in window 1 writes `\x1bPq#0;2;0;0;0` and then `#0~~\x1b\\`
- **THEN** the server collects one sixel sequence holding the joined bytes

#### Scenario: A non-sixel device control string is ignored
- **WHEN** the program in window 1 writes `\x1bP$qm\x1b\\`
- **THEN** the server collects no sixel sequence

### Requirement: Delivery to the attached clients
The server SHALL forward each sixel sequence of a session, with its placement, in the order the program wrote them, to every client attached to the session whose terminal supports sixel, in a sixel message as the wire-protocol capability defines. A sequence written while no supporting client is attached SHALL reach no client. A sequence written before a supporting client attached SHALL never reach that client. The server SHALL keep no sixel sequence after forwarding it, so a client that attaches later receives none sent before it attached.

The client SHALL draw each sixel message's bytes when its `window_sixel` option is `true`. A client whose `window_sixel` is `false` SHALL report no sixel support at attach, so the server forwards it no sixel message.

#### Scenario: Two supporting clients
- **WHEN** two clients whose terminals support sixel are attached to session `default` and the program in window 1 writes a sixel sequence
- **THEN** each client receives one sixel message naming window 1, the sequence's bytes and its position

#### Scenario: Order is kept
- **WHEN** the program in window 1 writes two sixel sequences `A` and then `B`
- **THEN** a supporting client receives a sixel message holding `A` and then one holding `B`

#### Scenario: No supporting client
- **WHEN** a client whose terminal does not support sixel is attached and the program in window 1 writes a sixel sequence
- **THEN** that client receives no sixel message

#### Scenario: Written before attaching
- **WHEN** a program writes a sixel sequence while no client is attached, and a supporting client attaches afterwards
- **THEN** the client receives no sixel message

### Requirement: Drawing a forwarded sixel at its position
The client SHALL draw a forwarded sixel between frames, after that frame's cells are drawn, by saving the cursor, moving it to the window's on-screen position plus the sequence's recorded row and column, writing the sequence's bytes unchanged, and restoring the cursor. It SHALL draw sixels in the order their messages arrive. The client SHALL write nothing for a sixel whose anchor cell lies outside the window's drawn area, so a sixel is never drawn at a wrong position. The client SHALL NOT clip a drawn sixel to the window, track it, or redraw it when the window scrolls or resizes; a program re-emits the image on its own redraw.

#### Scenario: Drawn at the recorded position
- **WHEN** a window is drawn with its top-left at screen column 10, row 2, and the client receives a sixel message for that window with the position row 3, column 5 and some bytes
- **THEN** the client moves the cursor to screen column 14, row 4, writes those bytes, and restores the cursor

#### Scenario: Anchor outside the window
- **WHEN** a sixel message's position names a row below the window's drawn area
- **THEN** the client writes no sixel bytes to its terminal

### Requirement: Offering sixel to the program
The server SHALL add attribute `4` to a window program's Primary Device Attributes reply, making it `\e[?62;4;22c`, only when at least one client attached to the session reported its terminal supports sixel, as the session-server capability's "Terminal query replies" defines. When no attached client reported sixel support, the reply SHALL stay `\e[?62;22c`.

#### Scenario: A supporting client is attached
- **WHEN** a client whose terminal supports sixel is attached and the program in window 1 writes `\e[c`
- **THEN** the program reads `\e[?62;4;22c` from its terminal

#### Scenario: No supporting client is attached
- **WHEN** no attached client supports sixel and the program in window 1 writes `\e[c`
- **THEN** the program reads `\e[?62;22c` from its terminal

### Requirement: Cell size in pixels
At startup, the client SHALL measure its outer terminal's cell size in pixels. When the terminal's window size, read with `TIOCGWINSZ`, holds nonzero columns, rows, pixel width and pixel height, the cell width SHALL be the pixel width divided by the columns and the cell height the pixel height divided by the rows, rounded down. Otherwise the client SHALL take the cell size from the terminal's reply `\e[6;<height>;<width>t` to the query `\e[16t`, written in the same write as the kitty graphics probe and before its Primary Device Attributes query. When neither gives a nonzero width and height, the cell size SHALL be unknown. The client SHALL report the cell size, or that it is unknown, in its attach request and in every resize message, as the wire-protocol capability defines. A resize message SHALL report the size `TIOCGWINSZ` gives at that resize when it gives one, and the startup cell size otherwise.

A session's cell size SHALL be the cell size last reported by the most recently attached client of the session that reported sixel support and a known cell size at attach. When that client detaches, the session's cell size SHALL become that of the next most recently attached such client, and SHALL be unknown when none remains. A resize message from any other client SHALL NOT change it.

While the session's cell size is known, the pixel size of a window's PTY SHALL be its columns times the cell width by its rows times the cell height; while it is unknown, the pixel size SHALL be 0 by 0. The server SHALL set the pixel size whenever it sets a PTY's size, as the session-server capability's "Window resizes" defines. When the session's cell size changes, the server SHALL set the new pixel size, once, on every window that "Window resizes" resizes when the session settles, and SHALL leave every other window's PTY until it is next resized. A change of the pixel size alone SHALL NOT send the clients a snapshot.

While the session's cell size is known, the server SHALL answer the cell size query `\e[16t` and the text area size query `\e[14t` of a window's program, as the session-server capability's "Terminal query replies" defines; while it is unknown, both SHALL stay unanswered.

#### Scenario: Cell size from the window size
- **WHEN** the outer terminal's `TIOCGWINSZ` gives 100 columns, 30 rows, 1000 pixels wide and 600 pixels high
- **THEN** the client's attach request reports the cell size 10 by 20 pixels

#### Scenario: Cell size from the query reply
- **WHEN** the outer terminal's `TIOCGWINSZ` gives 0 by 0 pixels and the terminal answers `\e[16t` with `\e[6;19;9t`
- **THEN** the client's attach request reports the cell size 9 by 19 pixels

#### Scenario: Unknown cell size
- **WHEN** the outer terminal's `TIOCGWINSZ` gives 0 by 0 pixels and the terminal does not answer `\e[16t`
- **THEN** the client's attach request reports an unknown cell size

#### Scenario: PTY pixel size
- **WHEN** a client that supports sixel attaches reporting the cell size 10 by 20 pixels, and window 1's PTY is 80 columns by 24 rows
- **THEN** window 1's PTY reports 800 by 480 pixels

#### Scenario: The latest sixel client wins
- **WHEN** a sixel client reporting 10 by 20 pixels attaches, then a sixel client reporting 8 by 16 pixels attaches, and the program in window 1 writes `\e[16t`
- **THEN** the program reads `\e[6;16;8t` from its terminal

#### Scenario: Fallback on detach
- **WHEN** the second of those two clients detaches and the program in window 1 writes `\e[16t`
- **THEN** the program reads `\e[6;20;10t` from its terminal

#### Scenario: No sixel client
- **WHEN** only a client without sixel support is attached and the program in window 1 writes `\e[16t` followed by `\e[c`
- **THEN** the only bytes the program reads from its terminal are `\e[?62;22c`
- **AND** window 1's PTY reports 0 by 0 pixels

### Requirement: Detecting the terminal's support
At startup, before it decodes any key, the client SHALL detect whether its outer terminal supports sixel from the Primary Device Attributes reply it requests for the kitty graphics probe, as the window-graphics capability's "Detecting the terminal's support" defines, in that probe's single write. The terminal SHALL be taken to support sixel when the Primary Device Attributes reply's parameters hold `4`. The client SHALL report sixel support in its attach request as the terminal's support AND its `window_sixel` option, as the wire-protocol capability defines.

#### Scenario: A terminal that advertises sixel
- **WHEN** the outer terminal's Primary Device Attributes reply is `\x1b[?62;4;22c`
- **THEN** the client's attach request reports sixel support

#### Scenario: A terminal that does not advertise sixel
- **WHEN** the outer terminal's Primary Device Attributes reply is `\x1b[?62;22c`
- **THEN** the client's attach request reports no sixel support

### Requirement: Window sixel option
The client SHALL have the built-in option `window_sixel`, alongside those the configuration capability's "Options and their values" lists, with the value a boolean, the default `true` and the description `whether a window program's sixel images are forwarded to your terminal`. A value of the wrong type SHALL be rejected as "Options and their values" defines. `gband.opt.list()` SHALL hold an entry for it with the type `"boolean"`. The default configuration SHALL set it to its default, with the other client options.

#### Scenario: Forwarded by default
- **WHEN** no `user/init.lua` exists, the outer terminal supports sixel, and the program in a window writes a sixel sequence
- **THEN** the client draws that sequence's bytes

#### Scenario: Turned off
- **WHEN** `user/init.lua` calls `gband.set { window_sixel = false }` and a program writes a sixel sequence
- **THEN** the client draws no sixel bytes
- **AND** the server forwards it no sixel message

#### Scenario: Wrong type
- **WHEN** line 2 of `user/init.lua` is `gband.set { window_sixel = "no" }`
- **THEN** loading fails with an error at `user/init.lua` line 2 naming `window_sixel`

#### Scenario: Default value
- **WHEN** the default configuration is evaluated alone and reads `gband.opt.window_sixel`
- **THEN** it reads `true`
