use std::io::{ErrorKind, Read, Write};
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use gband_protocol::test::{FromProcess, Role, ToProcess};
use gband_protocol::{Decoder, Value, encode};

const POLL: Duration = Duration::from_millis(10);

pub struct Listener {
    listener: UnixListener,
}

pub struct Endpoint {
    role: Role,
    stream: UnixStream,
    messages: Receiver<FromProcess>,
    next_id: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Waited {
    TimedOut,
    Closed,
}

impl Listener {
    pub fn bind(path: &Path) -> std::io::Result<Self> {
        let listener = UnixListener::bind(path)?;
        listener.set_nonblocking(true)?;
        Ok(Self { listener })
    }

    pub fn accept(
        &self,
        time: Option<i64>,
        deadline: Instant,
        mut alive: impl FnMut() -> Result<(), String>,
    ) -> Result<(Endpoint, Endpoint), String> {
        let mut client = None;
        let mut server = None;
        while client.is_none() || server.is_none() {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    let endpoint = Endpoint::greet(stream, time, deadline)?;
                    let slot = match endpoint.role {
                        Role::Client => &mut client,
                        Role::Server => &mut server,
                    };
                    if slot.is_some() {
                        return Err(format!(
                            "a second {} joined the test channel",
                            endpoint.role.name()
                        ));
                    }
                    *slot = Some(endpoint);
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    alive()?;
                    if Instant::now() >= deadline {
                        let missing = if client.is_none() { "client" } else { "server" };
                        return Err(format!(
                            "the {missing} did not join the test channel in time"
                        ));
                    }
                    thread::sleep(POLL);
                }
                Err(error) => return Err(format!("cannot accept on the test channel: {error}")),
            }
        }
        Ok((client.expect("joined"), server.expect("joined")))
    }

    pub fn accept_one(&self, time: Option<i64>, deadline: Instant) -> Result<Endpoint, String> {
        loop {
            match self.listener.accept() {
                Ok((stream, _)) => return Endpoint::greet(stream, time, deadline),
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return Err("no process joined the test channel in time".to_owned());
                    }
                    thread::sleep(POLL);
                }
                Err(error) => return Err(format!("cannot accept on the test channel: {error}")),
            }
        }
    }
}

fn frame(message: &ToProcess) -> Vec<u8> {
    encode(message).expect("test channel messages encode")
}

impl Endpoint {
    fn greet(mut stream: UnixStream, time: Option<i64>, deadline: Instant) -> Result<Self, String> {
        let failed =
            |error: std::io::Error| format!("cannot greet a process on the test channel: {error}");
        stream.set_nonblocking(false).map_err(failed)?;
        stream
            .set_read_timeout(Some(
                deadline.saturating_duration_since(Instant::now()).max(POLL),
            ))
            .map_err(failed)?;
        let mut decoder = Decoder::new();
        let mut buffer = [0u8; 1024];
        let role = loop {
            match decoder.next_message::<FromProcess>() {
                Ok(Some(FromProcess::Hello { role })) => break role,
                Ok(Some(other)) => return Err(format!("a process greeted with {other:?}")),
                Ok(None) => {}
                Err(error) => return Err(format!("a process greeted with garbage: {error}")),
            }
            let read = stream.read(&mut buffer).map_err(failed)?;
            if read == 0 {
                return Err("a process left the test channel before greeting".to_owned());
            }
            decoder
                .feed(&buffer[..read])
                .map_err(|error| format!("a process greeted with garbage: {error}"))?;
        };
        stream.set_read_timeout(None).map_err(failed)?;
        stream
            .write_all(&frame(&ToProcess::Start { time }))
            .map_err(failed)?;
        let (sender, messages) = mpsc::channel();
        let mut reader = stream.try_clone().map_err(failed)?;
        thread::spawn(move || {
            let mut buffer = [0u8; 64 * 1024];
            loop {
                while let Ok(Some(message)) = decoder.next_message::<FromProcess>() {
                    if sender.send(message).is_err() {
                        return;
                    }
                }
                match reader.read(&mut buffer) {
                    Ok(0) | Err(_) => return,
                    Ok(read) => {
                        if decoder.feed(&buffer[..read]).is_err() {
                            return;
                        }
                    }
                }
            }
        });
        Ok(Self {
            role,
            stream,
            messages,
            next_id: 1,
        })
    }

    pub fn role(&self) -> Role {
        self.role
    }

    fn send(&mut self, message: &ToProcess) -> Result<(), Waited> {
        self.stream
            .write_all(&frame(message))
            .map_err(|_| Waited::Closed)
    }

    fn wait<T>(
        &mut self,
        deadline: Instant,
        mut pick: impl FnMut(FromProcess) -> Option<T>,
    ) -> Result<T, Waited> {
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            match self.messages.recv_timeout(left) {
                Ok(message) => {
                    if let Some(picked) = pick(message) {
                        return Ok(picked);
                    }
                }
                Err(RecvTimeoutError::Timeout) => return Err(Waited::TimedOut),
                Err(RecvTimeoutError::Disconnected) => return Err(Waited::Closed),
            }
        }
    }

    fn id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn eval(
        &mut self,
        source: &str,
        args: Vec<Value>,
        deadline: Instant,
    ) -> Result<Result<Vec<Value>, String>, Waited> {
        let id = self.id();
        self.send(&ToProcess::Eval {
            id,
            source: source.to_owned(),
            args,
        })?;
        self.wait(deadline, |message| match message {
            FromProcess::Answer {
                id: answered,
                result,
            } if answered == id => Some(result),
            _ => None,
        })
    }

    pub fn reload(&mut self, deadline: Instant) -> Result<Option<String>, Waited> {
        let id = self.id();
        self.send(&ToProcess::Reload { id })?;
        self.wait(deadline, |message| match message {
            FromProcess::Reloaded {
                id: answered,
                error,
            } if answered == id => Some(error),
            _ => None,
        })
    }

    pub fn settle(
        &mut self,
        round: u32,
        input: Option<u64>,
        deadline: Instant,
    ) -> Result<u64, Waited> {
        self.send(&ToProcess::Settle { round, input })?;
        self.wait(deadline, |message| match message {
            FromProcess::Settled {
                round: settled,
                sent,
            } if settled == round => Some(sent),
            _ => None,
        })
    }

    pub fn set_time(&mut self, time: Option<i64>) -> Result<(), Waited> {
        self.send(&ToProcess::SetTime { time })
    }
}

impl Drop for Endpoint {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(Shutdown::Both);
    }
}
