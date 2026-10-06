use std::ffi::OsString;
use std::fs::{self, DirBuilder};
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{Context, Result, bail};
use gband_protocol::socket_path;

const SERVER_NAME_MAX: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerName(String);

impl ServerName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for ServerName {
    type Err = String;

    fn from_str(name: &str) -> Result<Self, String> {
        let valid = (1..=SERVER_NAME_MAX).contains(&name.len())
            && name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-');
        if !valid {
            return Err(format!(
                "a server name is 1 to {SERVER_NAME_MAX} ASCII letters, digits, `_` or `-`"
            ));
        }
        Ok(Self(name.to_owned()))
    }
}

pub fn resolve_socket(
    socket: Option<&Path>,
    server: Option<&ServerName>,
    gband: Option<OsString>,
    runtime_dir: &Path,
) -> Result<PathBuf> {
    if let Some(socket) = socket {
        return std::path::absolute(socket)
            .with_context(|| format!("cannot resolve socket path {}", socket.display()));
    }
    if let Some(server) = server {
        return Ok(runtime_dir.join(format!("{}.sock", server.as_str())));
    }
    match gband {
        Some(path) if !path.is_empty() => Ok(PathBuf::from(path)),
        _ => Ok(socket_path(runtime_dir)),
    }
}

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

    fn name(text: &str) -> ServerName {
        text.parse().unwrap()
    }

    #[test]
    fn valid_server_names_parse() {
        for text in ["feature", "a_b-1", &"x".repeat(64)] {
            assert_eq!(name(text).as_str(), text);
        }
    }

    #[test]
    fn invalid_server_names_are_refused() {
        for text in ["", &"x".repeat(65), "a.b", "a/b", ".."] {
            assert!(text.parse::<ServerName>().is_err(), "{text:?}");
        }
    }

    #[test]
    fn explicit_socket_wins_over_a_name() {
        let socket = resolve_socket(
            Some(Path::new("/tmp/s")),
            Some(&name("a")),
            Some("/run/window.sock".into()),
            Path::new("/run/gband"),
        )
        .unwrap();
        assert_eq!(socket, Path::new("/tmp/s"));
    }

    #[test]
    fn relative_socket_is_resolved_against_the_working_directory() {
        let socket =
            resolve_socket(Some(Path::new("target/s")), None, None, Path::new("/run")).unwrap();
        assert_eq!(socket, std::env::current_dir().unwrap().join("target/s"));
    }

    #[test]
    fn name_wins_over_the_window_socket() {
        let socket = resolve_socket(
            None,
            Some(&name("feature")),
            Some("/run/window.sock".into()),
            Path::new("/run/gband"),
        )
        .unwrap();
        assert_eq!(socket, Path::new("/run/gband/feature.sock"));
    }

    #[test]
    fn window_socket_wins_over_the_default() {
        let socket = resolve_socket(
            None,
            None,
            Some("/run/window.sock".into()),
            Path::new("/run/gband"),
        )
        .unwrap();
        assert_eq!(socket, Path::new("/run/window.sock"));
    }

    #[test]
    fn empty_window_socket_falls_back_to_the_default() {
        for gband in [None, Some(OsString::new())] {
            let socket = resolve_socket(None, None, gband, Path::new("/run/gband")).unwrap();
            assert_eq!(socket, Path::new("/run/gband/default.sock"));
        }
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
