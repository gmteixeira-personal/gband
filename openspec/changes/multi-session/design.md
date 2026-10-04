## Context

This change builds on layout-core, which it depends on. After layout-core, a server runs one session task that owns the `Layout`, the screen area and the panes. That task receives client sizes, session actions, pane exits and SIGTERM over an `mpsc` channel, publishes an `Arc<State>` through a `watch`, and sets an `ended` flag when the last pane leaves. Each client connection is a task that follows the published state. The server ends when `ended` is set: it sends `Exited`, removes the socket and returns.

The hello and its answer have a frozen encoding, and `Info` is the server's first message after accepting a hello. The debug-build replace path in the client reads `Info` before it changes the terminal. `postcard` is not self-describing, so any message change bumps `PROTOCOL_VERSION`. layout-core makes it 2. This change makes it 3.

Server and socket selection, `-S` and `-p`, are being designed in a separate change. This design touches neither. It uses whatever runtime directory and socket path the client and server already resolve.

Several MODIFIED requirements in `specs/session-server/spec.md` are added by layout-core: "Session ends with its last pane" and "Screen area follows the latest client". `openspec validate` reports that archive would refuse them today. The dependency on layout-core resolves this: layout-core is archived first, and its requirements exist by the time this change archives.

## Goals / Non-Goals

**Goals:**
- Several sessions in one server, each one exactly the session layout-core defines, so that layout-core's session code is reused unchanged per session.
- A single order for creating, attaching to and ending sessions, so that two clients racing on one name always meet in one session.
- Session listing and killing over the existing socket and handshake.

**Non-Goals:**
- Sharing a pane between sessions, or moving one across.
- Server-wide pane identifiers. A pane is identified by its session name and its pane identifier together.
- Per-session environment. Every session's panes get the server's environment, as today.

## Decisions

### A registry task owns the sessions

```
client task --request--> registry task --create / look up--> session task (one per session)
                                 ^                                 |
                                 +------- SessionEnded(name) ------+
SIGTERM ---> registry task ---> Terminate to every session task
```

One registry task owns a `BTreeMap<SessionName, SessionHandle>`. A handle is what layout-core's server holds for its one session today: the command sender, the state `watch` and the `ended` watch. The registry receives `Attach { name, cwd, size }`, `List`, `Kill { name }`, `SessionEnded(name)` and `Terminate` over an `mpsc` channel.

- **Attach** returns the existing handle, or spawns a new session task with the request's working directory and size, and returns that.
- **List** answers from each handle's latest published state: the pane count of the layout and the number of client tasks following it. The registry counts attached clients through a counter in the handle that each client task increments and decrements.
- **Kill** forwards a close-every-pane command to the session task, which reuses layout-core's close path and its SIGKILL timer. It returns the session's `ended` watch, which the client task awaits before sending `Killed`.
- **SessionEnded** removes the entry. When the map becomes empty, the registry sets the server-wide `ended` flag, and the existing farewell and socket-removal path runs.
- **Terminate** sends layout-core's SIGTERM command to every session task.

Serialising through one task makes create-or-attach atomic. Two clients attaching to a missing `work` at the same moment get one session. Alternative: a `Mutex<BTreeMap>` locked by each client task. Rejected because creating a session spawns a program, and holding a lock across a spawn is what layout-core's session task already avoids.

**A session leaves the map as soon as its last pane leaves**, before its farewell messages go out. An attach that arrives after that point creates a fresh session of the same name, rather than attaching to one that is ending.

### Session on start

`gband server` creates its first session before it binds the socket. The registry is then never empty while the server accepts connections. The "Program exits with no client" scenario keeps working: the first session's only program exits, the session ends, the map becomes empty, and the server exits.

### Wire protocol version 3

```rust
enum ClientMessage {
    Attach { session: SessionName, cwd: PathBuf },
    ListSessions,
    KillSession { session: SessionName },
    Key { pane, key }, Paste { pane, text }, Resize { cols, rows }, Action(SessionAction), Detach,
}
enum ServerMessage {
    Info { .. }, Layout { .. }, Snapshot { .. }, Update { .. }, Focus(PaneId), Exited,
    Sessions(Vec<SessionSummary>), Killed, NoSuchSession,
}
struct SessionSummary { name: SessionName, panes: u32, clients: u32 }
```

