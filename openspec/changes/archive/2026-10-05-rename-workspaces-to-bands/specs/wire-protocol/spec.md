## MODIFIED Requirements

### Requirement: Client messages
After an attach request, a client SHALL send only these messages:

| message | content |
|---|---|
| key | a pane identifier, and a key code with its Shift, Alt and Ctrl modifiers as the input-encoding capability defines them |
| paste | a pane identifier and the pasted text |
| resize | the client terminal's columns and rows |
| shown | the identifiers of every pane the client shows, as the layout-view capability defines; it replaces the set the client reported before |
| action | one session action, as the session-server capability defines it, with the pane or band it names |
| detach | nothing |

Pane and band identifiers SHALL name panes and bands of the client's session. The server SHALL ignore a pane identifier in a shown message that names no pane of the client's session. The open pane action SHALL name a band, optionally the pane whose column the new column follows, and optionally the program to run, either as a command line or as an argument list. Every other action SHALL name a pane, and consume or expel SHALL also name its direction. A client that detaches SHALL send detach, then close the connection.

#### Scenario: Detach message
- **WHEN** a client sends detach
- **THEN** the server stops sending to it and closes the connection
- **AND** every pane's program keeps running

#### Scenario: Action round trip
- **WHEN** a client sends consume or expel naming pane 3 and the direction left
- **THEN** the server decodes the same action, pane and direction

#### Scenario: Shown round trip
- **WHEN** a client sends shown naming panes 1 and 4
- **THEN** the server decodes a shown message naming panes 1 and 4

#### Scenario: Grow height round trip
- **WHEN** a client sends grow height naming pane 2
- **THEN** the server decodes the same action and pane

#### Scenario: Open pane with a program round trip
- **WHEN** a client sends open pane naming band 1, pane 2 and the argument list `htop`, `-d`, `10`
- **THEN** the server decodes the same band, pane and argument list

#### Scenario: Open pane without a program round trip
- **WHEN** a client sends open pane naming band 1 and no pane or program
- **THEN** the server decodes open pane naming band 1 with no pane and no program

### Requirement: Server messages
After the handshake, a server SHALL send only these messages:

| message | content |
|---|---|
| info | the server's process id and the identity of the executable it runs from |
| layout | the screen area's columns and rows, and the layout: its bands in order with their identifiers, each band's columns in order with their widths and full-width flags, and each column's panes in order with their identifiers and heights, each either automatic with its weight or fixed with its rows |
| snapshot | a pane identifier, the pane's columns and rows, and terminal output that, fed into an empty terminal grid of that size, reproduces that pane's screen on the server |
| update | a pane identifier, and terminal output that, fed into the grid the client built for that pane from every earlier snapshot and update of it, reproduces that pane's current screen on the server |
| focus | a pane identifier: the pane the client asked to open, which the client focuses |
| exited | nothing; the client's session has ended |
| sessions | for each session, its name, pane count and attached-client count |
| killed | nothing; the session a kill request named has ended |
| no such session | nothing; the server hosts no session of the name a kill request named |

The server SHALL send info exactly once, as its first message after accepting the hello, before any answer to a request. An executable's identity SHALL be the device and inode of the file the process was started from, taken when the process starts, so that replacing the file on disk, as a rebuild does, gives a new identity. The layout, snapshot, update, focus and exited messages SHALL concern only the session the client attached to.

A client SHALL keep one grid per pane. It SHALL build a pane's grid by replacing it with a new empty grid of the snapshot's size on every snapshot of that pane, then feeding the snapshot's output into it, and by feeding each update's output for that pane into its current grid. It SHALL discard the grid of a pane that the latest layout does not hold.

#### Scenario: Info before snapshot
- **WHEN** a client is accepted and sends an attach request
- **THEN** the first message it receives is info, carrying the server's process id
- **AND** the second is the layout, followed by a snapshot of each pane it holds

#### Scenario: Rebuilt executable has a new identity
- **WHEN** a server is started from `target/debug/gband` and `cargo build` then replaces that file with a new build
- **THEN** the identity the server reports differs from the identity a client started from the new file computes for itself

#### Scenario: Snapshot then updates
- **WHEN** a client applies every snapshot and every update it receives for a pane
- **THEN** its grid of that pane has the same visible cells, cursor and input modes as the server's

#### Scenario: Layout round trip
- **WHEN** a layout of two bands, one holding a full-width column of width 1/3 with two panes, is framed and decoded
- **THEN** the decoded layout equals the original

#### Scenario: Sessions round trip
- **WHEN** a sessions message holding `default` with 1 pane and 0 clients and `work` with 2 panes and 1 client is framed and decoded
- **THEN** the decoded message equals the original

#### Scenario: Session ends
- **WHEN** the last pane of a client's session leaves the layout
- **THEN** that client receives exited as its last message

#### Scenario: Heights in the layout
- **WHEN** a column holds pane 1 with a fixed height of 14 rows and pane 2 with an automatic height of weight 10/7, and the server sends the layout
- **THEN** the client decodes that column with pane 1 fixed at 14 rows and pane 2 automatic with weight 10/7
