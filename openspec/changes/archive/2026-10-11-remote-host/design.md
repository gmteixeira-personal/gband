## Context

`crates/client/src/transport.rs` defines `Transport` with `open`, `may_replace_server` and `replace_server`, and a `Link` of one boxed reader and one boxed writer. `UnixTransport` is the only production transport. `connect.rs` runs the handshake over any `Link`, and `crates/test-support` already has a `RelayTransport` that copies bytes through in-memory pipes, so the client is proven to work over a byte relay.

`src/main.rs` parses the command line with clap. `Selection::resolve` turns `-S`, `-p` and `GBAND` into a socket path before dispatch, and `client()` builds a `ClientConfig` and a `UnixTransport` for every subcommand that talks to a server. `kill-server` is different: `gband_server::kill` reads the pid record next to the socket and signals that process, so it only works on the server's machine.

`ClientMessage::Attach` carries `cwd: PathBuf`, filled from the client's own working directory in `gband_client::run`. The server's `Registry` creates a missing session with that path, and has no copy of the server's own working directory.

## Goals / Non-Goals

**Goals:**
- One remote mechanism: the system `ssh` binary, so the user's ssh configuration, agent, keys, jump hosts and multiplexing all apply unchanged.
- No wire protocol change and no protocol version bump.
- No new crate dependency.

**Non-Goals:**
- No built-in SSH implementation and no password handling in gband. ssh owns authentication on the terminal.
- No reconnect after a dropped connection. The client prints `[lost server]`, and running `gband -r` again reattaches.
- No copying of the local configuration, plugins or binary to the remote host. The remote host runs its own installed gband.
- No nesting check across machines. `gband -r localhost` from a window of the same server shows that server inside itself, as `ssh localhost gband` does today.
- No remote `server` subcommand. A foreground server over ssh dies with the connection.

## Decisions

### A relay subcommand on the remote end, over ssh's standard streams

`SshTransport::open` spawns `ssh -T -- <destination> <remote command>` with tokio's `Command`, standard input and output piped, and returns them as the `Link`. The remote command runs `gband relay`, which does what `UnixTransport::open` does on the remote machine, connect with the uid check or start a server, then copies bytes.

Alternatives considered:
- `ssh -L local.sock:remote.sock` forwarding. It needs a server already running on the remote host and a known remote socket path, so it cannot start a server or resolve `-S` there.
- `ssh host socat - UNIX-CONNECT:...`. It adds a dependency on the remote host and also cannot start a server.
- An SSH library such as `russh`. It would reimplement ssh configuration, agents and host key checking, which the user already has in OpenSSH.

`gband relay` reuses `UnixTransport` with `start_server: true`, so the uid check, the 5-second start timeout and the server start arguments stay in one place. The copy lives in a new `crates/client/src/relay.rs`: one task copies `tokio::io::stdin` to the socket's writer and shuts the writer down on end of input, the other copies the socket's reader to `tokio::io::stdout`, flushing after each read. The relay returns when both tasks end.

### The remote flag lives on the transport

`Transport` gains `fn remote(&self) -> bool`, `false` for `UnixTransport` and `RelayTransport`, `true` for `SshTransport`. `connect::handshake` skips the identity comparison when it is `true`, so `stale_server` stays `false`, and `may_replace_server` returns `false`. `gband_client::run` sends `PathBuf::new()` as the attach `cwd` when it is `true`. Putting the flag on `ClientConfig` was the alternative; the transport is what decides whether the server shares the client's filesystem, so the transport carries it.

### ssh diagnostics through a transport hook

ssh's standard error is piped. A task reads it line by line, writes each line to the log with `tracing::warn!`, and stores the latest line in an `Arc<Mutex<Option<String>>>` until the handshake completes. `Transport` gains `async fn diagnose(&self) -> Option<anyhow::Error>`, defaulting to `None`. When `connect` fails before the server's info arrives, it calls `diagnose`; `SshTransport` waits for the ssh child, then builds the 127 or `cannot reach` line the transport spec gives. `Transport` also gains `async fn finish(&self)`, a no-op by default, which `SshTransport` uses to close ssh's standard input, wait up to 2 seconds and kill the child. `gband_client::run` calls it after the terminal is restored.

ssh inherits the controlling terminal through `/dev/tty`, and the client changes the terminal only after the handshake, so password and host key prompts work. Standard error is piped, so nothing ssh prints later lands in the alternate screen.

