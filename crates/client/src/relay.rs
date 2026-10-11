use anyhow::{Context, Result};
use gband_core::geometry::Size;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::Instrument;

use crate::runtime;
use crate::transport::{Link, Transport, UnixTransport};

const BUFFER_LEN: usize = 64 * 1024;

pub fn run_relay(transport: UnixTransport, size: Size) -> Result<()> {
    runtime()?.block_on(relay(
        &transport,
        size,
        tokio::io::stdin(),
        tokio::io::stdout(),
    ))
}

pub async fn relay(
    transport: &UnixTransport,
    size: Size,
    mut input: impl AsyncRead + Send + Unpin + 'static,
    mut output: impl AsyncWrite + Send + Unpin + 'static,
) -> Result<()> {
    let Link {
        mut reader,
        mut writer,
    } = transport.open(size).await?;
    tracing::info!("relaying");
    let upstream = tokio::spawn(
        async move {
            let copied = tokio::io::copy(&mut input, &mut writer).await;
            let _ = writer.shutdown().await;
            copied.context("cannot copy standard input to the server")
        }
        .in_current_span(),
    );
    let downstream = tokio::spawn(
        async move {
            let mut buffer = vec![0; BUFFER_LEN];
            let copied = async {
                loop {
                    let n = reader.read(&mut buffer).await?;
                    if n == 0 {
                        break;
                    }
                    output.write_all(&buffer[..n]).await?;
                    output.flush().await?;
                }
                Ok::<_, std::io::Error>(())
            }
            .await;
            let _ = output.shutdown().await;
            copied.context("cannot copy the server to standard output")
        }
        .in_current_span(),
    );
    let (upstream, downstream) = tokio::join!(upstream, downstream);
    if let Err(error) = upstream? {
        tracing::info!("{error:#}");
    }
    if let Err(error) = downstream? {
        tracing::info!("{error:#}");
    }
    tracing::info!("relay finished");
    Ok(())
}
