# lua-commands Specification

## Purpose

Defines the registry of named commands that the configuration and plugins add through `gband.cmd`, so that later features can list and run them. This change surfaces commands to Lua only: no palette and no command-line integration.

## Requirements

### Requirement: Register commands
`gband.cmd.register(name, fn, opts)` SHALL register a command under its full name, as the plugins capability defines, and return that full name. `fn` SHALL be a Lua function. `opts.desc` SHALL be an optional string describing the command, and `opts.args` an optional list of strings naming its arguments, for display. A name that is not a non-empty string, an `fn` that is not a function, an `opts` field of the wrong type, or a full name already registered SHALL be an error at the line of the call. Commands SHALL be registered only while the configuration loads; calling `gband.cmd.register` later SHALL be an error.

#### Scenario: Register and list
- **WHEN** `user/init.lua` calls `gband.cmd.register("greet", fn, { desc = "Say hello", args = { "who" } })`
- **THEN** `gband.cmd.list()` holds one entry with `name` `greet`, `desc` `Say hello` and `args` `{ "who" }`

#### Scenario: Duplicate name
- **WHEN** line 3 of `user/init.lua` registers `greet` a second time
- **THEN** loading fails with an error at `user/init.lua` line 3 naming `greet`

### Requirement: Run commands
`gband.cmd.run(name, args)` SHALL call the function of the command whose full name is `name` with `args`, or with an empty table when `args` is nil, as a callback that belongs to the command's plugin. It SHALL return `true` when the function returns, and `false` when the function raises an error, which SHALL be reported as the configuration capability defines, or when the command is disabled. An unknown `name` SHALL be an error raised at the line of the call. Called inside a callback, the command's function MAY dispatch actions as a binding function does.

#### Scenario: Run with arguments
- **WHEN** a binding function calls `gband.cmd.run("greet", { who = "you" })`
- **THEN** the command's function receives a table whose `who` is `"you"`
- **AND** `gband.cmd.run` returns `true`

#### Scenario: Command dispatches an action
- **WHEN** a command's function calls `gband.action.focus_column_right()` and a binding function runs that command with the first of two columns focused
- **THEN** the second column is focused

#### Scenario: Failing command
- **WHEN** a command's function raises an error and a binding function runs it
- **THEN** `gband.cmd.run` returns `false` and the client shows the error

#### Scenario: Unknown command
- **WHEN** a binding function calls `gband.cmd.run("absent")` on line 8 of `user/init.lua`
- **THEN** the client shows an error at `user/init.lua` line 8 naming `absent`

### Requirement: List commands
`gband.cmd.list()` SHALL return one table per registered command, in ascending byte order of full names, each holding `name`, `desc` and `args` as registered. Changing a returned table SHALL NOT change the registry.

#### Scenario: Order
- **WHEN** the commands `b` and `a` are registered in that order
- **THEN** `gband.cmd.list()` returns `a` then `b`
