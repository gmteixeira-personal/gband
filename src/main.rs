use std::io::IsTerminal;
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use gband::logging::{self, Role};
use gband::{executable, paths};
use gband_client::{ClientConfig, Outcome};
use gband_server::ServerConfig;

const STALE_SERVER_NOTE: &str = "gband: the server runs a different gband build; stop it with gband kill-server and attach again";

#[derive(Parser)]
#[command(
    version,
    about = "A scrolling terminal multiplexer",
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Run the server that hosts the panes")]
    Server,
    #[command(about = "Attach a client to the server, starting one if needed")]
    Attach,
    #[command(about = "Stop the running server and its panes")]
    KillServer,
}

impl Command {
    fn role(&self) -> Role {
        match self {
            Command::Server => Role::Server,
            Command::Attach | Command::KillServer => Role::Client,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let role = cli.command.role();

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
    let result = match cli.command {
        Command::Server => server(),
        Command::Attach => attach(),
        Command::KillServer => kill_server(),
    };
    result.unwrap_or_else(|error| {
        tracing::error!("{error:#}");
        eprintln!("gband: {error:#}");
        ExitCode::FAILURE
    })
}

fn server() -> Result<ExitCode> {
    let runtime_dir = paths::runtime_dir();
    paths::prepare(&runtime_dir)?;
    let config = ServerConfig {
        runtime_dir,
        program: vec![gband_server::user_shell()],
        cwd: std::env::current_dir().context("cannot read the current directory")?,
        executable: executable::identity().context("cannot identify the gband executable")?,
    };
    tokio::runtime::Runtime::new()
        .context("cannot start the async runtime")?
        .block_on(gband_server::run(config))?;
    Ok(ExitCode::SUCCESS)
}

fn attach() -> Result<ExitCode> {
    if std::env::var_os("GBAND").is_some_and(|value| !value.is_empty()) {
        bail!("already inside a gband pane; attaching here would feed the session into itself");
    }
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        bail!("gband attach needs a terminal on standard input and standard output");
    }
    let config = ClientConfig {
        runtime_dir: paths::runtime_dir(),
        executable_path: std::env::current_exe().context("cannot locate the gband executable")?,
        identity: executable::identity().context("cannot identify the gband executable")?,
        replace_mismatched: cfg!(debug_assertions),
        log_dir: logging::log_directory()?,
    };
    let report = gband_client::run(config)?;
    let (line, status) = match report.outcome {
        Outcome::Detached => ("[detached]", ExitCode::SUCCESS),
        Outcome::Exited => ("[exited]", ExitCode::SUCCESS),
        Outcome::LostServer => ("[lost server]", ExitCode::FAILURE),
    };
    println!("{line}");
    if report.stale_server {
        eprintln!("{STALE_SERVER_NOTE}");
    }
    Ok(status)
}

fn kill_server() -> Result<ExitCode> {
    gband_server::kill(&paths::runtime_dir())?;
    Ok(ExitCode::SUCCESS)
}
