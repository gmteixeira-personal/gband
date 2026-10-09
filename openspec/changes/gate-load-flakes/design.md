## Context

See proposal.md, "Why", for the three causes and how each was reproduced.

`watch` in `crates/lua/src/watch.rs` builds a `notify::RecommendedWatcher` (inotify on Linux). It watches the `user` directory recursively, or the configuration directory when `user` does not exist yet (`Watching::Parent`). A `gband-config` thread receives the events, debounces them for 100 ms and calls `changed`. When the watcher cannot be created, `watch` logs a warning and returns an idle `Watcher`, so that process never reloads again. When neither path can be watched, it logs `not watching the configuration: … does not exist`, even if the real error was a limit.

The tests in `tests/` drive a real client in a PTY through `crates/harness`. `Attached::wait_for_prompt` waits for a focused tile with a line ending in `$`. `Attached::last_pid` reads the newest `pid=` line of the focused window, and `crates/harness/src/env.rs` has `children(pid)`, which reads `/proc/<pid>/task/*/children`.

## Goals / Non-Goals

**Goals:**
- A process that hits an inotify limit still reloads within one second.
- Each fixed test waits for the state it reads before it types.
- Each fix is checked on its own reproduction (below).

**Non-Goals:**
- Switching to polling after a reload has started, when inotify runs out of watches while adding a new subdirectory. `notify` reports that case as an `Err` event. It needs a new subdirectory to be created while the watch table is full, and it is left as today.
- A general audit of `thread::sleep` in `tests/`. Only the sleeps that guard a `cat -v` probe change.

## Decisions

**Fall back to `notify::PollWatcher` on a limit error.** `watch` keeps the native watcher first. It switches to a `PollWatcher` with a 250 ms poll interval when either of these fails with a limit error:
- creating the native watcher, which on inotify is `ErrorKind::Io` with `EMFILE` (instance limit);
- the first `watch` call on either path, which is `ErrorKind::MaxFilesWatch` (`ENOSPC`, watch limit).

The 250 ms interval plus the 100 ms debounce keeps a reload under the one-second bound with room for a slow machine. The fallback logs a warning: `watching the configuration by polling: <error>`. The stored watcher becomes `Arc<Mutex<Box<dyn notify::Watcher + Send>>>`, so `reload` and the `Parent`-to-`Directory` switch work unchanged for both backends. A watch that fails because the path does not exist keeps today's behaviour, and polling a missing directory would not help anyway.

Alternatives considered:
- *Always poll.* Rejected: it costs a directory scan every 250 ms in every gband process to fix a case that only occurs at a limit.
- *Retry the native watcher later.* Rejected: nothing frees an instance on a schedule, so the process could go without reloads indefinitely.
- *Lower the gate's parallelism.* Rejected: it only makes the limit less likely to be reached, and users hit the limit without running the gate.

**Test the fallback through a seam, check it end to end by hand.** `watch` calls `watch_with(config, changed, Backend::Native)`. A unit test beside the existing ones in `watch.rs` calls `watch_with(…, Backend::Poll)` and asserts, using the existing `next` helper, that an edit to `user/init.lua` reloads within one second. It is not done with an end-to-end test under a user namespace, because unprivileged user namespaces are disabled on some CI hosts (Ubuntu 24.04 AppArmor). The end-to-end check runs once, by hand, as task 1.3 describes.

**Mouse tests wait for focus and the prompt.** In both tests, the wait after prefix `u n Enter` also requires the single tile to be focused, then calls `wait_for_prompt`. The leaving band's tile is never drawn focused, so a focused single tile while the view is on the new band is the new window. `tests/attach.rs` already uses this shape after prefix `u` and `n`.

**A harness wait for a child program.** `Attached::wait_for_child(&self, shell: i32, program: &str)` waits, up to `TIMEOUT`, until a child of `shell` has `/proc/<child>/comm` equal to `program`. Bash restores the terminal's cooked mode when readline returns a line, before it forks, so once `cat` runs as a child, input is processed with `ICRNL` on. `echo_keys` reads `last_pid()` before it types `clear; cat -v`, because `clear` erases the pid line, then waits for `cat`. The four `tests/attach.rs` tests do the same with the pid `shell_pid` returned. The 200 ms sleeps between probe keys stay: they separate key events for the client's key parser, not readiness.

Alternatives considered:
- *A sentinel line echoed by `cat`.* Rejected: a sentinel sent too early has the same `ICRNL` race as the probe.
- *A longer sleep.* Rejected: it moves the threshold without removing it.

## Risks / Trade-offs

- [`/proc/<pid>/task/*/children` needs `CONFIG_PROC_CHILDREN`.] → `children` already relies on it for cleanup in `crates/harness/src/case.rs`, so tests already need it.
- [A `PollWatcher` misses a write that keeps the same mtime and size within one poll.] → Editors and the tests write whole files, which change the mtime. The unit test covers a plain write.
- [The fallback hides a misconfigured system.] → It logs a warning naming the error, and the spec requires that.
