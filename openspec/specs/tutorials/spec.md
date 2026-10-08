# tutorials Specification

## Purpose

Defines gband's tutorials: Markdown guides that teach the Lua API by building working code, and the checks in gband's test suite that keep each tutorial's code, screens, API version and coverage of the API equal to the build that ships it.

## Requirements

### Requirement: Scripting tutorial
gband SHALL include a scripting tutorial in `docs/tutorial/`. It SHALL consist of an index, `docs/tutorial/README.md`, and chapters named `NN-<slug>.md`, where `NN` is two digits counting from `00`. A reader reads the chapters in ascending order of `NN`. The chapters SHALL be:

| chapter | teaches |
|---|---|
| `00-setup.md` | the configuration directory, writing `user/init.lua` from an empty file, reloading, the Lua prompt, `print` and the client log, and the error list |
| `01-keys.md` | key bindings, key tables and modes, key styles, and mouse names |
| `02-actions.md` | actions, action targets, commands, and starting programs in windows |
| `03-options.md` | options and the settings window |
| `04-events.md` | events, event groups and `User` events |
| `05-layout.md` | reading the layout and the view, and acting on windows and bands |
| `06-colors.md` | highlight groups, colorschemes, themes and the terminal palette |
| `07-plugin-windows.md` | plugin windows |
| `08-bars.md` | side bars and the sidebar |
| `09-plugin.md` | moving the reader's code into a plugin: the manifest, the side guard, `gband.plugin`, options of a plugin, and ownership |
| `10-server.md` | the plugin's server half: server events, window state, server commands, `gband.emit` from the server, and `gband.rpc` |
| `11-testing.md` | writing the plugin's tests with `gband test`, and screenshots |

The chapters SHALL start from an empty `user/init.lua` and end with a plugin that has a client half, a server half and tests. A chapter SHALL link to the section of `docs/plugins.md` or `docs/testing.md` that documents each part of the API it introduces. The index SHALL link to every chapter, in order. The Scripting section of `README.md` and the introduction of `docs/plugins.md` SHALL link to the index.

#### Scenario: Index lists the chapters
- **WHEN** a reader opens `docs/tutorial/README.md`
- **THEN** it links to `00-setup.md` first and `11-testing.md` last, and to every other chapter between them in ascending order

#### Scenario: Reached from the README
- **WHEN** a reader opens the Scripting section of `README.md`
- **THEN** it links to `docs/tutorial/README.md`

### Requirement: Chapter directories
Each chapter `docs/tutorial/NN-<slug>.md` SHALL have a directory `examples/tutorial/NN-<slug>/`, and each directory under `examples/tutorial/` SHALL have a chapter. The directory SHALL hold every file the reader has written by the end of the chapter, laid out as the reader installs it:

| path in the chapter directory | installed at |
|---|---|
| `user/` | `user/` of the configuration directory |
| `plugins/<name>/` | `<name>/` of the plugins directory |

The directory SHALL also hold `tests/`, with at least one `_spec.lua` file and the screenshot references its cases compare. `gband test`, run in the chapter directory, SHALL pass every case. Each case SHALL start gband with the chapter's files: the text of `user/init.lua` and `user/server.lua` as the `config` and `server_config` of `g.start`, any other file under `user/` through `files`, and every plugin under `plugins/` through `plugins`. A plugin directory under `plugins/` MAY hold `tests/` of its own, the plugin's tests that the chapter has the reader write; `gband test` run in that plugin directory SHALL pass every case. gband's test suite SHALL run `gband test` in every chapter directory and in every plugin directory of a chapter that holds `tests/`, and SHALL fail when a case in one of them fails.

#### Scenario: Chapter tests run
- **WHEN** gband's test suite runs
- **THEN** `gband test` runs in `examples/tutorial/00-setup/` and in every other chapter directory, and every case passes

#### Scenario: Plugin tests run
- **WHEN** `examples/tutorial/11-testing/plugins/marks/tests/marks_spec.lua` exists and gband's test suite runs
- **THEN** `gband test` also runs in `examples/tutorial/11-testing/plugins/marks/`, with `marks` as the plugin under test, and every case passes

#### Scenario: Broken chapter
- **WHEN** a change renames `gband.keymap.set` and leaves `examples/tutorial/01-keys/user/init.lua` calling it
- **THEN** gband's test suite fails and names `examples/tutorial/01-keys`

