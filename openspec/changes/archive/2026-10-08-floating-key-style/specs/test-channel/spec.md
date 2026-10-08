## MODIFIED Requirements

### Requirement: Settle
The runner SHALL be able to ask the client and the server to settle. A settle SHALL complete once:

1. the client has read and handled all terminal input the runner wrote before asking,
2. the server has handled every message the client sent before that point, applied the actions they asked for, delivered the resulting events to its Lua handlers, and sent the resulting changes to the client,
3. the client has handled every message the server sent it before that point, delivered the resulting events to its Lua handlers, finished any running animation, sent the window move or resize that a pointer drag holds back for its next frame, and drawn a frame showing the result,
4. the runner's emulator has read every byte the client wrote to its terminal before that frame ended.

The client and the server SHALL repeat these steps for the effects of the effects until one round changes nothing, up to ten rounds. A settle SHALL NOT wait for output that programs in windows write in response. A settle that does not complete within 10 seconds SHALL fail with a message naming the step that did not finish.

#### Scenario: Key effect is drawn
- **WHEN** the runner writes the key that opens a window and then settles
- **THEN** the runner's screen shows two tiles as soon as the settle completes

#### Scenario: Server handler effect is drawn
- **WHEN** a server handler of `WindowOpened` sets a window state key, a client handler of `WindowStateChanged` writes the key's value into a bar with `gband.bar.set_lines`, and the runner opens a window and settles
- **THEN** the bar's new text is on the runner's screen as soon as the settle completes

#### Scenario: Two drags in a row
- **WHEN** the runner drags a floating window's border and settles, then at once writes a press, a drag and a release on its border again and settles
- **THEN** the window's box after the second settle holds both drags
