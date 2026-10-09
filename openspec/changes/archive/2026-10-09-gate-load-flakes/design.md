## Context

See proposal.md, "Why", for the three causes and how each was reproduced.

`watch` in `crates/lua/src/watch.rs` builds a `notify::RecommendedWatcher` (inotify on Linux). It watches the `user` directory recursively, or the configuration directory when `user` does not exist yet (`Watching::Parent`). A `gband-config` thread receives the events, debounces them for 100 ms and calls `changed`. When the watcher cannot be created, `watch` logs a warning and returns an idle `Watcher`, so that process never reloads again. When neither path can be watched, it logs `not watching the configuration: … does not exist`, even if the real error was a limit. `watch.rs` is the only user of the `notify` crate.

The tests in `tests/` drive a real client in a PTY through `crates/harness`. `Attached::wait_for_prompt` waits for a focused tile with a line ending in `$`. `Attached::last_pid` reads the newest `pid=` line of the focused window, and `crates/harness/src/env.rs` has `children(pid)`, which reads `/proc/<pid>/task/*/children`.

## Goals / Non-Goals

**Goals:**
- No inotify limit can stop a process from reloading within two seconds.
- Each fixed test waits for the state it reads before it types.
- Each fix is checked on its own reproduction (below).

**Non-Goals:**
- A general audit of `thread::sleep` in `tests/`. Only the sleeps that guard a `cat -v` probe change.

## Decisions

**Always poll, as niri does.** niri never uses inotify for its configuration: a thread compares the file's modification time and canonical path every 500 ms (`src/utils/watcher.rs`). `watch` does the same for the `user` directory, reading no file's contents. A `gband-config` thread takes a snapshot of every file under `user` whose name ends in `.lua`, keyed by path, holding the file's modification time at full precision and its size. `watch` takes the first snapshot before it returns, so a write right after `watch` returns counts as a change, as it did with inotify. The thread takes a new snapshot once a second. When the snapshot differs, it waits until a snapshot taken 100 ms later is unchanged, so a save still in progress is never loaded, then calls `changed`.

- A snapshot lists each directory under `user` and reads each entry's type from the listing, so a folder or a file that does not end in `.lua` costs no `stat`. Only `.lua` files are statted, plus symlinks, to learn what they point to.
- A created, written, replaced or removed `.lua` file changes the snapshot. A file that does not end in `.lua`, such as `07-settings.md`'s temporary file or a `notes.txt`, is never in it. `defaults` and the plugins directory are outside `user`, so they are never scanned.
- A missing `user` directory gives an empty snapshot, so its later creation with a `.lua` file in it is a change like any other. The `Parent`-to-`Directory` switch and the event filters go.
- A symlinked `.lua` file also records its canonical path, as niri does, which catches a symlink retargeted to another file with an older modification time. A symlinked directory is followed once per snapshot: its canonical path is kept, so a symlink loop ends.
- The modification time is compared at the precision the file system stores, nanoseconds on ext4, btrfs and xfs. `notify`'s `PollWatcher` truncates it to whole seconds, so it misses a second write within the same second unless it also hashes every file's contents on each poll.
- `Watcher` holds the sending end of a channel. The thread waits on the receiving end with a timeout, so it stops as soon as the `Watcher` is dropped.

The one-second interval plus the 100 ms quiet period can take a reload just past one second, so "Reload on change" and the scenarios in `settings` and `key-style` that wait for a reload allow two seconds. A snapshot lists a few directories and stats a handful of files once a second, which costs far less than the Lua load a reload runs.

Alternatives considered:
- *Fall back to polling only when inotify hits a limit.* Rejected: it keeps two code paths, the fallback is reached in tests only through a seam, and normal use saves only a few `stat` calls a second.
- *`notify::PollWatcher` with content hashing.* Rejected: it reads every file under `user` on every poll to make up for whole-second modification times.
- *Retry the native watcher later.* Rejected: nothing frees an instance on a schedule, so the process could go without reloads indefinitely.
- *Lower the gate's parallelism.* Rejected: it only makes the limit less likely to be reached, and users hit the limit without running the gate.
- *Stat only the files the last load read, as niri does with its includes.* Rejected: the `require` searcher would have to report every path it loaded, and a new `.lua` file that nothing requires yet would stop reloading, which changes "Reload on change".
- *Reload on the first snapshot that differs, as niri does.* Rejected: a save caught halfway would load once and report an error before the next snapshot fixed it.

**Test the poller directly, check it end to end by hand.** A unit test beside the existing ones in `watch.rs` writes `user/init.lua` twice in quick succession and asserts, using the existing `next` helper, that each write loads. It is not done with an end-to-end test under a user namespace, because unprivileged user namespaces are disabled on some CI hosts (Ubuntu 24.04 AppArmor). The end-to-end check runs once, by hand, as task 1.3 describes.

**Mouse tests wait for focus and the prompt.** In both tests, the wait after prefix `u n Enter` also requires the single tile to be focused, then calls `wait_for_prompt`. The leaving band's tile is never drawn focused, so a focused single tile while the view is on the new band is the new window. `tests/attach.rs` already uses this shape after prefix `u` and `n`.

**A harness wait for a child program.** `Attached::wait_for_child(&self, shell: i32, program: &str)` waits, up to `TIMEOUT`, until a child of `shell` has `/proc/<child>/comm` equal to `program`. Bash restores the terminal's cooked mode when readline returns a line, before it forks, so once `cat` runs as a child, input is processed with `ICRNL` on. `echo_keys` reads `last_pid()` before it types `clear; cat -v`, because `clear` erases the pid line, then waits for `cat`. The four `tests/attach.rs` tests do the same with the pid `shell_pid` returned. The 200 ms sleeps between probe keys stay: they separate key events for the client's key parser, not readiness.

Alternatives considered:
- *A sentinel line echoed by `cat`.* Rejected: a sentinel sent too early has the same `ICRNL` race as the probe.
- *A longer sleep.* Rejected: it moves the threshold without removing it.

## Risks / Trade-offs

- [`/proc/<pid>/task/*/children` needs `CONFIG_PROC_CHILDREN`.] → `children` already relies on it for cleanup in `crates/harness/src/case.rs`, so tests already need it.
- [A write that leaves the modification time unchanged is missed, for example on a file system with one-second timestamps, or a tool that restores the old time.] → niri accepts the same limit. ext4, btrfs and xfs store nanoseconds, and editors write whole files.
- [A reload can now take up to 1.1 s longer than with inotify.] → The spec allows two seconds, and the unit tests assert it. `gband reload` and the `reload` action still reload at once.
