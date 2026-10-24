## MODIFIED Requirements

### Requirement: Sidebar groups
The plugin SHALL define these groups as defaults, in the defaults layer the highlights capability defines:

| group | default |
|---|---|
| `SidebarMode` | `{ bold = true }` |
| `SidebarBand` | `{ dim = true }` |
| `SidebarBandActive` | `{ link = "SidebarBand", underline = true }` |
| `SidebarError` | `{ fg = 1, bold = true }` |
| `SidebarAgentBlocked` | `{ fg = "red" }` |
| `SidebarAgentBlockedActive` | `{ link = "SidebarAgentBlocked", underline = true }` |
| `SidebarAgentDone` | `{ fg = "green" }` |
| `SidebarAgentDoneActive` | `{ link = "SidebarAgentDone", underline = true }` |
| `SidebarAgentWorking` | `{ fg = "yellow" }` |
| `SidebarAgentWorkingActive` | `{ link = "SidebarAgentWorking", underline = true }` |
| `SidebarAgentIdle` | `{ fg = "blue" }` |
| `SidebarAgentIdleActive` | `{ link = "SidebarAgentIdle", underline = true }` |
| `SidebarAgentUnknown` | `{ link = "SidebarBand" }` |
| `SidebarAgentUnknownActive` | `{ link = "SidebarBandActive" }` |
| `SidebarDivider` | `{ link = "SidebarBand" }` |

So each `Active` group draws the label as its unfocused peer does, with an underline added. A change of a group's resolved style SHALL redraw the sidebar, as the bars capability defines.

#### Scenario: Defaults without a colorscheme
- **WHEN** `user/init.lua` sets up `gband.sidebar` and sets no group
- **THEN** `SidebarBand` resolves to `{ dim = true }` and `SidebarError` to `{ fg = 1, bold = true }`
- **AND** `SidebarAgentWorking` resolves to `{ fg = "yellow" }`, and `SidebarAgentUnknown` and `SidebarDivider` to `{ dim = true }`

#### Scenario: Active defaults add an underline
- **WHEN** `user/init.lua` sets up `gband.sidebar` and sets no group
- **THEN** `SidebarBandActive` and `SidebarAgentUnknownActive` resolve to `{ dim = true, underline = true }`
- **AND** `SidebarAgentWorkingActive` resolves to `{ fg = "yellow", underline = true }`, and `SidebarAgentBlockedActive` to `{ fg = "red", underline = true }`

#### Scenario: Viewed band underlined in the band colour
- **WHEN** the default configuration and the `default` colorscheme are in use, the layout holds bands 1 and 2, no window runs an agent, and the client views band 1
- **THEN** the labels `1` and `2` are both drawn dim and not bold
- **AND** only the label `1` is underlined

#### Scenario: Theme the viewed band
- **WHEN** `user/init.lua` calls `gband.hl.set("SidebarBandActive", { fg = 2 })`
- **THEN** the viewed band's label is drawn with foreground 2

#### Scenario: State colors follow the palette
- **WHEN** the agent at position 1 is `blocked`, the client focuses the agent at position 2, which is `blocked` too, and no group is set
- **THEN** the labels `1` and `2` are both drawn with palette index 1 and not bold
- **AND** only the label `2` is underlined

#### Scenario: Viewed agent band underlined in its state colour
- **WHEN** no group is set, band 1 holds one agent window whose state is `working`, and the client views band 1
- **THEN** the band label `1` is drawn with palette index 3, not bold, and underlined
