use std::fs;
use std::path::Path;

use gband::logging::{self, Role};

#[test]
fn keeps_at_most_seven_files_per_series() {
    let state = Path::new(env!("CARGO_TARGET_TMPDIR")).join("retention");
    let _ = fs::remove_dir_all(&state);
    let dir = state.join("gband").join("log");
    fs::create_dir_all(&dir).unwrap();
    for day in 1..=7 {
        fs::write(dir.join(format!("server.2000-01-{day:02}.log")), "old\n").unwrap();
    }
    let client = dir.join("client.2000-01-01.log");
    fs::write(&client, "client\n").unwrap();

    unsafe { std::env::set_var("XDG_STATE_HOME", &state) };
    let guard = logging::init(Role::Server).unwrap();
    drop(guard);

    let servers = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| name.starts_with("server."))
        .count();
    assert!(servers <= 7, "{servers} server files remain");
    assert_eq!(fs::read_to_string(&client).unwrap(), "client\n");
}
