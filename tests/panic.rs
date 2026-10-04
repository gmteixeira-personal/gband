use std::fs;
use std::path::Path;

use gband::logging::{self, Role};

#[test]
fn panic_reaches_the_log() {
    let state = Path::new(env!("CARGO_TARGET_TMPDIR")).join("panic");
    let _ = fs::remove_dir_all(&state);
    unsafe { std::env::set_var("XDG_STATE_HOME", &state) };
    let guard = logging::init(Role::Server).unwrap();

    let (handle, line) = (std::thread::spawn(|| panic!("boom")), line!());
    assert!(handle.join().is_err());
    drop(guard);

    let dir = state.join("gband").join("log");
    let text: String = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| fs::read_to_string(entry.unwrap().path()).unwrap())
        .collect();
    let location = format!("{}:{line}:", file!());
    assert!(
        text.lines().any(|entry| entry.contains(" ERROR ")
            && entry.contains("boom")
            && entry.contains(&location)),
        "{text}"
    );
}