#### Scenario: Chapter without a directory
- **WHEN** `docs/tutorial/12-more.md` exists and `examples/tutorial/12-more/` does not
- **THEN** gband's test suite fails and names `docs/tutorial/12-more.md`

### Requirement: Code blocks match the chapter directory
The text of every fenced code block whose info string's first word is `lua`, in a chapter, SHALL appear verbatim in a file under that chapter's directory, including its indentation. A code block whose info string's first word is not `lua` is not checked by this requirement. gband's test suite SHALL fail when a `lua` block appears in no file of its chapter's directory, and SHALL name the chapter and the first line of the block.

#### Scenario: Block found
- **WHEN** `docs/tutorial/01-keys.md` holds a `lua` block whose text appears in `examples/tutorial/01-keys/user/init.lua`
- **THEN** the check passes for that block

#### Scenario: Prompt line found in a spec
- **WHEN** `docs/tutorial/00-setup.md` holds a `lua` block with one line that the reader types at the Lua prompt, and `examples/tutorial/00-setup/tests/setup_spec.lua` types the same line
- **THEN** the check passes for that block

#### Scenario: Stale block
- **WHEN** `docs/tutorial/04-events.md` holds a `lua` block that no file under `examples/tutorial/04-events/` contains
- **THEN** gband's test suite fails and names `docs/tutorial/04-events.md` and the block's first line

### Requirement: Screens match screenshot references
A chapter SHALL show what the reader sees in fenced code blocks whose info string is `screen` followed by a path, relative to the chapter's directory, to a screenshot reference that the chapter's tests compare. The text of such a block SHALL equal the screen rows of that reference, in order, each without its `N|` prefix. The size line, the cursor and the cell attributes are not shown. gband's test suite SHALL fail when a `screen` block names a reference that does not exist, or when its text differs from the reference's rows, and SHALL name the chapter and the reference.

