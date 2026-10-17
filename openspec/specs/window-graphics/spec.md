# window-graphics Specification

## Purpose

Defines how gband carries a window program's kitty graphics images to the attached clients: the emulator collecting the program's kitty graphics APC sequences while leaving the window's screen unchanged, the server forwarding each sequence to the clients whose terminal supports kitty graphics and answering the support query by them, placement by Unicode placeholder, startup detection of the outer terminal's support, and the `window_graphics` option that turns it off.

## Requirements

### Requirement: Graphics sequences of a window program
The server SHALL recognise a kitty graphics sequence in the output of a window's program: an APC string `ESC _ G <control> ; <payload> ESC \`, where `<control>` holds the kitty control keys and `<payload>` is optional. It SHALL recognise a sequence whose bytes arrive across several writes, ending it at the first `ESC \`. Recognising a sequence SHALL change nothing on the window's screen, the same as today, because the emulator drops the APC string.

The server SHALL ignore an APC string that does not begin with `G`, and SHALL stop buffering and ignore a graphics sequence that has not ended within 16 MiB, so a program cannot grow the server's memory without bound.

#### Scenario: A transmit sequence is collected
- **WHEN** the program in window 1 writes `\x1b_Gf=24,s=1,v=1,a=T;AAAA\x1b\\`
- **THEN** the server collects one graphics sequence of window 1 holding those bytes
- **AND** window 1's screen is unchanged

#### Scenario: A sequence split across writes
- **WHEN** the program in window 1 writes `\x1b_Gf=24,s=1,v=1,` and then `a=T;AAAA\x1b\\`
- **THEN** the server collects one graphics sequence holding the joined bytes

#### Scenario: A non-graphics APC string is ignored
- **WHEN** the program in window 1 writes `\x1b_qMODE\x1b\\`
- **THEN** the server collects no graphics sequence

### Requirement: Delivery to the attached clients
The server SHALL forward each graphics sequence of a session, in the order the program wrote them, to every client attached to the session, in a graphics message as the wire-protocol capability defines. A sequence written while no client is attached SHALL reach no client. A sequence written before a client attached SHALL never reach that client. The server SHALL keep no graphics sequence after forwarding it to the attached clients, so a client that attaches later receives none sent before it attached.

The client SHALL write each graphics message's bytes to its terminal unchanged, between frames, in the order the messages arrive, when its `window_graphics` option is `true`. A client whose `window_graphics` is `false` SHALL report no graphics support at attach, so the server forwards it no graphics message.

#### Scenario: Two clients
- **WHEN** two clients that support graphics are attached to session `default` and the program in window 1 writes a graphics sequence
- **THEN** each client receives one graphics message naming window 1 and the sequence's bytes

#### Scenario: Order is kept
- **WHEN** the program in window 1 writes two graphics sequences `A` and then `B`
- **THEN** an attached client receives a graphics message holding `A` and then one holding `B`

#### Scenario: Written before attaching
- **WHEN** a program writes a graphics sequence while no client is attached, and a client attaches afterwards
- **THEN** the client receives no graphics message

#### Scenario: Other session
- **WHEN** one client is attached to session `a`, another to session `b`, and a program in session `a` writes a graphics sequence
- **THEN** only the client attached to `a` receives a graphics message

### Requirement: Placement by Unicode placeholder
The server SHALL carry an image's placement to a client only through the window's screen: a program places an image with U+10EEEE placeholder cells whose foreground colour holds the image id, and those cells reach the client in the window's snapshot and diff, as the wire-protocol capability defines, unchanged. The client SHALL NOT remap the foreground colour of a cell whose text begins with U+10EEEE when it applies a colorscheme palette, so a placeholder cell's image id survives to the terminal.

#### Scenario: Placeholder foreground is not remapped
- **WHEN** a colorscheme palette is set, and a window holds a cell whose text is U+10EEEE with a foreground of the 256-colour index 3
- **THEN** the client draws that cell with the 256-colour index 3 as its foreground

