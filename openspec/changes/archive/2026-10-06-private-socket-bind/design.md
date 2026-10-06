## Context

`bind` in `crates/server/src/lib.rs` removes a stale socket file, then sets the umask to `PRIVATE_SOCKET_MASK` (`0o177`), calls `UnixListener::bind`, and restores the previous umask. Linux takes a new socket file's mode from `0777` minus the umask, so this gives `0600`, but the umask is per process. `run_with_events` calls `bind` after `lock::acquire` has taken the socket's lock, after `scripting::start` has started the Lua thread, and after `registry.create` has spawned the first session's program, so other threads are already running when the umask changes.

The lock guarantees one server per socket path. It does not stop two servers on different paths in one process, which is how the lifecycle tests run, from changing the umask at overlapping times.

The binary is Linux-only in practice: it already reads `/proc/self/exe` and `/proc/<pid>/exe`.

## Goals / Non-Goals

**Goals:**
- The socket file never exists at its path with a mode other than `0600`.
- No step changes the process's umask.
- Every socket path the command line accepts, up to 107 bytes, keeps working.

**Non-Goals:**
- Changing where the socket lives, the runtime directory checks or the lock.
- Changing the client's check of the server's user.

## Decisions

### Bind in a private directory, then rename

`bind` creates a new directory beside the socket path with mode `0700`, binds the socket inside it under a short fixed name, sets that file to `0600` with `fchmodat`, and moves it to the socket path with `renameat`. Then it removes the directory. Inside a `0700` directory nobody else can reach the socket, whatever mode the umask gave it, and the rename makes it appear at its path with `0600` already set. A rename within one directory's file system is atomic, and the private directory sits in the socket's own directory, so the two paths are on one file system. The rename also replaces a stale socket file in one step. The existing removal of a stale file stays, so its log line keeps working.

The directory's name starts with `.gband-bind-` and ends with the process id and a counter. Creation retries with the next counter on `EEXIST`, as `mkdtemp` does, so a name another user created first in a shared directory such as `/tmp` costs a retry, not a failure. `mkdir` never follows a symlink at its last component, and the sticky bit on `/tmp` stops other users from removing or replacing the directory once it is created.

A failure after the directory exists removes the socket file inside it and the directory before returning the error.

*Alternative:* bind under the umask, then `chmod` to `0600`. Rejected: the socket sits at its path with the umask's mode until the `chmod`. With a umask of `002` or `000`, other users can connect in that window.

*Alternative:* a process-wide lock around the umask change. Rejected: it serialises binds but still changes the umask for every other thread, so the Lua thread, the tokio workers and anything they spawn see `0o177`.

*Alternative:* `fchmod` on the socket descriptor before `bind`. Rejected: on Linux it changes the socket's own inode, not the file `bind` creates.

### Bind through the directory's descriptor

The private directory is opened with `O_PATH | O_DIRECTORY`, and the socket is bound to `/proc/self/fd/<fd>/s`. That path is at most about 25 bytes, so the private directory's longer path never pushes the bind past the 107 bytes a Unix socket address holds, and a 107-byte socket path the command line accepted still binds. systemd binds long socket paths the same way.

*Alternative:* bind to the private directory's full path. Rejected: the directory's name adds bytes, so a socket path near 107 bytes, which `-p` accepts, would fail to bind.

### Remove `PRIVATE_SOCKET_MASK`

The constant and the `rustix::fs::Mode` import go, since nothing sets a umask any more.

### Regression tests beside `bind`

Unit tests in `crates/server/src/lib.rs` call `bind` directly:
- Eight tasks on a multi-threaded runtime with eight workers wait on one barrier and bind eight sockets in one directory at once. Every socket has mode `0600`, and the umask read from the `Umask:` line of `/proc/self/status` is the same before and after. Reading it from `/proc` leaves it unchanged, which calling `umask` to read it would not.
- A socket path of exactly 107 bytes binds and has mode `0600`.
- No `.gband-bind-` directory is left in the socket's directory after either test.

`socket_is_private` in `crates/server/tests/lifecycle.rs` stays unchanged and keeps covering the whole server.

## Risks / Trade-offs

- [`/proc` not mounted] → the bind fails with an error naming the path. The binary already depends on `/proc` to identify itself and its server, so this adds no new requirement.
- [The server is killed between `mkdir` and the cleanup] → a `.gband-bind-<pid>-<n>` directory stays beside the socket. It holds at most one dead socket file, and a later bind uses another name, so nothing breaks. The runtime directory is private, so the leftover is visible only to the user.
- [A file system where rename within one directory is not atomic] → none that a Unix socket can live on.
