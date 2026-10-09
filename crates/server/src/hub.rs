use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use gband_core::layout::WindowId;
use gband_lua::server::{Host, SessionView};
use gband_protocol::{Answer, Operation, Requirement, ServerMessage, SessionName, Value};
use tokio::sync::{mpsc, oneshot};

use crate::channel::Settling;
use crate::registry::SessionHandle;

pub const QUEUE_LIMIT: usize = 256;

pub type State = BTreeMap<String, Value>;

pub type Call = (u64, oneshot::Receiver<Answer>);

struct Queued {
    session: Option<SessionName>,
    name: String,
    data: Value,
    time: u64,
}

struct Client {
    session: SessionName,
    sender: mpsc::UnboundedSender<ServerMessage>,
}

#[derive(Default)]
struct Inner {
    sessions: BTreeMap<SessionName, Arc<SessionHandle>>,
    states: HashMap<(SessionName, WindowId), State>,
    clients: BTreeMap<u64, Client>,
    queue: VecDeque<Queued>,
    errors: Vec<String>,
    requirements: Vec<Requirement>,
    calls: HashMap<u64, oneshot::Sender<Answer>>,
    next_call: u64,
    typed: HashMap<SessionName, u64>,
}

#[derive(Default)]
pub struct Hub {
    inner: Mutex<Inner>,
    pub settling: Settling,
}

pub struct Attached {
    pub messages: Vec<ServerMessage>,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

impl Inner {
    fn send_to(&self, session: Option<&SessionName>, message: &ServerMessage) -> bool {
        let mut sent = false;
        for client in self.clients.values() {
            if session.is_none_or(|session| *session == client.session) {
                let _ = client.sender.send(message.clone());
                sent = true;
            }
        }
        sent
    }
}

impl Hub {
    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().expect("hub lock")
    }

    pub fn add_session(&self, handle: Arc<SessionHandle>) {
        self.lock().sessions.insert(handle.name.clone(), handle);
    }

    pub fn remove_session(&self, name: &SessionName, handle: &Arc<SessionHandle>) {
        let mut inner = self.lock();
        if inner
            .sessions
            .get(name)
            .is_some_and(|current| Arc::ptr_eq(current, handle))
        {
            inner.sessions.remove(name);
        }
    }

    pub fn session(&self, name: &str) -> Option<Arc<SessionHandle>> {
        let name: SessionName = name.parse().ok()?;
        self.lock().sessions.get(&name).cloned()
    }

    pub fn window_opened(&self, session: &SessionName, window: WindowId) {
        self.lock()
            .states
            .entry((session.clone(), window))
            .or_default();
    }

    pub fn window_closed(&self, session: &SessionName, window: WindowId) {
        self.lock().states.remove(&(session.clone(), window));
    }

    pub fn session_ended(&self, session: &SessionName) {
        self.lock().states.retain(|(name, _), _| name != session);
    }

    pub fn attach(
        &self,
        client: u64,
        session: &SessionName,
        sender: mpsc::UnboundedSender<ServerMessage>,
    ) -> Attached {
        let mut inner = self.lock();
        let mut states: Vec<(&(SessionName, WindowId), &State)> = inner
            .states
            .iter()
            .filter(|((name, _), _)| name == session)
            .collect();
        states.sort_by_key(|((_, window), _)| *window);
        let mut messages: Vec<ServerMessage> = states
            .into_iter()
            .flat_map(|((_, window), state)| {
                state.iter().map(|(key, value)| ServerMessage::WindowState {
                    window: *window,
                    key: key.clone(),
                    value: Some(value.clone()),
                })
            })
            .collect();
        messages.push(ServerMessage::Requirements(inner.requirements.clone()));
        messages.extend(inner.errors.last().cloned().map(ServerMessage::ServerError));
        let (delivered, kept): (VecDeque<Queued>, VecDeque<Queued>) =
            std::mem::take(&mut inner.queue)
                .into_iter()
                .partition(|queued| queued.session.as_ref().is_none_or(|name| name == session));
        inner.queue = kept;
        messages.extend(delivered.into_iter().map(|queued| ServerMessage::Event {
            name: queued.name,
            data: queued.data,
            queued: true,
            time: queued.time,
        }));
        inner.clients.insert(
            client,
            Client {
                session: session.clone(),
                sender,
            },
        );
        Attached { messages }
    }

