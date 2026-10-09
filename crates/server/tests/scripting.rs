use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use gband_core::input::{Key, KeyCode};
use gband_core::layout::{Proportion, WindowId};
use gband_lua::{LoadOptions, Locations, Side};
use gband_protocol::{
    Answer, ClientMessage, Entry, Key as DataKey, Operation, Process, Requirement, ServerMessage,
    SessionName, Target, Value,
};
use gband_server::{Reloader, Scripting, ServerConfig};
use gband_test_support::*;

struct Scripted {
    server: TestServer,
    reloader: Reloader,
    locations: Locations,
}

fn write(path: &Path, source: &str) -> PathBuf {
    let missing: Vec<&Path> = path
        .ancestors()
        .skip(1)
        .take_while(|directory| !directory.exists())
        .collect();
    for directory in missing.into_iter().rev() {
        fs::create_dir(directory).unwrap();
        fs::set_permissions(directory, fs::Permissions::from_mode(0o755)).unwrap();
    }
    fs::write(path, source).unwrap();
    path.to_path_buf()
}

fn locations(runtime_dir: &Path) -> Locations {
    Locations {
        config: runtime_dir.join("config"),
        plugins: Some(runtime_dir.join("plugins")),
    }
}

fn server_file(locations: &Locations) -> PathBuf {
    gband_lua::user_file(&locations.config, Side::Server)
}

fn plugin(runtime_dir: &Path, name: &str, manifest_extra: &str, server: &str) -> PathBuf {
    let directory = locations(runtime_dir).plugins.unwrap().join(name);
    write(
        &directory.join("plugin.lua"),
        &format!("return {{ name = '{name}', version = '0.1.0'{manifest_extra} }}"),
    );
    write(&directory.join("server.lua"), server)
}

fn load(locations: &Locations) -> (gband_lua::Config, Option<gband_lua::ConfigError>) {
    match gband_lua::load(locations, Side::Server, &LoadOptions::default()) {
        Ok(config) => (config, None),
        Err(error) => (
            gband_lua::load_defaults(locations, Side::Server, &LoadOptions::default()).unwrap(),
            Some(error),
        ),
    }
}

async fn start_in(runtime_dir: Scratch, source: &str) -> Scripted {
    let locations = locations(&runtime_dir);
    write(&server_file(&locations), source);
    let (loaded, error) = load(&locations);
    let (scripting, reloader) = Scripting::new(loaded, error);
    let loading = locations.clone();
    let scripting = scripting.with_loader(std::sync::Arc::new(move || {
        gband_lua::load(&loading, Side::Server, &LoadOptions::default())
    }));
    let config = ServerConfig {
        scripting: Some(scripting),
        channel: None,
        ..config(&runtime_dir, &["/bin/sh"])
    };
    let server = TestServer::start_with(runtime_dir, config).await;
    Scripted {
        server,
        reloader,
        locations,
    }
}

async fn start(name: &str, source: &str) -> Scripted {
    start_in(runtime_dir(name), source).await
}

impl Scripted {
    fn reload(&self, source: &str) {
        write(&server_file(&self.locations), source);
        self.reloader.reload(gband_lua::load(
            &self.locations,
            Side::Server,
            &LoadOptions::default(),
        ));
    }

    async fn attach(&self) -> TestClient {
        self.server.attach(80, 24).await
    }

    async fn server_answer(&self, operation: Operation) -> Answer {
        let answer = self
            .server
            .control("default", Target::Server, operation)
            .await;
        let ServerMessage::ControlResults(entries) = answer else {
            panic!("{answer:?}");
        };
        let [
            Entry {
                process: Process::Server,
                answer: Some(answer),
            },
        ] = &entries[..]
        else {
            panic!("{entries:?}");
        };
        answer.clone()
    }
}

fn text(value: &str) -> Value {
    Value::string(value)
}

fn field<'a>(data: &'a Value, name: &str) -> Option<&'a Value> {
    let Value::Table(entries) = data else {
        return None;
    };
    entries
        .iter()
        .find(|(key, _)| *key == DataKey::string(name))
        .map(|(_, value)| value)
}