#### Scenario: A placeholder cell is one column wide
- **WHEN** a window holds a cell whose text is U+10EEEE followed by two combining marks
- **THEN** the client draws it in one column

### Requirement: Answering the support query
The server SHALL answer a kitty graphics support query, a graphics sequence whose control holds `a=q`, by writing `ESC _ G i=<id> ; OK ESC \` back to the window's program, where `<id>` is the query's `i` value, or `0` when it names none. It SHALL write this answer only when at least one client attached to the session reported its terminal supports kitty graphics. When no attached client reported support, the server SHALL write nothing for the query. The server SHALL NOT forward a support query to the clients. The answer SHALL join the window emulator's write-back bytes, in order with the program's other query replies, as the terminal-emulator capability's "Output for the program" defines.

#### Scenario: A supporting client is attached
- **WHEN** a client that supports graphics is attached and the program in window 1 writes `\x1b_Gi=31,s=1,v=1,a=q;AAAA\x1b\\`
- **THEN** the program reads `\x1b_Gi=31;OK\x1b\\` from its terminal

#### Scenario: No supporting client is attached
- **WHEN** no attached client supports graphics and the program in window 1 writes `\x1b_Gi=31,a=q;AAAA\x1b\\`
- **THEN** the program reads nothing from its terminal for the query

#### Scenario: A query without an id
- **WHEN** a client that supports graphics is attached and the program writes `\x1b_Ga=q,s=1,v=1;AAAA\x1b\\`
- **THEN** the program reads `\x1b_Gi=0;OK\x1b\\` from its terminal

### Requirement: Detecting the terminal's support
At startup, before it decodes any key, the client SHALL detect whether its outer terminal supports kitty graphics. It SHALL write a kitty graphics query followed by a primary device attributes request (`ESC [ c`) to the terminal, and read the terminal's replies until the device attributes reply (`ESC [ ? ... c`) or a 1 second timeout. The terminal SHALL be taken to support kitty graphics when a graphics reply (`ESC _ G ... ESC \`) arrives before the device attributes reply. The client SHALL report graphics support in its attach request as the terminal's support AND its `window_graphics` option, as the wire-protocol capability defines.

#### Scenario: A terminal that answers the graphics query
- **WHEN** the outer terminal replies to the graphics query with `\x1b_Gi=1;OK\x1b\\` before its device attributes reply
- **THEN** the client's attach request reports graphics support

#### Scenario: A terminal that answers only device attributes
- **WHEN** the outer terminal replies only with its device attributes reply
- **THEN** the client's attach request reports no graphics support

### Requirement: Window graphics option
The client SHALL have the built-in option `window_graphics`, alongside those the configuration capability's "Options and their values" lists, with the value a boolean, the default `true` and the description `whether a window program's kitty graphics are forwarded to your terminal`. A value of the wrong type SHALL be rejected as "Options and their values" defines. `gband.opt.list()` SHALL hold an entry for it with the type `"boolean"`. The default configuration SHALL set it to its default, with the other client options.

#### Scenario: Forwarded by default
- **WHEN** no `user/init.lua` exists, the outer terminal supports graphics, and the program in a window writes a graphics sequence
- **THEN** the client writes that sequence's bytes to its terminal

#### Scenario: Turned off
- **WHEN** `user/init.lua` calls `gband.set { window_graphics = false }` and a program writes a graphics sequence
- **THEN** the client writes no graphics bytes to its terminal
- **AND** the server forwards it no graphics message

#### Scenario: Wrong type
- **WHEN** line 2 of `user/init.lua` is `gband.set { window_graphics = "no" }`
- **THEN** loading fails with an error at `user/init.lua` line 2 naming `window_graphics`

#### Scenario: Default value
- **WHEN** the default configuration is evaluated alone and reads `gband.opt.window_graphics`
- **THEN** it reads `true`
