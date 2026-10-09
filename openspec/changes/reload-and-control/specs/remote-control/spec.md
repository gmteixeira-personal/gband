## Purpose

Defines the `gband` subcommands that let a person or a program, such as a coding agent running in a gband window, reload the configuration of a running server and its clients, read their errors, run Lua in them and run their commands, and learn from the output and the exit status whether it worked.

## ADDED Requirements

### Requirement: Control subcommands
The `reload`, `errors`, `eval` and `cmd` subcommands SHALL each send one control request, as the wire-protocol capability's "Requests" defines, to the server the command-line capability's "Server selection" resolves, for the session its "Session option" resolves. They SHALL need no terminal, SHALL start no server, and SHALL run inside a gband window as well as outside one. Each SHALL print its result to standard output, as text by default and as one JSON document followed by a newline with the option `--json`.

When no server listens on the socket path, the subcommand SHALL print one line to standard error naming the path and exit with status 1. When the server hosts no session of the name, it SHALL print one line to standard error naming the session and exit with status 1. An invalid invocation, such as an unknown option, a missing argument or `--server` given with `--client`, SHALL print one line to standard error and exit with status 2 without connecting.

In text and in JSON, a process SHALL be named `server` for the server and `client <n>` for a client, where `<n>` is its client number as the wire-protocol capability defines. In text, each line break of an error message SHALL be printed as the two characters `\n`, so that every entry holds one line.

#### Scenario: No server
- **WHEN** no server listens on `/run/user/1000/gband/default.sock` and the user runs `gband errors` with `XDG_RUNTIME_DIR=/run/user/1000` and `GBAND` unset
- **THEN** standard error holds one line naming `/run/user/1000/gband/default.sock`
- **AND** the process exits with status 1

#### Scenario: No such session
- **WHEN** the server hosts only `default` and the user runs `gband reload -s absent`
- **THEN** standard error holds one line naming `absent`
- **AND** the process exits with status 1

#### Scenario: From a window
- **WHEN** an agent runs `gband errors` in a window of the session `work` on the server named `feature`
- **THEN** it reports the server named `feature` and the clients of `work`

#### Scenario: Without a terminal
- **WHEN** a program runs `gband errors` with standard input and standard output connected to pipes
- **THEN** it prints its result to the pipe and exits

### Requirement: Load number
The server and each client SHALL number the loads of their configuration. The load when the process starts SHALL be load 1. Every later load, after a file change, by the reload action, by a control request or by a test channel's reload request, SHALL add one, whether it succeeds or fails.

#### Scenario: Load after a file change
- **WHEN** a client started, `user/init.lua` was saved once, and the user runs `gband errors --json`
- **THEN** the entry of that client has the load number 2

#### Scenario: Failed load counted
- **WHEN** a client's load number is 3 and a reload of it fails
- **THEN** its load number is 4

### Requirement: Reload subcommand
`gband reload` SHALL make the server load the server configuration again, and then make every client attached to the session load its client configuration again, each as the configuration capability's "Forced reload" defines for its side, reading the plugins' files as they are then. A client's load SHALL end with `root` active in that client. The subcommand SHALL wait until each process has answered once its load has ended and taken effect, or has timed out as "Answer timeout" defines.

In text, it SHALL print one line per process, the server first and then the clients in ascending client number: the process's name, a tab, its load number, a tab, and `ok` for a load that succeeded or `error`, a tab and the load's error message for a load that failed. A client that did not answer SHALL have the line: its name, a tab, `-`, a tab and `no answer`. The subcommand SHALL exit with status 0 when every process answered and every load succeeded, and with status 1 otherwise.

#### Scenario: Everything loads
- **WHEN** the server's load number is 2, client 1 of the session has load number 5, and the user runs `gband reload`
- **THEN** standard output holds two lines: `server`, `3` and `ok` separated by tabs, then `client 1`, `6` and `ok` separated by tabs
- **AND** the process exits with status 0

#### Scenario: A client fails
- **WHEN** client 2's `user/init.lua` raises `bad` on line 1, and the user runs `gband reload`
- **THEN** the line of `client 2` holds `error` and a message holding `bad`
- **AND** the process exits with status 1
- **AND** client 2 keeps the configuration it last loaded

#### Scenario: Plugin change takes effect
- **WHEN** a plugin's `client.lua` draws `old` in a bar, an agent rewrites it to draw `new`, and runs `gband reload`
- **THEN** every client of the session draws `new` before the subcommand exits

#### Scenario: No client attached
- **WHEN** the session has no attached client and the user runs `gband reload`
- **THEN** standard output holds only the line of the server

#### Scenario: Clients of another session
- **WHEN** one client is attached to `work` and one to `other`, and the user runs `gband reload -s work`
- **THEN** the client of `other` does not load its configuration
- **AND** standard output holds the lines of the server and of the client of `work`

