## ADDED Requirements

### Requirement: Attach to a remote host
`gband attach` given `-r <destination>`, as the command-line capability's "Remote host option" defines, SHALL connect through the SSH transport the transport capability defines, and SHALL otherwise behave as this capability defines for a local attach, with these differences:

- It SHALL NOT apply "Refuse to nest", because the local `GBAND` variable names a server on this machine.
- It SHALL NOT start a server itself. The remote relay starts one when none is running, as the transport capability's "Relay subcommand" defines.
- It SHALL send an empty working directory with its attach request, so that a session the server creates takes the server's working directory, as the session-server capability's "Attach creates a missing session" defines.
- Its key bindings, modes, plugins, settings and client configuration SHALL be loaded from the local configuration, as for a local attach.

"Require a terminal" SHALL be checked before ssh starts, so `gband -r u@h > out.txt` starts no ssh.

#### Scenario: Remote attach from a gband window
- **WHEN** the user runs `gband -r u@h` in a window of the local default server
- **THEN** the client attaches to the session named `default` on `u@h`

#### Scenario: Remote session starts in the login directory
- **WHEN** no server runs on `u@h`, whose login directory is `/home/u`, and the user runs `gband -r u@h` in `/home/local/repo`
- **THEN** `pwd` in the window prints `/home/u`

#### Scenario: New remote session on a running server
- **WHEN** a server started in `/home/u` runs on `u@h` and the user runs `gband -r u@h -s work` in `/home/local/repo`
- **THEN** `pwd` in the window of `work` prints `/home/u`

#### Scenario: Local bindings over ssh
- **WHEN** the local `user/init.lua` binds `alt+h` to `gband.action.focus_column_left` and the user runs `gband -r u@h`, opens a second window and presses Alt+H
- **THEN** the first window is focused
- **AND** neither window receives the key

#### Scenario: Remote attach without a terminal
- **WHEN** the user runs `gband -r u@h > out.txt`
- **THEN** standard error says that `gband attach` needs a terminal
- **AND** the process exits with status 1
- **AND** ssh is not run

## MODIFIED Requirements

### Requirement: Version handshake
The client SHALL complete the wire protocol's handshake, and receive the server's info, before it changes the terminal. When the server speaks a different protocol version, a debug build of the client SHALL replace the server as the "Server from a different build" requirement defines, when that requirement allows it. Otherwise the client SHALL print one line to standard error naming both versions and the `gband kill-server` command that stops the running server, carrying the `-r` option and the selection option the client was given, and exit with status 1. The terminal SHALL be left as it was. In this capability, a debug build is one compiled with debug assertions, as cargo's default `dev` profile does, and a release build is any other.

#### Scenario: Incompatible server, debug build
- **WHEN** a server speaking protocol version 2 is running from the client's executable path and a debug-build client speaking version 1 attaches
- **THEN** the version 2 server and its shell are stopped
- **AND** the client attaches to a new version 1 server

#### Scenario: Incompatible server, release build
- **WHEN** a server speaking protocol version 2 is running and a release-build client speaking version 1 attaches
- **THEN** standard error names versions 1 and 2 and `gband kill-server`
- **AND** the client exits with status 1
- **AND** the terminal is not left in raw mode or the alternate screen

#### Scenario: Incompatible named server
- **WHEN** a server named `feature` speaks another protocol version and a release-build client runs `gband -S feature attach`
- **THEN** standard error names `gband -S feature kill-server`

#### Scenario: Incompatible remote server
- **WHEN** the server on `u@h` speaks another protocol version and the user runs `gband -r u@h -S feature`
- **THEN** standard error names `gband -r u@h -S feature kill-server`
- **AND** the remote server keeps running

### Requirement: Server from a different build
The client SHALL compare the executable identity in the server's info with the identity of its own executable, as the wire protocol defines it. When they differ, it SHALL record a warning in the client log naming the server's process id. A client attached through the SSH transport SHALL skip the comparison, record nothing, replace no server and print no note, because the server's identity names a file on another machine.

A debug build SHALL replace the server only when the server's process runs from the same executable path as the client, which is the case after a rebuild of that executable. It SHALL read that path from the operating system, using the process id in the server's pid record. A server from any other path, or whose path cannot be read, SHALL be treated as a release build treats it. This keeps a client built in one worktree from stopping a server built in another.

To replace the server, the client SHALL, before it changes the terminal, send detach and close the connection, stop the server as `gband kill-server` does for the same socket path, start a new server from its own executable as the "Start a server when none is running" requirement defines, and attach to that one. A client SHALL replace a server at most once per invocation. When the server it reaches after replacing still differs, the client SHALL continue as a release build does. When the old server does not stop within 5 seconds, the client SHALL print one line to standard error naming its process id, and exit with status 1.

A release build, and a debug build that does not replace, SHALL attach to the server unchanged and, after printing the line that "Leaving the client" defines, print to standard error: `gband: the server runs a different gband build; stop it with <command> and attach again`, where `<command>` is `gband kill-server` carrying the selection option the client was given.

#### Scenario: Debug build after a rebuild
- **WHEN** a server started from a debug build is running, the user runs `cargo build`, which replaces the executable, and runs `gband attach` from the new build
- **THEN** the old server and its shell are stopped
- **AND** the client attaches to a new server whose info carries the client's own executable identity

#### Scenario: Debug build, same executable
- **WHEN** a debug-build client attaches to a server started from the same unchanged executable
- **THEN** the client attaches to that server and its shell keeps running

#### Scenario: Debug build from another worktree
- **WHEN** a server started from `/w/a/target/debug/gband` is running and a debug-build client from `/w/b/target/debug/gband` attaches to it
- **THEN** the client attaches to that server and its shell keeps running
- **AND** after the user detaches, standard error holds the different-build note

#### Scenario: Release build after an upgrade
- **WHEN** a server started from a release build is running and the user replaces the executable and runs `gband attach`
- **THEN** the client attaches to the running server and its shell
- **AND** after the user detaches, standard error holds `gband: the server runs a different gband build; stop it with gband kill-server and attach again` after `[detached]`

#### Scenario: Note names the selected server
- **WHEN** a release-build client run as `gband -S feature attach` reaches a server from a different build and the user detaches
- **THEN** standard error holds `gband: the server runs a different gband build; stop it with gband -S feature kill-server and attach again`

#### Scenario: Server does not stop
- **WHEN** a debug-build client must replace a server that is still running 5 seconds after being told to stop
- **THEN** the client prints one line to standard error naming that server's process id
- **AND** it exits with status 1 without changing the terminal

#### Scenario: Remote server
- **WHEN** a client attaches with `-r u@h` to a server whose executable identity differs from the client's, and the user detaches
- **THEN** the server and its shell keep running
- **AND** standard error holds no different-build note
