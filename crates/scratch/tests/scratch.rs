use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::process::Command;

use gband_scratch::Scratch;

fn sibling(scratch: &Scratch, name: &str) -> PathBuf {
    let path = scratch.parent().unwrap().join(name);
    fs::create_dir_all(path.join("inner")).unwrap();
    path
}

fn dead_pid() -> u32 {
    let mut child = Command::new("true").spawn().unwrap();
    child.wait().unwrap();
    child.id()
}

#[test]
fn removes_on_drop() {
    let scratch = Scratch::new("scratch", "removes");
    let path = scratch.to_path_buf();
    fs::write(scratch.join("file"), "x").unwrap();
    assert!(path.is_dir());
    assert_eq!(
        path.file_name().unwrap().to_str().unwrap(),
        format!("gband-scratch-removes-{}", std::process::id())
    );
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o700
    );
    drop(scratch);
    assert!(!path.exists());
}

#[test]
fn keeps_on_panic() {
    let mut kept = PathBuf::new();
    let result = catch_unwind(AssertUnwindSafe(|| {
        let scratch = Scratch::new("scratch", "keeps");
        kept = scratch.to_path_buf();
        panic!("failing on purpose");
    }));
    assert!(result.is_err());
    assert!(kept.is_dir());
    fs::remove_dir_all(&kept).unwrap();
}

#[test]
fn sweeps_dead_siblings() {
    let scratch = Scratch::new("scratch", "sweep");
    let dead = sibling(&scratch, &format!("gband-scratch-sweep-{}", dead_pid()));
    drop(scratch);
    assert!(!dead.exists());
}

#[test]
fn spares_live_and_unrelated_siblings() {
    let mut live_process = Command::new("sleep").arg("30").spawn().unwrap();
    let scratch = Scratch::new("scratch", "spare");
    let live = sibling(
        &scratch,
        &format!("gband-scratch-spare-{}", live_process.id()),
    );
    let other = sibling(
        &scratch,
        &format!("gband-scratch-spare-other-{}", dead_pid()),
    );
    let named = sibling(&scratch, &format!("gband-scratch-spare-{}x", dead_pid()));
    drop(scratch);
    live_process.kill().unwrap();
    live_process.wait().unwrap();
    for path in [&live, &other, &named] {
        assert!(path.is_dir(), "{} was swept", path.display());
        fs::remove_dir_all(path).unwrap();
    }
}

fn lock(path: &Path, mode: u32) {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

#[test]
fn removes_read_only_trees() {
    let scratch = Scratch::new("scratch", "read-only");
    let path = scratch.to_path_buf();
    let outer = scratch.join("outer");
    let inner = outer.join("inner");
    fs::create_dir_all(&inner).unwrap();
    fs::write(inner.join("file"), "x").unwrap();
    fs::write(outer.join("file"), "x").unwrap();
    lock(&inner, 0o000);
    lock(&outer, 0o500);
    drop(scratch);
    assert!(!path.exists());
}
