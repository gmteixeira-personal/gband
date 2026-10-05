## MODIFIED Requirements

### Requirement: Server selection
The `server`, `attach` and `kill-server` subcommands SHALL address one server, identified by its socket path. Two options select it, and each SHALL be accepted before or after the subcommand:

| option | selects |
|---|---|
| `-S <name>`, `--server <name>` | the server named `<name>` |
| `-p <path>`, `--socket <path>` | the server listening on `<path>` |

A server name SHALL be 1 to 64 bytes long and hold only ASCII letters, digits, `_` and `-`. A relative `<path>` SHALL be resolved against the working directory. A command SHALL resolve its socket path as follows, first match winning:

1. `-p <path>`: that path.
2. `-S <name>`: `<name>.sock` in the gband runtime directory the session-server capability defines.
3. The `GBAND` environment variable, when it is set and not empty: the path it holds.
4. `default.sock` in the gband runtime directory.

Giving both options, a name that breaks the rule, or an empty path SHALL be an invalid invocation. `gband test` addresses no server: when `-S` or `-p` is given to it, `gband` SHALL print one line to standard error saying that the option does not apply to that subcommand and exit with status 2, without creating or writing a log file. A resolved socket path longer than 107 bytes SHALL be refused with one line on standard error that names the path and the limit, and exit status 1.

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
- **WHEN** the user runs `gband attach` outside a gband pane with `XDG_RUNTIME_DIR=/run/user/1000`
- **THEN** the client connects to `/run/user/1000/gband/default.sock`

#### Scenario: Inside a pane
- **WHEN** the user runs `gband kill-server` without an option in a pane of the server named `feature`
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
- **WHEN** the user runs `gband server -p` with a path of 120 bytes
- **THEN** standard error names the path and the 107-byte limit
- **AND** the process exits with status 1
- **AND** no shell is started

#### Scenario: Server option given to test
- **WHEN** the user runs `gband test -S feature`
- **THEN** standard error says that `-S` does not apply to `test`
- **AND** the process exits with status 2


### Requirement: Session option
The `gband` binary SHALL accept the option `-s <name>`, long form `--session <name>`, before or after the subcommand. It names a session as the session-server capability defines session names. When it is absent, the session name SHALL be `default`. `gband server`, `gband attach` and `gband kill-session` SHALL use it. When the name is not a valid session name, `gband` SHALL print one line to standard error stating the rule for session names and exit with status 2, without creating or writing a log file. When `-s` is given to `kill-server`, `list-sessions` or `test`, which do not select a session, `gband` SHALL print one line to standard error saying that the option does not apply to that subcommand and exit with status 2, without creating or writing a log file.

#### Scenario: Option after the subcommand
- **WHEN** the user runs `gband attach -s work`
- **THEN** the client attaches to the session named `work`

#### Scenario: Option before the subcommand
- **WHEN** the user runs `gband -s work attach`
- **THEN** the client attaches to the session named `work`

#### Scenario: Invalid name
- **WHEN** the user runs `gband attach -s 'a/b'`
- **THEN** standard error states which characters a session name may hold
- **AND** the process exits with status 2
- **AND** no log file is created

#### Scenario: Option that does not apply
- **WHEN** the user runs `gband kill-server -s work`
- **THEN** standard error says that `-s` does not apply to `kill-server`
- **AND** the process exits with status 2

#### Scenario: Session option given to test
- **WHEN** the user runs `gband test -s work`
- **THEN** standard error says that `-s` does not apply to `test`
- **AND** the process exits with status 2

