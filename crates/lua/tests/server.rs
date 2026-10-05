mod common;

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use common::*;
use gband_core::layout::{
    BandId, Layout, LayoutOptions, PaneContent, PaneId, Program, SessionAction,
};
use gband_lua::server::{Caller, Event, Host, SessionView};
use gband_lua::{Config, Dispatch, Outcome};
use gband_protocol::{Key, Value};

type States = BTreeMap<(String, u32), BTreeMap<String, Value>>;

#[derive(Default)]
struct Fake {
    states: Mutex<States>,
    writes: Mutex<usize>,
    emitted: Mutex<Vec<(String, Value, Option<String>)>>,
    sessions: Mutex<BTreeMap<String, (Layout, Vec<u64>)>>,
}

impl Fake {
    fn with_pane(session: &str, pane: u32) -> Arc<Self> {
        let fake = Arc::new(Fake::default());
        fake.states
            .lock()
            .unwrap()
            .insert((session.to_owned(), pane), BTreeMap::new());
        fake
    }

    fn state(&self, session: &str, pane: u32) -> BTreeMap<String, Value> {
        self.states.lock().unwrap()[&(session.to_owned(), pane)].clone()
    }

    fn emitted(&self) -> Vec<(String, Value, Option<String>)> {
        self.emitted.lock().unwrap().clone()
    }
}

impl Host for Fake {
    fn sessions(&self) -> Vec<String> {
        self.sessions.lock().unwrap().keys().cloned().collect()
    }

    fn session(&self, name: &str) -> Option<SessionView> {
        let sessions = self.sessions.lock().unwrap();
        let (layout, clients) = sessions.get(name)?;
        Some(SessionView {
            layout: layout.clone(),
            clients: clients.clone(),
        })
    }

    fn pane_state(&self, session: &str, pane: PaneId) -> Option<BTreeMap<String, Value>> {
        self.states
            .lock()
            .unwrap()
            .get(&(session.to_owned(), pane.0))
            .cloned()
    }

    fn set_pane_state(&self, session: &str, pane: PaneId, key: &str, value: Option<Value>) {
        *self.writes.lock().unwrap() += 1;
        let mut states = self.states.lock().unwrap();
        let state = states.get_mut(&(session.to_owned(), pane.0)).unwrap();
        match value {
            Some(value) => state.insert(key.to_owned(), value),
            None => state.remove(key),
        };
    }

    fn emit(&self, name: String, data: Value, session: Option<String>) {
        self.emitted.lock().unwrap().push((name, data, session));
    }
}

fn server(source: &str, host: Arc<Fake>) -> (Scratch, Config) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let scratch = Scratch::new(&format!("server-{}", NEXT.fetch_add(1, Ordering::Relaxed)));
    scratch.server(source);
    let config = scratch.loaded_server();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    config.runtime.set_host(host);
    (scratch, config)
}

fn opened(session: &str, pane: u32) -> Event {
    Event::PaneOpened {
        session: session.to_owned(),
        pane: PaneId(pane),
        band: BandId(1),
    }
}

fn text(value: &str) -> Value {
    Value::string(value)
}

#[test]
fn user_event_refused() {
    let scratch = Scratch::new("user-event");
    let path = scratch.server("gband.on('User', function() end)");
    let error = scratch.load_server().err().unwrap();
    assert_error_at(&error, &path, 1, "`User`");
}

#[test]
fn server_event_name_in_the_client() {
    let scratch = Scratch::new("server-event-in-client");
    let path = scratch.write("\n\n\ngband.on('PaneOutput', function() end)");
    let error = scratch.load().err().unwrap();
    assert_error_at(&error, &path, 4, "`PaneOutput`");
    assert!(error.message.contains("server"), "{error}");
}

#[test]
fn client_event_in_the_server() {
    let scratch = Scratch::new("client-event-in-server");
    let path = scratch.server("gband.on('FocusChanged', function() end)");
    let error = scratch.load_server().err().unwrap();
    assert_error_at(&error, &path, 1, "`FocusChanged`");
    assert!(error.message.contains("client"), "{error}");
}