#### Scenario: Screen shown
- **WHEN** `docs/tutorial/08-bars.md` holds a block opened by `` ```screen tests/screenshots/bars_spec/shows-the-bar--two-windows.txt `` and the block's text equals that reference's rows without their prefixes
- **THEN** the check passes for that block

#### Scenario: Screen changed
- **WHEN** a change redraws the sidebar, `gband test --update` rewrites `examples/tutorial/08-bars/tests/screenshots/bars_spec/shows-the-bar--two-windows.txt`, and `docs/tutorial/08-bars.md` still shows the old rows
- **THEN** gband's test suite fails and names `docs/tutorial/08-bars.md` and the reference

### Requirement: Tutorial API version
At least one Lua file under `examples/tutorial/` SHALL set a plugin's `api` field, as the plugins capability defines it. Every `api` field that a Lua file under `examples/tutorial/` sets SHALL equal the `gband.api_version` of the build. gband's test suite SHALL fail when one differs, and SHALL name the file.

#### Scenario: Version raised
- **WHEN** a change raises `gband.api_version` to 2 and `examples/tutorial/09-plugin/plugins/marks/lua/marks/init.lua` still sets `api = 1`
- **THEN** gband's test suite fails and names that file

### Requirement: Every namespace taught
For every top-level field of the client's `gband` table and of the server's `gband` table, collected with only the API installed and before any configuration file runs, the text `gband.<field>` SHALL appear in at least one chapter. The field `core` is exempt. gband's test suite SHALL fail when a field appears in no chapter, and SHALL name the field.

#### Scenario: Every field named
- **WHEN** a test collects the top-level fields of the client's and the server's `gband` tables
- **THEN** each one other than `core`, such as `keymap`, `window_state` and `sessions`, appears as `gband.<field>` in some chapter

#### Scenario: New namespace without a chapter
- **WHEN** a change adds a top-level field `gband.menu` to the client and no chapter names `gband.menu`
- **THEN** gband's test suite fails and names `gband.menu`

### Requirement: Internals tutorial
gband SHALL include an internals tutorial in `docs/internals/`. It SHALL explain gband's bundled Lua as the files are, in the order the prelude loads the API. It SHALL consist of an index, `docs/internals/README.md`, and chapters named `NN-<slug>.md`, where `NN` is two digits counting from `00`. A reader reads the chapters in ascending order of `NN`. The chapters SHALL be:

| chapter | teaches |
|---|---|
| `00-boundary.md` | what the executable provides and what Lua builds: `gband.core`, the providers, the prelude, `require` and the runtimepath, the copies under `defaults/`, and replacing a bundled file by copying it under `user/` |
| `01-themes.md` | colorscheme files, through the `gruvbox` theme |
| `02-highlights.md` | the modules `gband.palette` and `gband.hl`, and the styles provider |
| `03-colorschemes.md` | the modules `gband.colorscheme` and `gband.theme` |
| `04-bars.md` | the module `gband.bar` and the bars provider |
| `05-plugin-windows.md` | the state, drawing and API of the module `gband.win` |
| `06-window-provider.md` | the windows provider of `gband.win`: its input, layout and drawing functions |
| `07-settings.md` | the module `gband.settings` and the settings provider |
| `08-key-styles.md` | the module `gband.keystyle`, the modal, direct and floating presets, and the bundled plugin `gband.desktop` that the floating preset sets up |
| `09-sidebar-and-errors.md` | the bundled plugins `gband.sidebar` and `gband.errors` |
| `10-keylist-and-prompt.md` | the bundled plugins `gband.keylist` and `gband.prompt`, and the module `gband.keyform` |
| `11-default-configs.md` | the default client and server configurations |
| `12-own-prelude.md` | writing the reader's own `gband.prelude` |

The index SHALL link to every chapter, in order. The Scripting section of `README.md`, the section "The prelude and the API modules" of `docs/plugins.md`, and the scripting tutorial's index SHALL link to the internals tutorial's index.

#### Scenario: Index lists the chapters
- **WHEN** a reader opens `docs/internals/README.md`
- **THEN** it links to `00-boundary.md` first and `12-own-prelude.md` last, and to every other chapter between them in ascending order

#### Scenario: Reached from the plugin guide
- **WHEN** a reader opens the section "The prelude and the API modules" of `docs/plugins.md`
- **THEN** it links to `docs/internals/README.md`

#### Scenario: Reached from the scripting tutorial
- **WHEN** a reader opens `docs/tutorial/README.md`
- **THEN** it links to `docs/internals/README.md`

#### Scenario: Floating style explained
- **WHEN** a reader opens `docs/internals/08-key-styles.md`
- **THEN** it quotes `defaults/keystyle/floating.lua` and `defaults/lua/gband/desktop.lua`, and names both paths outside fenced code blocks

### Requirement: Quotes of bundled files
A fenced code block in an internals chapter whose info string is `lua` followed by a second word is a quote. The second word SHALL be the path, relative to the configuration directory, of a file that gband writes under `defaults/`, such as `defaults/lua/gband/bar.lua` or `defaults/colors/gruvbox.lua`. The quote's lines SHALL equal a run of consecutive lines of that file's text, verbatim, indentation and blank lines included. The run SHALL occur at exactly one position in the file. gband's test suite SHALL fail when a quote names a path that gband does not write, matches no run of the file, or matches more than one, and SHALL name the chapter, the path and the quote's first line.

#### Scenario: Quote found
- **WHEN** `docs/internals/04-bars.md` holds a block opened by `` ```lua defaults/lua/gband/bar.lua `` whose lines equal lines 1 to 12 of the text gband writes at `defaults/lua/gband/bar.lua`
- **THEN** the check passes for that quote

#### Scenario: Stale quote
- **WHEN** a change edits a line of the bundled `gband.sidebar` module and `docs/internals/09-sidebar-and-errors.md` still quotes the old line
- **THEN** gband's test suite fails and names `docs/internals/09-sidebar-and-errors.md`, `defaults/lua/gband/sidebar.lua` and the quote's first line

#### Scenario: Ambiguous quote
- **WHEN** a quote of `defaults/lua/gband/win.lua` holds only the line `  end`
- **THEN** gband's test suite fails, because the line occurs at more than one position, and names the chapter, the path and `  end`