fn list(data: &Value) -> Vec<Value> {
    let Value::Table(entries) = data else {
        return Vec::new();
    };
    let mut entries = entries.clone();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries.into_iter().map(|(_, value)| value).collect()
}

fn events(client: &TestClient, wanted: &str) -> Vec<(Value, bool)> {
    client
        .bridge
        .iter()
        .filter_map(|message| match message {
            ServerMessage::Event {
                name, data, queued, ..
            } if name == wanted => Some((data.clone(), *queued)),
            _ => None,
        })
        .collect()
}

async fn event(client: &mut TestClient, wanted: &str) -> Value {
    client
        .wait_until(|client| !events(client, wanted).is_empty())
        .await;
    events(client, wanted).remove(0).0
}

fn results(client: &TestClient, call: u64) -> Option<Result<Value, String>> {
    client.bridge.iter().find_map(|message| match message {
        ServerMessage::Result {
            call: answered,
            result,
        } if *answered == call => Some(result.clone()),
        _ => None,
    })
}

async fn call(
    client: &mut TestClient,
    call: u64,
    name: &str,
    args: Value,
) -> Result<Value, String> {
    client
        .send(&ClientMessage::Command {
            call,
            name: name.to_owned(),
            args,
        })
        .await;
    client
        .wait_until(|client| results(client, call).is_some())
        .await;
    results(client, call).unwrap()
}

fn errors(client: &TestClient) -> Vec<String> {
    client
        .bridge
        .iter()
        .filter_map(|message| match message {
            ServerMessage::ServerError(error) => Some(error.clone()),
            _ => None,
        })
        .collect()
}

const SETTLE: Duration = Duration::from_millis(500);