#[test]
fn pattern_is_refused_for_server_events() {
    let scratch = Scratch::new("server-pattern");
    let path = scratch.server("gband.on('PaneOpened', function() end, { pattern = 'x' })");
    let error = scratch.load_server().err().unwrap();
    assert_error_at(&error, &path, 1, "pattern");
}

#[test]
fn payload_of_each_server_event() {
    let names = [
        "SessionCreated",
        "SessionEnded",
        "PaneOpened",
        "PaneClosed",
        "PaneExited",
        "PaneOutput",
        "PaneInput",
        "ClientAttached",
        "ClientDetached",
        "ConfigReloaded",
    ];
    let mut source = "log = {}\n".to_owned();
    for name in names {
        source.push_str(&format!(
            "gband.on('{name}', function(ev)
               local keys = {{}}
               for k, v in pairs(ev) do keys[#keys + 1] = k .. '=' .. tostring(v) end
               table.sort(keys)
               log[#log + 1] = '{name} ' .. table.concat(keys, ' ')
             end)\n"
        ));
    }
    let (_scratch, config) = server(&source, Arc::new(Fake::default()));
    let session = || "work".to_owned();
    let events = [
        Event::SessionCreated { session: session() },
        Event::SessionEnded { session: session() },
        opened("work", 2),
        Event::PaneClosed {
            session: session(),
            pane: PaneId(2),
            band: BandId(3),
        },
        Event::PaneExited {
            session: session(),
            pane: PaneId(2),
            code: Some(3),
            signal: None,
        },
        Event::PaneExited {
            session: session(),
            pane: PaneId(2),
            code: None,
            signal: Some(9),
        },
        Event::PaneOutput {
            session: session(),
            pane: PaneId(2),
            data: b"hi\r\n".to_vec(),
        },
        Event::PaneInput {
            session: session(),
            pane: PaneId(2),
            client: 7,
        },
        Event::ClientAttached {
            session: session(),
            client: 7,
        },
        Event::ClientDetached {
            session: session(),
            client: 7,
        },
        Event::ConfigReloaded,
    ];
    for event in &events {
        clean(&config.runtime.emit_server(event));
    }
    let log: Vec<String> = global(&config, "log");
    assert_eq!(
        log,
        [
            "SessionCreated session=work",
            "SessionEnded session=work",
            "PaneOpened band=1 pane=2 session=work",
            "PaneClosed band=3 pane=2 session=work",
            "PaneExited code=3 pane=2 session=work",
            "PaneExited pane=2 session=work signal=9",
            "PaneOutput data=hi\r\n pane=2 session=work",
            "PaneInput client=7 pane=2 session=work",
            "ClientAttached client=7 session=work",
            "ClientDetached client=7 session=work",
            "ConfigReloaded ",
        ]
    );
    assert!(config.runtime.handles("PaneOutput"));
}

#[test]
fn handles_only_registered_events() {
    let (_scratch, config) = server(
        "gband.on('PaneOpened', function() end)",
        Arc::new(Fake::default()),
    );
    assert!(config.runtime.handles("PaneOpened"));
    assert!(!config.runtime.handles("PaneOutput"));
    assert!(!config.runtime.handles("PaneInput"));
}

#[test]
fn emit_reaches_the_host() {
    let host = Arc::new(Fake::default());
    let (_scratch, config) = server(
        "gband.on('PaneOpened', function(ev)
           gband.emit('agent.waiting', { pane = ev.pane, title = 'build' })
           gband.emit('agent.done', nil, { session = 'work' })
         end)",
        Arc::clone(&host),
    );
    clean(&config.runtime.emit_server(&opened("work", 1)));
    let sorted: Vec<_> = host
        .emitted()
        .into_iter()
        .map(|(name, data, session)| match data {
            Value::Table(mut entries) => {
                entries.sort_by(|a, b| a.0.cmp(&b.0));
                (name, Value::Table(entries), session)
            }
            other => (name, other, session),
        })
        .collect();
    assert_eq!(
        sorted,
        [
            (
                "agent.waiting".to_owned(),
                Value::Table(vec![
                    (Key::string("pane"), Value::Int(1)),
                    (Key::string("title"), text("build")),
                ]),
                None
            ),
            ("agent.done".to_owned(), Value::Nil, Some("work".to_owned())),
        ]
    );
}

