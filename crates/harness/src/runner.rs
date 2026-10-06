use std::ffi::OsString;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use gband_core::geometry::Size;
use gband_lua::keys::parse_key;
use gband_lua::plain;
use gband_protocol::Value as Data;
use mlua::{Function, HookTriggers, Lua, MultiValue, Table, Value, VmState};

use crate::case::{Case, DEFAULT_TIME, Setup, parse_size, parse_time, plugin_name};
use crate::screenshot::{self, Colour, Mismatch, References, Style, reference_path};

const MODULE: &str = include_str!("test.lua");
const FILE_TIMEOUT: f64 = 30.0;
const CASE_TIMEOUT: f64 = 30.0;
const WAIT_TIMEOUT: f64 = 5.0;
const CALL_TIMEOUT: Duration = Duration::from_secs(10);
const STEP: u32 = 10_000;

pub struct Options {
    pub executable: PathBuf,
    pub files: Vec<PathBuf>,
    pub update: bool,
    pub show: bool,
    pub filter: Option<String>,
    pub plugins: Vec<PathBuf>,
}

pub const STDIN: &str = "-";

struct TestFile {
    name: String,
    path: PathBuf,
    source: Vec<u8>,
    standard_input: bool,
}

fn out(text: &str) {
    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(text.as_bytes());
    let _ = stdout.flush();
}

fn discover(cwd: &Path) -> Vec<PathBuf> {
    fn walk(directory: &Path, found: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(directory) else {
            return;
        };
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, found);
            } else if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with("_spec.lua"))
            {
                found.push(path);
            }
        }
    }
    let mut found = Vec::new();
    walk(&cwd.join("tests"), &mut found);
    let mut found: Vec<PathBuf> = found
        .into_iter()
        .map(|path| {
            path.strip_prefix(cwd)
                .map_or(path.clone(), Path::to_path_buf)
        })
        .collect();
    found.sort_by(|a, b| {
        a.as_os_str()
            .as_encoded_bytes()
            .cmp(b.as_os_str().as_encoded_bytes())
    });
    found
}

fn files(options: &Options, cwd: &Path) -> Result<Vec<TestFile>, String> {
    let standard_input = options.files.iter().any(|file| file.as_os_str() == STDIN);
    if standard_input && options.files.len() > 1 {
        return Err(
            "`-` reads one test file from standard input and takes no other file".to_owned(),
        );
    }
    if standard_input {
        let mut source = Vec::new();
        std::io::stdin()
            .read_to_end(&mut source)
            .map_err(|error| format!("cannot read standard input: {error}"))?;
        return Ok(vec![TestFile {
            name: "stdin".to_owned(),
            path: cwd.join("stdin"),
            source,
            standard_input: true,
        }]);
    }
    let paths = if options.files.is_empty() {
        let found = discover(cwd);
        if found.is_empty() {
            return Err(format!(
                "no test file found: no file under {} ends in _spec.lua",
                cwd.join("tests").display()
            ));
        }
        found
    } else {
        options.files.clone()
    };
    paths
        .into_iter()
        .map(|path| {
            let source = fs::read(&path).map_err(|error| {
                format!("cannot read the test file {}: {error}", path.display())
            })?;
            Ok(TestFile {
                name: path.display().to_string(),
                path: if path.is_absolute() {
                    path
                } else {
                    cwd.join(path)
                },
                source,
                standard_input: false,
            })
        })
        .collect()
}

fn plugins(options: &Options, cwd: &Path) -> Result<Vec<PathBuf>, String> {
    let mut plugins = Vec::new();
    if cwd.join("plugin.lua").is_file() {
        plugins.push(cwd.to_path_buf());
    }
    for plugin in &options.plugins {
        if !plugin.is_dir() {
            return Err(format!(
                "the plugin {} is not a directory",
                plugin.display()
            ));
        }
        plugins.push(plugin.clone());
    }
    let mut names = Vec::new();
    for plugin in &plugins {
        let name = plugin_name(plugin)
            .ok_or_else(|| format!("the plugin path {} has no name", plugin.display()))?;
        if names.contains(&name) {
            return Err(format!("two plugins are named `{name}`"));
        }
        names.push(name);
    }
    Ok(plugins)
}

