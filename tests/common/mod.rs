#![allow(dead_code, unused_imports)]

use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

pub use gband_emulator::{Emulator, Grid};
pub use gband_harness::env::children;
pub use gband_harness::{
    Attached, TIMEOUT, Tile, focused_lines, is_running, tiles, wait_process_exit, wait_until,
};

pub const GBAND: &str = env!("CARGO_BIN_EXE_gband");

const FNV_OFFSET: u32 = 0x811c_9dc5;
const FNV_PRIME: u32 = 0x0100_0193;

pub fn scratch_root(prefix: &str, name: &str) -> PathBuf {
    let worktree = env!("CARGO_MANIFEST_DIR")
        .bytes()
        .fold(FNV_OFFSET, |hash, byte| {
            (hash ^ u32::from(byte)).wrapping_mul(FNV_PRIME)
        });
    std::env::temp_dir().join(format!("{prefix}-{worktree:08x}-{name}"))
}

pub struct TestEnv(gband_harness::TestEnv);

impl TestEnv {
    pub fn new(name: &str) -> Self {
        Self(gband_harness::TestEnv::new(
            scratch_root("gband-e2e", name),
            GBAND,
        ))
    }
}

impl Deref for TestEnv {
    type Target = gband_harness::TestEnv;

    fn deref(&self) -> &gband_harness::TestEnv {
        &self.0
    }
}

impl DerefMut for TestEnv {
    fn deref_mut(&mut self) -> &mut gband_harness::TestEnv {
        &mut self.0
    }
}

pub fn session_of(pid: i32) -> i32 {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).unwrap();
    let fields: Vec<&str> = stat
        .rsplit(')')
        .next()
        .unwrap()
        .split_whitespace()
        .collect();
    fields[3].parse().unwrap()
}
