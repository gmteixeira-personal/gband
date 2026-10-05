# local-actions Specification

## Purpose

Defines the functions client Lua uses to act on the user's own machine and terminal: desktop notifications, the bell, the clipboard and opening URLs or files, so that a server event becomes something the user notices wherever the client runs.

## Requirements

### Requirement: Notifications
`gband.notify(text, opts)` SHALL ask the client's terminal to show a desktop notification of `text`, a string, with `opts.title`, an optional string. The client option `notify_style` SHALL choose how:

| `notify_style` | output to the client's terminal |
|---|---|
| `"osc9"` | `ESC ] 9 ; <text> BEL`; the title is left out |
| `"osc777"` | `ESC ] 777 ; notify ; <title> ; <text> BEL`, with the title `gband` when none is given |
| `"bell"` | `BEL` |
| `"none"` | nothing |

Control characters, and `;` in the title, SHALL be removed from the title and the text, and each SHALL be cut to 1024 bytes at a character boundary. The client SHALL write the sequence between frames, so that it never splits a frame's output. An argument of the wrong type SHALL be an error at the line of the call. `gband.notify` SHALL be callable only in a callback.

#### Scenario: Default style
- **WHEN** `notify_style` is unset and a handler calls `gband.notify("Agent waiting: build")`
- **THEN** the client writes `\x1b]9;Agent waiting: build\x07` to its terminal

#### Scenario: Title with osc777
- **WHEN** `notify_style` is `"osc777"` and a handler calls `gband.notify("done", { title = "agent" })`
- **THEN** the client writes `\x1b]777;notify;agent;done\x07`

#### Scenario: Escape removed
- **WHEN** a handler calls `gband.notify("a\27]52;c;eA==\7b")`
- **THEN** the text written is `a]52;c;eA==b` inside one notification sequence

### Requirement: Bell
`gband.bell()` SHALL write `BEL` to the client's terminal between frames. It SHALL be callable only in a callback.

#### Scenario: Ring
- **WHEN** a binding function calls `gband.bell()`
- **THEN** the client writes `\x07` to its terminal

### Requirement: Clipboard
`gband.clipboard(text)` SHALL set the clipboard of the terminal the client runs in to `text`, a string of at most 1 MiB, by writing `ESC ] 52 ; c ; <base64 of text> BEL` between frames. It SHALL be callable only in a callback, and a text of the wrong type or size SHALL be an error at the line of the call.

#### Scenario: Copy
- **WHEN** a handler calls `gband.clipboard("hi")`
- **THEN** the client writes `\x1b]52;c;aGk=\x07` to its terminal

### Requirement: Open locally
`gband.open(target)` SHALL open `target`, a non-empty string naming a URL or a path, with the opener of the machine the client runs on: `open` on macOS and `xdg-open` elsewhere. It SHALL start the opener with `target` as its only argument, without a shell, with no input and its output discarded, and SHALL NOT wait for it to finish. It SHALL return `true` when the opener started, and `false` when it could not start, recording the reason in the client's log. It SHALL be callable only in a callback.

#### Scenario: Open a URL
- **WHEN** a binding function calls `gband.open("https://example.com/a b")` on Linux
- **THEN** the client starts `xdg-open` with the single argument `https://example.com/a b`

#### Scenario: Opener missing
- **WHEN** no opener is installed and a binding function calls `gband.open("x")`
- **THEN** `gband.open` returns `false`, the client log records the reason, and the client keeps running