pub fn run(options: Options) -> u8 {
    let prepared = std::env::current_dir()
        .map_err(|error| format!("cannot read the current directory: {error}"))
        .and_then(|cwd| Ok((files(&options, &cwd)?, plugins(&options, &cwd)?)));
    let (files, plugins) = match prepared {
        Ok(prepared) => prepared,
        Err(message) => {
            eprintln!("gband test: {message}");
            return 2;
        }
    };
    let run = Arc::new(Run {
        executable: options.executable,
        update: options.update,
        show: options.show,
        plugins,
        references: Mutex::new(References::default()),
    });
    let mut totals = Totals::default();
    for file in files {
        run_file(&run, &file, options.filter.as_deref(), &mut totals);
    }
    let mut line = format!("{} passed, {} failed", totals.passed, totals.failed);
    if totals.files_failed > 0 {
        line.push_str(&format!(", {} files failed", totals.files_failed));
    }
    out(&format!("\n{line}\n"));
    u8::from(totals.failed > 0 || totals.files_failed > 0)
}

#[derive(Default)]
struct Totals {
    passed: usize,
    failed: usize,
    files_failed: usize,
}

struct Run {
    executable: PathBuf,
    update: bool,
    show: bool,
    plugins: Vec<PathBuf>,
    references: Mutex<References>,
}

struct Registered {
    name: String,
    timeout: f64,
    function: Function,
}

#[derive(Default)]
struct Deadline(Option<(Instant, String)>);

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn message(error: &mlua::Error) -> String {
    match error {
        mlua::Error::CallbackError { cause, .. } | mlua::Error::WithContext { cause, .. } => {
            message(cause)
        }
        mlua::Error::RuntimeError(text) | mlua::Error::SyntaxError { message: text, .. } => text
            .split_once("\nstack traceback:")
            .map_or(text.as_str(), |(text, _)| text)
            .to_owned(),
        other => other.to_string(),
    }
}

const WRAP: &str = "gband.test.wrap";

fn fail(text: impl Into<String>) -> mlua::Error {
    mlua::Error::runtime(text.into())
}

#[derive(Debug)]
struct Raised(String);

impl std::fmt::Display for Raised {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Raised {}

fn raised(error: &mlua::Error) -> mlua::Error {
    mlua::Error::external(Raised(message(error)))
}

fn passed_through(error: &mlua::Error) -> Option<String> {
    match error {
        mlua::Error::CallbackError { cause, .. } | mlua::Error::WithContext { cause, .. } => {
            passed_through(cause)
        }
        mlua::Error::ExternalError(external) => external
            .downcast_ref::<Raised>()
            .map(|raised| raised.0.clone()),
        _ => None,
    }
}

fn protect<A: mlua::FromLuaMulti + 'static>(
    lua: &Lua,
    call: impl Fn(&Lua, A) -> mlua::Result<MultiValue> + Send + 'static,
) -> mlua::Result<Function> {
    let raw = lua.create_function(move |lua, args: A| match call(lua, args) {
        Ok(mut values) => {
            values.push_front(Value::Boolean(true));
            Ok(values)
        }
        Err(error) => {
            let (text, as_is) = match passed_through(&error) {
                Some(text) => (text, true),
                None => (message(&error), false),
            };
            Ok(MultiValue::from_iter([
                Value::Boolean(false),
                text.into_lua(lua)?,
                Value::Boolean(as_is),
            ]))
        }
    })?;
    lua.named_registry_value::<Function>(WRAP)?.call(raw)
}

fn seconds(seconds: f64) -> String {
    if seconds == 1.0 {
        "1 second".to_owned()
    } else if seconds.fract() == 0.0 {
        format!("{seconds:.0} seconds")
    } else {
        format!("{seconds} seconds")
    }
}

fn new_state(
    file: &TestFile,
    deadline: &Arc<Mutex<Deadline>>,
) -> mlua::Result<(Lua, Arc<Mutex<Vec<Registered>>>)> {
    let lua = Lua::new();
    gband_lua::install_test(&lua, |line| out(&format!("{line}\n")))?;
    let directory = file
        .path
        .parent()
        .unwrap_or(Path::new("."))
        .to_string_lossy()
        .into_owned();
    let package: Table = lua.globals().get("package")?;
    let path: String = package.get("path")?;
    package.set(
        "path",
        format!("{directory}/?.lua;{directory}/?/init.lua;{path}"),
    )?;
    let registered = Arc::new(Mutex::new(Vec::new()));
    let host = lua.create_table()?;
    let cases = Arc::clone(&registered);
    host.set(
        "register",
        lua.create_function(
            move |_, (name, timeout, function): (String, Option<f64>, Function)| {
                lock(&cases).push(Registered {
                    name,
                    timeout: timeout.unwrap_or(CASE_TIMEOUT),
                    function,
                });
                Ok(())
            },
        )?,
    )?;
    let module = lua.load(MODULE).set_name("=gband.test").into_function()?;
    let loaded: Value = module.call(&host)?;
    lua.set_named_registry_value(WRAP, host.get::<Function>("wrap")?)?;
    let preload: Table = package.get("preload")?;
    let loaded = lua.create_function(move |_, ()| Ok(loaded.clone()))?;
    preload.set("gband.test", loaded)?;
    let limit = Arc::clone(deadline);
    lua.set_global_hook(
        HookTriggers::new().every_nth_instruction(STEP),
        move |_, _| match &lock(&limit).0 {
            Some((deadline, text)) if Instant::now() >= *deadline => {
                Err(mlua::Error::runtime(text.clone()))
            }
            _ => Ok(VmState::Continue),
        },
    )?;
    Ok((lua, registered))
}

