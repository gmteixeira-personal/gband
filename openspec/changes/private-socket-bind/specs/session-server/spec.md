## MODIFIED Requirements

### Requirement: Socket location
The server SHALL listen on a Unix stream socket at the socket path the command-line capability's server selection resolves. Without a selection option, that path is `default.sock` in the gband runtime directory. The runtime directory SHALL be `$XDG_RUNTIME_DIR/gband/` when `XDG_RUNTIME_DIR` is set to an absolute path, and `/tmp/gband-<uid>/` otherwise, where `<uid>` is the user's numeric user id. When the socket path lies in the runtime directory, the server SHALL create the runtime directory with mode `0700` when it is missing. When the socket path was given with `-p`, the server SHALL NOT create its parent directory, and a missing parent SHALL be refused with one line on standard error naming the directory, and exit status 1. The server SHALL create the socket file with mode `0600`. The socket file SHALL have mode `0600` from the moment it appears at the socket path, so no other user can connect to it at any time, whatever the user's file mode creation mask and whatever the permissions of the socket's directory. Creating the socket SHALL NOT change the server process's file mode creation mask, even for a moment, so a file, directory or program another thread of the server creates meanwhile gets the mask the server started with. The server and `gband attach` SHALL resolve the same path from the same environment and options.

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

#### Scenario: Servers started together in one process
- **WHEN** eight servers on eight socket paths start at the same time in one process whose file mode creation mask is `022`
- **THEN** every socket file has mode `0600`
- **AND** the process's file mode creation mask is still `022` once every server listens

#### Scenario: Long explicit socket path
- **WHEN** the user runs `gband server -p` with a socket path of exactly 107 bytes in an existing directory
- **THEN** the server listens on that path with mode `0600`

#### Scenario: Missing parent of an explicit socket
- **WHEN** the user runs `gband server -p /tmp/missing/s.sock` and `/tmp/missing/` does not exist
- **THEN** standard error names `/tmp/missing/`
- **AND** the process exits with status 1
- **AND** `/tmp/missing/` is not created
