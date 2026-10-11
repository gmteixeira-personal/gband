## Why

tabbed-columns added the `tabbed` and `tab_rows` fields to each column of the layout the server sends, and left the protocol version at 12, the version release 0.1.1 speaks. A client built after tabbed-columns that attaches to a running 0.1.1 server passes the handshake, then fails with `cannot decode message: Found an Option discriminant that wasn't 0 or 1` and names no remedy. Release 0.2.0 would ship that failure to every user who upgrades with a server still running.

## What Changes

- Raise the protocol version from 12 to 13, so a 0.2.0 client and a 0.1.1 server refuse each other at the handshake. The client then prints the line client-attach's "Version handshake" defines, naming both versions and `gband kill-server`, and a debug build replaces a server from its own executable path.
- No message changes encoding. The layout already carries `tabbed` and `tab_rows`; version 13 names that encoding.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `wire-protocol`: "Handshake" sets the current protocol version to 13 and adds the scenario of a version 12 client meeting a version 13 server.

## Impact

- Protocol: `PROTOCOL_VERSION` in `crates/protocol/src/message.rs` becomes 13.
- Tests: `crates/protocol/tests/messages.rs` checks the version is 13; `crates/server/tests/sync.rs` checks a version 12 client is rejected with version 13.
- Users: a server started from 0.1.1 or from a build before this change refuses a 0.2.0 client, which names `gband kill-server`. Stopping the old server and attaching again starts a version 13 server.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- crates/protocol/src/message.rs
- crates/protocol/tests/messages.rs
- crates/server/tests/sync.rs
- openspec/changes/protocol-13/
