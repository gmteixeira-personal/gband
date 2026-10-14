# window-clipboard Specification

## Purpose

Defines how a program inside a gband window sets the clipboard of the terminal each client runs in: the server recognises the program's OSC 52 write, sends it to the attached clients, and each client copies it unless its Lua configuration turns copying off.

## Requirements

### Requirement: Clipboard writes of a window program
The server SHALL recognise a clipboard write in the output of a window's program: `ESC ] 52 ; <selection> ; <data>`, ended by `BEL` or by `ESC \`. It SHALL accept the write when `<selection>` is empty or holds the character `c`, and `<data>` is base64 that decodes to at most 1 MiB. The written text SHALL be the decoded bytes read as UTF-8, with each invalid sequence replaced by U+FFFD.

The server SHALL ignore every other OSC 52 sequence: a `<selection>` without `c` that is not empty, `<data>` that is `?` or is not base64, and decoded data over 1 MiB. It SHALL never answer an OSC 52 sequence, so a program cannot read any clipboard through gband. Recognising a write SHALL change nothing on the window's screen.

#### Scenario: Write with the clipboard selection
- **WHEN** the program in window 1 writes `\x1b]52;c;aGk=\x07`
- **THEN** the server records a clipboard write of window 1 with the text `hi`

#### Scenario: Empty selection
- **WHEN** the program in window 1 writes `\x1b]52;;aGk=\x1b\\`
- **THEN** the server records a clipboard write of window 1 with the text `hi`

#### Scenario: Primary selection alone
- **WHEN** the program in window 1 writes `\x1b]52;p;aGk=\x07`
- **THEN** the server records no clipboard write

#### Scenario: Query is never answered
- **WHEN** the program in window 1 writes `\x1b]52;c;?\x07`
- **THEN** the server records no clipboard write and writes nothing to window 1's program

#### Scenario: Text over 1 MiB
- **WHEN** the program in window 1 writes a clipboard write whose data decodes to 1 MiB and one byte
- **THEN** the server records no clipboard write

### Requirement: Delivery to the attached clients
The server SHALL keep the latest clipboard write of each session: the writing window and its text. It SHALL send that write in a clipboard message, as the wire-protocol capability defines, to every client attached to the session. A client SHALL receive each write at most once. When a program makes several writes before a client is sent one, the client SHALL receive only the latest of them. A write made while no client is attached, and a write made before a client attached, SHALL never reach that client.

#### Scenario: Two clients
- **WHEN** two clients are attached to session `default` and the program in window 1 writes `\x1b]52;c;aGk=\x07`
- **THEN** each client receives one clipboard message naming window 1 and the text `hi`

#### Scenario: Other session
- **WHEN** one client is attached to session `a`, another to session `b`, and a program in session `a` makes a clipboard write
- **THEN** only the client attached to `a` receives a clipboard message

#### Scenario: Write before attaching
- **WHEN** a program makes a clipboard write while no client is attached, and a client attaches afterwards
- **THEN** the client receives no clipboard message

#### Scenario: Latest write wins
- **WHEN** a program writes the text `a` and then `b` before the server sends either to a client
- **THEN** the client receives one clipboard message, with the text `b`

### Requirement: Window clipboard option
The client SHALL have the built-in option `window_clipboard`, alongside those the configuration capability's "Options and their values" lists, with the value a boolean, the default `true` and the description `whether a window program's clipboard write sets your terminal's clipboard`. A value of the wrong type SHALL be rejected as "Options and their values" defines. `gband.opt.list()` SHALL hold an entry for it with the type `"boolean"`. The default configuration SHALL set it to its default, with the other client options.

On each clipboard message, the client SHALL first emit the built-in event `WindowClipboard` with `window` and `text`, as the lua-events capability defines. Then, when `window_clipboard` is `true`, it SHALL write `ESC ] 52 ; c ; <base64 of text> BEL` to its terminal between frames, the sequence `gband.clipboard(text)` writes. When `window_clipboard` is `false`, the client SHALL write nothing for the message, and a `WindowClipboard` handler MAY still call `gband.clipboard`.

#### Scenario: Copied by default
- **WHEN** no `user/init.lua` exists and the program in the focused window runs `printf '\033]52;c;aGk=\007'`
- **THEN** the client writes `\x1b]52;c;aGk=\x07` to its terminal

#### Scenario: Turned off
- **WHEN** `user/init.lua` calls `gband.set { window_clipboard = false }` and a program makes a clipboard write
- **THEN** the client writes no OSC 52 sequence to its terminal
- **AND** `WindowClipboard` handlers still run

#### Scenario: Copy from one window only
- **WHEN** `user/init.lua` sets `window_clipboard` to `false` and registers a `WindowClipboard` handler that calls `gband.clipboard(event.text)` only when `event.window` is 2, and the programs in windows 1 and 2 each make a clipboard write
- **THEN** the client writes one OSC 52 sequence, holding window 2's text

#### Scenario: Wrong type
- **WHEN** line 2 of `user/init.lua` is `gband.set { window_clipboard = "no" }`
- **THEN** loading fails with an error at `user/init.lua` line 2 naming `window_clipboard`

#### Scenario: Default value
- **WHEN** the default configuration is evaluated alone and reads `gband.opt.window_clipboard`
- **THEN** it reads `true`