fn failed_handler(source: &str) -> (Outcome, Arc<Fake>) {
    let host = Fake::with_pane("work", 1);
    let (_scratch, config) = server(source, Arc::clone(&host));
    let outcome = config.runtime.emit_server(&opened("work", 1));
    (outcome, host)
}

#[test]
fn function_inside_emitted_data() {
    let (outcome, host) = failed_handler(
        "gband.on('PaneOpened', function()
           local x = 1
           gband.emit('bad', { cb = { run = function() end } })
         end)",
    );
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert_eq!(error.location.as_ref().map(|(_, line)| *line), Some(3));
    assert!(error.message.contains("`data.cb.run`"), "{error}");
    assert!(host.emitted().is_empty());
}

#[test]
fn emit_validation_errors_at_the_call_line() {
    for (call, mentions) in [
        ("gband.emit('', {})", "event name"),
        ("gband.emit(3, {})", "event name"),
        ("gband.emit('x', {}, { sesion = 'w' })", "`sesion`"),
        ("gband.emit('x', {}, { session = 3 })", "`session`"),
        ("gband.emit('x', {}, 'work')", "options"),
        ("local t = {} t.self = t gband.emit('x', t)", "`data.self`"),
    ] {
        let (outcome, host) =
            failed_handler(&format!("gband.on('PaneOpened', function()\n{call}\nend)"));
        let [error] = outcome.errors.as_slice() else {
            panic!("{call}: {:?}", outcome.errors);
        };
        assert_eq!(
            error.location.as_ref().map(|(_, line)| *line),
            Some(2),
            "{call}: {error}"
        );
        assert!(error.message.contains(mentions), "{call}: {error}");
        assert!(host.emitted().is_empty());
    }
}

#[test]
fn emit_while_loading_is_an_error() {
    let scratch = Scratch::new("emit-loading");
    let path = scratch.server("\ngband.emit('x', {})");
    let error = scratch.load_server().err().unwrap();
    assert_error_at(&error, &path, 2, "gband.emit");
}

#[test]
fn set_and_clear_pane_state() {
    let (outcome, host) = failed_handler(
        "gband.on('PaneOpened', function(ev)
           local state = gband.pane_state(ev.session, ev.pane)
           state.agent = 'waiting'
           state.count = 2
           state.count = nil
         end)",
    );
    clean(&outcome);
    assert_eq!(
        host.state("work", 1),
        BTreeMap::from([("agent".to_owned(), text("waiting"))])
    );
}

#[test]
fn pane_state_reads_return_copies() {
    let host = Fake::with_pane("work", 1);
    let (_scratch, config) = server(
        "gband.on('PaneOpened', function(ev)
           local state = gband.pane_state(ev.session, ev.pane)
           state.list = { 1, 2 }
           local copy = state.list
           copy[1] = 9
           first = state.list[1]
           seen = state.list ~= state.list
           missing = gband.pane_state(ev.session, 99) == nil
           keys = {}
           for k in pairs(state) do keys[#keys + 1] = k end
         end)",
        Arc::clone(&host),
    );
    clean(&config.runtime.emit_server(&opened("work", 1)));
    assert_eq!(global::<i64>(&config, "first"), 1);
    assert!(global::<bool>(&config, "seen"));
    assert!(global::<bool>(&config, "missing"));
    assert_eq!(global::<Vec<String>>(&config, "keys"), ["list"]);
}

#[test]
fn function_refused_in_pane_state() {
    let (outcome, host) = failed_handler(
        "gband.on('PaneOpened', function(ev)
           local state = gband.pane_state(ev.session, ev.pane)
           state.agent = 'waiting'
           local x = 1
           state.agent = function() end
         end)",
    );
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert_eq!(error.location.as_ref().map(|(_, line)| *line), Some(5));
    assert!(error.message.contains("state.agent"), "{error}");
    assert_eq!(
        host.state("work", 1),
        BTreeMap::from([("agent".to_owned(), text("waiting"))])
    );
}

