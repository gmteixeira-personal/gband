## MODIFIED Requirements

### Requirement: Reload on change
Each process SHALL watch the `user` directory and every directory beneath it. Within two seconds of a file whose name ends in `.lua` being created, written, replaced or removed there, the process SHALL load its side's configuration again in a new Lua state and, when loading succeeds, use the new configuration from then on. Changes to the `defaults` directory and to the plugins directory SHALL NOT cause a reload. Server options SHALL apply to session actions received after the server reloads, and no column's width SHALL change because of a reload. A key sequence in progress SHALL end without effect when the client reloads. The process SHALL detect these changes by checking the `user` directory periodically, without file-change notifications from the operating system, so that no limit on such notifications can stop reloads.

#### Scenario: New binding without restart
- **WHEN** a client is attached and the user adds `gband.bind("alt+l", gband.action.focus_column_right)` to `user/init.lua`
- **THEN** within two seconds Alt+L focuses the column to the right

#### Scenario: Editor replaces the file
- **WHEN** an editor saves `user/init.lua` by writing a new file and renaming it over the old one
- **THEN** the client loads the new content

#### Scenario: User plugin file edited
- **WHEN** a client is attached, `user/init.lua` calls `require("keys")`, and the user saves `user/lua/keys.lua` binding `alt+k`
- **THEN** within two seconds Alt+K runs the new binding

#### Scenario: New default width
- **WHEN** a session holds a column of width 1/2 and the user sets `default_column_width` to `1/3` in `user/server.lua`
- **THEN** that column keeps width 1/2
- **AND** a window opened after the reload has a column of width 1/3

#### Scenario: File removed
- **WHEN** `user/init.lua` bound `alt+h` and the user deletes it
- **THEN** the default configuration applies and Alt+H reaches the focused window

#### Scenario: Defaults file edited while running
- **WHEN** a client is attached and the user saves a change to `defaults/init.lua`
- **THEN** the bindings stay as they were

#### Scenario: Notification limit reached
- **WHEN** a client starts while the user's processes already hold as many file-change notifications as the system allows, and the user then adds `gband.bind("alt+l", gband.action.focus_column_right)` to `user/init.lua`
- **THEN** within two seconds Alt+L focuses the column to the right
