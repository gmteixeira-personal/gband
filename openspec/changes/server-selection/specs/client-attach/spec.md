## MODIFIED Requirements

### Requirement: Connect to the session
`gband attach` SHALL connect to the socket path the command-line capability's server selection resolves from its own environment and options. Before it sends anything, the client SHALL read the user id of the process listening on the socket from the operating system. When that user id differs from the client's own, the client SHALL print one line to standard error naming the socket path and the listening user id, and exit with status 1 without changing the terminal.

#### Scenario: Server already running
- **WHEN** a server is running and the user runs `gband attach` with the same environment and options
- **THEN** the client attaches to that server's session

#### Scenario: Named server
- **WHEN** servers named `a` and `b` are running and the user runs `gband -S b attach`
- **THEN** the client attaches to the session of the server named `b`

#### Scenario: Socket owned by another user
- **WHEN** a process of another user listens on `/tmp/s.sock` and the user runs `gband attach -p /tmp/s.sock`
- **THEN** standard error names `/tmp/s.sock` and that user's id
- **AND** the process exits with status 1
- **AND** nothing is sent on the socket

### Requirement: Start a server when none is running
When nothing accepts connections on the socket path, `gband attach` SHALL start `gband server` from its own executable, addressing the same socket path with `-p`, detached from the client's terminal and session, with the client's working directory and environment, and with standard input, output and error on `/dev/null`. It SHALL then connect to it. If no connection succeeds within 5 seconds, the client SHALL print one line to standard error naming the socket path and the server log, and exit with status 1.

#### Scenario: First attach starts the session
- **WHEN** no server is running and the user runs `gband attach` in `~/repos/gband`
- **THEN** a shell prompt appears in the client
- **AND** `pwd` in the pane prints the path of `~/repos/gband`

#### Scenario: Named server started on demand
- **WHEN** no server named `feature` is running and the user runs `gband -S feature attach`
- **THEN** a server listening on `feature.sock` in the runtime directory starts
- **AND** `echo $GBAND` in its pane prints that socket path

#### Scenario: Started server survives its terminal
- **WHEN** `gband attach` started the server and the terminal emulator window running the client is closed
- **THEN** the server and its shell keep running

#### Scenario: Server cannot start
- **WHEN** the server that `gband attach` starts exits before accepting a connection
- **THEN** the client prints one line to standard error naming the socket path and the server log
- **AND** it exits with status 1 within 5 seconds

### Requirement: Refuse to nest
`gband attach` SHALL refuse to run when the `GBAND` environment variable is set, is not empty, and holds the socket path the client resolved, because the client would then show the pane's own server inside one of its panes. It SHALL print one line to standard error saying so, and exit with status 1 without connecting. Attaching from a pane to a server on another socket path SHALL be allowed.

#### Scenario: Attach inside a pane
- **WHEN** the user runs `gband attach` without an option in a gband pane
- **THEN** standard error says that the client is already inside a pane of that server
- **AND** the process exits with status 1

#### Scenario: Same server named explicitly
- **WHEN** the user runs `gband -S feature attach` in a pane of the server named `feature`
- **THEN** the process exits with status 1 without connecting

#### Scenario: Another server from a pane
- **WHEN** the user runs `gband -S feature attach` in a pane of the default server
- **THEN** the client attaches to the server named `feature`

### Requirement: Version handshake
The client SHALL complete the wire protocol's handshake, and receive the server's info, before it changes the terminal. When the server speaks a different protocol version, a debug build of the client SHALL replace the server as the "Server from a different build" requirement defines, when that requirement allows it. Otherwise the client SHALL print one line to standard error naming both versions and the `gband kill-server` command that stops the running server, carrying the selection option the client was given, and exit with status 1. The terminal SHALL be left as it was. In this capability, a debug build is one compiled with debug assertions, as cargo's default `dev` profile does, and a release build is any other.

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

### Requirement: Server from a different build
The client SHALL compare the executable identity in the server's info with the identity of its own executable, as the wire protocol defines it. When they differ, it SHALL record a warning in the client log naming the server's process id.

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