    pub fn detach(&self, client: u64) {
        let mut inner = self.lock();
        if let Some(detached) = inner.clients.remove(&client)
            && inner.typed.get(&detached.session) == Some(&client)
        {
            inner.typed.remove(&detached.session);
        }
    }

    pub fn typed(&self, client: u64) {
        let mut inner = self.lock();
        if let Some(session) = inner
            .clients
            .get(&client)
            .map(|typed| typed.session.clone())
        {
            inner.typed.insert(session, client);
        }
    }

    pub fn chosen(&self, session: &SessionName) -> Option<u64> {
        let inner = self.lock();
        inner.typed.get(session).copied().or_else(|| {
            inner
                .clients
                .iter()
                .rev()
                .find(|(_, client)| client.session == *session)
                .map(|(&id, _)| id)
        })
    }

    pub fn control(&self, client: u64, operation: Operation) -> Option<Call> {
        let mut inner = self.lock();
        let call = inner.next_call;
        inner.next_call += 1;
        let (sender, answer) = oneshot::channel();
        let sent = inner
            .clients
            .get(&client)?
            .sender
            .send(ServerMessage::Control { call, operation });
        if sent.is_err() {
            return None;
        }
        inner.calls.insert(call, sender);
        Some((call, answer))
    }

    pub fn send(&self, client: u64, message: ServerMessage) {
        if let Some(client) = self.lock().clients.get(&client) {
            let _ = client.sender.send(message);
        }
    }

    pub fn answer(&self, call: u64, answer: Answer) {
        if let Some(sender) = self.lock().calls.remove(&call) {
            let _ = sender.send(answer);
        }
    }

    pub fn forget(&self, call: u64) {
        self.lock().calls.remove(&call);
    }

    pub fn clients_of(&self, session: &SessionName) -> Vec<u64> {
        self.lock()
            .clients
            .iter()
            .filter(|(_, client)| client.session == *session)
            .map(|(&id, _)| id)
            .collect()
    }

    pub fn state(&self, session: &SessionName, window: WindowId) -> Option<State> {
        self.lock().states.get(&(session.clone(), window)).cloned()
    }

    pub fn set_state(
        &self,
        session: &SessionName,
        window: WindowId,
        key: &str,
        value: Option<Value>,
    ) {
        let mut inner = self.lock();
        let Some(state) = inner.states.get_mut(&(session.clone(), window)) else {
            return;
        };
        match &value {
            Some(value) => state.insert(key.to_owned(), value.clone()),
            None => state.remove(key),
        };
        inner.send_to(
            Some(session),
            &ServerMessage::WindowState {
                window,
                key: key.to_owned(),
                value,
            },
        );
    }

    pub fn emit(&self, name: String, data: Value, session: Option<SessionName>) {
        let time = now();
        let mut inner = self.lock();
        let message = ServerMessage::Event {
            name: name.clone(),
            data: data.clone(),
            queued: false,
            time,
        };
        if inner.send_to(session.as_ref(), &message) {
            return;
        }
        if inner.queue.len() == QUEUE_LIMIT {
            let dropped = inner.queue.pop_front().expect("the queue is full");
            tracing::warn!(
                "dropping the queued event `{}` because {QUEUE_LIMIT} events wait for a client",
                dropped.name
            );
        }
        inner.queue.push_back(Queued {
            session,
            name,
            data,
            time,
        });
    }

    pub fn report(&self, error: String) {
        let mut inner = self.lock();
        inner.send_to(None, &ServerMessage::ServerError(error.clone()));
        inner.errors.push(error);
    }

    pub fn clear_errors(&self) {
        self.lock().errors.clear();
    }

    pub fn errors(&self) -> Vec<String> {
        self.lock().errors.clone()
    }

    pub fn set_requirements(&self, requirements: Vec<Requirement>) {
        self.lock().requirements = requirements;
    }
}

