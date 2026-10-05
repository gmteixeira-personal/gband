use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use gband_core::input::{Key, KeyCode};
use gband_core::layout::Proportion;
use gband_lua::{LoadOptions, Locations, Side};
use gband_protocol::{ClientMessage, Key as DataKey, Requirement, ServerMessage, Value};
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

async fn start_in(runtime_dir: PathBuf, source: &str) -> Scripted {
    let locations = locations(&runtime_dir);
    write(&server_file(&locations), source);
    let (loaded, error) = load(&locations);
    let (scripting, reloader) = Scripting::new(loaded, error);
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
gband.on('PaneExited', function(ev) log[#log + 1] = 'exited ' .. ev.pane .. ' ' .. tostring(ev.code) end)
gband.on('PaneClosed', function(ev)
  log[#log + 1] = 'closed ' .. ev.pane
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
        "gband.on('PaneOpened', function() gband.emit('one') end)",
    )
    .await;
    let mut client = scripted.attach().await;
    scripted.reload(
        "gband.on('PaneOpened', function() gband.emit('two') end)
gband.on('ConfigReloaded', function() gband.emit('reloaded') end)",
    );
    event(&mut client, "reloaded").await;
    client.bridge.clear();
    let first = client.first();
    client.open_after(first).await;
    event(&mut client, "two").await;
    assert!(events(&client, "one").is_empty());
}

const AGENT: &str = "gband.on('PaneOpened', function(ev)
  gband.pane_state(ev.session, ev.pane).agent = 'waiting'
  gband.emit('hello', { pane = ev.pane })
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
            ServerMessage::PaneState {
                pane: first,
                key: "agent".to_owned(),
                value: Some(text("waiting")),
            },
            ServerMessage::Requirements(vec![Requirement {
                plugin: "agent-status".to_owned(),
                requirement: ">= 0.1".to_owned(),
            }]),
            ServerMessage::Event {
                name: "hello".to_owned(),
                data: Value::Table(vec![(DataKey::string("pane"), Value::Int(1))]),
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
        "gband.on('PaneOpened', function()\n  local a = 1\n  local b = 2\n  error('boom')\nend)",
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
gband.on('PaneOutput', function(ev)
  local buffer = (buffers[ev.pane] or '') .. ev.data
  buffers[ev.pane] = buffer
  if buffer:find('\\nto proceed?', 1, true) then
    gband.emit('found', { buffer = buffer })
    buffers[ev.pane] = ''
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
        "gband.on('PaneInput', function(ev)
  local keys = {}
  for key in pairs(ev) do keys[#keys + 1] = key end
  table.sort(keys)
  gband.emit('input', { pane = ev.pane, client = ev.client, keys = table.concat(keys, ',') })
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
        assert_eq!(field(&data, "pane"), Some(&Value::Int(i64::from(first.0))));
        assert!(matches!(field(&data, "client"), Some(Value::Int(_))));
        assert_eq!(field(&data, "keys"), Some(&text("client,pane,session")));
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn close_from_a_handler() {
    let scripted = start(
        "lua-close",
        "gband.on('PaneOutput', function(ev)
  if ev.data:find('bye', 1, true) then
    gband.action.close_pane({ session = ev.session, pane = ev.pane })
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
  gband.action.open_pane({ session = ctx.session, band = 1, after = args.after, program = 'sleep 100' })
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
    client.wait_until(|client| client.panes().len() == 2).await;
    client.pump(Duration::from_millis(300)).await;
    assert!(client.focus.is_empty(), "{:?}", client.focus);
}

#[tokio::test(flavor = "multi_thread")]
async fn action_from_the_servers_lua() {
    let scripted = start(
        "lua-action",
        "gband.on('PaneOpened', function(ev)
  if ev.pane > 1 then
    gband.action.grow_column_width({ session = ev.session, pane = ev.pane })
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
                .find(|column| column.panes.contains(&second))
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
        "gband.on('PaneOpened', function(ev)
  if ev.pane == 2 then
    gband.action.toggle_pane_floating({ session = ev.session, pane = ev.pane })
    gband.action.open_pane({ session = ev.session, band = ev.band, floating = true })
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
        .map(|floating| floating.pane)
        .collect();
    assert_eq!(floating[0], second);
    assert_eq!(client.layout.bands()[0].columns.len(), 1);
}

const COMMANDS: &str = "gband.cmd.register('focus', function(args, ctx)
  ctx.focus(args.pane)
  return args.pane
end)
gband.cmd.register('boom', function() error('boom') end)
gband.cmd.register('broadcast', function(args, ctx)
  gband.emit('agent.waiting', { pane = 1 })
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
    other.wait_until(|client| client.panes().len() == 2).await;
    let focused_before = caller.focus.len();
    let args = Value::Table(vec![(
        DataKey::string("pane"),
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
    assert_eq!(field(&data, "pane"), Some(&Value::Int(1)));
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
    panes = #session.bands[1].columns,
    clients = #session.clients,
    missing = gband.pane_state(ctx.session, 99) == nil,
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
    assert_eq!(field(&described, "panes"), Some(&Value::Int(1)));
    assert_eq!(field(&described, "clients"), Some(&Value::Int(1)));
    assert_eq!(field(&described, "missing"), Some(&Value::Bool(true)));
}
