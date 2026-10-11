## ADDED Requirements

### Requirement: SSH transport
The client SHALL provide an SSH transport, and `gband attach` given `-r <destination>`, as the command-line capability's "Remote host option" defines, SHALL use it. Opening it SHALL run the `ssh` executable found on the local `PATH` with the arguments `-T`, `--`, the destination exactly as given, and one remote command. The remote command SHALL be the remote gband command, followed by `-S <name>` or `-p <path>` when the user gave one, then `relay`, `-s <session>` with the session the client attaches to, and `--size <cols>x<rows>` with the client's reported size, as "Ribbon area beside the bars" defines. The remote gband command SHALL be the value of `--remote-gband` written unquoted, so that the remote shell expands `~` and variables in it, or `gband` without that option. Every other argument of the remote command SHALL be quoted for a POSIX shell, so that the remote shell receives each one as a single word.

The client SHALL write to ssh's standard input as its stream to the server and read ssh's standard output as its stream from the server. ssh SHALL inherit the client's controlling terminal, so it can ask for a password or a host key confirmation there before the client changes the terminal. ssh's standard error SHALL be read by the client and written to the client log, one event per line. Each line read before the handshake completes SHALL also be kept as the last line from ssh.

When the connection closes before the server's info arrives, the client SHALL wait for ssh to exit, print one line to standard error and exit with status 1 without changing the terminal. The line SHALL be:

| ssh exits with | line |
|---|---|
| 127 | `gband: <command> was not found on <destination>; install gband there or pass --remote-gband`, where `<command>` is the remote gband command |
| any other status | `gband: cannot reach <destination>: <last line>`, where `<last line>` is the last line from ssh, or `ssh exited with status <n>` when ssh wrote none |

When the `ssh` executable cannot be started, the client SHALL print one line to standard error naming `ssh` and the reason, and exit with status 1. When the client leaves, it SHALL close ssh's standard input, wait up to 2 seconds for ssh to exit, and then kill it. The SSH transport SHALL never replace the server.

#### Scenario: Remote command line
- **WHEN** the user runs `gband -r u@h -S feature -s work` in an 80×24 terminal with the default configuration
- **THEN** the client runs `ssh -T -- u@h gband '-S' 'feature' 'relay' '-s' 'work' '--size' '79x24'`

#### Scenario: Remote gband with a tilde
- **WHEN** the user runs `gband -r u@h --remote-gband '~/.cargo/bin/gband'`
- **THEN** the remote command starts with `~/.cargo/bin/gband` unquoted

#### Scenario: Socket path with a space
- **WHEN** the user runs `gband -r u@h -p '/tmp/my socket'`
- **THEN** the remote command holds `'-p' '/tmp/my socket'`, so the remote relay receives the path as one argument

#### Scenario: Unreachable host
- **WHEN** ssh prints `ssh: Could not resolve hostname nowhere: Name or service not known` and exits with status 255
- **THEN** standard error holds `gband: cannot reach nowhere: ssh: Could not resolve hostname nowhere: Name or service not known`
- **AND** the process exits with status 1
- **AND** the terminal is not left in raw mode or the alternate screen

#### Scenario: gband missing on the remote host
- **WHEN** the remote shell finds no `gband` and ssh exits with status 127
- **THEN** standard error holds `gband: gband was not found on u@h; install gband there or pass --remote-gband`
- **AND** the process exits with status 1

#### Scenario: Connection drops
- **WHEN** a client attached with `-r` loses its ssh connection
- **THEN** the client restores the terminal, prints `[lost server]` and exits with status 1

### Requirement: Relay subcommand
`gband relay` SHALL run on the remote host, in the remote command the SSH transport sends. It SHALL resolve its socket path as the command-line capability's "Server selection" defines, and connect to it, checking the listening user as client-attach's "Connect to the session" defines. When nothing accepts connections there, it SHALL start a server as client-attach's "Start a server when none is running" defines, with the session `-s` names and the size `--size` gives, and the relay's own working directory as the server's working directory. It SHALL then copy its standard input to the socket and the socket to its standard output, without reading or changing the bytes, until both directions have ended. It SHALL exit with status 0 once both have ended.

`gband relay` SHALL accept `--size <cols>x<rows>` as `gband server` accepts it, and refuse an invalid value the same way. When it cannot connect or start a server, it SHALL print one line to standard error naming the socket path, as `gband attach` would, and exit with status 1 without writing to standard output. It SHALL NOT require a terminal on its standard input or output, which are ssh's pipes, and SHALL NOT apply client-attach's "Refuse to nest".

#### Scenario: Relay to a running server
- **WHEN** a server is running and `gband relay` receives a hello on its standard input
- **THEN** the server's hello reply appears on the relay's standard output

#### Scenario: Relay starts the server
- **WHEN** no server is running and the user runs `gband relay -s work --size 100x30` in `/home/u`
- **THEN** a server starts that hosts exactly one session, named `work`, whose working directory is `/home/u`
- **AND** the session's first window, in a column of width 1/2, has a PTY of 48 columns by 28 rows

#### Scenario: Server ends the session
- **WHEN** the only window of the relayed session exits
- **THEN** the relay's standard output closes after the server's last message
- **AND** the relay exits with status 0

## MODIFIED Requirements

### Requirement: Relay transparency
A transport that only copies bytes between the client and the server's Unix socket SHALL NOT change the session's behaviour. The SSH transport, through `gband relay`, is such a transport.

#### Scenario: Attach through a relay
- **WHEN** a client attaches through a relay that copies bytes in both directions, runs `printf 'relayed\n'` in the window and detaches
- **THEN** the client shows `relayed` in the window
- **AND** the session keeps running after the detach

#### Scenario: Attach through ssh
- **WHEN** the `ssh` on `PATH` runs its remote command in a local shell, the user runs `gband -r u@h`, runs `printf 'relayed\n'` in the window and detaches
- **THEN** the client shows `relayed` in the window
- **AND** the client printed `[detached]` and exited with status 0
- **AND** `gband list-sessions` lists the session `default` with 0 clients