### Requirement: Errors subcommand
`gband errors` SHALL report the error list of the server and of every client attached to the session, without loading any configuration. The server's error list SHALL hold every configuration and plugin error of the server's Lua since its last load without an error, oldest first. A client's error list SHALL hold the errors that `gband.errors()` returns, as the configuration capability defines, leaving out those the client received from the server.

In text, it SHALL print one line per error, the server's errors first and then each client's, in ascending client number: the process's name, a tab, its load number, a tab and the error message. A process with no error SHALL print no line. A client that did not answer SHALL have the line that "Reload subcommand" defines for it. The subcommand SHALL exit with status 0 when every process answered and no error is listed, and with status 1 otherwise.

#### Scenario: No errors
- **WHEN** no process has an error and the user runs `gband errors`
- **THEN** standard output is empty
- **AND** the process exits with status 0

#### Scenario: Error after a save
- **WHEN** an agent saves a `user/init.lua` that raises `bad` on line 2, waits for client 1's load, and runs `gband errors`
- **THEN** standard output holds one line that starts with `client 1`, a tab and the new load number, and holds `user/init.lua:2` and `bad`
- **AND** the process exits with status 1

#### Scenario: Server error listed once
- **WHEN** a plugin's `server.lua` handler raises `boom` and client 1 shows `server: ` followed by that error
- **THEN** `gband errors` prints that error on a line of `server` only

#### Scenario: Errors cleared by a good load
- **WHEN** `user/init.lua` raised an error, is fixed, and the client has loaded it
- **THEN** `gband errors` prints no line for that client

### Requirement: Eval subcommand
`gband eval <source> [<arg>...]` SHALL run the chunk `<source>` in one process, with the strings `<arg>` as `...`, and print its return values. With `-` as `<source>`, it SHALL read the chunk from standard input. The chunk SHALL run in the client that "Chosen client" defines, or in the server with the option `--server`. It SHALL run as the test-channel capability's "Evaluate a chunk" defines: in the process's current Lua state, as a callback that belongs to no plugin, with the actions it dispatches taking effect after it returns, and under the instruction limit. Its error SHALL NOT be reported as a configuration or plugin error.

In text, it SHALL print each return value on a line of its own, in order: a string as it is, `nil`, `true` and `false` as Lua writes them, a number as Lua's `tostring` writes it, and a table as JSON, as "JSON values" defines. With `--json`, it SHALL print a JSON array of the return values. It SHALL exit with status 0 when the chunk returned. When the chunk fails to compile, raises an error, hits the instruction limit, returns a value that is not plain data, or its process does not answer, it SHALL print one line to standard error holding the reason and exit with status 1.

#### Scenario: Read the client
- **WHEN** the user runs `gband eval 'return gband.side, select("#", ...)' a b`
- **THEN** standard output holds the lines `client` and `2`
- **AND** the process exits with status 0

#### Scenario: Read the server
- **WHEN** the user runs `gband eval --server 'return gband.side'`
- **THEN** standard output holds the line `server`

#### Scenario: Act in the client
- **WHEN** the session holds two columns, the first focused in the chosen client, and the user runs `gband eval 'gband.action.focus_column_right()'`
- **THEN** the chosen client focuses the second column
- **AND** standard output is empty

#### Scenario: Table as JSON
- **WHEN** the user runs `gband eval --json 'return { 1, 2 }, { a = true }'`
- **THEN** standard output holds `[[1,2],{"a":true}]`

#### Scenario: Chunk from standard input
- **WHEN** the user runs `gband eval -` with `return 40 + 2` on standard input
- **THEN** standard output holds the line `42`

#### Scenario: Error answered
- **WHEN** the user runs `gband eval 'error("boom")'`
- **THEN** standard error holds one line holding `boom`
- **AND** the process exits with status 1
- **AND** the client shows no error

### Requirement: Cmd subcommand
`gband cmd <name> [<args>]` SHALL run the command whose full name is `<name>` with the arguments that the JSON text `<args>` gives, converted as "JSON values" defines, or with an empty table when `<args>` is absent. Without `--server`, it SHALL run a command registered with `gband.cmd` in the client that "Chosen client" defines, as `gband.cmd.run` does, and its result SHALL be the value the command's function returns. With `--server`, it SHALL run the server command of that name, as a command message from the chosen client would, or with no calling client when the session has none, and its result SHALL be the server command's result.

It SHALL print the result as `gband eval` prints one return value, and print nothing for `nil`. It SHALL exit with status 0 when the command returned. When the name is unknown, the command is disabled, its function raises an error, its process does not answer, or `<args>` is not JSON, it SHALL print one line to standard error holding the reason and exit with status 1, or with status 2 for `<args>` that is not JSON. A client command that raises an error SHALL also be reported in the client, as `gband.cmd.run` reports it.