#[tokio::test(flavor = "multi_thread")]
async fn exit_then_close() {
    let scripted = start(
        "lua-exit-close",
        "local log = {}
gband.on('WindowExited', function(ev) log[#log + 1] = 'exited ' .. ev.window .. ' ' .. tostring(ev.code) end)
gband.on('WindowClosed', function(ev)
  log[#log + 1] = 'closed ' .. ev.window
  gband.emit('log', log)
end)",
    )
    .await;
    let mut client = scripted.attach().await;
    let first = client.first();
    let second = client.open_after(first).await;
    client.type_line_to(second, "exit 3").await;
    let log = event(&mut client, "log").await;
    assert_eq!(
        list(&log),
        [
            text(&format!("exited {second} 3")),
            text(&format!("closed {second}"))
        ]
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn handler_replaced() {
    let scripted = start(
        "lua-replaced",
        "gband.on('WindowOpened', function() gband.emit('one') end)",
    )
    .await;
    let mut client = scripted.attach().await;
    scripted.reload(
        "gband.on('WindowOpened', function() gband.emit('two') end)
gband.on('ConfigReloaded', function() gband.emit('reloaded') end)",
    );
    event(&mut client, "reloaded").await;
    client.bridge.clear();
    let first = client.first();
    client.open_after(first).await;
    event(&mut client, "two").await;
    assert!(events(&client, "one").is_empty());
}

const AGENT: &str = "gband.on('WindowOpened', function(ev)
  gband.window_state(ev.session, ev.window).agent = 'waiting'
  gband.emit('hello', { window = ev.window })
end)";

#[tokio::test(flavor = "multi_thread")]
async fn kept_across_reload() {
    let scripted = start("lua-kept", AGENT).await;
    tokio::time::sleep(SETTLE).await;
    let mut client = scripted.attach().await;
    let first = client.first();
    assert_eq!(client.states[&first]["agent"], text("waiting"));
    scripted.reload("gband.on('ConfigReloaded', function() gband.emit('reloaded') end)");
    event(&mut client, "reloaded").await;
    let later = scripted.attach().await;
    assert_eq!(later.states[&first]["agent"], text("waiting"));
}

#[tokio::test(flavor = "multi_thread")]
async fn every_load_counts_failed_ones_included() {
    let scripted = start("lua-load-number", "").await;
    assert_eq!(
        scripted.server_answer(Operation::Errors).await,
        Answer::Errors {
            load: 1,
            errors: Vec::new(),
        }
    );
    scripted.reload("local = 1");
    let Answer::Errors { load: 2, errors } = scripted.server_answer(Operation::Errors).await else {
        panic!("the failed load was not counted");
    };
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].contains("server.lua:1:"), "{errors:?}");
    write(&server_file(&scripted.locations), "");
    assert_eq!(
        scripted.server_answer(Operation::Reload).await,
        Answer::Loaded {
            load: 3,
            error: None,
        }
    );
    assert_eq!(
        scripted.server_answer(Operation::Errors).await,
        Answer::Errors {
            load: 3,
            errors: Vec::new(),
        }
    );
    write(&server_file(&scripted.locations), "error('bad')");
    let Answer::Loaded {
        load: 4,
        error: Some(error),
    } = scripted.server_answer(Operation::Reload).await
    else {
        panic!("the failed forced load was not counted");
    };
    assert!(error.contains("bad"), "{error}");
}

#[tokio::test(flavor = "multi_thread")]
async fn server_errors_listed_since_the_last_good_load() {
    let scripted = start(
        "lua-error-list",
        "gband.on('WindowOpened', function() error('boom') end)",
    )
    .await;
    let mut client = scripted.attach().await;
    let first = client.first();
    let second = client.open_after(first).await;
    client.open_after(second).await;
    client
        .wait_until(|client| {
            client
                .bridge
                .iter()
                .filter(|message| matches!(message, ServerMessage::ServerError(_)))
                .count()
                >= 2
        })
        .await;
    let Answer::Errors { load: 1, errors } = scripted.server_answer(Operation::Errors).await else {
        panic!("wrong load number");
    };
    assert!(errors.len() >= 2, "{errors:?}");
    assert!(
        errors.iter().all(|error| error.contains("boom")),
        "{errors:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn eval_and_command_in_the_server() {
    let scripted = start(
        "lua-control-eval",
        "gband.cmd.register('double', function(args) return args.n * 2 end)",
    )
    .await;
    assert_eq!(
        scripted
            .server_answer(Operation::Eval {
                source: "return gband.side, ...".to_owned(),
                args: vec![text("a")],
            })
            .await,
        Answer::Values(Ok(vec![text("server"), text("a")]))
    );
    assert_eq!(
        scripted
            .server_answer(Operation::Command {
                name: "double".to_owned(),
                args: Value::Table(vec![(DataKey::string("n"), Value::Int(21))]),
            })
            .await,
        Answer::Values(Ok(vec![Value::Int(42)]))
    );
    let Answer::Values(Err(reason)) = scripted
        .server_answer(Operation::Command {
            name: "absent".to_owned(),
            args: Value::Nil,
        })
        .await
    else {
        panic!("an unknown command answered");
    };
    assert!(reason.contains("absent"), "{reason}");
}

#[tokio::test(flavor = "multi_thread")]
async fn attach_order_with_window_names() {
    let scripted = start("lua-attach-names", AGENT).await;
    let mut client = scripted.attach().await;
    let first = client.first();
    client.open_after(first).await;
    tokio::time::sleep(SETTLE).await;
    let (mut peer, _) = TestClient::accepted(&scripted.server.socket(), 80, 24).await;
    peer.send(&ClientMessage::Attach {
        session: SessionName::default(),
        cwd: "/".into(),
    })
    .await;
    let mut received = Vec::new();
    loop {
        let message: ServerMessage = peer.recv().await.unwrap();
        received.push(match message {
            ServerMessage::Requirements(_) => break,
            ServerMessage::Layout { .. } => "layout".to_owned(),
            ServerMessage::Snapshot { window, .. } => format!("snapshot {window}"),
            ServerMessage::WindowName { window, .. } => format!("name {window}"),
            ServerMessage::WindowState { window, .. } => format!("state {window}"),
            other => format!("{other:?}"),
        });
    }
    assert_eq!(
        received,
        [
            "layout",
            "snapshot 1",
            "snapshot 2",
            "name 1",
            "name 2",
            "state 1",
            "state 2"
        ]
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn attach_order_with_plugin_state() {
    let runtime_dir = runtime_dir("lua-attach-order");
    plugin(&runtime_dir, "agent-status", ", client = '>= 0.1'", AGENT);
    let scripted = start_in(runtime_dir, "").await;
    tokio::time::sleep(SETTLE).await;
    let mut client = scripted.attach().await;
    let first = client.first();
    event(&mut client, "hello").await;
    assert_eq!(
        client.bridge,
        [
            ServerMessage::WindowState {
                window: first,
                key: "agent".to_owned(),
                value: Some(text("waiting")),
            },
            ServerMessage::Requirements(vec![Requirement {
                plugin: "agent-status".to_owned(),
                requirement: ">= 0.1".to_owned(),
            }]),
            ServerMessage::Event {
                name: "hello".to_owned(),
                data: Value::Table(vec![(DataKey::string("window"), Value::Int(1))]),
                queued: true,
                time: match &client.bridge[2] {
                    ServerMessage::Event { time, .. } => *time,
                    other => panic!("{other:?}"),
                },
            },
        ]
    );
    let mut second = scripted.attach().await;
    second.pump(Duration::from_millis(300)).await;
    assert!(events(&second, "hello").is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn handler_error_shown() {
    let runtime_dir = runtime_dir("lua-handler-error");
    let file = plugin(
        &runtime_dir,
        "broken",
        "",
        "gband.on('WindowOpened', function()\n  local a = 1\n  local b = 2\n  error('boom')\nend)",
    );
    let scripted = start_in(runtime_dir, "").await;
    let mut client = scripted.attach().await;
    let first = client.first();
    client.open_after(first).await;
    let expected = format!("broken: {}:4: boom", file.display());
    client
        .wait_until(|client| errors(client).contains(&expected))
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn agent_prompt_detected() {
    let scripted = start(
        "lua-prompt",
        "local buffers = {}
gband.on('WindowOutput', function(ev)
  local buffer = (buffers[ev.window] or '') .. ev.data
  buffers[ev.window] = buffer
  if buffer:find('\\nto proceed?', 1, true) then
    gband.emit('found', { buffer = buffer })
    buffers[ev.window] = ''
  end
end)",
    )
    .await;
    let mut client = scripted.attach().await;
    client.wait_for_prompt(client.first()).await;
    client
        .type_line("printf 'Do you want\\nto proceed?\\n'")
        .await;
    let found = event(&mut client, "found").await;
    let Some(Value::Bytes(buffer)) = field(&found, "buffer") else {
        panic!("{found:?}");
    };
    let buffer = String::from_utf8_lossy(buffer);
    assert!(buffer.contains("Do you want\r\nto proceed?"), "{buffer:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn input_notice() {
    let scripted = start(
        "lua-input",
        "gband.on('WindowInput', function(ev)
  local keys = {}
  for key in pairs(ev) do keys[#keys + 1] = key end
  table.sort(keys)
  gband.emit('input', { window = ev.window, client = ev.client, keys = table.concat(keys, ',') })
end)",
    )
    .await;
    let mut client = scripted.attach().await;
    let first = client.first();
    client.key(Key::plain(KeyCode::Char('l'))).await;
    client.key(Key::plain(KeyCode::Char('s'))).await;
    client
        .wait_until(|client| events(client, "input").len() == 2)
        .await;
    for (data, _) in events(&client, "input") {
        assert_eq!(
            field(&data, "window"),
            Some(&Value::Int(i64::from(first.0)))
        );
        assert!(matches!(field(&data, "client"), Some(Value::Int(_))));
        assert_eq!(field(&data, "keys"), Some(&text("client,session,window")));
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn close_from_a_handler() {
    let scripted = start(
        "lua-close",
        "gband.on('WindowOutput', function(ev)
  if ev.data:find('bye', 1, true) then
    gband.action.close_window({ session = ev.session, window = ev.window })
  end
end)",
    )
    .await;
    let mut client = scripted.attach().await;
    let first = client.first();
    let second = client.open_after(first).await;
    client.type_line_to(second, "echo by''e").await;
    client
        .wait_until(|client| !client.layout.contains(second))
        .await;
    assert!(client.layout.contains(first));
}

#[tokio::test(flavor = "multi_thread")]
async fn open_without_stealing_focus() {
    let scripted = start(
        "lua-open",
        "gband.cmd.register('open', function(args, ctx)
  gband.action.open_window({ session = ctx.session, band = 1, after = args.after, program = 'sleep 100' })
  return true
end)",
    )
    .await;
    let mut client = scripted.attach().await;
    let first = client.first();
    let band = client.layout.bands()[0].id.0;
    assert_eq!(band, 1);
    let args = Value::Table(vec![(
        DataKey::string("after"),
        Value::Int(i64::from(first.0)),
    )]);
    assert_eq!(
        call(&mut client, 1, "open", args).await,
        Ok(Value::Bool(true))
    );
    client
        .wait_until(|client| client.windows().len() == 2)
        .await;
    client.pump(Duration::from_millis(300)).await;
    assert!(client.focus.is_empty(), "{:?}", client.focus);
}

#[tokio::test(flavor = "multi_thread")]
async fn action_from_the_servers_lua() {
    let scripted = start(
        "lua-action",
        "gband.on('WindowOpened', function(ev)
  if ev.window > 1 then
    gband.action.grow_column_width({ session = ev.session, window = ev.window })
  end
end)",
    )
    .await;
    let mut client = scripted.attach().await;
    let first = client.first();
    let second = client.open_after(first).await;
    client
        .wait_until(|client| {
            client.layout.bands()[0]
                .columns
                .iter()
                .find(|column| column.windows.contains(&second))
                .is_some_and(|column| {
                    u64::from(column.width.num) * u64::from(Proportion::ONE_HALF.den)
                        > u64::from(Proportion::ONE_HALF.num) * u64::from(column.width.den)
                })
        })
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn float_and_tile_from_the_servers_lua() {
    let scripted = start(
        "lua-floating",
        "gband.on('WindowOpened', function(ev)
  if ev.window == 2 then
    gband.action.toggle_window_floating({ session = ev.session, window = ev.window })
    gband.action.open_window({ session = ev.session, band = ev.band, floating = true })
  end
end)",
    )
    .await;
    let mut client = scripted.attach().await;
    let first = client.first();
    let second = client.open_after(first).await;
    client
        .wait_until(|client| client.layout.bands()[0].floating.len() == 2)
        .await;
    let floating: Vec<_> = client.layout.bands()[0]
        .floating
        .iter()
        .map(|floating| floating.window)
        .collect();
    assert_eq!(floating[0], second);
    assert_eq!(client.layout.bands()[0].columns.len(), 1);
}

async fn first_grown(client: &mut TestClient, first: WindowId) {
    client
        .wait_until(|client| {
            client.layout.bands()[0]
                .columns
                .iter()
                .find(|column| column.windows.contains(&first))
                .is_some_and(|column| column.width != Proportion::ONE_HALF)
        })
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn two_handlers_float_one_window() {
    let scripted = start(
        "lua-float-twice",
        "for _ = 1, 2 do
  gband.on('WindowOpened', function(ev)
    if ev.window == 2 then
      gband.action.toggle_window_floating({ session = ev.session, window = ev.window, floating = true })
    end
  end)
end
gband.on('WindowOpened', function(ev)
  if ev.window == 2 then
    gband.action.grow_column_width({ session = ev.session, window = 1 })
  end
end)",
    )
    .await;
    let mut client = scripted.attach().await;
    let first = client.first();
    let second = client.open_after(first).await;
    first_grown(&mut client, first).await;
    let floating: Vec<_> = client.layout.bands()[0]
        .floating
        .iter()
        .map(|floating| floating.window)
        .collect();
    assert_eq!(floating, [second]);
    assert!(client.layout.locate(second).is_none());
}

#[tokio::test(flavor = "multi_thread")]
async fn tile_a_tiled_window_from_the_server() {
    let scripted = start(
        "lua-tile-tiled",
        "gband.on('WindowOpened', function(ev)
  if ev.window == 2 then
    gband.action.toggle_window_floating({ session = ev.session, window = ev.window, floating = false })
    gband.action.grow_column_width({ session = ev.session, window = 1 })
  end
end)",
    )
    .await;
    let mut client = scripted.attach().await;
    let first = client.first();
    let second = client.open_after(first).await;
    first_grown(&mut client, first).await;
    let band = &client.layout.bands()[0];
    assert!(band.floating.is_empty());
    let columns: Vec<_> = band
        .columns
        .iter()
        .map(|column| column.windows.clone())
        .collect();
    assert_eq!(columns, [vec![first], vec![second]]);
}

const COMMANDS: &str = "gband.cmd.register('focus', function(args, ctx)
  ctx.focus(args.window)
  return args.window
end)
gband.cmd.register('boom', function() error('boom') end)
gband.cmd.register('broadcast', function(args, ctx)
  gband.emit('agent.waiting', { window = 1 })
  gband.emit('mine', {}, { session = ctx.session })
  return true
end)";

#[tokio::test(flavor = "multi_thread")]
async fn command_focuses_for_its_caller() {
    let scripted = start("lua-focus", COMMANDS).await;
    let mut caller = scripted.attach().await;
    let mut other = scripted.attach().await;
    let first = caller.first();
    let second = caller.open_after(first).await;
    other.wait_until(|client| client.windows().len() == 2).await;
    let focused_before = caller.focus.len();
    let args = Value::Table(vec![(
        DataKey::string("window"),
        Value::Int(i64::from(first.0)),
    )]);
    assert_eq!(
        call(&mut caller, 7, "focus", args).await,
        Ok(Value::Int(i64::from(first.0)))
    );
    caller
        .wait_until(|client| client.focus.len() > focused_before)
        .await;
    assert_eq!(caller.focus.last(), Some(&first));
    other.pump(Duration::from_millis(300)).await;
    assert!(other.focus.is_empty(), "{:?}", other.focus);
    assert_ne!(first, second);
}

#[tokio::test(flavor = "multi_thread")]
async fn unknown_and_failing_commands() {
    let scripted = start("lua-failing", COMMANDS).await;
    let mut client = scripted.attach().await;
    let unknown = call(&mut client, 1, "absent", Value::Nil).await;
    assert!(unknown.unwrap_err().contains("absent"));
    let failed = call(&mut client, 2, "boom", Value::Nil).await;
    assert!(failed.unwrap_err().contains("boom"));
    client
        .wait_until(|client| errors(client).iter().any(|error| error.contains("boom")))
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn commands_without_a_server_runtime() {
    let server = TestServer::start("lua-none", &["/bin/sh"]).await;
    let mut client = server.attach(80, 24).await;
    let result = call(&mut client, 1, "anything", Value::Nil).await;
    assert!(result.unwrap_err().contains("anything"));
    assert_eq!(client.bridge[0], ServerMessage::Requirements(Vec::new()));
}

#[tokio::test(flavor = "multi_thread")]
async fn broadcast_and_one_session() {
    let scripted = start("lua-broadcast", COMMANDS).await;
    let mut work = scripted
        .server
        .attach_to("work", Path::new("/"), 80, 24)
        .await;
    let mut play = scripted
        .server
        .attach_to("play", Path::new("/"), 80, 24)
        .await;
    assert_eq!(
        call(&mut work, 1, "broadcast", Value::Nil).await,
        Ok(Value::Bool(true))
    );
    let data = event(&mut play, "agent.waiting").await;
    assert_eq!(field(&data, "window"), Some(&Value::Int(1)));
    event(&mut work, "agent.waiting").await;
    event(&mut work, "mine").await;
    play.pump(Duration::from_millis(300)).await;
    assert!(events(&play, "mine").is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn sessions_and_layout() {
    let scripted = start(
        "lua-structure",
        "gband.cmd.register('describe', function(args, ctx)
  local session = gband.session(ctx.session)
  return {
    names = gband.sessions(),
    windows = #session.bands[1].columns,
    clients = #session.clients,
    missing = gband.window_state(ctx.session, 99) == nil,
  }
end)",
    )
    .await;
    let mut client = scripted.attach().await;
    let described = call(&mut client, 1, "describe", Value::Nil).await.unwrap();
    assert_eq!(
        field(&described, "names").map(list),
        Some(vec![text("default")])
    );
    assert_eq!(field(&described, "windows"), Some(&Value::Int(1)));
    assert_eq!(field(&described, "clients"), Some(&Value::Int(1)));
    assert_eq!(field(&described, "missing"), Some(&Value::Bool(true)));
}
