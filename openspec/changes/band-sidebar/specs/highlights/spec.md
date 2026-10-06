## MODIFIED Requirements

### Requirement: Highlight change events
After the configuration has loaded, each call to `gband.hl.set` or `gband.hl.default` that changes a group's settings SHALL emit `HighlightChanged` with the group's name, as the lua-events capability defines. Calls while the configuration loads, and settings a colorscheme makes as it loads, SHALL NOT emit it.

#### Scenario: Change in a callback
- **WHEN** a binding function calls `gband.hl.set("SidebarMode", { fg = 3 })`
- **THEN** `HighlightChanged` runs once with `group` `SidebarMode`

#### Scenario: No event during the load
- **WHEN** `user/init.lua` registers a `HighlightChanged` handler and then calls `gband.hl.set("Title", { fg = 1 })`
- **THEN** the handler does not run
