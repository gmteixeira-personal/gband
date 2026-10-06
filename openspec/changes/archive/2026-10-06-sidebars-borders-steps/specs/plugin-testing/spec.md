## MODIFIED Requirements

### Requirement: Case environment
`g.start(opts)` SHALL start the case's gband. It SHALL be called once, before any other function of the handle; calling it again, or calling another function first, SHALL be an error. `opts` fields, all optional:

| field | meaning | default |
|---|---|---|
| `size` | the terminal size, `"<cols>x<rows>"` | `"80x24"` |
| `config` | the contents of `user/init.lua` | no file: the default configuration applies |
| `server_config` | the contents of `user/server.lua` | no file: the default server configuration applies |
| `files` | a table from paths relative to the configuration directory to file contents | none |
| `plugins` | a list of paths of further plugins for this case | none |
| `env` | a table of environment variables to set, or to remove with `false` | none |
| `time` | the instant the test-channel capability freezes, as Unix seconds or `"YYYY-MM-DD HH:MM:SS"` in UTC, or `false` for the real time | `"2025-01-01 12:00:00"` |

Each case SHALL run in a new directory tree of its own under the system's temporary directory, holding the configuration, data, state and runtime directories of the case and a working directory. The plugin under test, every `--plugin` path and every path in `plugins` SHALL be linked into the case's plugins directory under its last path component. The directory first in `PATH` SHALL hold the programs `xdg-open` and `open`, which record their argument for `g.opened()` and open nothing. The case's gband SHALL run with an environment holding `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_STATE_HOME` and `XDG_RUNTIME_DIR` set to the tree, `SHELL=/bin/sh`, `PS1` set to `$ `, `TERM=xterm-256color`, `COLORTERM=truecolor`, `TZ=UTC`, `GBAND_ANIMATIONS=off`, `INPUTRC=/dev/null`, `PATH` with a directory of the tree first, and `GBAND_TEST_SOCKET` set as the test-channel capability defines, and without `GBAND`, `GBAND_SESSION`, `GBAND_WINDOW` and `GBAND_LOG`, then changed by `env`. `g.start` SHALL run `gband attach` from the executable running the test, in the working directory of the tree, in a new PTY of the size given, and SHALL return once the client has drawn its first frame with the configuration loaded.

When the case ends, whether it passed or not, the runner SHALL stop the case's client and server and every process they started, and remove the tree.

#### Scenario: Configuration of the case
- **WHEN** a case starts with `config = [[gband.plugin("gband.statusline", { side = "right" })]]`
- **THEN** the case's status line is drawn on the terminal's last 20 columns
- **AND** the user's own `~/.config/gband/user/init.lua` is not read

#### Scenario: Plugin under test is installed
- **WHEN** `gband test` runs in the plugin directory `window` and a case starts with `config = [[gband.plugin("window")]]`
- **THEN** the plugin's segment is drawn, read from the working directory's files

#### Scenario: Nothing left behind
- **WHEN** a case opens three windows and then fails
- **THEN** after the run, none of the case's processes is running and its directory tree does not exist

#### Scenario: Run from inside a window
- **WHEN** the user runs `gband test` from a window of their own gband server
- **THEN** each case's client starts its own server in the case's tree instead of refusing to nest
