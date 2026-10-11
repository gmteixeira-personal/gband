# command-line Specification

## Purpose

Defines the `gband` binary's command line: the subcommands it accepts, the help it prints and the exit status each invocation returns.

## Requirements

### Requirement: Subcommands run the session
The `gband` binary SHALL accept the subcommands `server`, `attach`, `list-sessions`, `kill-session`, `kill-server`, `reload`, `errors`, `eval`, `cmd` and `relay`. When no subcommand is given, `gband` SHALL run `attach`, with every option given applied to it as if `attach` had been named. Each of these ten subcommands SHALL start logging for its role and record one log event stating that it started, before doing anything else. Subcommands that other capabilities define, such as shell-completions' `completions` and `install-completions`, SHALL log only as those capabilities state. `gband server` SHALL run the session server the session-server capability defines, in the foreground, until its last session ends. `gband attach` SHALL run the client the client-attach capability defines. `gband list-sessions`, `gband kill-session` and `gband kill-server` SHALL list the sessions, end a session and stop the running server, as the session-server capability defines. `gband reload`, `gband errors`, `gband eval` and `gband cmd` SHALL act as the remote-control capability defines. `gband relay` SHALL act as the transport capability's "Relay subcommand" defines, and SHALL be left out of the usage `gband -h` prints, because only the SSH transport runs it. No subcommand SHALL write log events to standard output or standard error.

#### Scenario: Run the server
- **WHEN** the user runs `gband server` with `SHELL=/bin/true`
- **THEN** the process exits with status 0
- **AND** it prints nothing to standard output or standard error
- **AND** the server log file holds an event stating that the server started

#### Scenario: Server runs until its program exits
- **WHEN** the user runs `gband server` with the default shell
- **THEN** the process keeps running until every window's shell in every session has exited

#### Scenario: Run attach
- **WHEN** the user runs `gband attach` and then detaches
- **THEN** the client log file holds an event stating that the client started

#### Scenario: No subcommand runs attach
- **WHEN** the user runs `gband` with no arguments and then detaches
- **THEN** the client attaches to the session named `default` on the default server
- **AND** the client log file holds an event stating that the client started

#### Scenario: No subcommand with options
- **WHEN** the user runs `gband -S feature -s work`
- **THEN** the client attaches to the session named `work` on the server named `feature`

#### Scenario: No subcommand without a terminal
- **WHEN** the user runs `gband` with no arguments and standard input is not a terminal
- **THEN** standard error holds the one line `gband attach` prints when it has no terminal
- **AND** the process exits with status 1
- **AND** the client log file holds an event stating that the client started

#### Scenario: Run kill-server
- **WHEN** the user runs `gband kill-server` with no server running
- **THEN** the client log file holds an event stating that the client started
- **AND** the process exits with status 1

#### Scenario: Run list-sessions
- **WHEN** the user runs `gband list-sessions` with no server running
- **THEN** the client log file holds an event stating that the client started
- **AND** the process exits with status 1

#### Scenario: Run reload
- **WHEN** the user runs `gband reload` with no server running
- **THEN** the client log file holds an event stating that the client started
- **AND** the process exits with status 1

#### Scenario: Run kill-session
- **WHEN** the user runs `gband kill-session` with no server running
- **THEN** the client log file holds an event stating that the client started
- **AND** the process exits with status 1

#### Scenario: Run relay
- **WHEN** the user runs `gband relay < /dev/null` with no server running and `SHELL=/bin/true`
- **THEN** the client log file holds an event stating that the client started

#### Scenario: Relay is hidden
- **WHEN** the user runs `gband --help`
- **THEN** standard output does not list `relay`

### Requirement: Help and version
`gband -h` and `gband --help` SHALL each print usage to standard output that lists the `server`, `attach`, `list-sessions`, `kill-session`, `kill-server`, `reload`, `errors`, `eval` and `cmd` subcommands and the `-s` and `-r` options, marks `attach` as the subcommand run when none is given, and exit with status 0 without creating or writing a log file. `gband --version` SHALL print the binary's version and exit with status 0.

#### Scenario: Print help
- **WHEN** the user runs `gband --help`
- **THEN** standard output lists the `server`, `attach`, `list-sessions`, `kill-session`, `kill-server`, `reload`, `errors`, `eval` and `cmd` subcommands and the `-s` and `-r` options
- **AND** the `attach` entry states that it is the default subcommand
- **AND** the process exits with status 0
- **AND** no log file is created

