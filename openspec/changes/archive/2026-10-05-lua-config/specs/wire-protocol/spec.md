## MODIFIED Requirements

### Requirement: Client messages
After an attach request, a client SHALL send only these messages:

| message | content |
|---|---|
| key | a pane identifier, and a key code with its Shift, Alt and Ctrl modifiers as the input-encoding capability defines them |
| paste | a pane identifier and the pasted text |
| resize | the client terminal's columns and rows |
| shown | the identifiers of every pane the client shows, as the layout-view capability defines; it replaces the set the client reported before |
| action | one session action, as the session-server capability defines it, with the pane or workspace it names |
| detach | nothing |

Pane and workspace identifiers SHALL name panes and workspaces of the client's session. The server SHALL ignore a pane identifier in a shown message that names no pane of the client's session. The open pane action SHALL name a workspace, optionally the pane whose column the new column follows, and optionally the program to run, either as a command line or as an argument list. Every other action SHALL name a pane, and consume or expel SHALL also name its direction. A client that detaches SHALL send detach, then close the connection.

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
- **WHEN** a client sends open pane naming workspace 1, pane 2 and the argument list `htop`, `-d`, `10`
- **THEN** the server decodes the same workspace, pane and argument list

#### Scenario: Open pane without a program round trip
- **WHEN** a client sends open pane naming workspace 1 and no pane or program
- **THEN** the server decodes open pane naming workspace 1 with no pane and no program
