## MODIFIED Requirements

### Requirement: Socket location
The server SHALL listen on a Unix stream socket at the socket path the command-line capability's server selection resolves. Without a selection option, that path is `default.sock` in the gband runtime directory. The runtime directory SHALL be `$XDG_RUNTIME_DIR/gband/` when `XDG_RUNTIME_DIR` is set to an absolute path, and `/tmp/gband-<uid>/` otherwise, where `<uid>` is the user's numeric user id. When the socket path lies in the runtime directory, the server SHALL create the runtime directory with mode `0700` when it is missing. When the socket path was given with `-p`, the server SHALL NOT create its parent directory, and a missing parent SHALL be refused with one line on standard error naming the directory, and exit status 1. The server SHALL create the socket file with mode `0600`. The server and `gband attach` SHALL resolve the same path from the same environment and options.

#### Scenario: Runtime directory from the environment
- **WHEN** the user runs `gband server` with `XDG_RUNTIME_DIR=/run/user/1000`
- **THEN** the server listens on `/run/user/1000/gband/default.sock`

#### Scenario: No runtime directory in the environment
- **WHEN** the user with uid 1000 runs `gband server` with `XDG_RUNTIME_DIR` unset
- **THEN** the server listens on `/tmp/gband-1000/default.sock`
- **AND** `/tmp/gband-1000/` has mode `0700`

#### Scenario: Named server
- **WHEN** the user runs `gband server -S feature` with `XDG_RUNTIME_DIR=/run/user/1000`
- **THEN** the server listens on `/run/user/1000/gband/feature.sock`

#### Scenario: Socket is private
- **WHEN** a server is listening
- **THEN** its socket file has mode `0600`

#### Scenario: Missing parent of an explicit socket
- **WHEN** the user runs `gband server -p /tmp/missing/s.sock` and `/tmp/missing/` does not exist
- **THEN** standard error names `/tmp/missing/`
- **AND** the process exits with status 1
- **AND** `/tmp/missing/` is not created

### Requirement: Runtime directory ownership
The socket gives whoever can open it control of a shell. When the socket path lies in the runtime directory, the server SHALL refuse to start when the runtime directory exists but is not owned by the user, or grants any permission to group or others. It SHALL print one line to standard error that names the directory and the problem, and exit with status 1. A socket path given with `-p` SHALL rely on the socket's own mode and on the client's check of the server's user instead.

#### Scenario: Directory open to others
- **WHEN** `/tmp/gband-1000/` exists with mode `0777` and the user with uid 1000 runs `gband server` with `XDG_RUNTIME_DIR` unset
- **THEN** standard error names `/tmp/gband-1000/` and its permissions
- **AND** the process exits with status 1
- **AND** no shell is started

#### Scenario: Explicit socket in a shared directory
- **WHEN** the user runs `gband server -p /tmp/s.sock`
- **THEN** the server listens on `/tmp/s.sock` with mode `0600`

### Requirement: Single server per socket
At most one server SHALL serve a socket path. Servers selected with different options that resolve to the same socket path SHALL share one guard. A server that finds another live server on its socket path SHALL print one line to standard error naming the socket path, and exit with status 1 without starting a shell. A socket file that no live server holds SHALL be replaced. Servers on different socket paths SHALL run side by side, each with its own panes.

#### Scenario: Second server refused
- **WHEN** a server is running and the user runs `gband server` again with the same environment and options
- **THEN** the second process prints that a server is already running on the socket path
- **AND** it exits with status 1
- **AND** the first server and its shell keep running

#### Scenario: Same socket through both options
- **WHEN** a server started with `-S feature` is running and the user runs `gband server -p "$XDG_RUNTIME_DIR/gband/feature.sock"`
- **THEN** the second process prints that a server is already running on that socket path
- **AND** it exits with status 1

#### Scenario: Stale socket replaced
- **WHEN** a socket file remains from a server that was killed with SIGKILL and the user runs `gband server`
- **THEN** the new server starts and listens on that path

#### Scenario: Two named servers
- **WHEN** a server named `a` is running and the user runs `gband server -S b`
- **THEN** both servers run, each with its own shell

### Requirement: Pid record
While it holds the single-server guard, the server SHALL record its process id, in decimal followed by a newline, in its lock file. The lock file's path SHALL be the socket path with a final `.sock` replaced by `.lock`, or with `.lock` appended when the socket path does not end in `.sock`. The record SHALL be in place before the server accepts its first connection.

#### Scenario: Pid readable while running
- **WHEN** a server is running without a selection option
- **THEN** `default.lock` in the runtime directory holds that server's process id

#### Scenario: Named server record
- **WHEN** a server named `feature` is running
- **THEN** `feature.lock` in the runtime directory holds that server's process id

#### Scenario: Explicit socket record
- **WHEN** a server started with `-p /tmp/s` is running
- **THEN** `/tmp/s.lock` holds that server's process id

### Requirement: Kill-server subcommand
`gband kill-server` SHALL stop the server for the socket path it resolves, using only the server's pid record and signals, so that it stops a server of any protocol version. It SHALL leave every server on another socket path running. When no server holds the single-server guard, it SHALL print one line to standard error naming the socket path and exit with status 1. Otherwise it SHALL send SIGTERM to the recorded process and wait up to 5 seconds for the guard to be released. It SHALL exit with status 0, printing nothing, when the guard is released. When the guard is still held after 5 seconds, it SHALL print one line to standard error naming the process id and exit with status 1. It SHALL NOT create the runtime directory or any directory of a socket path.

#### Scenario: Stop a running server
- **WHEN** a server is running and the user runs `gband kill-server`
- **THEN** the server and its shell are no longer running
- **AND** `gband kill-server` exits with status 0 and prints nothing

#### Scenario: Stop one named server
- **WHEN** servers named `a` and `b` are running and the user runs `gband -S a kill-server`
- **THEN** the server named `a` is no longer running
- **AND** the server named `b` and its shell keep running

#### Scenario: No server running
- **WHEN** no server is running and the user runs `gband kill-server`
- **THEN** standard error names the socket path and says no server is running
- **AND** the process exits with status 1
- **AND** the runtime directory is not created

#### Scenario: Next attach starts a fresh server
- **WHEN** the user runs `gband kill-server` and then `gband attach`
- **THEN** the client shows a new shell with a different process id
