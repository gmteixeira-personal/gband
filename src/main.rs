use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use clap::error::ErrorKind;
use clap::{Args, CommandFactory, Parser, Subcommand};
use gband::completions::{self, Shell};
use gband::executable;
use gband::logging::{self, Role};
use gband::paths::{self, ServerName};
use gband_client::{ClientConfig, Configuration, Outcome, UnixTransport};
use gband_lua::{Config, ConfigError, LoadOptions, Locations};
use gband_protocol::SessionName;
use gband_server::{SUN_PATH_MAX, ServerConfig};
use tokio::sync::watch;

#[derive(Parser)]
#[command(
    version,
    about = "A scrolling terminal multiplexer",
    disable_help_subcommand = true
)]
struct Cli {
    #[command(flatten)]
    selection: Selection,
    #[arg(
        short = 's',
        long = "session",
        value_name = "NAME",
        global = true,
        help = "Select the session named NAME [default: default]"
    )]
    session: Option<SessionName>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Args)]
struct Selection {
    #[arg(
        short = 'S',
        long = "server",
        value_name = "NAME",
        global = true,
        help = "Address the server named NAME"
    )]
    server: Option<ServerName>,
    #[arg(
        short = 'p',
        long = "socket",
        value_name = "PATH",
        global = true,
        help = "Address the server listening on PATH"
    )]
    socket: Option<PathBuf>,
}

impl Selection {
    fn resolve(&self, runtime_dir: &Path) -> Result<PathBuf> {
        let socket = paths::resolve_socket(
            self.socket.as_deref(),
            self.server.as_ref(),
            std::env::var_os("GBAND"),
            runtime_dir,
        )?;
        if socket.as_os_str().len() > SUN_PATH_MAX {
            bail!(
                "socket path {} is longer than the {SUN_PATH_MAX} bytes a Unix socket allows",
                socket.display()
            );
        }
        Ok(socket)
    }

    fn kill_command(&self) -> String {
        match (&self.server, &self.socket) {
            (Some(name), _) => format!("gband -S {} kill-server", name.as_str()),
            (None, Some(socket)) => format!("gband -p {} kill-server", socket.display()),
            (None, None) => "gband kill-server".to_owned(),
        }
    }
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Run the server that hosts the panes")]
    Server,
    #[command(
        about = "Attach a client to a session, starting a server if needed; runs when no command is given"
    )]
    Attach,
    #[command(about = "List the sessions of the running server")]
    ListSessions,
    #[command(about = "End a session and its panes")]
    KillSession,
    #[command(about = "Stop the running server and every session")]
    KillServer,
    #[command(about = "Print the completion script for SHELL")]
    Completions {
        #[arg(value_name = "SHELL")]
        shell: Shell,
    },
    #[command(about = "Install the completion script where SHELL loads it")]
    InstallCompletions {
        #[arg(value_name = "SHELL")]
        shell: Shell,
    },
}

impl Command {
    fn role(&self) -> Option<Role> {
        match self {
            Command::Server => Some(Role::Server),
            Command::Attach
            | Command::ListSessions
            | Command::KillSession
            | Command::KillServer => Some(Role::Client),
            Command::Completions { .. } | Command::InstallCompletions { .. } => None,
        }
    }

