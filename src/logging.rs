use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use directories::ProjectDirs;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{Builder, Rotation};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const FILTER_VARIABLE: &str = "GBAND_LOG";
const DEFAULT_FILTER: &str = "info";
const MAX_LOG_FILES: usize = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Server,
    Client,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Server => "server",
            Role::Client => "client",
        }
    }
}

pub fn init(role: Role) -> Result<WorkerGuard> {
    let directory = log_directory()?;
    std::fs::create_dir_all(&directory)
        .with_context(|| format!("cannot create log directory {}", directory.display()))?;

    let appender = Builder::new()
        .rotation(Rotation::DAILY)
        .filename_prefix(role.as_str())
        .filename_suffix("log")
        .max_log_files(MAX_LOG_FILES)
        .build(&directory)
        .with_context(|| format!("cannot open log file in {}", directory.display()))?;
    let (writer, guard) = tracing_appender::non_blocking(appender);

    let requested = std::env::var(FILTER_VARIABLE).ok();
    let (filter, rejected) = match requested.as_deref().map(EnvFilter::try_new) {
        None => (EnvFilter::new(DEFAULT_FILTER), None),
        Some(Ok(filter)) => (filter, None),
        Some(Err(error)) => (EnvFilter::new(DEFAULT_FILTER), Some(error)),
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(writer),
        )
        .try_init()
        .context("cannot install the log subscriber")?;

    if let (Some(value), Some(error)) = (requested, rejected) {
        tracing::warn!(
            "{FILTER_VARIABLE} value {value:?} is invalid, using {DEFAULT_FILTER}: {error}"
        );
    }
    tracing::debug!(directory = %directory.display(), "logging started");

    install_panic_hook();
    Ok(guard)
}

pub fn log_directory() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "gband")
        .ok_or_else(|| anyhow!("cannot determine the home directory"))?;
    let base = dirs.state_dir().unwrap_or_else(|| dirs.data_local_dir());
    Ok(base.join("log"))
}

fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let payload = info.payload();
        let message = payload
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
            .unwrap_or("Box<dyn Any>");
        match info.location() {
            Some(location) => tracing::error!(
                "panic at {}:{}:{}: {message}",
                location.file(),
                location.line(),
                location.column()
            ),
            None => tracing::error!("panic: {message}"),
        }
        previous(info);
    }));
}
