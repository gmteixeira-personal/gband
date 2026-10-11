## Why

gband reaches only a server on the local machine. Working on another machine means running `ssh host` and then `gband` inside it, so the remote client loads the remote configuration, sees keys through ssh's terminal and loses the local key bindings, plugins and clipboard. The transport capability already splits the client from the byte streams that carry the wire protocol, and its "Relay transparency" requirement names an SSH proxy as the shape a later transport takes.

## What Changes

- Add the global option `-r <destination>`, long form `--remote <destination>`. `gband -r user@host` attaches the local client to the session named `default` on the default server of `host`, starting that server when none runs there. `-s`, `-S` and `-p` keep their meaning and select the session and server on the remote host.
- Add the option `--remote-gband <command>`, valid only with `-r`, naming the gband executable on the remote host. Without it the remote login shell finds `gband` on its `PATH`.
- `attach` with `-r` keeps the client local: local configuration, local terminal, local key bindings. It reaches the remote server through a new SSH transport that runs `ssh -T -- <destination> <remote gband> relay ...` and uses ssh's standard input and output as the two byte streams. ssh asks for passwords and host keys on the terminal before the client takes it.
- Add the hidden subcommand `gband relay`. It runs on the remote host, connects to the server it selects, starting one at the client's reported size when none runs, then copies its standard input to the socket and the socket to its standard output.
- `list-sessions`, `kill-session`, `kill-server`, `reload`, `errors`, `eval` and `cmd` with `-r` run the same subcommand with the same options through `ssh` on the remote host, and exit with its status.
- A remote attach sends an empty working directory, and the server gives a session it creates for an empty or missing directory its own working directory. A remote attach never replaces the server and prints no different-build note, because an executable's identity is a device and inode on another machine.
- The version-mismatch line names the kill command with `-r <destination>`, so `gband -r user@host kill-server` stops the remote server.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `transport`: adds "SSH transport" and "Relay subcommand"; "Relay transparency" names the SSH transport as the relay it describes.
- `command-line`: adds "Remote host option"; "Subcommands run the session" adds the hidden `relay`; "Help and version" lists `-r`.
- `client-attach`: adds "Attach to a remote host"; "Version handshake" carries `-r` in the kill command; "Server from a different build" skips the comparison over the SSH transport.
- `session-server`: "Attach creates a missing session" falls back to the server's working directory for an empty or missing one.
- `logging`: "Log to a file per role" logs `relay` to the `client` series; "Events name their server" names the destination for a subcommand run with `-r`.

## Impact

- CLI: `src/main.rs` gains `-r`, `--remote-gband`, the hidden `relay` subcommand and the remote dispatch; a new `src/remote.rs` builds and quotes the ssh command line and runs a remote subcommand.
- Client: a new `crates/client/src/ssh.rs` holds `SshTransport`; a new `crates/client/src/relay.rs` copies standard input and output to the socket; `crates/client/src/connect.rs` and `crates/client/src/lib.rs` skip the identity comparison and send an empty working directory for a remote transport; `crates/client/src/transport.rs` gives the `Transport` trait the remote flag.
- Server: `crates/server/src/registry.rs` falls back to the server's working directory.
- Tests: a new `tests/remote.rs` drives `gband -r` through a fake `ssh` on `PATH` that runs the remote command locally; `crates/server/tests/sessions.rs` covers the working directory fallback.
- Docs: `README.md` Usage gains remote attach.
- Dependencies: none new. The local machine needs an OpenSSH client, and the remote machine needs a gband speaking the same protocol version.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- README.md
- crates/client/src/connect.rs
- crates/client/src/lib.rs
- crates/client/src/relay.rs
- crates/client/src/ssh.rs
- crates/client/src/transport.rs
- crates/client/tests/transport.rs
- crates/server/src/lib.rs
- crates/server/src/registry.rs
- crates/server/tests/sessions.rs
- crates/test-support/src/lib.rs
- openspec/changes/remote-host/
- src/main.rs
- src/remote.rs
- tests/remote.rs
- tests/subcommands.rs
