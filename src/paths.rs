use std::ffi::OsString;
use std::fs::{self, DirBuilder};
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

pub fn runtime_dir() -> PathBuf {
    resolve(
        std::env::var_os("XDG_RUNTIME_DIR"),
        rustix::process::getuid().as_raw(),
    )
}

fn resolve(xdg_runtime_dir: Option<OsString>, uid: u32) -> PathBuf {
    match xdg_runtime_dir.map(PathBuf::from) {
        Some(base) if base.is_absolute() => base.join("gband"),
        _ => PathBuf::from(format!("/tmp/gband-{uid}")),
    }
}

pub fn prepare(directory: &Path) -> Result<()> {
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(directory)
        .with_context(|| format!("cannot create runtime directory {}", directory.display()))?;
    let metadata = fs::symlink_metadata(directory)
        .with_context(|| format!("cannot inspect runtime directory {}", directory.display()))?;
    if !metadata.is_dir() {
        bail!(
            "runtime directory {} is not a directory",
            directory.display()
        );
    }
    let uid = rustix::process::getuid().as_raw();
    if metadata.uid() != uid {
        bail!(
            "runtime directory {} is owned by uid {}, not by uid {uid}",
            directory.display(),
            metadata.uid()
        );
    }
    let mode = metadata.mode() & 0o777;
    if mode & 0o077 != 0 {
        bail!(
            "runtime directory {} has mode {mode:04o}, which grants access to group or others",
            directory.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("gband-paths-{}-{name}", std::process::id()));
        if path.exists() {
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
            fs::remove_dir_all(&path).unwrap();
        }
        path
    }

    #[test]
    fn absolute_xdg_runtime_dir_is_used() {
        assert_eq!(
            resolve(Some("/run/user/1000".into()), 1000),
            Path::new("/run/user/1000/gband")
        );
    }

    #[test]
    fn relative_xdg_runtime_dir_falls_back_to_tmp() {
        assert_eq!(
            resolve(Some("run/user".into()), 1000),
            Path::new("/tmp/gband-1000")
        );
    }

    #[test]
    fn unset_xdg_runtime_dir_falls_back_to_tmp() {
        assert_eq!(resolve(None, 1234), Path::new("/tmp/gband-1234"));
    }

    #[test]
    fn missing_directory_is_created_private() {
        let directory = scratch("missing").join("gband");
        prepare(&directory).unwrap();
        let mode = fs::metadata(&directory).unwrap().mode() & 0o777;
        assert_eq!(mode, 0o700);
        fs::remove_dir_all(directory.parent().unwrap()).unwrap();
    }

    #[test]
    fn directory_open_to_others_is_refused() {
        let directory = scratch("open");
        fs::create_dir_all(&directory).unwrap();
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o777)).unwrap();
        let message = format!("{:#}", prepare(&directory).unwrap_err());
        assert!(message.contains(directory.to_str().unwrap()), "{message}");
        assert!(message.contains("0777"), "{message}");
        fs::remove_dir_all(&directory).unwrap();
    }
}