    fn ignores_session(&self) -> Option<&'static str> {
        match self {
            Command::ListSessions => Some("list-sessions"),
            Command::KillServer => Some("kill-server"),
            Command::Server
            | Command::Attach
            | Command::KillSession
            | Command::Completions { .. }
            | Command::InstallCompletions { .. } => None,
        }
    }

    fn is_standalone(&self) -> Option<&'static str> {
        match self {
            Command::Completions { .. } => Some("completions"),
            Command::InstallCompletions { .. } => Some("install-completions"),
            Command::Server
            | Command::Attach
            | Command::ListSessions
            | Command::KillSession
            | Command::KillServer => None,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Command::Attach);
    if cli.selection.server.is_some() && cli.selection.socket.is_some() {
        Cli::command()
            .error(
                ErrorKind::ArgumentConflict,
                "the argument '--server <NAME>' cannot be used with '--socket <PATH>'",
            )
            .exit();
    }
    if cli.session.is_some()
        && let Some(subcommand) = command.ignores_session()
    {
        Cli::command()
            .error(
                ErrorKind::ArgumentConflict,
                format!(
                    "the option '-s' does not apply to '{subcommand}', which selects no session"
                ),
            )
            .exit();
    }
    if let Some(subcommand) = command.is_standalone() {
        let flag = [
            (cli.session.is_some(), "-s"),
            (cli.selection.server.is_some(), "-S"),
            (cli.selection.socket.is_some(), "-p"),
        ]
        .into_iter()
        .find_map(|(set, flag)| set.then_some(flag));
        if let Some(flag) = flag {
            Cli::command()
                .error(
                    ErrorKind::ArgumentConflict,
                    format!(
                        "the option '{flag}' does not apply to '{subcommand}', which addresses no \
                         server"
                    ),
                )
                .exit();
        }
    }
    let Some(role) = command.role() else {
        return standalone(command).unwrap_or_else(|error| {
            eprintln!("gband: {error:#}");
            ExitCode::FAILURE
        });
    };
    let session = cli.session.unwrap_or_default();

    let _guard = match logging::init(role) {
        Ok(guard) => guard,
        Err(error) => {
            eprintln!("gband: {error:#}");
            return ExitCode::FAILURE;
        }
    };

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        pid = std::process::id(),
        "{} started",
        role.as_str()
    );
    let runtime_dir = paths::runtime_dir();
    let socket = cli.selection.resolve(&runtime_dir);
    let _span = socket
        .as_ref()
        .ok()
        .map(|socket| tracing::error_span!("gband", socket = %socket.display()).entered());
    let result = socket.and_then(|socket| match command {
        Command::Server => server(socket, session, &runtime_dir),
        Command::Attach => attach(socket, session, &cli.selection),
        Command::ListSessions => list_sessions(socket, &cli.selection),
        Command::KillSession => kill_session(socket, session, &cli.selection),
        Command::KillServer => kill_server(&socket),
        Command::Completions { .. } | Command::InstallCompletions { .. } => {
            unreachable!("standalone subcommands return before logging starts")
        }
    });
    result.unwrap_or_else(|error| {
        tracing::error!("{error:#}");
        eprintln!("gband: {error:#}");
        ExitCode::FAILURE
    })
}

fn standalone(command: Command) -> Result<ExitCode> {
    match command {
        Command::Completions { shell } => print_completions(shell),
        Command::InstallCompletions { shell } => install_completions(shell),
        _ => unreachable!("only standalone subcommands have no role"),
    }
}

fn print_completions(shell: Shell) -> Result<ExitCode> {
    let mut script = Vec::new();
    completions::generate(shell, &mut Cli::command(), &mut script);
    std::io::stdout()
        .write_all(&script)
        .context("cannot write the completion script to standard output")?;
    Ok(ExitCode::SUCCESS)
}