fn run_file(run: &Arc<Run>, file: &TestFile, filter: Option<&str>, totals: &mut Totals) {
    let deadline = Arc::new(Mutex::new(Deadline::default()));
    let file_failed = |text: String, totals: &mut Totals| {
        totals.files_failed += 1;
        out(&format!("FAIL {}\n{}\n", file.name, indent(&text)));
    };
    let (lua, registered) = match new_state(file, &deadline) {
        Ok(state) => state,
        Err(error) => return file_failed(message(&error), totals),
    };
    lock(&deadline).0 = Some((
        Instant::now() + Duration::from_secs_f64(FILE_TIMEOUT),
        format!(
            "the test file ran longer than its time limit of {}",
            seconds(FILE_TIMEOUT)
        ),
    ));
    let ran = lua
        .load(file.source.as_slice())
        .set_name(format!("@{}", file.name))
        .exec();
    lock(&deadline).0 = None;
    if let Err(error) = ran {
        return file_failed(message(&error), totals);
    }
    let cases = std::mem::take(&mut *lock(&registered));
    for case in cases {
        if filter.is_some_and(|filter| !case.name.contains(filter)) {
            continue;
        }
        let passed = run_case(run, file, &lua, case, &deadline);
        if passed {
            totals.passed += 1;
        } else {
            totals.failed += 1;
        }
    }
}

