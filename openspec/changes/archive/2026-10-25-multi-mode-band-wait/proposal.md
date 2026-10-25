## Why

`band_change_moves_the_input` in `tests/multi_mode.rs` failed 2 of 7 runs during the 0.6.3 release gate, each time with `ParseIntError { kind: InvalidDigit }` at `tests/multi_mode.rs:58`. Built from `dev` at `490b6a3`, the `multi_mode` binary failed 5 of 40 runs idle and 4 of 40 with all 6 cores busy. The test sends prefix `u n Enter`, waits for a prompt and reads `gband.view().window`. Prefix `u` views the new, empty band, where the client has no focused window until the new shell opens. `wait_for_prompt` accepts the leaving band's prompt, which the first frame of the band switch still draws. `gband.view().window` is then nil, `gband eval` prints `nil`, and `focused_window` panics parsing it.

`other_bands_receive_nothing` in the same file opens its band with the same three lines. It has not been seen to fail, but it reads the same nil.

## What Changes

- `tests/multi_mode.rs` gains `open_band`, which sends prefix `u n Enter` and waits until `gband.view().window` is a window other than the one focused before. It sits beside `open_column` and `open_floating`, which already wait for a new focused window.
- `viewed_window` returns `gband.view().window` as an `Option<u32>`, so the wait retries while it is nil. `focused_window` unwraps it.
- `band_change_moves_the_input` and `other_bands_receive_nothing` call `open_band` in place of sending the keys and reading the focus at once.
- Test code only: no behaviour, spec or documentation changes.

Out of scope:
- Changing the client to keep a focused window while it views an empty band, or to skip drawing the leaving band's first frame. Both are harmless in use, and the tests can wait for the state they read.
- `tests/keystyle.rs`'s `with_bands`, which sends the same keys. It already waits with `wait_for_focused` for a prompt without the first band's label.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. The change edits tests only, so `.openspec.yaml` sets `skip_specs: true`.

## Impact

- `tests/multi_mode.rs`: the `open_band` and `viewed_window` helpers and their two callers.
- No product code, wire protocol, Lua API or documentation change.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- openspec/changes/multi-mode-band-wait/
- tests/multi_mode.rs
