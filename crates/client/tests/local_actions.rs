use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use gband_client::animation::Animations;
use gband_client::{Controls, Display, Step};
use gband_core::geometry::Size;
use gband_core::layout::{Layout, LayoutOptions};
use gband_lua::keys::parse_key;
use gband_lua::{LoadOptions, Locations, Side};
use gband_protocol::ServerMessage;

const CHILD: &str = "GBAND_LOCAL_ACTIONS_CHILD";
const RECORD: &str = "GBAND_OPENED";

struct Client {
    root: PathBuf,
    display: Display,
    controls: Controls,
}

impl Client {
    fn new(name: &str, source: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("gband-client-local-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let config = root.join("config");
        let file = gband_lua::user_file(&config, Side::Client);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, source).unwrap();
        let locations = Locations {
            config,
            plugins: None,
        };
        let config = gband_lua::load(&locations, Side::Client, &LoadOptions::default()).unwrap();
        let mut display = Display::new(Size::new(80, 24), Animations::Off);
        let mut controls = Controls::new(config, &mut display);
        let mut layout = Layout::new();
        let pane = layout.allocate_pane();
        layout.open(
            pane,
            layout.bands()[0].id,
            None,
            None,
            &LayoutOptions::default(),
        );
        controls.receive(
            &mut display,
            [
                ServerMessage::Layout {
                    cols: 80,
                    rows: 24,
                    layout,
                },
                ServerMessage::Requirements(Vec::new()),
            ],
        );
        Self {
            root,
            display,
            controls,
        }
    }

    fn press(&mut self, name: &str) -> Vec<Step> {
        self.controls
            .press(&mut self.display, parse_key(name).unwrap())
    }

    fn written(&mut self, name: &str) -> Vec<Vec<u8>> {
        self.press(name)
            .into_iter()
            .filter_map(|step| match step {
                Step::Write(bytes) => Some(bytes),
                _ => None,
            })
            .collect()
    }

