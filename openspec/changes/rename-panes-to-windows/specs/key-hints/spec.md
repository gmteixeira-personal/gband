## MODIFIED Requirements

### Requirement: Hint labels
A hint's label SHALL be the first of these that applies:
1. For a binding to an action named in the `labels` option, that option's string. A binding to an action that `labels` maps to `false` SHALL be left out.
2. The binding's `desc`, when it is a non-empty string that differs from the description `gband.action.list()` gives its action.
3. For a built-in action, its short label in the table below.
4. The description `gband.action.list()` gives a registered action, when it is non-empty.
5. The action's full name.

A binding to a function SHALL be labelled by its `desc`, and SHALL be left out when its `desc` is nil or empty.

| action | short label |
|---|---|
| `focus_column_left` | `left` |
| `focus_column_right` | `right` |
| `focus_window_down` | `down` |
| `focus_window_up` | `up` |
| `focus_band_down` | `band down` |
| `focus_band_up` | `band up` |
| `open_window` | `new` |
| `close_window` | `close` |
| `consume_or_expel_left` | `stack left` |
| `consume_or_expel_right` | `stack right` |
| `cycle_column_width` | `width` |
| `toggle_full_width` | `full` |
| `grow_column_width` | `wider` |
| `shrink_column_width` | `narrower` |
| `grow_window_height` | `taller` |
| `shrink_window_height` | `shorter` |
| `reset_window_height` | `reset height` |
| `detach` | `detach` |
| `send_prefix` | `send prefix` |

#### Scenario: Default description gives the short label
- **WHEN** the default configuration binds `prefix r` to `cycle_column_width` with the action's description
- **THEN** its hint shows `r width`

#### Scenario: Own description
- **WHEN** `user/init.lua` binds `prefix g` to `gband.action.focus_column_left` with description `go west`
- **THEN** its hint shows `g go west`

#### Scenario: Label option
- **WHEN** the hints segment is set up with `{ labels = { close_window = "kill" } }` and the default bindings are in use
- **THEN** the hint for `q` shows `q kill`

#### Scenario: Label option hides an action
- **WHEN** the hints segment is set up with `{ labels = { send_prefix = false } }` and the user presses Ctrl+Space
- **THEN** no hint shows `send prefix`

#### Scenario: Registered action
- **WHEN** the plugin `hello` registers `greet` with description `say hi` and binds `alt+g` in `root` to it
- **THEN** the root hints show `C-space prefix  A-g say hi`

#### Scenario: Registered action without a description
- **WHEN** the plugin `hello` registers `greet` with no description and binds `alt+g` in `root` to it
- **THEN** its hint shows `A-g hello.greet`

#### Scenario: Function without a description
- **WHEN** `user/init.lua` binds `prefix x` to a Lua function with no description
- **THEN** the prefix table's hints hold no hint for `x`