#### Scenario: Print help with the short flag
- **WHEN** the user runs `gband -h`
- **THEN** standard output lists the `server`, `attach`, `list-sessions`, `kill-session`, `kill-server`, `reload`, `errors`, `eval` and `cmd` subcommands and the `-s` and `-r` options
- **AND** the process exits with status 0
- **AND** no log file is created

#### Scenario: Print version
- **WHEN** the user runs `gband --version`
- **THEN** standard output holds `gband` followed by the package version
- **AND** the process exits with status 0

### Requirement: Invalid invocations
`gband` run with a subcommand or option it does not recognise SHALL print usage to standard error and exit with status 2. It SHALL NOT create or write a log file. A missing subcommand is not an invalid invocation: it runs `attach`.

#### Scenario: No subcommand
- **WHEN** the user runs `gband` with no arguments
- **THEN** it runs `attach` rather than printing usage
- **AND** the process does not exit with status 2

#### Scenario: Unknown subcommand
- **WHEN** the user runs `gband frobnicate`
- **THEN** standard error names `frobnicate` as unrecognised
- **AND** the process exits with status 2
- **AND** no log file is created

#### Scenario: Unknown option without a subcommand
- **WHEN** the user runs `gband --frobnicate`
- **THEN** standard error names `--frobnicate` as unrecognised
- **AND** the process exits with status 2
- **AND** no log file is created

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
- **WHEN** the user runs `gband server -p` with a path of 120 bytes
- **THEN** standard error names the path and the 107-byte limit
- **AND** the process exits with status 1
- **AND** no shell is started

#### Scenario: Remote server selection
- **WHEN** the user runs `gband -r u@h -p target/gband.sock attach` in `/home/u/repo`
- **THEN** the remote relay receives `-p target/gband.sock` unresolved
- **AND** no local socket path is checked

#### Scenario: Server option given to test
- **WHEN** the user runs `gband test -S feature`
- **THEN** standard error says that `-S` does not apply to `test`
- **AND** the process exits with status 2

### Requirement: Session option
The `gband` binary SHALL accept the option `-s <name>`, long form `--session <name>`, before or after the subcommand. It names a session as the session-server capability defines session names. When it is absent, the session name SHALL be `default`, except for `reload`, `errors`, `eval` and `cmd`: they SHALL use the session that the `GBAND_SESSION` environment variable names when it is set and holds a valid session name, and `default` otherwise, so that a command run in a gband window addresses that window's session. `gband server`, `gband attach`, `gband kill-session`, `gband reload`, `gband errors`, `gband eval`, `gband cmd` and `gband relay` SHALL use it. Given `-r`, `reload`, `errors`, `eval` and `cmd` SHALL NOT read `GBAND_SESSION`, and pass `-s` to the remote host only when the user gave it, as "Remote host option" defines. When the name is not a valid session name, `gband` SHALL print one line to standard error stating the rule for session names and exit with status 2, without creating or writing a log file. When `-s` is given to `kill-server`, `list-sessions` or `test`, which do not select a session, `gband` SHALL print one line to standard error saying that the option does not apply to that subcommand and exit with status 2, without creating or writing a log file.

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
- **AND** no server is stopped

#### Scenario: Session option given to test
- **WHEN** the user runs `gband test -s work`
- **THEN** standard error says that `-s` does not apply to `test`
- **AND** the process exits with status 2

#### Scenario: Session of the window
- **WHEN** the user runs `gband errors` without `-s` in a window of the session `work`
- **THEN** it reports the session `work`

#### Scenario: Option over the window's session
- **WHEN** the user runs `gband reload -s other` in a window of the session `work`
- **THEN** it reloads the clients of the session `other`

#### Scenario: Outside a window
- **WHEN** `GBAND_SESSION` is unset and the user runs `gband errors`
- **THEN** it reports the session `default`

### Requirement: Server starting size
`gband server` SHALL accept the option `--size <cols>x<rows>`, where both are whole numbers above 0. The session the server creates on start SHALL start at that screen area, as the session-server capability's "Session on start" defines. When the option is absent, that area SHALL be 80 columns by 24 rows. When the value is not of that form, `gband` SHALL print one line to standard error naming the value and the form `COLSxROWS`, and exit with status 2, without creating or writing a log file.

#### Scenario: Server started at a size
- **WHEN** the user runs `gband server --size 100x30` and no client has attached
- **THEN** the session's first window, in a column of width 1/2, has a PTY of 48 columns by 28 rows