The requests extend `ClientMessage` rather than forming a separate type, so that one decoder serves the connection. The client task is a small state machine: after `Info` it accepts only a request, and after an attach only the attached messages.

Alternative: put the session name in `Hello`. Rejected, because the hello's encoding is frozen so that every version can detect every other. Alternative: send the request before `Info`. Rejected, because the debug replace path must see `Info` before it commits to anything, and listing or killing sessions on a server it is about to replace would be wasted.

The hello's terminal size stays where it is. It seeds a new session's screen area and is the size an attaching client reports. List and kill clients send whatever size they have, or 80×24 when standard output is not a terminal, and the server ignores it.

### `SessionName` lives in gband-protocol

`SessionName` is a newtype over `String` with `FromStr` enforcing the 1 to 64 byte, `[A-Za-z0-9._-]` rule. Its `Deserialize` goes through the same check, so the server rejects an invalid name at decode time and closes the connection. The root package uses the same `FromStr` as clap's `value_parser` for `-s`. One rule then guards the command line and the wire. The character set keeps names safe to print in a tab-separated list and to embed in log fields.

### Command line

`-s`/`--session` is a clap `global = true` option of type `Option<SessionName>`. `main` resolves `None` to `default`. When the subcommand is `kill-server` or `list-sessions` and the option is present, `main` raises a clap error of kind `ArgumentConflict` before logging starts. clap prints it with the usage and exits with status 2, which matches "Invalid invocations".

`gband attach` spawns `<current_exe> server -s <name>`. The server it starts then holds the requested session from its first moment, and the client's attach request finds it.

### List and kill clients

The client crate's connect path gains a flag that turns off spawning. `list-sessions` and `kill-session` connect with spawning off: `NotFound` or `ConnectionRefused` becomes "no server is running on <socket>". They complete the handshake, read `Info`, send their request and read one answer. A rejected hello gives the release-build version error for both build types: these commands never replace a server. A different executable identity is ignored, because neither command keeps a session attached that a stale-build note would be about.

`kill-session` waits up to 5 s for `Killed`. The server's own 2 s SIGKILL normally bounds the wait. The client timeout covers a program the kernel cannot kill promptly, such as one in uninterruptible sleep.

### Logging

Each session task runs inside a `tracing` span carrying the session name. Pane, layout and exit events then name their session without changing each call site. The registry logs each session's creation and removal.

## Risks / Trade-offs

- [A debug client replacing a server from a different build ends every session of that server, not only the one it asked for] → Unchanged from today in kind, since the server is replaced as a whole. The separate server-selection change gives each worktree its own server, which keeps a debug build away from the user's everyday server.
- [An attach racing a server whose last session just ended reaches a server that is shutting down] → The client sees the connection close and prints `[lost server]`. Running `gband attach` again starts a new server. The window is the farewell period of at most 1 s.
- [Pane identifiers repeat across sessions, so `GBAND_PANE` alone does not identify a pane in a server] → Panes also carry `GBAND_SESSION`. A server-wide identifier can be added later without changing the per-session layout.
- [`-s` on `kill-server` or `list-sessions` is an error rather than ignored] → An ignored option would read as `gband -s work kill-server` stopping only `work`. The error names the right subcommand.

## Migration Plan

The protocol version becomes 3. A debug client replaces a version 2 server through the existing path, which ends that server's one session. A release client reports the version mismatch, and the user runs `gband kill-server`, then `gband attach`. No file on disk changes format.

## Open Questions

- Whether `gband attach` without `-s` should later attach to the most recently used session, as tmux does, rather than to `default`. The explicit default is predictable for scripts and tests, and a later change can add the fallback without changing this one's messages.
- Whether commands run inside a pane should later default to that pane's session through `GBAND_SESSION`. Today `attach` there is refused as nesting, and `kill-session` without `-s` would target `default`.
- Whether a session created on attach should take some of the attaching client's environment, as tmux's `update-environment` does.
