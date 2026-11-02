## MODIFIED Requirements

### Requirement: Bands mode
The `bands` table SHALL hold these bindings, each with the description its row gives. A row whose `does` names an action SHALL be bound to that action, and every other row SHALL be a Lua function:

| key | Lua binding | does | description |
|---|---|---|---|
| `1` to `9` | `bands 1` to `bands 9` | view the band at that position of `gband.layout().bands`, as `gband.band.view` does; when there is no band at that position, change nothing | `view band 1` to `view band 9` |
| `k` | `bands k` | the `focus_band_up` action | `view the band above` |
| Up | `bands up` | the `focus_band_up` action | `view the band above` |
| `j` | `bands j` | the `focus_band_down` action | `view the band below` |
| Down | `bands down` | the `focus_band_down` action | `view the band below` |
| `a` | `bands a` | enter agents mode | `agents mode` |
| Escape | `bands escape` | return to interactive mode | `interactive mode` |
| Enter | `bands enter` | return to interactive mode | `interactive mode` |

Only Escape and Enter SHALL leave bands mode. The digits, `k`, `j`, Up and Down SHALL keep it active, so the user can view band after band in one visit. `k`, `j`, Up and Down SHALL stop at the first and last band as those actions do, whatever `band_ends` holds. The position of a band SHALL be its label in the sidebar.

#### Scenario: Jump to a band
- **WHEN** the layout holds bands 1, 2 and 3, the client views band 1, and the user presses Ctrl+Space, `b`, then `3`
- **THEN** the client views band 3 and bands mode stays active
- **AND** after `2` the client views band 2, and after Escape `root` is the active table and band 2 is still viewed

#### Scenario: Band past the layout
- **WHEN** the layout holds bands 1 and 2, the client views band 1, and the user presses Ctrl+Space, `b`, then `7`
- **THEN** the client still views band 1 and bands mode stays active

#### Scenario: Step through bands
- **WHEN** the layout holds bands 1, 2 and 3, the client views band 1, and the user presses Ctrl+Space, `b`, Down, Down
- **THEN** the client views band 3 and bands mode stays active
- **AND** after Up it views band 2

#### Scenario: Switch between the modes
- **WHEN** agents mode is active and the user presses `b`
- **THEN** bands mode is active and the sidebar's mode letter is `B`
- **AND** after `a` agents mode is active and the mode letter is `A`

#### Scenario: Step bands with j and k
- **WHEN** the layout holds bands 1, 2 and 3, the client views band 1, and the user presses Ctrl+Space, `b`, `j`, `j`, then `k`
- **THEN** the client views band 2 and bands mode stays active