#### Scenario: Invalid size
- **WHEN** the user runs `gband server --size 0x24`
- **THEN** standard error names `0x24` and the form `COLSxROWS`
- **AND** the process exits with status 2
- **AND** no log file is created

### Requirement: Remote host option
The `gband` binary SHALL accept the option `-r <destination>`, long form `--remote <destination>`, before or after the subcommand. It addresses a server on the host the destination names, reached through `ssh`. The destination SHALL be passed to `ssh` exactly as given, so `host`, `user@host`, an alias from the ssh configuration and `ssh://user@host:port` all work. An empty destination SHALL be an invalid invocation.

`-r` SHALL apply to `attach`, `list-sessions`, `kill-session`, `kill-server`, `reload`, `errors`, `eval` and `cmd`, and to `gband` run without a subcommand. When it is given to `server`, `relay`, `test`, `completions` or `install-completions`, `gband` SHALL print one line to standard error saying that `-r` does not apply to that subcommand and exit with status 2, without creating or writing a log file.

The `gband` binary SHALL accept the option `--remote-gband <command>`, before or after the subcommand. It names the gband executable the remote host runs, and is written into the remote command unquoted. Without it, the remote host SHALL run `gband`. Giving `--remote-gband` without `-r`, or with an empty value, SHALL be an invalid invocation.

With `-r`, the subcommand SHALL resolve no local socket path, SHALL NOT read the local `GBAND` or `GBAND_SESSION` variables, and SHALL NOT check a socket path's length. The `-S`, `-p` and `-s` options SHALL be passed to the remote host as given, and a relative `-p` path SHALL be resolved there, against the remote login directory. The rules for server names and session names SHALL still be checked locally.

`gband attach` with `-r` SHALL run the client locally over the SSH transport the transport capability defines. Each other subcommand with `-r` SHALL run `ssh -T -- <destination>` with a remote command made of the remote gband command, then every argument of the local command line in its order, with `-r`, `--remote-gband` and their values left out, each quoted for a POSIX shell. ssh SHALL get the local standard output and standard error. ssh SHALL get the local standard input for `eval -` and `/dev/null` for every other subcommand. `gband` SHALL exit with ssh's exit status, except that when ssh exits with status 255, `gband` SHALL print one line to standard error naming the destination and exit with status 1.

#### Scenario: Attach to a remote host
- **WHEN** the user runs `gband -r u@h`
- **THEN** the client attaches to the session named `default` on the default server of `u@h`

#### Scenario: Remote named session
- **WHEN** the user runs `gband -r u@h -s work`
- **THEN** the client attaches to the session named `work` on the default server of `u@h`, creating it when it does not exist

#### Scenario: Option after the subcommand
- **WHEN** the user runs `gband attach -r u@h`
- **THEN** the client attaches to the session named `default` on `u@h`

#### Scenario: Remote list-sessions
- **WHEN** the user runs `gband -r u@h -S feature list-sessions`
- **THEN** `gband` runs `ssh -T -- u@h gband '-S' 'feature' 'list-sessions'`
- **AND** standard output holds the remote listing
- **AND** the process exits with the remote command's status

#### Scenario: Remote eval from standard input
- **WHEN** the user runs `echo 'return 1 + 1' | gband -r u@h eval -`
- **THEN** the remote `gband eval -` reads `return 1 + 1` from its standard input

#### Scenario: Window variables stay local
- **WHEN** the user runs `gband -r u@h errors` in a window of the local session `work`
- **THEN** the remote command holds no `-s`, so the remote host reports its session `default`

#### Scenario: ssh fails for a subcommand
- **WHEN** the user runs `gband -r nowhere kill-server` and ssh exits with status 255
- **THEN** standard error names `nowhere`
- **AND** the process exits with status 1

#### Scenario: Remote option given to server
- **WHEN** the user runs `gband server -r u@h`
- **THEN** standard error says that `-r` does not apply to `server`
- **AND** the process exits with status 2
- **AND** no log file is created

#### Scenario: Remote gband without a remote host
- **WHEN** the user runs `gband --remote-gband /opt/gband attach`
- **THEN** standard error holds the usage
- **AND** the process exits with status 2

#### Scenario: Empty destination
- **WHEN** the user runs `gband -r ''`
- **THEN** the process exits with status 2
- **AND** no log file is created
