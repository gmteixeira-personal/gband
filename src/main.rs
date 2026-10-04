use std::process::ExitCode;

use clap::{Parser, Subcommand};
use gband::logging::{self, Role};

#[derive(Parser)]
#[command(version, about = "A scrolling terminal multiplexer")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Run the server that hosts the panes")]
    Server,
    #[command(about = "Attach a client to the server")]
    Attach,
}

impl Command {
    fn role(&self) -> Role {
        match self {
            Command::Server => Role::Server,
            Command::Attach => Role::Client,
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
    ExitCode::SUCCESS
}
