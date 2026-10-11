## Context

`PROTOCOL_VERSION` in `crates/protocol/src/message.rs` is 12, as in release 0.1.1. tabbed-columns appended `tabbed` and `tab_rows` to each column in the layout encoding, so two builds that both speak "version 12" disagree on the layout's bytes, and the handshake cannot tell them apart.

## Goals / Non-Goals

**Goals:**
- A client and a server that disagree on the layout encoding refuse each other at the handshake.

**Non-Goals:**
- No compatibility with version 12 servers. The client refuses them and names `gband kill-server`, as for every earlier bump.
- No check that catches the next field added without a bump. That is a separate change.

## Decisions

### Bump only the constant

The hello's leading version and the answer keep their encoding in every version, so 13 is the only edit the handshake needs. client-attach's "Version handshake" already prints the refusal and the kill command, and a debug build already replaces a mismatched server started from its own executable path.

### Keep the version 12 scenario's test

`version_eleven_client_is_rejected_by_version_twelve` sends a literal 11 and expects a literal 12. It becomes the version 11 against version 13 test, and a new test sends a literal 12 and expects 13, matching the new scenario. The literals stay, so each test pins the versions its scenario names.

## Risks / Trade-offs

- [A user upgrades with a 0.1.1 server running] → The 0.2.0 client refuses with one line naming `gband kill-server`. The windows of that server are lost when it is stopped, as with any protocol bump.
