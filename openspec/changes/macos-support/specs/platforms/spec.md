## Purpose

Defines the operating systems gband builds and runs on, and what each one must provide so that the client, the server and the test suite behave the same on all of them.

## ADDED Requirements

### Requirement: Supported operating systems
gband SHALL build with `cargo install` and run on Linux and on macOS 13 or later, on x86-64 and arm64. Every subcommand, capability and option SHALL behave on macOS as its specification defines for Linux, except where a specification names a difference. `cargo test --workspace` SHALL pass on both operating systems. Building SHALL need no tool beyond the Rust toolchain and a C compiler: on macOS, the Xcode Command Line Tools provide one.

#### Scenario: Install on macOS
- **WHEN** a user on macOS with the Rust toolchain and the Xcode Command Line Tools runs the README's `cargo install` command
- **THEN** the command succeeds and `gband` is in `~/.cargo/bin/`

#### Scenario: Local session on macOS
- **WHEN** a user on macOS runs `gband` in Terminal
- **THEN** a server starts, the client attaches, and the first window runs the user's shell, as the session-server capability's "Window program" defines

#### Scenario: Mac client, Linux server
- **WHEN** a user on macOS runs `gband -r user@host` against a Linux host where gband of the same protocol version is installed
- **THEN** the client attaches to the remote session

### Requirement: Peer user check
Where a specification requires gband to check the user on the other end of a local Unix socket, the check SHALL read that user's id from the kernel for the connected socket: `SO_PEERCRED` on Linux and `getpeereid` on macOS. When the kernel cannot report it, the check SHALL fail as a mismatch does.

#### Scenario: Socket owned by another user on macOS
- **WHEN** gband runs on macOS, a process of another user listens on `/tmp/s.sock`, and the user runs `gband attach -p /tmp/s.sock`
- **THEN** the client refuses the server as the client-attach capability's "Socket owned by another user" defines

### Requirement: Process information
Where a specification requires gband to read a running process's arguments, command name or executable path, gband SHALL read them from the operating system without starting another program: from `/proc/<pid>/` on Linux, and from the kernel's process records on macOS. A record that cannot be read SHALL count as absent, and the reading capability's fallback SHALL apply.

#### Scenario: Debug server replaced on macOS
- **WHEN** gband runs on macOS, a server started from a debug build is running, the user runs `cargo build`, which replaces the executable, and runs `gband attach` from the new build
- **THEN** the client reads the server's executable path, finds it equal to its own, and replaces the server as the client-attach capability's "Server from a different build" defines

#### Scenario: Process that exited
- **WHEN** gband reads the arguments of a process id whose process has exited
- **THEN** the read counts as absent and the reading capability's fallback applies