fn indent(text: &str) -> String {
    text.lines()
        .map(|line| format!("  {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

struct Slot {
    case: Option<Case>,
    started: bool,
    detail: Option<String>,
    logs: Option<(Vec<String>, Vec<String>)>,
}

struct Context {
    run: Arc<Run>,
    file: String,
    directory: PathBuf,
    path: PathBuf,
    standard_input: bool,
    name: String,
    deadline: Instant,
    timeout: f64,
    slot: Mutex<Slot>,
}

impl Context {
    fn slot(&self) -> MutexGuard<'_, Slot> {
        lock(&self.slot)
    }

    fn until(&self, timeout: Duration) -> Instant {
        (Instant::now() + timeout).min(self.deadline)
    }

    fn overdue(&self) -> Option<String> {
        (Instant::now() >= self.deadline).then(|| self.limit())
    }

    fn limit(&self) -> String {
        format!(
            "the case ran longer than its time limit of {}",
            seconds(self.timeout)
        )
    }

    fn show(&self, name: Option<&str>, shot: &str) {
        let mut title = format!("screenshot {} > {}", self.file, self.name);
        if let Some(name) = name {
            title.push_str(" > ");
            title.push_str(name);
        }
        out(&format!("{title}\n{shot}"));
    }
}

fn run_case(
    run: &Arc<Run>,
    file: &TestFile,
    lua: &Lua,
    case: Registered,
    deadline: &Arc<Mutex<Deadline>>,
) -> bool {
    let started = Instant::now();
    let context = Arc::new(Context {
        run: Arc::clone(run),
        file: file.name.clone(),
        directory: file.path.parent().unwrap_or(Path::new(".")).to_path_buf(),
        path: file.path.clone(),
        standard_input: file.standard_input,
        name: case.name.clone(),
        deadline: started + Duration::from_secs_f64(case.timeout),
        timeout: case.timeout,
        slot: Mutex::new(Slot {
            case: None,
            started: false,
            detail: None,
            logs: None,
        }),
    });
    lock(deadline).0 = Some((context.deadline, context.limit()));
    let result = handle(lua, &context).and_then(|g| case.function.call::<()>(g));
    lock(deadline).0 = None;
    let mut slot = context.slot();
    if result.is_err()
        && let Some(running) = &slot.case
    {
        slot.logs = Some((
            noteworthy(running.log("client")),
            noteworthy(running.log("server")),
        ));
    }
    if let Some(running) = slot.case.take() {
        running.finish();
    }
    let elapsed = started.elapsed().as_secs_f64();
    match result {
        Ok(()) => {
            out(&format!(
                "PASS {} > {} ({elapsed:.2}s)\n",
                file.name, case.name
            ));
            true
        }
        Err(error) => {
            let mut report = message(&error);
            if let Some(detail) = slot.detail.take() {
                report.push('\n');
                report.push_str(detail.trim_end());
            }
            if let Some((client, server)) = slot.logs.take() {
                for (side, lines) in [("client", client), ("server", server)] {
                    if !lines.is_empty() {
                        report.push_str(&format!("\n{side} log:\n{}", indent(&lines.join("\n"))));
                    }
                }
            }
            out(&format!(
                "FAIL {} > {} ({elapsed:.2}s)\n{}\n",
                file.name,
                case.name,
                indent(&report)
            ));
            false
        }
    }
}

fn noteworthy(lines: Vec<String>) -> Vec<String> {
    lines
        .into_iter()
        .filter(|line| {
            line.contains(" gband_lua::runtime: ") || line.contains("configuration error")
        })
        .collect()
}

type Call<A> = fn(&Lua, &Context, A) -> mlua::Result<MultiValue>;

fn bind<A: mlua::FromLuaMulti + 'static>(
    lua: &Lua,
    g: &Table,
    name: &'static str,
    context: &Arc<Context>,
    call: Call<A>,
) -> mlua::Result<()> {
    let context = Arc::clone(context);
    let function = protect(lua, move |lua, args: A| {
        if name != "start" && context.slot().case.is_none() {
            return Err(fail(format!("call g.start before g.{name}")));
        }
        if let Some(limit) = context.overdue() {
            return Err(fail(limit));
        }
        call(lua, &context, args)
    })?;
    g.set(name, function)
}

fn handle(lua: &Lua, context: &Arc<Context>) -> mlua::Result<Table> {
    let g = lua.create_table()?;
    bind(lua, &g, "start", context, start)?;
    bind(lua, &g, "keys", context, keys)?;
    bind(lua, &g, "type", context, type_text)?;
    bind(lua, &g, "paste", context, paste)?;
    bind(lua, &g, "run", context, run_line)?;
    bind(lua, &g, "resize", context, resize)?;
    bind(lua, &g, "write", context, write)?;
    bind(lua, &g, "reload", context, reload)?;
    bind(lua, &g, "settle", context, settle)?;
    bind(lua, &g, "wait", context, wait)?;
    bind(lua, &g, "wait_text", context, wait_text)?;
    bind(lua, &g, "client", context, client)?;
    bind(lua, &g, "server", context, server)?;
    bind(lua, &g, "set_time", context, set_time)?;
    bind(lua, &g, "screen", context, screen)?;
    bind(lua, &g, "notifications", context, notifications)?;
    bind(lua, &g, "clipboard", context, clipboard)?;
    bind(lua, &g, "opened", context, opened)?;
    bind(lua, &g, "bells", context, bells)?;
    bind(lua, &g, "log", context, log)?;
    bind(lua, &g, "screenshot", context, take_screenshot)?;
    bind(lua, &g, "expect_screenshot", context, expect_screenshot)?;
    Ok(g)
}

fn none() -> mlua::Result<MultiValue> {
    Ok(MultiValue::new())
}

fn one(lua: &Lua, value: impl mlua::IntoLua) -> mlua::Result<MultiValue> {
    Ok(MultiValue::from_iter([value.into_lua(lua)?]))
}

fn with_case<T>(
    context: &Context,
    act: impl FnOnce(&mut Case) -> Result<T, String>,
) -> mlua::Result<T> {
    let mut slot = context.slot();
    let case = slot.case.as_mut().expect("checked before every call");
    act(case).map_err(fail)
}

fn text_arg(value: &Value, what: &str) -> mlua::Result<String> {
    match value {
        Value::String(text) => Ok(text.to_str()?.to_owned()),
        other => Err(fail(format!(
            "{what} must be a string, found {}",
            other.type_name()
        ))),
    }
}

fn options_arg(value: Value, what: &str) -> mlua::Result<Option<Table>> {
    match value {
        Value::Nil => Ok(None),
        Value::Table(table) => Ok(Some(table)),
        other => Err(fail(format!(
            "{what} must be a table, found {}",
            other.type_name()
        ))),
    }
}

fn timeout_arg(opts: Option<&Table>, default: f64, what: &str) -> mlua::Result<f64> {
    let Some(opts) = opts else {
        return Ok(default);
    };
    match opts.get::<Value>("timeout")? {
        Value::Nil => Ok(default),
        Value::Integer(seconds) if seconds > 0 => Ok(seconds as f64),
        Value::Number(seconds) if seconds > 0.0 => Ok(seconds),
        _ => Err(fail(format!(
            "the timeout of {what} must be a positive number of seconds"
        ))),
    }
}

fn start(_: &Lua, context: &Context, opts: Value) -> mlua::Result<MultiValue> {
    if context.slot().started {
        return Err(fail("g.start was already called in this case"));
    }
    let opts = options_arg(opts, "the options of g.start")?;
    let field = |name: &str| -> mlua::Result<Value> {
        opts.as_ref()
            .map_or(Ok(Value::Nil), |opts| opts.get::<Value>(name))
    };
    let optional_text = |name: &str| -> mlua::Result<Option<String>> {
        match field(name)? {
            Value::Nil => Ok(None),
            value => text_arg(&value, &format!("`{name}` of g.start")).map(Some),
        }
    };
    let size = match optional_text("size")? {
        None => Size::new(80, 24),
        Some(text) => parse_size(&text).ok_or_else(|| {
            fail(format!(
                "`size` of g.start must be \"<cols>x<rows>\", found {text:?}"
            ))
        })?,
    };
    let mut files = Vec::new();
    match field("files")? {
        Value::Nil => {}
        Value::Table(table) => {
            for pair in table.pairs::<Value, Value>() {
                let (path, contents) = pair?;
                files.push((
                    text_arg(&path, "a path in `files` of g.start")?,
                    text_arg(&contents, "the contents of a file in `files` of g.start")?,
                ));
            }
        }
        other => {
            return Err(fail(format!(
                "`files` of g.start must be a table, found {}",
                other.type_name()
            )));
        }
    }
    let mut plugins = context.run.plugins.clone();
    match field("plugins")? {
        Value::Nil => {}
        Value::Table(table) => {
            for path in table.sequence_values::<Value>() {
                let path = PathBuf::from(text_arg(&path?, "a path in `plugins` of g.start")?);
                let path = if path.is_absolute() {
                    path
                } else {
                    context.directory.join(path)
                };
                plugins.push(path);
            }
        }
        other => {
            return Err(fail(format!(
                "`plugins` of g.start must be a list, found {}",
                other.type_name()
            )));
        }
    }
    let mut env = Vec::new();
    match field("env")? {
        Value::Nil => {}
        Value::Table(table) => {
            for pair in table.pairs::<Value, Value>() {
                let (name, value) = pair?;
                let name = OsString::from(text_arg(&name, "a name in `env` of g.start")?);
                let value = match value {
                    Value::Boolean(false) => None,
                    value => Some(OsString::from(text_arg(
                        &value,
                        "a value in `env` of g.start",
                    )?)),
                };
                env.push((name, value));
            }
        }
        other => {
            return Err(fail(format!(
                "`env` of g.start must be a table, found {}",
                other.type_name()
            )));
        }
    }
    let time = time_arg(field("time")?, "`time` of g.start", Some(DEFAULT_TIME))?;
    let keystyle = match field("keystyle")? {
        Value::Nil => Some("modal".to_owned()),
        Value::Boolean(false) => None,
        Value::String(style) if matches!(style.as_bytes().as_ref(), b"modal" | b"direct") => {
            Some(style.to_str()?.to_owned())
        }
        Value::String(style) => {
            return Err(fail(format!(
                "`keystyle` of g.start must be \"modal\", \"direct\" or false, found {:?}",
                style.to_string_lossy()
            )));
        }
        other => {
            return Err(fail(format!(
                "`keystyle` of g.start must be \"modal\", \"direct\" or false, found {}",
                other.type_name()
            )));
        }
    };
    context.slot().started = true;
    let setup = Setup {
        executable: context.run.executable.clone(),
        size,
        config: optional_text("config")?,
        server_config: optional_text("server_config")?,
        keystyle,
        files,
        plugins,
        env,
        time,
    };
    let case = Case::start(setup).map_err(fail)?;
    context.slot().case = Some(case);
    none()
}

fn time_arg(value: Value, what: &str, default: Option<i64>) -> mlua::Result<Option<i64>> {
    match value {
        Value::Nil => Ok(default),
        Value::Boolean(false) => Ok(None),
        Value::Integer(seconds) => Ok(Some(seconds)),
        Value::Number(seconds) if seconds.fract() == 0.0 => Ok(Some(seconds as i64)),
        Value::String(text) => {
            let text = text.to_str()?.to_owned();
            parse_time(&text).map(Some).ok_or_else(|| {
                fail(format!(
                    "{what} must be \"YYYY-MM-DD HH:MM:SS\", found {text:?}"
                ))
            })
        }
        other => Err(fail(format!(
            "{what} must be Unix seconds, \"YYYY-MM-DD HH:MM:SS\" or false, found {}",
            other.type_name()
        ))),
    }
}

fn keys(_: &Lua, context: &Context, keys: Value) -> mlua::Result<MultiValue> {
    let keys = text_arg(&keys, "the keys of g.keys")?;
    let parsed = keys
        .split_whitespace()
        .map(|name| parse_key(name).map_err(|_| fail(format!("unknown key `{name}` in g.keys"))))
        .collect::<mlua::Result<Vec<_>>>()?;
    with_case(context, |case| case.keys(&parsed))?;
    none()
}

fn type_text(_: &Lua, context: &Context, text: Value) -> mlua::Result<MultiValue> {
    let text = text_arg(&text, "the text of g.type")?;
    with_case(context, |case| case.type_text(&text))?;
    none()
}

fn paste(_: &Lua, context: &Context, text: Value) -> mlua::Result<MultiValue> {
    let text = text_arg(&text, "the text of g.paste")?;
    with_case(context, |case| case.paste(&text))?;
    none()
}

fn run_line(_: &Lua, context: &Context, line: Value) -> mlua::Result<MultiValue> {
    let line = text_arg(&line, "the line of g.run")?;
    with_case(context, |case| case.type_text(&format!("{line}\r")))?;
    none()
}

fn resize(_: &Lua, context: &Context, size: Value) -> mlua::Result<MultiValue> {
    let text = text_arg(&size, "the size of g.resize")?;
    let size = parse_size(&text).ok_or_else(|| {
        fail(format!(
            "the size of g.resize must be \"<cols>x<rows>\", found {text:?}"
        ))
    })?;
    with_case(context, |case| case.resize(size))?;
    none()
}

fn write(_: &Lua, context: &Context, (path, contents): (Value, Value)) -> mlua::Result<MultiValue> {
    let path = text_arg(&path, "the path of g.write")?;
    let contents = text_arg(&contents, "the contents of g.write")?;
    with_case(context, |case| case.write_file(&path, &contents))?;
    none()
}

fn reload(lua: &Lua, context: &Context, (): ()) -> mlua::Result<MultiValue> {
    let deadline = context.until(CALL_TIMEOUT);
    let error = with_case(context, |case| case.reload(deadline))?;
    one(lua, error)
}

fn settle(_: &Lua, context: &Context, (): ()) -> mlua::Result<MultiValue> {
    with_case(context, Case::settle)?;
    none()
}

fn screen_table(lua: &Lua, screen: vt100::Screen) -> mlua::Result<Table> {
    let screen = Arc::new(screen);
    let (rows, cols) = screen.size();
    let (row, col) = screen.cursor_position();
    let table = lua.create_table()?;
    table.set("cols", cols)?;
    table.set("rows", rows)?;
    let cursor = lua.create_table()?;
    cursor.set("row", row)?;
    cursor.set("col", col)?;
    cursor.set("visible", !screen.hide_cursor())?;
    table.set("cursor", cursor)?;
    let shown = Arc::clone(&screen);
    table.set(
        "row",
        protect(lua, move |lua, row: Value| {
            let row = match row {
                Value::Integer(row) if (0..i64::from(rows)).contains(&row) => row as u16,
                other => {
                    return Err(fail(format!(
                        "row must be an integer from 0 to {}, found {other:?}",
                        rows - 1
                    )));
                }
            };
            one(lua, screenshot::row_text(&shown, row))
        })?,
    )?;
    let shown = Arc::clone(&screen);
    table.set(
        "text",
        protect(lua, move |lua, ()| one(lua, screen_text(&shown)))?,
    )?;
    let shown = Arc::clone(&screen);
    table.set(
        "cell",
        protect(lua, move |lua, (row, col): (Value, Value)| {
            let coordinate = |value: Value, limit: u16, what: &str| match value {
                Value::Integer(n) if (0..i64::from(limit)).contains(&n) => Ok(n as u16),
                other => Err(fail(format!(
                    "{what} must be an integer from 0 to {}, found {other:?}",
                    limit - 1
                ))),
            };
            let (row, col) = (coordinate(row, rows, "row")?, coordinate(col, cols, "col")?);
            let cell = lua.create_table()?;
            let Some(source) = shown.cell(row, col) else {
                return one(lua, cell);
            };
            let style = Style::of(source);
            cell.set(
                "char",
                if source.has_contents() {
                    source.contents().to_owned()
                } else {
                    " ".to_owned()
                },
            )?;
            let colour = |colour: Option<Colour>| -> mlua::Result<Value> {
                match colour {
                    None => Ok(Value::Nil),
                    Some(Colour::Palette(index)) => Ok(Value::Integer(i64::from(index))),
                    Some(rgb) => rgb.to_string().into_lua(lua),
                }
            };
            cell.set("fg", colour(style.fg)?)?;
            cell.set("bg", colour(style.bg)?)?;
            cell.set("bold", style.bold)?;
            cell.set("dim", style.dim)?;
            cell.set("italic", style.italic)?;
            cell.set("underline", style.underline)?;
            cell.set("inverse", style.inverse)?;
            one(lua, cell)
        })?,
    )?;
    Ok(table)
}

use mlua::IntoLua as _;

fn screen_text(screen: &vt100::Screen) -> String {
    let (rows, _) = screen.size();
    (0..rows)
        .map(|row| screenshot::row_text(screen, row))
        .collect::<Vec<_>>()
        .join("\n")
}

fn current_screen(context: &Context) -> (vt100::Screen, String) {
    let slot = context.slot();
    let case = slot.case.as_ref().expect("checked before every call");
    (case.screen(), case.screenshot(true))
}

fn wait(
    lua: &Lua,
    context: &Context,
    (predicate, opts): (Value, Value),
) -> mlua::Result<MultiValue> {
    let Value::Function(predicate) = predicate else {
        return Err(fail("g.wait expects a function"));
    };
    let opts = options_arg(opts, "the options of g.wait")?;
    let timeout = timeout_arg(opts.as_ref(), WAIT_TIMEOUT, "g.wait")?;
    let deadline = context.until(Duration::from_secs_f64(timeout));
    loop {
        let (screen, shot) = current_screen(context);
        let value: Value = predicate
            .call(screen_table(lua, screen)?)
            .map_err(|error| raised(&error))?;
        if !matches!(value, Value::Nil | Value::Boolean(false)) {
            return one(lua, value);
        }
        if Instant::now() >= deadline {
            return Err(gave_up(
                context,
                format!("g.wait gave up after {}", seconds(timeout)),
                &shot,
            ));
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn gave_up(context: &Context, text: String, shot: &str) -> mlua::Error {
    if let Some(limit) = context.overdue() {
        return mlua::Error::runtime(limit);
    }
    fail(format!("{text}\n{shot}"))
}

fn wait_text(_: &Lua, context: &Context, (text, opts): (Value, Value)) -> mlua::Result<MultiValue> {
    let text = text_arg(&text, "the text of g.wait_text")?;
    let opts = options_arg(opts, "the options of g.wait_text")?;
    let timeout = timeout_arg(opts.as_ref(), WAIT_TIMEOUT, "g.wait_text")?;
    let deadline = context.until(Duration::from_secs_f64(timeout));
    loop {
        let (screen, shot) = current_screen(context);
        if screen_text(&screen).contains(&text) {
            return none();
        }
        if Instant::now() >= deadline {
            return Err(gave_up(
                context,
                format!(
                    "g.wait_text gave up waiting for {text:?} after {}",
                    seconds(timeout)
                ),
                &shot,
            ));
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn evaluate(
    lua: &Lua,
    context: &Context,
    server: bool,
    args: MultiValue,
) -> mlua::Result<MultiValue> {
    let side = if server { "server" } else { "client" };
    let mut args = args.into_iter();
    let chunk = text_arg(
        &args.next().unwrap_or(Value::Nil),
        &format!("the chunk of g.{side}"),
    )?;
    let args = args
        .enumerate()
        .map(|(index, value)| {
            plain::from_lua(&value, &format!("argument {}", index + 2)).map_err(fail)
        })
        .collect::<mlua::Result<Vec<Data>>>()?;
    let deadline = context.until(CALL_TIMEOUT);
    let answer = with_case(context, |case| case.eval(server, &chunk, args, deadline))?;
    match answer {
        Ok(results) => results
            .iter()
            .map(|result| plain::into_lua(lua, result))
            .collect(),
        Err(reason) => Err(fail(format!("the {side}'s chunk failed: {reason}"))),
    }
}

fn client(lua: &Lua, context: &Context, args: MultiValue) -> mlua::Result<MultiValue> {
    evaluate(lua, context, false, args)
}

fn server(lua: &Lua, context: &Context, args: MultiValue) -> mlua::Result<MultiValue> {
    evaluate(lua, context, true, args)
}

fn set_time(_: &Lua, context: &Context, time: Value) -> mlua::Result<MultiValue> {
    if matches!(time, Value::Nil) {
        return Err(fail(
            "g.set_time expects Unix seconds, \"YYYY-MM-DD HH:MM:SS\" or false",
        ));
    }
    let time = time_arg(time, "the time of g.set_time", None)?;
    with_case(context, |case| case.set_time(time))?;
    none()
}

fn screen(lua: &Lua, context: &Context, (): ()) -> mlua::Result<MultiValue> {
    let (screen, _) = current_screen(context);
    one(lua, screen_table(lua, screen)?)
}

fn notifications(lua: &Lua, context: &Context, (): ()) -> mlua::Result<MultiValue> {
    let list = lua.create_table()?;
    for (title, body) in with_case(context, |case| Ok(case.notifications()))? {
        let entry = lua.create_table()?;
        entry.set("title", title)?;
        entry.set("body", body)?;
        list.push(entry)?;
    }
    one(lua, list)
}

fn clipboard(lua: &Lua, context: &Context, (): ()) -> mlua::Result<MultiValue> {
    let texts = with_case(context, |case| Ok(case.clipboard()))?;
    one(lua, lua.create_sequence_from(texts)?)
}

fn opened(lua: &Lua, context: &Context, (): ()) -> mlua::Result<MultiValue> {
    let opened = with_case(context, |case| Ok(case.opened()))?;
    one(lua, lua.create_sequence_from(opened)?)
}

fn bells(lua: &Lua, context: &Context, (): ()) -> mlua::Result<MultiValue> {
    let bells = with_case(context, |case| Ok(case.bells()))?;
    one(lua, bells)
}

fn log(lua: &Lua, context: &Context, side: Value) -> mlua::Result<MultiValue> {
    let side = text_arg(&side, "the side of g.log")?;
    if side != "client" && side != "server" {
        return Err(fail(format!(
            "g.log expects \"client\" or \"server\", found {side:?}"
        )));
    }
    let lines = with_case(context, |case| Ok(case.log(&side)))?;
    one(lua, lua.create_sequence_from(lines)?)
}

fn styles(opts: Option<&Table>, what: &str) -> mlua::Result<bool> {
    let Some(opts) = opts else {
        return Ok(true);
    };
    match opts.get::<Value>("styles")? {
        Value::Nil => Ok(true),
        Value::Boolean(styles) => Ok(styles),
        other => Err(fail(format!(
            "`styles` of {what} must be a boolean, found {}",
            other.type_name()
        ))),
    }
}

fn take_screenshot(lua: &Lua, context: &Context, opts: Value) -> mlua::Result<MultiValue> {
    let opts = options_arg(opts, "the options of g.screenshot")?;
    let styles = styles(opts.as_ref(), "g.screenshot")?;
    let shot = with_case(context, |case| Ok(case.screenshot(styles)))?;
    if context.run.show {
        context.show(None, &shot);
    }
    one(lua, shot)
}

fn expect_screenshot(
    _: &Lua,
    context: &Context,
    (name, opts): (Value, Value),
) -> mlua::Result<MultiValue> {
    let (name, opts) = match (name, opts) {
        (Value::Table(opts), Value::Nil) => (None, Some(opts)),
        (Value::Nil, opts) => (
            None,
            options_arg(opts, "the options of g.expect_screenshot")?,
        ),
        (name, opts) => (
            Some(text_arg(&name, "the name of g.expect_screenshot")?),
            options_arg(opts, "the options of g.expect_screenshot")?,
        ),
    };
    let styles = styles(opts.as_ref(), "g.expect_screenshot")?;
    let shot = with_case(context, |case| Ok(case.screenshot(styles)))?;
    if context.standard_input || context.run.show {
        context.show(name.as_deref(), &shot);
    }
    if context.standard_input {
        return none();
    }
    let reference = reference_path(&context.path, &context.name, name.as_deref());
    let compared = lock(&context.run.references).compare(&reference, &shot, context.run.update);
    match compared {
        Ok(()) => none(),
        Err(mismatch) => {
            let detail = match &mismatch {
                Mismatch::Missing(_) => Some(shot),
                Mismatch::Differs { diff, .. } => Some(diff.clone()),
                Mismatch::Duplicate(_) | Mismatch::Io(_) => None,
            };
            context.slot().detail = detail;
            Err(fail(mismatch.to_string()))
        }
    }
}
