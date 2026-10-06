## ADDED Requirements

### Requirement: Titles turned off
The client SHALL read the environment variable `GBAND_WINDOW_TITLES` when it starts. When its value is `off`, the client SHALL draw no border title, whatever the `window_titles` option holds. When it is unset or `on`, the client SHALL draw border titles, as "Border title" defines, while the client option `window_titles` is `true`, and SHALL draw none while it is `false`. Any other value SHALL be logged as a warning in the client's log and read as unset.

Turning titles off SHALL change only drawing. The server SHALL keep and send names, `gband.layout()` SHALL hold `name` and `manual_name`, and the rename prompt SHALL rename, as with titles on. A reload that changes `window_titles` SHALL take effect from the next frame the client draws.

#### Scenario: Option off
- **WHEN** `user/init.lua` calls `gband.set({ window_titles = false })` and the only window runs bash
- **THEN** the tile's top border shows `─` across its width
- **AND** `gband.layout()` gives the window the `name` `bash`

#### Scenario: Environment turns titles off
- **WHEN** a client starts with `GBAND_WINDOW_TITLES=off` and `user/init.lua` sets `window_titles` to `true`
- **THEN** no tile's top border shows its name

#### Scenario: Unknown value
- **WHEN** a client starts with `GBAND_WINDOW_TITLES=no`
- **THEN** the client's log holds a warning naming `GBAND_WINDOW_TITLES` value `no`
- **AND** tiles show their names on their top borders

#### Scenario: Turned back on by a reload
- **WHEN** `user/init.lua` sets `window_titles` to `false`, the user changes it to `true`, and the configuration reloads
- **THEN** the next frame shows each tile's name on its top border
