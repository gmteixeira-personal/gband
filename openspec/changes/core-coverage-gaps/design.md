## Context

See proposal.md for the motivation. `gband-core` has no inline test modules. All 79 of its tests are integration tests in `crates/core/tests/`, and they use only the crate's public API. Nothing in the repository measures coverage, and there is no CI.

## Goals / Non-Goals

**Goals:**
- Every line of `crates/core/src/` is run by a test in `crates/core/tests/`, as `cargo llvm-cov` reports it.
- Each new test states one behaviour in its name, in the style of the existing tests.

**Non-Goals:**
- Counting lines that only server or client tests run. Core is measured by its own tests alone.
- Region or branch coverage targets. They are reported for information, not enforced.
- Refactoring core so that it is easier to test.

## Decisions

### `cargo llvm-cov`, scoped to `gband-core`

Measure with `cargo llvm-cov -p gband-core --show-missing-lines`. It runs on the stable toolchain through `llvm-tools-preview`, and it reports the exact lines that are missing. The `-p` flag limits the run to core's own tests. Without it, server tests that drive the layout would hide gaps in core's tests.

Alternative: `cargo tarpaulin`. It works on Linux only and is less accurate on async code elsewhere in the workspace. Alternative: reading the code only. That is how the gaps in the proposal were found, and it misses branches inside expressions.

### Tests stay in the existing files and use the public API

New tests go into `layout.rs`, `view.rs`, `geometry.rs` and, where an operation's events are untested, `events.rs` under `crates/core/tests/`, and reuse each file's helpers. No `#[cfg(test)]` module is added to `src/`. A line that the public API cannot reach is reported as dead code, not covered from inside the crate.

### Assert the spec, else pin current behaviour

When a spec in `openspec/specs/` (mainly `layout`, `layout-view`, `input-encoding` and `session-events`) states the behaviour, the test asserts the spec. When no spec states it, the test pins what the code does now. Every pinned behaviour is listed in the change's `coverage.md`, so a reviewer can tell a chosen rule from an accidental one. A column holding more panes than the area has rows is the known example: it produces tiles of height 0.

### A failing test stops the work on that behaviour

When a new test shows that the code contradicts a spec, the test is not committed in a failing or `#[ignore]` state, and the code is not fixed here. The finding goes into `coverage.md` and is reported to the operator, who proposes the fix as its own change. This change modifies no file in `src/`.

## Risks / Trade-offs

- [A pinned behaviour is a bug, and the test now protects it] → `coverage.md` lists every pinned behaviour, and the report names them.
- [100% line coverage can be met by tests that assert nothing] → Each test asserts the observable result (layout shape, focus, camera, tiles, events or bytes), never only that a call returns.
- [`cargo-llvm-cov` is not installed on this machine] → Installing it changes the developer's toolchain. The implementer asks before running `rustup component add llvm-tools-preview` and `cargo install cargo-llvm-cov`.