#[test]
fn invalid_keys_are_refused() {
    for assignment in ["state[1] = 'x'", "state[''] = 'x'"] {
        let (outcome, host) = failed_handler(&format!(
            "gband.on('PaneOpened', function(ev)
               local state = gband.pane_state(ev.session, ev.pane)
               {assignment}
             end)"
        ));
        assert_eq!(outcome.errors.len(), 1, "{assignment}");
        assert!(host.state("work", 1).is_empty());
    }
}

#[test]
fn state_size_limit() {
    let (outcome, host) = failed_handler(
        "gband.on('PaneOpened', function(ev)
           local state = gband.pane_state(ev.session, ev.pane)
           state.a = string.rep('x', 40000)
           state.b = string.rep('y', 40000)
         end)",
    );
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert!(error.message.contains("64 KiB"), "{error}");
    assert_eq!(error.location.as_ref().map(|(_, line)| *line), Some(4));
    let state = host.state("work", 1);
    assert_eq!(state.keys().collect::<Vec<_>>(), ["a"]);
}

#[test]
fn equal_assignment_changes_nothing() {
    let (outcome, host) = failed_handler(
        "gband.on('PaneOpened', function(ev)
           local state = gband.pane_state(ev.session, ev.pane)
           state.agent = { name = 'build', n = 1 }
           state.agent = { name = 'build', n = 1 }
           state.absent = nil
         end)",
    );
    clean(&outcome);
    assert_eq!(*host.writes.lock().unwrap(), 1);
}

fn layout_of(panes: usize) -> Layout {
    let mut layout = Layout::new();
    let band = layout.bands()[0].id;
    let mut after = None;
    for _ in 0..panes {
        let pane = layout.allocate_pane();
        layout.open(pane, band, after, None, &LayoutOptions::default());
        after = Some(pane);
    }
    layout
}

#[test]
fn layout_of_a_session() {
    let host = Arc::new(Fake::default());
    host.sessions
        .lock()
        .unwrap()
        .insert("work".to_owned(), (layout_of(2), vec![4]));
    host.sessions
        .lock()
        .unwrap()
        .insert("play".to_owned(), (layout_of(1), vec![]));
    let (_scratch, config) = server(
        "gband.on('ConfigReloaded', function()
           names = gband.sessions()
           local work = gband.session('work')
           second = work.bands[1].columns[2].panes[1].pane
           width = work.bands[1].columns[1].width
           clients = #work.clients
           band = work.bands[1].band
           unknown = gband.session('absent') == nil
         end)",
        host,
    );
    clean(&config.runtime.emit_server(&Event::ConfigReloaded));
    assert_eq!(global::<Vec<String>>(&config, "names"), ["play", "work"]);
    assert_eq!(global::<i64>(&config, "second"), 2);
    assert_eq!(global::<f64>(&config, "width"), 0.5);
    assert_eq!(global::<i64>(&config, "clients"), 1);
    assert_eq!(global::<i64>(&config, "band"), 1);
    assert!(global::<bool>(&config, "unknown"));
}

#[test]
fn open_pane_target_with_a_program_list() {
    let (_scratch, config) = server(
        "gband.on('PaneOpened', function(ev)
           gband.action.open_pane({ session = ev.session, band = 1, after = ev.pane, program = { 'htop', '-d', '10' } })
           gband.action.grow_column_width({ session = ev.session, pane = ev.pane })
         end)",
        Arc::new(Fake::default()),
    );
    let outcome = config.runtime.emit_server(&opened("work", 1));
    clean(&outcome);
    assert_eq!(
        outcome.dispatched,
        [
            Dispatch::Targeted {
                session: "work".to_owned(),
                action: SessionAction::OpenPane {
                    band: BandId(1),
                    after: Some(PaneId(1)),
                    width: None,
                    focus: false,
                    content: PaneContent::Program(Some(Program::Argv(vec![
                        "htop".to_owned(),
                        "-d".to_owned(),
                        "10".to_owned()
                    ]))),
                },
            },
            Dispatch::Targeted {
                session: "work".to_owned(),
                action: SessionAction::StepWidth {
                    pane: PaneId(1),
                    step: gband_core::layout::Step::Grow,
                },
            },
        ]
    );
}

