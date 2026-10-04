use std::time::Duration;

use anyhow::{Result, bail};
use gband_protocol::{ClientMessage, ServerMessage, SessionSummary};

use crate::{ClientConfig, connect, runtime};

const KILL_TIMEOUT: Duration = Duration::from_secs(5);

pub fn list_sessions(config: &ClientConfig) -> Result<Vec<SessionSummary>> {
    runtime()?.block_on(async {
        let mut connection = connect(config).await?;
        connection.send(&ClientMessage::ListSessions).await?;
        match connection.receive().await? {
            ServerMessage::Sessions(sessions) => Ok(sessions),
            _ => bail!("the server sent an unexpected answer to a list request"),
        }
    })
}

pub fn kill_session(config: &ClientConfig) -> Result<()> {
    let session = &config.session;
    runtime()?.block_on(async {
        let mut connection = connect(config).await?;
        connection
            .send(&ClientMessage::KillSession {
                session: session.clone(),
            })
            .await?;
        let Ok(answer) = tokio::time::timeout(KILL_TIMEOUT, connection.receive()).await else {
            bail!(
                "session {session} did not end within {} seconds",
                KILL_TIMEOUT.as_secs()
            );
        };
        match answer? {
            ServerMessage::Killed => Ok(()),
            ServerMessage::NoSuchSession => {
                bail!("no session named {session} on {}", config.socket.display())
            }
            _ => bail!("the server sent an unexpected answer to a kill request"),
        }
    })
}
