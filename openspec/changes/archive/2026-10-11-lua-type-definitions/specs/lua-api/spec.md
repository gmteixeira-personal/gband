## ADDED Requirements

### Requirement: Type definitions
gband SHALL bundle one type definition file for lua-language-server: a Lua file whose first line is `---@meta` and which declares the `gband` global. It SHALL declare every path that the walk of the "Documented API" requirement collects from the client's `gband` table and from the server's `gband` table, with every bundled plugin of the "Bundled Lua consumes the API" requirement loaded on the client. It SHALL declare no other path under `gband`, except that `gband.action` SHALL accept any string key as a function, so that a plugin's action such as `gband.action["keylist.open"]` resolves. Each declared function SHALL carry the parameter and return types that `docs/plugins.md` gives it, and each declaration SHALL say whether the field exists on the client, on the server, or on both.

Loading SHALL never run the type definition file, and it SHALL lie in no runtimepath entry. The file changes nothing that a chunk can observe, so adding or changing it SHALL NOT change `gband.api_version`.

`docs/plugins.md` SHALL describe the type definition file and the editor settings file that the configuration capability's "Configuration directory" writes, and SHALL give the lua-language-server setting that makes the type definition file apply to a plugin directory outside the configuration directory.

#### Scenario: Every path declared
- **WHEN** a test walks the client's `gband` table with every bundled plugin loaded and the server's `gband` table, collecting each path such as `gband.core.present_window` and `gband.keymap.set`
- **THEN** the type definition file declares every collected path

#### Scenario: No stale declaration
- **WHEN** a test collects every path the type definition file declares under `gband`, other than the string-keyed entry of `gband.action`
- **THEN** the walk of the client's or the server's `gband` table collected each of them

#### Scenario: A new field without a declaration fails
- **WHEN** a build adds a field to the client's `gband` table and does not declare it in the type definition file
- **THEN** the test of "Every path declared" fails and names the field

#### Scenario: Default key style checks clean
- **WHEN** lua-language-server checks a configuration directory that a client has prepared, with that directory as its workspace
- **THEN** it reports no undefined global `gband` and no undefined field of `gband` in `defaults/keystyle/modal.lua`, `defaults/init.lua` or `defaults/server.lua`

#### Scenario: Definitions are not loaded
- **WHEN** the user writes `error("boom")` at the end of `defaults/meta/gband.lua` and reloads
- **THEN** the reload succeeds and no error is reported
