use std::fs;

pub fn command_name(argument: &str) -> Option<String> {
    let base = argument.rsplit('/').next().unwrap_or(argument);
    let base = base.strip_prefix('-').unwrap_or(base);
    (!base.is_empty()).then(|| base.to_owned())
}

pub fn foreground_command(group: Option<i32>, program: &str) -> String {
    group
        .filter(|&group| group > 0)
        .and_then(|group| from_cmdline(group).or_else(|| from_comm(group)))
        .or_else(|| command_name(program))
        .unwrap_or_default()
}

fn from_cmdline(pid: i32) -> Option<String> {
    let cmdline = fs::read(format!("/proc/{pid}/cmdline")).ok()?;
    let first = cmdline.split(|&byte| byte == 0).next()?;
    command_name(&String::from_utf8_lossy(first))
}

fn from_comm(pid: i32) -> Option<String> {
    let comm = fs::read_to_string(format!("/proc/{pid}/comm")).ok()?;
    command_name(comm.trim_end_matches('\n'))
}

#[cfg(test)]
mod tests {
    use std::os::unix::process::CommandExt;
    use std::process::{Child, Command};
    use std::time::{Duration, Instant};

    use super::*;

    struct Running(Child);

    impl Drop for Running {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    fn sleeping(arg0: &str) -> Running {
        let mut command = Command::new("sleep");
        command.arg0(arg0).arg("30");
        let running = Running(command.spawn().unwrap());
        let cmdline = format!("/proc/{}/cmdline", running.0.id());
        let deadline = Instant::now() + Duration::from_secs(5);
        while !fs::read(&cmdline).is_ok_and(|read| read.ends_with(b"\x0030\x00")) {
            assert!(Instant::now() < deadline, "sleep never started");
            std::thread::sleep(Duration::from_millis(1));
        }
        running
    }

    fn pid(running: &Running) -> Option<i32> {
        Some(running.0.id() as i32)
    }

    #[test]
    fn base_name_is_taken() {
        assert_eq!(command_name("/usr/bin/sleep").as_deref(), Some("sleep"));
        assert_eq!(command_name("vim").as_deref(), Some("vim"));
    }

    #[test]
    fn leading_dash_is_stripped() {
        assert_eq!(command_name("-zsh").as_deref(), Some("zsh"));
        assert_eq!(command_name("/bin/-bash").as_deref(), Some("bash"));
        assert_eq!(command_name("a-b").as_deref(), Some("a-b"));
    }

    #[test]
    fn empty_names_are_none() {
        assert_eq!(command_name(""), None);
        assert_eq!(command_name("-"), None);
        assert_eq!(command_name("dir/"), None);
    }

    #[test]
    fn command_read_from_cmdline() {
        let running = sleeping("/usr/bin/sleep");
        assert_eq!(foreground_command(pid(&running), "bash"), "sleep");
    }

    #[test]
    fn login_shell() {
        let running = sleeping("-zsh");
        assert_eq!(foreground_command(pid(&running), "sh"), "zsh");
    }

    #[test]
    fn empty_argument_falls_back_to_comm() {
        let running = sleeping("");
        assert_eq!(foreground_command(pid(&running), "bash"), "sleep");
    }

    #[test]
    fn unreadable_process_falls_back_to_the_program() {
        assert_eq!(foreground_command(Some(i32::MAX), "/bin/bash"), "bash");
        assert_eq!(foreground_command(None, "-sh"), "sh");
    }
}
