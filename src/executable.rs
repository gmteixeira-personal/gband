use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;

use gband_protocol::ExecutableId;

pub fn identity() -> io::Result<ExecutableId> {
    let metadata = fs::metadata("/proc/self/exe")
        .or_else(|_| std::env::current_exe().and_then(fs::metadata))?;
    Ok(ExecutableId {
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_matches_the_current_executable() {
        let metadata = fs::metadata(std::env::current_exe().unwrap()).unwrap();
        assert_eq!(
            identity().unwrap(),
            ExecutableId {
                device: metadata.dev(),
                inode: metadata.ino(),
            }
        );
    }
}
