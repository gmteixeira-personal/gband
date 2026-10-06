use std::io::{Write, stdout};

use anyhow::{Context, Result};
use gband_lua::{Config, ConfigError};
use gband_protocol::test::{FromProcess, ToProcess};
use gband_protocol::{MessageReader, MessageWriter, ServerMessage};
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};

use crate::{Connection, Controls, Display, Outcome, perform};

pub type Loader = Box<dyn FnMut() -> Result<Config, ConfigError>>;

pub struct TestChannel {
    pub stream: std::os::unix::net::UnixStream,
    pub reload: Loader,
}

pub(crate) struct Channel {
    reader: MessageReader<OwnedReadHalf>,
    writer: MessageWriter<OwnedWriteHalf>,
    reload: Loader,
    read: u64,
    holding: bool,
    input: Option<(u32, u64)>,
    pub(crate) draw: Option<u32>,
    reported: u64,
}

pub(crate) enum Served {
    Continue,
    Finished(Outcome),
}

impl Channel {
    pub(crate) fn new(channel: TestChannel) -> Result<Self> {
        channel
            .stream
            .set_nonblocking(true)
            .context("cannot use the test channel")?;
        let (reader, writer) = UnixStream::from_std(channel.stream)
            .context("cannot use the test channel")?
            .into_split();
        Ok(Self {
            reader: MessageReader::new(reader),
            writer: MessageWriter::new(writer),
            reload: channel.reload,
            read: 0,
            holding: false,
            input: None,
            draw: None,
            reported: 0,
        })
    }

    pub(crate) async fn fill(&mut self) -> bool {
        matches!(self.reader.fill().await, Ok(true))
    }

    async fn answer(&mut self, message: &FromProcess) -> Result<()> {
        self.writer
            .send(message)
            .await
            .context("cannot answer on the test channel")
    }

    async fn settled(&mut self, round: u32, sent: u64) -> Result<()> {
        let since = sent - self.reported;
        self.reported = sent;
        self.answer(&FromProcess::Settled { round, sent: since })
            .await
    }

    pub(crate) async fn read(&mut self, bytes: u64, holding: bool) -> Result<()> {
        self.read += bytes;
        self.holding = holding;
        self.check_input().await
    }

    async fn check_input(&mut self) -> Result<()> {
        match self.input {
            Some((round, input)) if self.read >= input && !self.holding => {
                self.input = None;
                self.answer(&FromProcess::Settled { round, sent: 0 }).await
            }
            _ => Ok(()),
        }
    }

    pub(crate) async fn drawn(&mut self, sent: u64) -> Result<()> {
        let Some(round) = self.draw.take() else {
            return Ok(());
        };
        let mut out = stdout();
        out.write_all(format!("\x1b]7777;settle;{round}\x07").as_bytes())
            .and_then(|()| out.flush())
            .context("cannot write the settle marker")?;
        self.settled(round, sent).await
    }

    pub(crate) async fn serve(
        &mut self,
        controls: &mut Controls,
        display: &mut Display,
        connection: &mut Connection,
    ) -> Result<Served> {
        while let Some(request) = self.reader.try_recv::<ToProcess>()? {
            match request {
                ToProcess::Eval { id, source, args } => {
                    let (result, steps) = controls.eval(display, &source, &args);
                    if let Some(outcome) = perform(connection, steps).await? {
                        return Ok(Served::Finished(outcome));
                    }
                    self.answer(&FromProcess::Answer { id, result }).await?;
                }
                ToProcess::Reload { id } => {
                    let loaded = (self.reload)();
                    let error = loaded.as_ref().err().map(ToString::to_string);
                    let steps = controls.reload(display, loaded);
                    if let Some(outcome) = perform(connection, steps).await? {
                        return Ok(Served::Finished(outcome));
                    }
                    self.answer(&FromProcess::Reloaded { id, error }).await?;
                }
                ToProcess::SetTime { time } => gband_lua::freeze_time(time),
                ToProcess::Settle {
                    round,
                    input: Some(input),
                } => {
                    self.input = Some((round, input));
                    self.check_input().await?;
                }
                ToProcess::Settle { round, input: None } => {
                    if !connection.reader.fill_ready().await.unwrap_or(false) {
                        return Ok(Served::Finished(Outcome::LostServer));
                    }
                    let mut messages = Vec::new();
                    while let Some(message) = connection.reader.try_recv::<ServerMessage>()? {
                        messages.push(message);
                    }
                    if !messages.is_empty() {
                        let received = controls.receive(display, messages);
                        if let Some(outcome) = received.outcome {
                            return Ok(Served::Finished(outcome));
                        }
                        if let Some(outcome) = perform(connection, received.steps).await? {
                            return Ok(Served::Finished(outcome));
                        }
                    }
                    self.draw = Some(round);
                }
                ToProcess::Start { .. } => {}
            }
        }
        Ok(Served::Continue)
    }
}
