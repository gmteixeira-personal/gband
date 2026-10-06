## Why

The server makes its socket private by setting the process's file mode creation mask to `0o177` around `UnixListener::bind` and restoring it afterwards. The mask belongs to the whole process, not to the thread that sets it, so the step is not thread-safe. Two binds that overlap in one process can interleave so that one socket is created under the default mask, with mode `0755`, and the process keeps `0o177` afterwards. The lifecycle test `socket_is_private` failed this way during a gate run, and running that test binary 60 times with 16 test threads reproduced it once. In the real server, the Lua scripting thread and the tokio workers already run while the socket is bound, so any file, directory or child process they create in that moment gets the restrictive mask: a directory created then is unusable.

## What Changes

- The server never changes the file mode creation mask.
- The server binds its socket inside a new private directory beside the socket path, sets the socket to mode `0600` there, and moves it to the socket path in one atomic rename, then removes the directory. The socket is never reachable at its path with any mode but `0600`.
- The bind goes through the directory's file descriptor, so a socket path of up to 107 bytes keeps working however long the private directory's path is.
- A regression test binds many sockets at once in one process and checks that every socket has mode `0600` and that the process's mask is unchanged afterwards. `socket_is_private` stays as it is.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `session-server`: "Socket location" states that the socket has mode `0600` from the moment it appears at its path, and that creating it leaves the process's file mode creation mask unchanged.

## Impact

- `crates/server/src/lib.rs`: `bind`, the `PRIVATE_SOCKET_MASK` constant, and unit tests for concurrent binds and a 107-byte path.
- `crates/server/tests/lifecycle.rs`: unchanged; `socket_is_private` keeps covering the whole server.
- No protocol, client or configuration change. `rustix` already provides the calls the new bind needs.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- openspec/changes/private-socket-bind/
- crates/server/src/lib.rs
