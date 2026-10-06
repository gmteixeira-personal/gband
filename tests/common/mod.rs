#![allow(dead_code, unused_imports)]

use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

pub use gband_emulator::{Emulator, Grid};
pub use gband_harness::env::children;
pub use gband_harness::{
    Attached, TIMEOUT, Tile, focused_lines, is_running, tiles, wait_process_exit, wait_until,
};
use gband_scratch::Scratch;

pub const GBAND: &str = env!("CARGO_BIN_EXE_gband");

pub struct TestEnv {
    env: gband_harness::TestEnv,
    scratch: Scratch,
}

impl TestEnv {
    pub fn new(name: &str) -> Self {
        let env = Self::without_settings(name);
        env.save_key_style("modal");
        env.save_theme("terminal");
        env
    }

    pub fn without_settings(name: &str) -> Self {
        let scratch = Scratch::new("e2e", name);
        Self {
            env: gband_harness::TestEnv::new(scratch.to_path_buf(), GBAND),
            scratch,
        }
    }

    pub fn key_style_lua(&self) -> PathBuf {
        self.config_dir().join("user").join("keystyle.lua")
    }

    pub fn save_theme(&self, theme: &str) {
        let path = self.config_dir().join("user").join("theme.lua");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, format!("return \"{theme}\"\n")).unwrap();
    }

    pub fn save_key_style(&self, style: &str) {
        let path = self.key_style_lua();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, format!("return \"{style}\"\n")).unwrap();
    }
}

impl Deref for TestEnv {
    type Target = gband_harness::TestEnv;

    fn deref(&self) -> &gband_harness::TestEnv {
        &self.env
    }
}

impl DerefMut for TestEnv {
    fn deref_mut(&mut self) -> &mut gband_harness::TestEnv {
        &mut self.env
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

pub fn sidebar_cell(screen: &Grid, row: usize) -> String {
    screen
        .contents()
        .lines()
        .nth(row)
        .unwrap_or_default()
        .chars()
        .next()
        .filter(|cell| *cell != ' ')
        .map(String::from)
        .unwrap_or_default()
}

pub fn sidebar_mode(screen: &Grid) -> String {
    sidebar_cell(screen, 0)
}

pub fn sidebar_error(screen: &Grid) -> bool {
    let rows = usize::from(screen.size().rows);
    rows > 0 && sidebar_cell(screen, rows - 1) == "!"
}
