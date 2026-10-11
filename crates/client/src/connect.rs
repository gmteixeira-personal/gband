use anyhow::{Context, Result, anyhow, bail};
use gband_core::geometry::Size;
use gband_protocol::{
    ClientMessage, ExecutableId, Hello, HelloReply, IoError, MessageReader, MessageWriter,
    PROTOCOL_VERSION, ServerMessage,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::ClientConfig;
use crate::transport::{Link, Transport};

pub struct Connection {
    pub reader: MessageReader<Box<dyn AsyncRead + Send + Unpin>>,
    pub writer: MessageWriter<Box<dyn AsyncWrite + Send + Unpin>>,
    pub pid: u32,
    pub executable: ExecutableId,
    pub stale_server: bool,
    pub sent: u64,
}

impl Connection {
    pub async fn send(&mut self, message: &ClientMessage) -> Result<()> {
        self.sent += 1;
        write_frame(&mut self.writer, message).await
    }

    pub async fn receive(&mut self) -> Result<ServerMessage> {
        read_frame(&mut self.reader).await
    }
}

enum Handshake {
    Ready(Connection),
    Replace(String),
    Lost(anyhow::Error),
}

pub(crate) fn terminal_size() -> Size {
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    Size::new(cols, rows)
}

pub async fn connect(
    config: &ClientConfig,
    transport: &impl Transport,
    size: Size,
) -> Result<Connection> {
    let mut replaced = false;
    loop {
        let link = transport.open(size).await?;
        let may_replace = !replaced && config.replace_mismatched && transport.may_replace_server();
        match handshake(link, config, size, may_replace, transport.remote()).await? {
            Handshake::Ready(connection) => return Ok(connection),
            Handshake::Lost(error) => return Err(transport.diagnose().await.unwrap_or(error)),
            Handshake::Replace(reason) => {
                tracing::info!("replacing the server: {reason}");
                transport.replace_server().await?;
                replaced = true;
            }
        }
    }
}

async fn handshake(
    link: Link,
    config: &ClientConfig,
    size: Size,
    may_replace: bool,
    remote: bool,
) -> Result<Handshake> {
    let mut reader = MessageReader::new(link.reader);
    let mut writer = MessageWriter::new(link.writer);
    let hello = Hello {
        version: PROTOCOL_VERSION,
        cols: size.cols,
        rows: size.rows,
    };
    if let Err(error) = write_frame(&mut writer, &hello).await {
        return Ok(Handshake::Lost(error));
    }
    let reply = match receive(&mut reader).await? {
        Ok(reply) => reply,
        Err(lost) => return Ok(Handshake::Lost(lost)),
    };
    match reply {
        HelloReply::Accepted { .. } => {}
        HelloReply::Rejected { version } if may_replace => {
            return Ok(Handshake::Replace(format!(
                "it speaks protocol version {version}"
            )));
        }
        HelloReply::Rejected { version } => bail!(
            "the server speaks protocol version {version} and this client speaks version \
             {PROTOCOL_VERSION}; stop the server with {}",
            config.kill_command
        ),
    }
    let info = match receive(&mut reader).await? {
        Ok(info) => info,
        Err(lost) => return Ok(Handshake::Lost(lost)),
    };
    let ServerMessage::Info { pid, executable } = info else {
        bail!("the server did not send its info after the handshake");
    };
    let stale_server = !remote && executable != config.identity;
    if stale_server {
        tracing::warn!(pid, "the server runs a different gband build");
        if may_replace {
            let _ = write_frame(&mut writer, &ClientMessage::Detach).await;
            return Ok(Handshake::Replace(format!(
                "process {pid} runs a different build"
            )));
        }
    }
    Ok(Handshake::Ready(Connection {
        reader,
        writer,
        pid,
        executable,
        stale_server,
        sent: 0,
    }))
}

async fn receive<T: DeserializeOwned>(
    reader: &mut MessageReader<impl AsyncRead + Unpin>,
) -> Result<Result<T>> {
    match reader.recv().await {
        Ok(Some(message)) => Ok(Ok(message)),
        Ok(None) => Ok(Err(anyhow!("the server closed the connection"))),
        Err(IoError::Io(error)) => Ok(Err(
            anyhow::Error::from(error).context("cannot read from the server")
        )),
        Err(error) => Err(error).context("cannot read from the server"),
    }
}

async fn write_frame<T: Serialize>(
    writer: &mut MessageWriter<impl AsyncWrite + Unpin>,
    message: &T,
) -> Result<()> {
    writer
        .send(message)
        .await
        .context("cannot write to the server")
}

async fn read_frame<T: DeserializeOwned>(
    reader: &mut MessageReader<impl AsyncRead + Unpin>,
) -> Result<T> {
    match reader.recv().await.context("cannot read from the server")? {
        Some(message) => Ok(message),
        None => bail!("the server closed the connection"),
    }
}
