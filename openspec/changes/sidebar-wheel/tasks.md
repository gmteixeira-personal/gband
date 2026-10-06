## 1. Sidebar plugin

- [x] 1.1 In `crates/lua/src/runtime/gband/sidebar.lua`, register a `MouseScrolled` handler in `setup` that, for a step with `target == "outside"`, no Ctrl, Alt or Shift, on a shown sidebar's column, dispatches `gband.action.focus_band_down()` for `"down"` and `gband.action.focus_band_up()` for `"up"`, and ignores every other step; verify `cargo build` succeeds

## 2. Tests

- [x] 2.1 In `crates/client/tests/sidebar.rs`, add a wheel helper beside `click` and one test per "Band wheel" scenario: wheel down over the labels, two steps, wheel up, any row (rows 0, 1 and 23), past the last band, the right sidebar, Alt held with the default keys, and navigation mode; verify `cargo test -p gband-client --test sidebar` passes
- [x] 2.2 In `tests/lua/sidebar_spec.lua`, add a case that scrolls down over the sidebar with `g.mouse("scroll", "down", 0, 2)`, settles, and checks the viewed band and the bold label, following the "click a band label" case; verify `cargo test --test lua_specs` passes

## 3. Documentation

- [x] 3.1 In the README's Sidebar section, after the click sentence, say that the wheel over the sidebar views the band above or below, one band per step, in any mode; verify the section reads correctly beside the click sentence

## 4. Gate

- [x] 4.1 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace`, and verify all pass