fn install_completions(shell: Shell) -> Result<ExitCode> {
    let path = completions::install_path(shell, |name| std::env::var_os(name))?;
    completions::install(shell, &mut Cli::command(), &path)?;
    println!("{}", path.display());
    if shell == Shell::Zsh
        && let Some(directory) = path.parent()
    {
        println!(
            "add fpath=({} $fpath) to ~/.zshrc before compinit runs",
            directory.display()
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn server(socket: PathBuf, session: SessionName, runtime_dir: &Path) -> Result<ExitCode> {
    match socket.parent() {
        Some(parent) if parent == runtime_dir => paths::prepare(runtime_dir)?,
        Some(parent) if !parent.is_dir() => {
            bail!("socket directory {} does not exist", parent.display())
        }
        _ => {}
    }
    let loaded = configuration();
    let (options_tx, options) = watch::channel(loaded.config.options.layout);
    let _watcher = loaded.locations.map(|locations| {
        gband_lua::watch(
            locations,
            LoadOptions::default(),
            move |result| match result {
                Ok(config) => {
                    tracing::info!("configuration reloaded");
                    log_errors(&config);
                    options_tx.send_replace(config.options.layout);
                }
                Err(error) => tracing::warn!("configuration error: {error}"),
            },
        )
    });
    let config = ServerConfig {
        socket,
        session,
        program: vec![gband_server::user_shell()],
        cwd: std::env::current_dir().context("cannot read the current directory")?,
        executable: executable::identity().context("cannot identify the gband executable")?,
        options,
    };
    tokio::runtime::Runtime::new()
        .context("cannot start the async runtime")?
        .block_on(gband_server::run(config))?;
    Ok(ExitCode::SUCCESS)
}

struct Loaded {
    config: Config,
    error: Option<ConfigError>,
    locations: Option<Locations>,
}

fn log_errors(config: &Config) {
    for error in &config.errors {
        tracing::warn!("configuration error: {error}");
    }
}

fn configuration() -> Loaded {
    let Some(locations) = Locations::from_env() else {
        tracing::info!("no configuration directory, using the defaults");
        return Loaded {
            config: gband_lua::defaults(),
            error: None,
            locations: None,
        };
    };
    if let Err(error) = gband_lua::prepare(&locations.config) {
        tracing::warn!(
            "cannot prepare the configuration directory {}: {error}",
            locations.config.display()
        );
    }
    let options = LoadOptions::default();
    let (config, error) = match gband_lua::load(&locations, &options) {
        Ok(config) => (config, None),
        Err(error) => {
            tracing::warn!("configuration error: {error}");
            let config = gband_lua::load_defaults(&locations, &options).unwrap_or_else(|error| {
                tracing::warn!("configuration error: {error}");
                gband_lua::defaults()
            });
            (config, Some(error))
        }
    };
    log_errors(&config);
    Loaded {
        config,
        error,
        locations: Some(locations),
    }
}

fn client(
    socket: PathBuf,
    session: SessionName,
    selection: &Selection,
    attaching: bool,
) -> Result<(ClientConfig, UnixTransport)> {
    let transport = UnixTransport {
        socket,
        executable_path: std::env::current_exe().context("cannot locate the gband executable")?,
        session: session.clone(),
        start_server: attaching,
        log_dir: logging::log_directory()?,
    };
    let config = ClientConfig {
        session,
        identity: executable::identity().context("cannot identify the gband executable")?,
        replace_mismatched: attaching && cfg!(debug_assertions),
        kill_command: selection.kill_command(),
    };
    Ok((config, transport))
}

fn attach(socket: PathBuf, session: SessionName, selection: &Selection) -> Result<ExitCode> {
    if std::env::var_os("GBAND").is_some_and(|pane| Path::new(&pane) == socket) {
        bail!(
            "already inside a gband pane of the server on {}; attaching here would feed the \
             session into itself",
            socket.display()
        );
    }
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        bail!("gband attach needs a terminal on standard input and standard output");
    }
    let (config, transport) = client(socket, session, selection, true)?;
    let kill_command = config.kill_command.clone();
    let loaded = configuration();
    let (reloads_tx, reloads) = tokio::sync::mpsc::unbounded_channel();
    let _watcher = loaded.locations.map(|locations| {
        gband_lua::watch(locations, LoadOptions::default(), move |result| {
            let _ = reloads_tx.send(result);
        })
    });
    let configuration = Configuration {
        config: loaded.config,
        error: loaded.error,
        reloads,
    };
    let report = gband_client::run(config, transport, configuration)?;
    let (line, status) = match report.outcome {
        Outcome::Detached => ("[detached]", ExitCode::SUCCESS),
        Outcome::Exited => ("[exited]", ExitCode::SUCCESS),
        Outcome::LostServer => ("[lost server]", ExitCode::FAILURE),
    };
    println!("{line}");
    if report.stale_server {
        eprintln!(
            "gband: the server runs a different gband build; stop it with {kill_command} and \
             attach again"
        );
    }
    Ok(status)
}

fn list_sessions(socket: PathBuf, selection: &Selection) -> Result<ExitCode> {
    let (config, transport) = client(socket, SessionName::default(), selection, false)?;
    let mut listing = String::new();
    for session in gband_client::list_sessions(&config, &transport)? {
        listing.push_str(&format!(
            "{}\t{}\t{}\n",
            session.name, session.panes, session.clients
        ));
    }
    print!("{listing}");
    Ok(ExitCode::SUCCESS)
}

fn kill_session(socket: PathBuf, session: SessionName, selection: &Selection) -> Result<ExitCode> {
    let (config, transport) = client(socket, session, selection, false)?;
    gband_client::kill_session(&config, &transport)?;
    Ok(ExitCode::SUCCESS)
}

fn kill_server(socket: &Path) -> Result<ExitCode> {
    gband_server::kill(socket)?;
    Ok(ExitCode::SUCCESS)
}