    fn global<T: mlua::FromLua>(&self, name: &str) -> T {
        self.controls.runtime().lua().globals().get(name).unwrap()
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn bound(call: &str) -> String {
    format!("gband.bind('alt+x', function() {call} end)")
}

#[test]
fn default_style() {
    let mut client = Client::new(
        "default-style",
        &bound("gband.notify('Agent waiting: build')"),
    );
    assert_eq!(
        client.written("alt+x"),
        [b"\x1b]9;Agent waiting: build\x07".to_vec()]
    );
}

#[test]
fn title_with_osc777() {
    let mut client = Client::new(
        "osc777",
        &format!(
            "gband.opt.notify_style = 'osc777'\n{}",
            bound("gband.notify('done', { title = 'agent' }) gband.notify('plain')")
        ),
    );
    assert_eq!(
        client.written("alt+x"),
        [
            b"\x1b]777;notify;agent;done\x07".to_vec(),
            b"\x1b]777;notify;gband;plain\x07".to_vec()
        ]
    );
}

#[test]
fn bell_and_none_styles() {
    let mut client = Client::new(
        "bell-style",
        &format!(
            "gband.opt.notify_style = 'bell'\n{}",
            bound("gband.notify('x')")
        ),
    );
    assert_eq!(client.written("alt+x"), [b"\x07".to_vec()]);
    let mut client = Client::new(
        "none-style",
        &format!(
            "gband.opt.notify_style = 'none'\n{}",
            bound("gband.notify('x')")
        ),
    );
    assert!(client.written("alt+x").is_empty());
}

#[test]
fn escape_removed() {
    let mut client = Client::new("escape", &bound("gband.notify('a\\27]52;c;eA==\\7b')"));
    assert_eq!(
        client.written("alt+x"),
        [b"\x1b]9;a]52;c;eA==b\x07".to_vec()]
    );
}

#[test]
fn notify_argument_errors() {
    for call in [
        "gband.notify(3)",
        "gband.notify('x', 'title')",
        "gband.notify('x', { title = 3 })",
        "gband.clipboard(3)",
        "gband.clipboard(string.rep('x', 1024 * 1024 + 1))",
        "gband.open('')",
    ] {
        let mut client = Client::new("argument-errors", &bound(call));
        assert!(client.written("alt+x").is_empty(), "{call}");
        assert!(client.display.banner().is_some(), "{call}");
    }
}

#[test]
fn outside_a_callback() {
    for call in [
        "gband.notify('x')",
        "gband.bell()",
        "gband.clipboard('x')",
        "gband.open('x')",
    ] {
        let root =
            std::env::temp_dir().join(format!("gband-client-local-outside-{}", std::process::id()));
        let file = gband_lua::user_file(&root, Side::Client);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, call).unwrap();
        let locations = Locations {
            config: root.clone(),
            plugins: None,
        };
        let error = gband_lua::load(&locations, Side::Client, &LoadOptions::default())
            .err()
            .unwrap_or_else(|| panic!("{call} loaded"));
        assert!(error.message.contains("callback"), "{call}: {error}");
        let _ = fs::remove_dir_all(&root);
    }
}

#[test]
fn ring() {
    let mut client = Client::new("ring", &bound("gband.bell()"));
    assert_eq!(client.written("alt+x"), [b"\x07".to_vec()]);
}

#[test]
fn copy() {
    let mut client = Client::new("copy", &bound("gband.clipboard('hi')"));
    assert_eq!(client.written("alt+x"), [b"\x1b]52;c;aGk=\x07".to_vec()]);
}

fn in_child(test: &str, path: &Path, record: &Path) -> bool {
    if std::env::var_os(CHILD).is_some() {
        return true;
    }
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--nocapture"])
        .env(CHILD, "1")
        .env("PATH", path)
        .env(RECORD, record)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{stdout}");
    assert!(stdout.contains("1 passed"), "{stdout}");
    false
}

fn scratch(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("gband-client-opener-{name}"));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

#[derive(Clone, Default)]
struct Captured(Arc<Mutex<Vec<u8>>>);

impl Write for Captured {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

const OPENER: &str = if cfg!(target_os = "macos") {
    "open"
} else {
    "xdg-open"
};

#[test]
fn open_a_url() {
    let bin = scratch(&format!("bin-{}", std::process::id()));
    let record = bin.join("record");
    if !in_child("open_a_url", &bin, &record) {
        let _ = fs::remove_dir_all(&bin);
        return;
    }
    let record = PathBuf::from(std::env::var_os(RECORD).unwrap());
    let opener = record.with_file_name(OPENER);
    fs::write(
        &opener,
        "#!/bin/sh\nprintf '%s|%s' \"$#\" \"$1\" > \"$GBAND_OPENED\"\n",
    )
    .unwrap();
    fs::set_permissions(&opener, fs::Permissions::from_mode(0o755)).unwrap();
    let mut client = Client::new(
        "open-url",
        &bound("opened = gband.open('https://example.com/a b')"),
    );
    client.press("alt+x");
    assert!(client.global::<bool>("opened"));
    let expected = "1|https://example.com/a b";
    let deadline = Instant::now() + Duration::from_secs(10);
    while fs::read_to_string(&record).ok().as_deref() != Some(expected) {
        assert!(Instant::now() < deadline, "the opener never ran");
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn opener_missing() {
    let empty = scratch(&format!("empty-{}", std::process::id()));
    if !in_child("opener_missing", &empty, &empty.join("record")) {
        let _ = fs::remove_dir_all(&empty);
        return;
    }
    let captured = Captured::default();
    let writer = captured.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || writer.clone())
        .with_ansi(false)
        .finish();
    let mut client = Client::new("opener-missing", &bound("opened = gband.open('x')"));
    tracing::subscriber::with_default(subscriber, || client.press("alt+x"));
    assert!(!client.global::<bool>("opened"));
    assert!(client.display.banner().is_none());
    let log = String::from_utf8(captured.0.lock().unwrap().clone()).unwrap();
    assert!(log.contains(OPENER), "{log}");
}