#### Scenario: Unknown path
- **WHEN** a chapter holds a block opened by `` ```lua defaults/lua/gband/menu.lua `` and gband writes no such file
- **THEN** gband's test suite fails and names the chapter and `defaults/lua/gband/menu.lua`

### Requirement: Every bundled file covered
The covered files SHALL be every file that gband writes under `defaults/`, except the files under `defaults/colors/` other than `defaults/colors/gruvbox.lua`, and except `defaults/lua/gband/theme/catppuccin.lua`. Every non-blank line of every covered file SHALL be in exactly one quote across the internals chapters, and no line of any file SHALL be in more than one quote. Quotes MAY appear in any order and in any chapter. Every covered file SHALL be named by its path, such as `defaults/colors/gruvbox.lua`, outside fenced code blocks in at least one internals chapter. gband's test suite SHALL fail when a line of a covered file is quoted in no chapter, naming the file, the line's number and its text; when a line is quoted twice, naming the file, the line's number and both chapters; and when a covered file is named in no chapter, naming the file.

#### Scenario: Whole file quoted
- **WHEN** gband's test suite runs on the tutorial as written
- **THEN** every non-blank line of `defaults/lua/gband/win.lua`, `defaults/keystyle/modal.lua`, `defaults/init.lua`, `defaults/colors/gruvbox.lua` and every other covered file is in exactly one quote

#### Scenario: New function not explained
- **WHEN** a change adds a function to the bundled `gband.bar` module and no chapter quotes it
- **THEN** gband's test suite fails and names `defaults/lua/gband/bar.lua`, the number of the function's first line and that line's text

#### Scenario: Line quoted twice
- **WHEN** `05-plugin-windows.md` and `06-window-provider.md` both quote the line of `defaults/lua/gband/win.lua` that registers the windows provider
- **THEN** gband's test suite fails and names the file, the line's number and both chapters

#### Scenario: Covered theme changed
- **WHEN** a change adds a field to the `ui` table of the bundled `gruvbox` theme and no chapter quotes the new line
- **THEN** gband's test suite fails and names `defaults/colors/gruvbox.lua`, the new line's number and its text

#### Scenario: Other theme data not covered
- **WHEN** a change edits a color of the bundled `nord` theme and a color in `defaults/lua/gband/theme/catppuccin.lua`, and adds a bundled colorscheme `defaults/colors/ocean.lua`, and no chapter quotes or names any of them
- **THEN** the coverage and naming checks pass

#### Scenario: File never named
- **WHEN** a change adds a bundled module whose copy is `defaults/lua/gband/notes.lua`, every line of it is quoted, and no chapter names `defaults/lua/gband/notes.lua` outside fenced code blocks
- **THEN** gband's test suite fails and names `defaults/lua/gband/notes.lua`

### Requirement: Every primitive taught
For every field of the client's `gband.core` table, at every depth, collected with only the API installed and before any configuration file runs, the text `gband.core.<path>`, such as `gband.core.provide` or `gband.core.palette.set`, SHALL appear outside fenced code blocks in at least one internals chapter. gband's test suite SHALL fail when a field appears in no chapter, and SHALL name the field.

#### Scenario: Primitive explained
- **WHEN** `docs/internals/00-boundary.md` explains `gband.core.provide` in its text
- **THEN** the check passes for `gband.core.provide`

#### Scenario: Primitive only quoted
- **WHEN** the only text that names the timer primitive is a quote of bundled code that calls `core.timer`
- **THEN** gband's test suite fails and names `gband.core.timer`

### Requirement: Internals chapter directories
An internals chapter MAY have a directory `examples/internals/NN-<slug>/` with the chapter's name, and each directory under `examples/internals/` SHALL have a chapter. A chapter SHALL have one when it holds a fenced block whose info string is `lua` alone or starts with `screen`. The directory SHALL follow the layout, the test cases and the runs that the scripting tutorial's chapter directories follow: `user/`, `plugins/<name>/` and `tests/` with at least one `_spec.lua` file, and `gband test` run in it SHALL pass every case. A `lua` block whose info string is `lua` alone SHALL appear verbatim in a file under the chapter's directory. A `screen` block SHALL follow the scripting tutorial's rule for screens, with paths relative to the chapter's directory. `12-own-prelude.md` SHALL have a directory whose `user/lua/gband/prelude.lua` is the prelude the chapter has the reader write, and whose tests show the client loading it. gband's test suite SHALL run `gband test` in every directory under `examples/internals/`, and SHALL fail on a failed case, a missing directory, a `lua` block found in no file, or a `screen` block that differs from its reference, naming the chapter.

#### Scenario: Own prelude runs
- **WHEN** gband's test suite runs
- **THEN** `gband test` runs in `examples/internals/12-own-prelude/`, every case passes, and a case shows that the client loaded `user/lua/gband/prelude.lua` in place of the bundled prelude

#### Scenario: Reader code without a directory
- **WHEN** `docs/internals/04-bars.md` holds a block opened by `` ```lua `` alone and `examples/internals/04-bars/` does not exist
- **THEN** gband's test suite fails and names `docs/internals/04-bars.md`

#### Scenario: Quotes need no directory
- **WHEN** `docs/internals/02-highlights.md` holds only quotes and blocks whose info string starts with neither `lua` nor `screen`
- **THEN** no directory `examples/internals/02-highlights/` is required
