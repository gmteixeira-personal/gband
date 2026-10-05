## MODIFIED Requirements

### Requirement: Handshake
The first frame a client sends SHALL be a hello. The hello's payload SHALL begin with the client's protocol version, encoded the same way in every protocol version, followed by the client's terminal size. The first frame the server sends SHALL answer it. The server SHALL decode the leading version on its own, and compare it with its own version before it decodes anything that follows it. When the versions differ, the answer SHALL reject the client and carry the server's protocol version, whatever bytes follow the version, and the server SHALL then close the connection. When the versions are equal and the rest of the hello decodes, the answer SHALL accept the client and carry the server's protocol version. The leading version and the answer SHALL keep the same encoding in every later protocol version, so that two versions can always detect each other. The fields after the leading version MAY change in a later protocol version. The current protocol version SHALL be 8.

#### Scenario: Matching versions
- **WHEN** a version 2 client sends its hello to a version 2 server
- **THEN** the server accepts it
- **AND** sends server info as its next frame, then the layout, then a snapshot of each pane

#### Scenario: Mismatched versions
- **WHEN** a version 1 client sends its hello to a version 2 server
- **THEN** the server rejects it with version 2 and closes the connection
- **AND** the server records the rejected version in its log and keeps running

#### Scenario: Later hello with other fields
- **WHEN** a client sends a hello frame holding version 3 followed by bytes that do not decode as a version 2 terminal size
- **THEN** the server rejects it with version 2 and closes the connection

#### Scenario: First frame is not a hello
- **WHEN** a client's first frame begins with version 2 but the rest does not decode as a hello
- **THEN** the server closes the connection without accepting it

#### Scenario: Empty first frame
- **WHEN** a client's first frame has an empty payload
- **THEN** the server closes the connection without answering

#### Scenario: Version 3 client meets a version 4 server
- **WHEN** a client speaking protocol version 3 sends its hello to a server speaking version 4
- **THEN** the server rejects it with version 4 and closes the connection

#### Scenario: Version 4 client meets a version 5 server
- **WHEN** a client speaking protocol version 4 sends its hello to a server speaking version 5
- **THEN** the server rejects it with version 5 and closes the connection

#### Scenario: Version 5 client meets a version 6 server
- **WHEN** a client speaking protocol version 5 sends its hello to a server speaking version 6
- **THEN** the server rejects it with version 6 and closes the connection

#### Scenario: Version 7 client meets a version 8 server
- **WHEN** a client speaking protocol version 7 sends its hello to a server speaking version 8
- **THEN** the server rejects it with version 8 and closes the connection

### Requirement: Client messages
After an attach request, a client SHALL send only these messages:

| message | content |
|---|---|
| key | a pane identifier, and a key code with its Shift, Alt and Ctrl modifiers as the input-encoding capability defines them |
| paste | a pane identifier and the pasted text |
| resize | the client terminal's columns and rows |
| shown | the identifiers of every pane the client shows, as the layout-view capability defines; it replaces the set the client reported before |
| action | one session action, as the session-server capability defines it, with the pane or band it names |
| content | a pane identifier, and terminal output for the plugin pane's screen, as the session-server capability defines |
| command | a call number chosen by the client, a server command's full name, and its arguments as a plain data value |
| detach | nothing |

Pane and band identifiers SHALL name panes and bands of the client's session. The server SHALL ignore a pane identifier in a shown message that names no pane of the client's session. The open pane action SHALL name a band, optionally the pane whose column the new column follows, optionally the new column's width, whether the pane floats, whether the client asks to focus the new pane, and what the pane holds: either a program, which is optionally named as a command line or as an argument list, or plugin content with a request number. Every other action SHALL name a pane. Consume or expel and move column SHALL also name their direction, left or right. Move pane SHALL also name its direction, down or up. Toggle floating SHALL also name optionally the tiled pane to tile after. Set position SHALL also name a column and a row. Set width SHALL also name a width. Set height SHALL also name either a number of rows or a weight. Grow and shrink of a width and of a height SHALL also name a step. A width, a weight and a step SHALL each be a fraction in lowest terms. A client SHALL NOT reuse a call number while its call is unanswered. A client that detaches SHALL send detach, then close the connection.

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
- **WHEN** a client sends grow height naming pane 2 and the step 1/10
- **THEN** the server decodes the same action, pane and step

#### Scenario: Open pane with a program round trip
- **WHEN** a client sends open pane naming band 1, pane 2 and the argument list `htop`, `-d`, `10`
- **THEN** the server decodes the same band, pane and argument list

#### Scenario: Open pane without a program round trip
- **WHEN** a client sends open pane naming band 1 and no pane or program
- **THEN** the server decodes open pane naming band 1 with no pane and no program

#### Scenario: Open plugin pane round trip
- **WHEN** a client sends open pane naming band 1, pane 2, the width 1/4, no focus, and plugin content with request number 7
- **THEN** the server decodes the same band, pane, width, focus flag, content kind and request number

#### Scenario: Set height round trip
- **WHEN** a client sends set height naming pane 2 and the weight 3/2
- **THEN** the server decodes set height naming pane 2 and the weight 3/2

#### Scenario: Content round trip
- **WHEN** a client sends content naming pane 4 and the output `\x1b[1;1Hhello`
- **THEN** the server decodes content naming pane 4 and the same bytes

#### Scenario: Command round trip
- **WHEN** a client sends command number 7 naming `agents.next_waiting` with the arguments `{ state = "agent" }`
- **THEN** the server decodes the same number, name and arguments

#### Scenario: Toggle floating round trip
- **WHEN** a client sends toggle floating naming pane 3 and pane 1 as the pane to tile after
- **THEN** the server decodes toggle floating naming pane 3 and pane 1

#### Scenario: Move round trip
- **WHEN** a client sends move column naming pane 2 and the direction right, and move pane naming pane 2 and the direction up
- **THEN** the server decodes the same actions, pane and directions

#### Scenario: Set position round trip
- **WHEN** a client sends set position naming pane 4, column 12 and row 3
- **THEN** the server decodes set position naming pane 4, column 12 and row 3

#### Scenario: Open floating pane round trip
- **WHEN** a client sends open pane naming band 1, no pane, the width 1/3, floating, and no program
- **THEN** the server decodes the same band, width and floating flag

