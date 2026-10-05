## MODIFIED Requirements

### Requirement: Pane program
On creating a session, the server SHALL open that session's first pane. Every pane other than a plugin pane SHALL run, in its own new PTY, the program its open pane action names, or the user's shell when the action names none and for the first pane. The user's shell SHALL be `$SHELL` when it is set, otherwise the user's login shell. A program named as a command line SHALL run as the user's shell with the arguments `-c` and that command line. A program named as an argument list SHALL run as that list. The program SHALL start in its session's working directory. Its environment SHALL be the server's without `GBAND_TEST_SOCKET`, with `TERM=xterm-256color`, `COLORTERM=truecolor`, `GBAND` set to the socket path, `GBAND_SESSION` set to the session's name and `GBAND_PANE` set to the pane's identifier in decimal. The PTY's initial size SHALL be the terminal size the layout gives the pane for its session's current screen area.

#### Scenario: Environment of the pane
- **WHEN** a client attaches and the user runs `echo $TERM $COLORTERM $GBAND` in the pane
- **THEN** the pane prints `xterm-256color truecolor` followed by the socket path

#### Scenario: Session named in the pane
- **WHEN** a client attaches with `-s work` and the user runs `echo $GBAND_SESSION` in the pane
- **THEN** the pane prints `work`

#### Scenario: Panes are told apart
- **WHEN** a session holds two panes and the user runs `echo $GBAND_PANE` in each
- **THEN** the two panes print different identifiers

#### Scenario: Size before any client
- **WHEN** the server starts and no client has attached
- **THEN** the first pane's PTY is 38 columns by 22 rows

#### Scenario: Command line program
- **WHEN** a client asks to open a pane naming the command line `echo $GBAND_PANE; sleep 5`
- **THEN** the new pane prints its identifier, as the user's shell expands it

#### Scenario: Argument list program
- **WHEN** a client asks to open a pane naming the argument list `printf`, `%s-%s`, `a`, `b`
- **THEN** the new pane prints `a-b` without a shell expanding it

#### Scenario: Program that cannot start
- **WHEN** a client asks to open a pane naming the argument list `/nonexistent`
- **THEN** the server records the reason in its log and the layout is unchanged

#### Scenario: Test channel kept out of panes
- **WHEN** a server runs with `GBAND_TEST_SOCKET` set and the user runs `echo "[$GBAND_TEST_SOCKET]"` in a pane
- **THEN** the pane prints `[]`