#### Scenario: Run a client command
- **WHEN** `user/init.lua` registers the command `greet` whose function returns `"hi " .. args.who`, and the user runs `gband cmd greet '{"who":"you"}'`
- **THEN** standard output holds the line `hi you`
- **AND** the process exits with status 0

#### Scenario: Run a server command
- **WHEN** the agent-status plugin is installed, a window waits, and the user runs `gband cmd --server agent-status.next_waiting`
- **THEN** the chosen client focuses the waiting window

#### Scenario: Unknown command
- **WHEN** the user runs `gband cmd absent`
- **THEN** standard error holds one line naming `absent`
- **AND** the process exits with status 1

#### Scenario: Arguments that are not JSON
- **WHEN** the user runs `gband cmd greet '{who'`
- **THEN** the process exits with status 2 without connecting

### Requirement: Chosen client
`gband eval` and `gband cmd` SHALL accept the option `--client <n>`, which chooses the client of the session whose client number is `<n>`. Without it, the chosen client SHALL be the client of the session that most recently sent the server a key, paste or mouse message, or, when none has, the client that attached to the session last. When `<n>` names no client attached to the session, or the session has no client and `--server` is not given, the subcommand SHALL print one line to standard error saying so and exit with status 1.

#### Scenario: Last client typed in
- **WHEN** clients 1 and 2 are attached to the session, the user typed last in client 1, and runs `gband eval 'return 1'` from a window
- **THEN** the chunk runs in client 1

#### Scenario: Client by number
- **WHEN** clients 1 and 2 are attached to the session and the user runs `gband eval --client 2 'return 1'`
- **THEN** the chunk runs in client 2

#### Scenario: No client
- **WHEN** the session has no attached client and the user runs `gband eval 'return 1'`
- **THEN** standard error holds one line saying that the session has no client
- **AND** the process exits with status 1

### Requirement: Answer timeout
The server SHALL wait at most 5 seconds for a client to answer a control message. A client that has not answered by then SHALL count as not answering, and the server SHALL leave its later answer unused. The server SHALL apply no timeout to its own work on a control request.

#### Scenario: Stopped client
- **WHEN** client 2 of the session is stopped with SIGSTOP and the user runs `gband reload`
- **THEN** within 6 seconds standard output holds a line of `client 2`, `-` and `no answer` separated by tabs
- **AND** the process exits with status 1

### Requirement: JSON output
With `--json`, `gband reload` and `gband errors` SHALL print a JSON array that holds one object per process, in the order their text lines would take. Each object SHALL hold `process`, `"server"` or `"client"`; `client`, the client number, for a client only; and `answered`, a boolean. An object whose process answered SHALL also hold `load`, its load number. For `gband reload`, it SHALL also hold `error`, the load's error message, or `null` when the load succeeded. For `gband errors`, it SHALL also hold `errors`, an array of the error messages, oldest first. Every process the request addressed SHALL have an object, with or without errors. Error messages SHALL be kept as they are, line breaks included.

#### Scenario: Reload as JSON
- **WHEN** the server's load succeeds as load 3, client 1 answers with load 6 and no error, and the user runs `gband reload --json`
- **THEN** standard output holds `[{"process":"server","answered":true,"load":3,"error":null},{"process":"client","client":1,"answered":true,"load":6,"error":null}]` with its keys in any order

#### Scenario: Errors as JSON
- **WHEN** no process has an error and the session has client 1, and the user runs `gband errors --json`
- **THEN** standard output holds an array of two objects, each with `errors` set to `[]`

### Requirement: JSON values
A plain data value, as the plugin-bridge capability defines, SHALL be written as JSON as follows: nil as `null`; a boolean as itself; an integer and a finite float as a number; a float that is not finite as `null`; a string as a JSON string, with each byte sequence that is not UTF-8 replaced by U+FFFD; a table whose keys are exactly the integers 1 to n, for n of at least 1, as an array in key order; and any other table, an empty one included, as an object whose keys are its string keys and the decimal text of its integer keys.

A JSON value SHALL be read as a plain data value as follows: `null` as nil, a boolean as itself, a number without a fraction or exponent that fits a 64-bit integer as an integer and any other number as a float, a string as a string, an array as a table with the keys 1 to n, and an object as a table with its keys as strings.

#### Scenario: Array and object
- **WHEN** a chunk returns `{ "a", "b" }` and `{ [1] = "a", x = 2 }` to `gband eval --json`
- **THEN** standard output holds `[["a","b"],{"1":"a","x":2}]`

#### Scenario: Arguments read from JSON
- **WHEN** a command's function returns `type(args.n) .. " " .. type(args.list)` and the user runs `gband cmd <name> '{"n":1,"list":[1,2]}'`
- **THEN** standard output holds the line `number table`
