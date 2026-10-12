## MODIFIED Requirements

### Requirement: Server selection
The `server`, `attach`, `kill-server`, `reload`, `errors`, `eval`, `cmd` and `relay` subcommands SHALL address one server, identified by its socket path. Two options select it, and each SHALL be accepted before or after the subcommand:

| option | selects |
|---|---|
| `-S <name>`, `--server <name>` | the server named `<name>` |
| `-p <path>`, `--socket <path>` | the server listening on `<path>` |

A server name SHALL be 1 to 64 bytes long and hold only ASCII letters, digits, `_` and `-`. A relative `<path>` SHALL be resolved against the working directory. A command given `-r` SHALL resolve no socket path, and SHALL pass `-S` or `-p` to the remote host, as "Remote host option" defines. Any other command SHALL resolve its socket path as follows, first match winning:

1. `-p <path>`: that path.
2. `-S <name>`: `<name>.sock` in the gband runtime directory the session-server capability defines.
3. The `GBAND` environment variable, when it is set and not empty: the path it holds.
4. `default.sock` in the gband runtime directory.

Giving both options, a name that breaks the rule, or an empty path SHALL be an invalid invocation. `gband test` addresses no server: when `-S` or `-p` is given to it, `gband` SHALL print one line to standard error saying that the option does not apply to that subcommand and exit with status 2, without creating or writing a log file. A resolved socket path SHALL be refused, with one line on standard error that names the path and the limit it breaks, and exit status 1, when it breaks the limit of the operating system: on Linux, a path longer than 107 bytes; on macOS, a path longer than 103 bytes, or a path whose directory's path is longer than 80 bytes. The macOS directory limit leaves room for the private directory the session-server capability's "Socket location" binds in. When a path breaks both macOS limits, the line SHALL name the 103-byte limit.

#### Scenario: Named server
- **WHEN** the user runs `gband -S feature attach` with `XDG_RUNTIME_DIR=/run/user/1000`
- **THEN** the client connects to `/run/user/1000/gband/feature.sock`

#### Scenario: Option after the subcommand
- **WHEN** the user runs `gband kill-server -S feature`
- **THEN** it stops the server named `feature`

#### Scenario: Explicit socket
- **WHEN** the user runs `gband attach -p target/gband.sock` in `/home/u/repo`
- **THEN** the client connects to `/home/u/repo/target/gband.sock`

#### Scenario: Default server
- **WHEN** the user runs `gband attach` outside a gband window with `XDG_RUNTIME_DIR=/run/user/1000`
- **THEN** the client connects to `/run/user/1000/gband/default.sock`

#### Scenario: Inside a window
- **WHEN** the user runs `gband kill-server` without an option in a window of the server named `feature`
- **THEN** it stops the server named `feature`

#### Scenario: Both options
- **WHEN** the user runs `gband -S a -p /tmp/b.sock attach`
- **THEN** standard error holds the usage
- **AND** the process exits with status 2

#### Scenario: Invalid name
- **WHEN** the user runs `gband -S a.b attach`
- **THEN** standard error names `a.b` as an invalid server name
- **AND** the process exits with status 2

#### Scenario: Path too long
- **WHEN** gband runs on Linux and the user runs `gband server -p` with a path of 120 bytes
- **THEN** standard error names the path and the 107-byte limit
- **AND** the process exits with status 1
- **AND** no shell is started

#### Scenario: Path too long on macOS
- **WHEN** gband runs on macOS and the user runs `gband attach -p` with a path of 104 bytes
- **THEN** standard error names the path and the 103-byte limit
- **AND** the process exits with status 1
- **AND** no server is started

#### Scenario: Directory too long on macOS
- **WHEN** gband runs on macOS and the user runs `gband attach -p` with a path of 90 bytes whose directory's path holds 81 bytes
- **THEN** standard error names the path and the 80-byte directory limit
- **AND** the process exits with status 1
- **AND** no server is started

#### Scenario: Remote server selection
- **WHEN** the user runs `gband -r u@h -p target/gband.sock attach` in `/home/u/repo`
- **THEN** the remote relay receives `-p target/gband.sock` unresolved
- **AND** no local socket path is checked

#### Scenario: Server option given to test
- **WHEN** the user runs `gband test -S feature`
- **THEN** standard error says that `-S` does not apply to `test`
- **AND** the process exits with status 2

