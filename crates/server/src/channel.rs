use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{Context as _, Result};
use gband_protocol::test::{FromProcess, ToProcess};
use gband_protocol::{MessageReader, MessageWriter, ServerMessage};
use tokio::net::UnixStream;
use tokio::sync::{mpsc, oneshot};

use crate::hub::Hub;
use crate::scripting::Taps;

pub struct TestChannel {
    pub stream: std::os::unix::net::UnixStream,
}

pub(crate) enum Barrier {
    Read(oneshot::Sender<()>),
    Flush(oneshot::Sender<()>),
}

#[derive(Default)]
pub struct Settling {
    connections: Mutex<HashMap<u64, mpsc::UnboundedSender<Barrier>>>,
    forward: Mutex<Option<mpsc::UnboundedSender<oneshot::Sender<()>>>>,
    sent: AtomicU64,
}

impl Settling {
    pub(crate) fn register(&self, client: u64, barriers: mpsc::UnboundedSender<Barrier>) {
        self.connections
            .lock()
            .expect("settling lock")
            .insert(client, barriers);
    }

    pub(crate) fn unregister(&self, client: u64) {
        self.connections
            .lock()
            .expect("settling lock")
            .remove(&client);
    }

    pub(crate) fn set_forward(&self, forward: mpsc::UnboundedSender<oneshot::Sender<()>>) {
        *self.forward.lock().expect("settling lock") = Some(forward);
    }

    pub(crate) fn sent(&self, message: &ServerMessage) {
        if !matches!(
            message,
            ServerMessage::Update { .. } | ServerMessage::Snapshot { .. }
        ) {
            self.count();
        }
    }

    pub(crate) fn count(&self) {
        self.sent.fetch_add(1, Ordering::Relaxed);
    }

    async fn connections(&self, barrier: fn(oneshot::Sender<()>) -> Barrier) {
        let reached: Vec<oneshot::Receiver<()>> = self
            .connections
            .lock()
            .expect("settling lock")
            .values()
            .filter_map(|connection| {
                let (sender, reached) = oneshot::channel();
                connection.send(barrier(sender)).ok().map(|()| reached)
            })
            .collect();
        for reached in reached {
            let _ = reached.await;
        }
    }

    async fn handlers(&self) {
        let forward = self.forward.lock().expect("settling lock").clone();
        let Some(forward) = forward else {
            return;
        };
        let (sender, reached) = oneshot::channel();
        if forward.send(sender).is_ok() {
            let _ = reached.await;
        }
    }

    async fn settle(&self) -> u64 {
        self.connections(Barrier::Read).await;
        self.handlers().await;
        self.connections(Barrier::Flush).await;
        self.sent.swap(0, Ordering::Relaxed)
    }
}

pub(crate) async fn serve(channel: TestChannel, hub: Arc<Hub>, taps: Arc<Taps>) {
    match run(channel, &hub, &taps).await {
        Ok(()) => tracing::info!("the test channel closed"),
        Err(error) => tracing::warn!("the test channel failed: {error:#}"),
    }
}

async fn run(channel: TestChannel, hub: &Hub, taps: &Taps) -> Result<()> {
    let TestChannel { stream } = channel;
    stream
        .set_nonblocking(true)
        .context("cannot use the test channel")?;
    let (reader, writer) = UnixStream::from_std(stream)
        .context("cannot use the test channel")?
        .into_split();
    let mut reader = MessageReader::new(reader);
    let mut writer = MessageWriter::new(writer);
    while let Some(request) = reader.recv::<ToProcess>().await? {
        let answer = match request {
            ToProcess::Eval { id, source, args } => Some(FromProcess::Answer {
                id,
                result: taps.eval(source, args).await,
            }),
            ToProcess::Reload { id } => {
                let error = match taps.load().await {
                    Some((_, error)) => error,
                    None => Some("the server runs no Lua".to_owned()),
                };
                Some(FromProcess::Reloaded { id, error })
            }
            ToProcess::SetTime { time } => {
                gband_lua::freeze_time(time);
                None
            }
            ToProcess::Settle { round, .. } => Some(FromProcess::Settled {
                round,
                sent: hub.settling.settle().await,
            }),
            ToProcess::Start { .. } => None,
        };
        if let Some(answer) = answer {
            writer
                .send(&answer)
                .await
                .context("cannot answer on the test channel")?;
        }
    }
    Ok(())
}