pub struct HubHost(pub Arc<Hub>);

fn session_name(name: &str) -> Option<SessionName> {
    name.parse().ok()
}

impl Host for HubHost {
    fn sessions(&self) -> Vec<String> {
        self.0
            .lock()
            .sessions
            .keys()
            .map(|name| name.as_str().to_owned())
            .collect()
    }

    fn session(&self, name: &str) -> Option<SessionView> {
        let handle = self.0.session(name)?;
        let (layout, area) = {
            let state = handle.state.borrow();
            (state.layout.clone(), state.area)
        };
        Some(SessionView {
            layout,
            area,
            clients: self.0.clients_of(&handle.name),
        })
    }

    fn window_state(&self, session: &str, window: WindowId) -> Option<State> {
        self.0.state(&session_name(session)?, window)
    }

    fn set_window_state(&self, session: &str, window: WindowId, key: &str, value: Option<Value>) {
        if let Some(session) = session_name(session) {
            self.0.set_state(&session, window, key, value);
        }
    }

    fn emit(&self, name: String, data: Value, session: Option<String>) {
        let session = match session {
            Some(session) => match session_name(&session) {
                Some(session) => Some(session),
                None => {
                    tracing::warn!(
                        "dropping the event `{name}` for the invalid session `{session}`"
                    );
                    return;
                }
            },
            None => None,
        };
        self.0.emit(name, data, session);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(text: &str) -> SessionName {
        text.parse().unwrap()
    }

    fn events(messages: &[ServerMessage]) -> Vec<(String, bool)> {
        messages
            .iter()
            .filter_map(|message| match message {
                ServerMessage::Event { name, queued, .. } => Some((name.clone(), *queued)),
                _ => None,
            })
            .collect()
    }

    fn drain(receiver: &mut mpsc::UnboundedReceiver<ServerMessage>) -> Vec<ServerMessage> {
        let mut messages = Vec::new();
        while let Ok(message) = receiver.try_recv() {
            messages.push(message);
        }
        messages
    }

    #[test]
    fn queued_while_detached() {
        let hub = Hub::default();
        hub.emit("a".to_owned(), Value::Nil, None);
        hub.emit("b".to_owned(), Value::Nil, None);
        let (sender, _receiver) = mpsc::unbounded_channel();
        let attached = hub.attach(1, &name("work"), sender);
        assert_eq!(
            events(&attached.messages),
            [("a".to_owned(), true), ("b".to_owned(), true)]
        );
        let (sender, _receiver) = mpsc::unbounded_channel();
        let later = hub.attach(2, &name("work"), sender);
        assert!(events(&later.messages).is_empty());
    }

    #[test]
    fn queue_limit() {
        let hub = Hub::default();
        for index in 0..300 {
            hub.emit(index.to_string(), Value::Int(index), None);
        }
        let (sender, _receiver) = mpsc::unbounded_channel();
        let names: Vec<String> = events(&hub.attach(1, &name("work"), sender).messages)
            .into_iter()
            .map(|(name, _)| name)
            .collect();
        let expected: Vec<String> = (44..300).map(|index: i32| index.to_string()).collect();
        assert_eq!(names, expected);
    }

    #[test]
    fn one_session() {
        let hub = Hub::default();
        let (work, mut work_rx) = mpsc::unbounded_channel();
        let (play, mut play_rx) = mpsc::unbounded_channel();
        hub.attach(1, &name("work"), work);
        hub.attach(2, &name("play"), play);
        hub.emit("x".to_owned(), Value::Nil, Some(name("work")));
        hub.emit("y".to_owned(), Value::Nil, None);
        assert_eq!(
            events(&drain(&mut work_rx)),
            [("x".to_owned(), false), ("y".to_owned(), false)]
        );
        assert_eq!(events(&drain(&mut play_rx)), [("y".to_owned(), false)]);
    }

    #[test]
    fn event_for_a_detached_session_waits_for_it() {
        let hub = Hub::default();
        let (play, mut play_rx) = mpsc::unbounded_channel();
        hub.attach(1, &name("play"), play);
        hub.emit("x".to_owned(), Value::Nil, Some(name("work")));
        assert!(drain(&mut play_rx).is_empty());
        let (work, _work_rx) = mpsc::unbounded_channel();
        let attached = hub.attach(2, &name("work"), work);
        assert_eq!(events(&attached.messages), [("x".to_owned(), true)]);
    }

    #[test]
    fn states_are_sent_and_removed_on_close() {
        let hub = Hub::default();
        let work = name("work");
        hub.window_opened(&work, WindowId(1));
        hub.set_state(&work, WindowId(1), "agent", Some(Value::string("waiting")));
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let attached = hub.attach(1, &work, sender);
        assert_eq!(
            attached.messages[0],
            ServerMessage::WindowState {
                window: WindowId(1),
                key: "agent".to_owned(),
                value: Some(Value::string("waiting")),
            }
        );
        assert!(matches!(
            attached.messages[1],
            ServerMessage::Requirements(_)
        ));
        hub.set_state(&work, WindowId(1), "agent", None);
        assert_eq!(
            drain(&mut receiver),
            [ServerMessage::WindowState {
                window: WindowId(1),
                key: "agent".to_owned(),
                value: None,
            }]
        );
        hub.window_closed(&work, WindowId(1));
        assert_eq!(hub.state(&work, WindowId(1)), None);
        hub.set_state(&work, WindowId(1), "agent", Some(Value::Nil));
        assert!(drain(&mut receiver).is_empty());
    }

    #[test]
    fn every_error_kept_and_latest_sent_until_cleared() {
        let hub = Hub::default();
        let (sender, mut receiver) = mpsc::unbounded_channel();
        hub.attach(1, &name("work"), sender);
        hub.report("first".to_owned());
        hub.report("second".to_owned());
        assert_eq!(drain(&mut receiver).len(), 2);
        assert_eq!(hub.errors(), ["first", "second"]);
        let (sender, _receiver) = mpsc::unbounded_channel();
        let attached = hub.attach(2, &name("work"), sender);
        assert!(
            attached
                .messages
                .contains(&ServerMessage::ServerError("second".to_owned()))
        );
        hub.clear_errors();
        assert!(hub.errors().is_empty());
        let (sender, _receiver) = mpsc::unbounded_channel();
        let attached = hub.attach(3, &name("work"), sender);
        assert_eq!(attached.messages, [ServerMessage::Requirements(Vec::new())]);
    }

    #[test]
    fn chosen_client_typed_last_or_attached_last() {
        let hub = Hub::default();
        let work = name("work");
        assert_eq!(hub.chosen(&work), None);
        let (first, _first_rx) = mpsc::unbounded_channel();
        let (second, _second_rx) = mpsc::unbounded_channel();
        let (other, _other_rx) = mpsc::unbounded_channel();
        hub.attach(1, &work, first);
        hub.attach(2, &work, second);
        hub.attach(3, &name("play"), other);
        assert_eq!(hub.chosen(&work), Some(2));
        hub.typed(1);
        assert_eq!(hub.chosen(&work), Some(1));
        hub.typed(3);
        assert_eq!(hub.chosen(&work), Some(1));
        hub.detach(1);
        assert_eq!(hub.chosen(&work), Some(2));
    }

    #[test]
    fn control_answer_reaches_its_call_once() {
        let hub = Hub::default();
        let (sender, mut receiver) = mpsc::unbounded_channel();
        hub.attach(1, &name("work"), sender);
        assert!(hub.control(2, Operation::Errors).is_none());
        let (call, mut answer) = hub.control(1, Operation::Errors).unwrap();
        assert_eq!(
            drain(&mut receiver),
            [ServerMessage::Control {
                call,
                operation: Operation::Errors
            }]
        );
        let loaded = Answer::Loaded {
            load: 1,
            error: None,
        };
        hub.answer(call, loaded.clone());
        assert_eq!(answer.try_recv().unwrap(), loaded);
        let (late, mut forgotten) = hub.control(1, Operation::Reload).unwrap();
        hub.forget(late);
        hub.answer(late, loaded);
        assert!(forgotten.try_recv().is_err());
    }
}