### Non-attach subcommands run whole on the remote host

`list-sessions`, `kill-session`, `kill-server`, `reload`, `errors`, `eval` and `cmd` with `-r` run `ssh -T -- <destination> <remote gband> <arguments>` with `std::process::Command`, standard output and error inherited, and standard input inherited only for `eval -`. Routing them through the relay was the alternative: it works for the six that only send requests, but `kill-server` signals a pid on the server's machine, and one rule for all seven is easier to state and test.

The arguments come from `std::env::args_os` with `-r`, `--remote-gband` and their values removed. A walk over the tokens finds them, reading which options take a value from `Cli::command()`: a value attached as `-rhost` or `--remote=host` removes one token and a separate value removes two, and nothing after `--` or inside the trailing arguments of `eval` is removed. The result is parsed again with clap, and the command refuses to run unless that parse succeeds without `-r` or `--remote-gband`. clap's `ArgMatches::indices_of` was the first choice and does not work: each subcommand counts its indices from 0, and a global option given after the subcommand overwrites the index recorded before it. Rebuilding the arguments from the parsed structs was the alternative; it would need a serializer per subcommand that tracks every future option.

### Quoting

A new `src/remote.rs` holds `shell_quote`, which wraps an argument in single quotes and writes each `'` inside as `'\''`. Every argument is quoted, including ones that need no quoting, so the scenarios can name exact command lines. The `--remote-gband` value is written as given, so `~/.cargo/bin/gband` expands on the remote host. `SshTransport` lives in `crates/client/src/ssh.rs` and takes the finished remote command as a `Vec<OsString>` from `src/main.rs`, so the quoting rule exists once.

### CLI shape

`Cli` gains `remote: Option<Destination>` and `remote_gband: Option<OsString>`, both `global = true`. `Destination` is a newtype whose parser refuses an empty value. `Command` gains `#[command(hide = true)] Relay { size: Option<Size> }`. `main` refuses `-r` for `Server`, `Relay`, `Test`, `Completions` and `InstallCompletions` next to the existing `ignores_session` and `is_standalone` checks, and refuses `--remote-gband` without `-r` with `ErrorKind::MissingRequiredArgument`. With `-r`, `main` skips `Selection::resolve`, opens the `gband` log span with `remote = <destination>`, and dispatches `Attach` to `attach_remote` and every other subcommand to `remote::run`. `Selection::kill_command` gains the destination, giving `gband -r u@h -S feature kill-server`.

### Server working directory fallback

`Registry` gains `cwd: PathBuf`, set from `ServerConfig::cwd`. `Request::Attach` creates a missing session with the client's `cwd` when it is a directory, and with the registry's otherwise. An empty path is not a directory, so one check covers both cases. The server started by `gband relay` inherits the relay's working directory, which sshd sets to the login directory.

### Tests through a fake ssh

`tests/remote.rs` writes an executable `ssh` script into a scratch directory and puts that directory first on `PATH`. The script drops `-T`, `--` and the destination, records its arguments and destination to a file, and runs `exec sh -c "$*"`, which joins and reparses the words as sshd's login shell does. Tests pass `--remote-gband` with the built binary's path, so the "remote" gband is the one under test, in the same isolated runtime and state directories. A second script variant prints a fixed line to standard error and exits with 255 or 127, for the failure scenarios. No test needs a real sshd.

## Risks / Trade-offs

- [The remote host runs another gband version] → The handshake already refuses another protocol version, and the line names `gband -r <destination> kill-server`. Builds of one protocol version interoperate.
- [ssh's `ControlMaster` or a `RemoteCommand` in the user's ssh configuration changes how the command runs] → `-T` and an explicit command override `RemoteCommand`; a shared master connection still carries the same standard streams.
- [A login shell that prints to standard output, such as a `.bashrc` with `echo`, corrupts the byte stream before the handshake] → ssh runs a non-interactive command, which most shells start without printing. When it happens, the handshake fails with a frame error and the client exits with status 1 before changing the terminal. The README notes the cause.
- [The relay keeps running after a silent network drop] → sshd closes the relay's standard input when its keepalive fails, the relay shuts the socket down, and the server treats the client as gone. Users who want faster detection set `ServerAliveInterval` in their ssh configuration.
- [The executable identity comparison is skipped for remote servers] → A stale remote build is noticed only through a protocol version mismatch. Same-version builds behave the same on the wire.
