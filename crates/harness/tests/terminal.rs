use std::ops::Deref;

use gband_harness::{Attached, TestEnv};
use gband_scratch::Scratch;

struct Env {
    env: TestEnv,
    _scratch: Scratch,
}

impl Deref for Env {
    type Target = TestEnv;

    fn deref(&self) -> &TestEnv {
        &self.env
    }
}

fn env(name: &str) -> Env {
    let scratch = Scratch::new("harness", name);
    Env {
        env: TestEnv::new(scratch.to_path_buf(), "/bin/sh"),
        _scratch: scratch,
    }
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
