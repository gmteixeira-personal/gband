## ADDED Requirements

### Requirement: Error marker bars
`gband.bar.mark_errors(id, shown)` SHALL record whether the bar `id`'s lines show the error marker: `shown` `true` records that they do, and `false` that they do not. A bar SHALL start with `false` when it is added. `id` SHALL be resolved as `gband.bar.set_lines` resolves it, and `shown` SHALL be a boolean. A bar that does not exist, or a `shown` of another type, SHALL raise an error at the line of the call. The call SHALL be allowed while the configuration loads and in any callback, and SHALL change no cell of any bar.

A bar's error marker SHALL count as drawn while the client reports an error, as the configuration capability defines, the bar is shown, and its latest call of `gband.bar.mark_errors` recorded `true`. While any bar's error marker counts as drawn, the client SHALL draw no error banner. A bar that is removed, or is not shown, SHALL stop counting at once.

#### Scenario: Marked bar hides the banner
- **WHEN** `user/init.lua` adds a left bar `status` of size 3, calls `gband.bar.mark_errors("status", true)`, and a binding function raises `boom`
- **THEN** no banner is drawn

#### Scenario: Unmarked bar leaves the banner
- **WHEN** `user/init.lua` adds a left bar `status` of size 3 and never calls `gband.bar.mark_errors`, and a binding function raises `boom`
- **THEN** the error is drawn as the banner on the ribbon area's bottom row

#### Scenario: Hidden bar stops counting
- **WHEN** the bar `status` of size 3 marks errors, an error is reported, and the client's terminal shrinks to 3 columns, so the bar is not shown
- **THEN** the error is drawn as the banner on the ribbon area's bottom row

#### Scenario: Unmarked again
- **WHEN** the bar `status` marks errors, an error is reported, and a callback calls `gband.bar.mark_errors("status", false)`
- **THEN** the error is drawn as the banner once that callback returns

#### Scenario: Unknown bar
- **WHEN** line 6 of `user/init.lua` calls `gband.bar.mark_errors("nope", true)`
- **THEN** loading fails with an error at `user/init.lua` line 6

## MODIFIED Requirements

### Requirement: Bars and reloads
A reload SHALL remove every bar the previous configuration added, before the new configuration's `ConfigReloaded` handlers run. The bars the new configuration adds SHALL take their place, and the ribbon area SHALL change at most once for the reload. A bar the new configuration adds SHALL start unmarked, as "Error marker bars" defines, whatever the bar of the same id in the previous configuration recorded. A failed reload SHALL keep the bars as they were, with their marks. A bar SHALL be removed when the plugin it belongs to is marked failed.

#### Scenario: Reload keeps an unchanged bar
- **WHEN** `user/init.lua` adds a left bar of size 20, the user saves it unchanged, and the reload succeeds
- **THEN** the ribbon area does not change

#### Scenario: Failed plugin
- **WHEN** the plugin `tabs` adds a left bar in its `setup` and then raises an error
- **THEN** the bar is removed and the ribbon area is the whole terminal

#### Scenario: Mark not carried over a reload
- **WHEN** `user/init.lua` adds the bar `status` and marks it in a binding, and the user then removes the binding and saves, and the reloaded file reports an error from a plugin
- **THEN** the error is drawn as the banner on the ribbon area's bottom row
