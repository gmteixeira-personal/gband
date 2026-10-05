use gband_harness::{Attached, TestEnv};

fn env(name: &str) -> TestEnv {
    let root = std::env::temp_dir().join(format!("gband-harness-{}-{name}", std::process::id()));
    TestEnv::new(root, "/bin/sh")
}

#[test]
fn terminal_queries_are_answered() {
    let env = env("queries");
    let script = "stty raw -echo; printf '\\033[c'; \
                  reply=$(dd bs=1 count=9 2>/dev/null); \
                  printf 'got:%s\\r\\n' \"$reply\" | cat -v; sleep 5";
    let attached = Attached::start_with(&env, "/bin/sh", &["-c", script], 40, 5, |_| {});
    attached.wait_for_text("got:^[[?62;22c");
}
