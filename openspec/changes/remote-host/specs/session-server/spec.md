## MODIFIED Requirements

### Requirement: Attach creates a missing session
When a client asks to attach to a session the server does not host, the server SHALL create that session and attach the client to it. The new session's working directory SHALL be the working directory the client sent with its request, or the server's working directory when the client sent an empty path or one that is not a directory on the server's machine, and its screen area SHALL start as the size the client's hello carries, which is its reported size. A client attaching to a session that exists SHALL NOT create one.

#### Scenario: Second session
- **WHEN** a server hosts `default` and the user runs `gband attach -s work` in `/tmp`
- **THEN** the server hosts `default` and `work`
- **AND** `pwd` in the window of `work` prints `/tmp`
- **AND** the windows of `default` keep running

#### Scenario: Reattach by name
- **WHEN** the user runs `sleep 100` in session `work`, detaches, and runs `gband attach -s work` again
- **THEN** the client shows `sleep 100` still running

#### Scenario: Empty working directory
- **WHEN** a server started in `/home/u` hosts `default` and a client asks to attach to `work` with an empty working directory
- **THEN** `pwd` in the window of `work` prints `/home/u`

#### Scenario: Missing working directory
- **WHEN** a server started in `/home/u` hosts `default` and a client asks to attach to `work` with the working directory `/no/such/dir`
- **THEN** `pwd` in the window of `work` prints `/home/u`