#[test]
fn invalid_action_targets() {
    for (call, mentions) in [
        ("gband.action.close_pane({ pane = 1 })", "`session`"),
        ("gband.action.close_pane({ session = 'w' })", "`pane`"),
        (
            "gband.action.close_pane({ session = 'w', pane = 'x' })",
            "`pane`",
        ),
        (
            "gband.action.close_pane({ session = 'w', pane = 1, band = 1 })",
            "`band`",
        ),
        ("gband.action.open_pane({ session = 'w' })", "`band`"),
        (
            "gband.action.open_pane({ session = 'w', band = 1, program = {} })",
            "`program`",
        ),
        ("gband.action.close_pane(1)", "target table"),
    ] {
        let (outcome, _) =
            failed_handler(&format!("gband.on('PaneOpened', function()\n{call}\nend)"));
        let [error] = outcome.errors.as_slice() else {
            panic!("{call}: {:?}", outcome.errors);
        };
        assert!(error.message.contains(mentions), "{call}: {error}");
        assert_eq!(error.location.as_ref().map(|(_, line)| *line), Some(2));
        assert!(outcome.dispatched.is_empty(), "{call}");
    }
}

#[test]
fn actions_while_loading_are_an_error() {
    let scratch = Scratch::new("action-loading");
    let path = scratch.server("gband.action.close_pane({ session = 'w', pane = 1 })");
    let error = scratch.load_server().err().unwrap();
    assert_error_at(&error, &path, 1, "callback");
}

const COMMANDS: &str = "
gband.cmd.register('list', function(args, ctx)
  seen = { session = ctx.session, client = ctx.client }
  ctx.focus(3)
  return { count = args.n }
end)
gband.cmd.register('bad', function() return function() end end)
gband.cmd.register('boom', function() error('boom') end)
gband.on('ConfigReloaded', function()
  ran = gband.cmd.run('list', { n = 1 })
  local_client = seen.client
  bad = gband.cmd.run('bad')
end)
";

#[test]
fn local_run() {
    let (_scratch, config) = server(COMMANDS, Arc::new(Fake::default()));
    let outcome = config.runtime.emit_server(&Event::ConfigReloaded);
    assert!(global::<bool>(&config, "ran"));
    assert!(global::<Option<i64>>(&config, "local_client").is_none());
    assert!(!global::<bool>(&config, "bad"));
    assert_eq!(outcome.errors.len(), 1, "{:?}", outcome.errors);
}

#[test]
fn command_from_a_client() {
    let (_scratch, config) = server(COMMANDS, Arc::new(Fake::default()));
    let focused = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&focused);
    let caller = Caller {
        session: "work".to_owned(),
        client: 7,
        focus: Arc::new(move |pane| sink.lock().unwrap().push(pane)),
    };
    let args = Value::Table(vec![(Key::string("n"), Value::Int(4))]);
    let (result, outcome) = config.runtime.command("list", args, Some(caller));
    clean(&outcome);
    assert_eq!(
        result,
        Ok(Value::Table(vec![(Key::string("count"), Value::Int(4))]))
    );
    assert_eq!(*focused.lock().unwrap(), [PaneId(3)]);
    let session: String = eval(&config, "return seen.session");
    assert_eq!(session, "work");
    assert_eq!(eval::<i64>(&config, "return seen.client"), 7);
}

#[test]
fn failing_commands() {
    let (_scratch, config) = server(COMMANDS, Arc::new(Fake::default()));
    let (result, _) = config.runtime.command("absent", Value::Nil, None);
    assert!(result.unwrap_err().contains("absent"));
    let (result, outcome) = config.runtime.command("boom", Value::Nil, None);
    assert!(result.unwrap_err().contains("boom"));
    assert_eq!(outcome.errors.len(), 1);
    let (result, outcome) = config.runtime.command("bad", Value::Nil, None);
    assert!(result.unwrap_err().contains("function"));
    assert_eq!(outcome.errors.len(), 1);
}
