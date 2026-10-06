use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::ValueEnum;

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Shell {
    Fish,
    Bash,
    Zsh,
}

impl From<Shell> for clap_complete::Shell {
    fn from(shell: Shell) -> Self {
        match shell {
            Shell::Fish => clap_complete::Shell::Fish,
            Shell::Bash => clap_complete::Shell::Bash,
            Shell::Zsh => clap_complete::Shell::Zsh,
        }
    }
}

pub fn generate(shell: Shell, command: &mut clap::Command, writer: &mut dyn Write) {
    clap_complete::generate(clap_complete::Shell::from(shell), command, "gband", writer);
    if shell == Shell::Fish {
        complete_fish_positionals(command, writer);
    }
}

// clap_complete's fish generator emits no completions for positional arguments.
fn complete_fish_positionals(command: &clap::Command, writer: &mut dyn Write) {
    for subcommand in command.get_subcommands() {
        for argument in subcommand.get_positionals() {
            let values: Vec<_> = argument
                .get_possible_values()
                .into_iter()
                .filter(|value| !value.is_hide_set())
                .map(|value| value.get_name().to_owned())
                .collect();
            if values.is_empty() {
                continue;
            }
            writeln!(
                writer,
                "complete -c gband -n \"__fish_gband_using_subcommand {}\" -f -a \"{}\"",
                subcommand.get_name(),
                values.join(" ")
            )
            .expect("failed to write completion file");
        }
    }
}

pub fn install_path(shell: Shell, env: impl Fn(&str) -> Option<OsString>) -> Result<PathBuf> {
    let absolute = |name: &str| {
        env(name)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
    };
    let base = |name: &str, default: &str| {
        absolute(name)
            .or_else(|| absolute("HOME").map(|home| home.join(default)))
            .ok_or_else(|| {
                anyhow!(
                    "cannot determine the install directory: neither {name} nor HOME is set to \
                     an absolute path"
                )
            })
    };
    Ok(match shell {
        Shell::Fish => base("XDG_CONFIG_HOME", ".config")?.join("fish/completions/gband.fish"),
        Shell::Bash => match absolute("BASH_COMPLETION_USER_DIR") {
            Some(directory) => directory.join("completions/gband"),
            None => {
                base("XDG_DATA_HOME", ".local/share")?.join("bash-completion/completions/gband")
            }
        },
        Shell::Zsh => base("XDG_DATA_HOME", ".local/share")?.join("zsh/site-functions/_gband"),
    })
}

pub fn install(shell: Shell, command: &mut clap::Command, path: &Path) -> Result<()> {
    let mut script = Vec::new();
    generate(shell, command, &mut script);
    let directory = path
        .parent()
        .with_context(|| format!("install path {} has no parent directory", path.display()))?;
    fs::create_dir_all(directory)
        .with_context(|| format!("cannot create directory {}", directory.display()))?;
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(format!(".{}.tmp", std::process::id()));
    let temporary = PathBuf::from(temporary);
    let written = fs::write(&temporary, &script)
        .and_then(|()| fs::rename(&temporary, path))
        .with_context(|| format!("cannot write {}", path.display()));
    if written.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    written
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::os::unix::fs::PermissionsExt;

    use gband_scratch::Scratch;

    use super::*;

    fn command() -> clap::Command {
        clap::Command::new("gband")
            .subcommand(clap::Command::new("completions"))
            .subcommand(
                clap::Command::new("install-completions")
                    .arg(clap::Arg::new("shell").value_parser(clap::value_parser!(Shell))),
            )
    }

    fn script(shell: Shell) -> Vec<u8> {
        let mut output = Vec::new();
        generate(shell, &mut command(), &mut output);
        output
    }

    fn path(shell: Shell, vars: &[(&str, &str)]) -> Result<PathBuf> {
        let vars: HashMap<String, OsString> = vars
            .iter()
            .map(|(name, value)| (name.to_string(), OsString::from(value)))
            .collect();
        install_path(shell, |name| vars.get(name).cloned())
    }

    #[test]
    fn every_shell_script_names_the_subcommands() {
        for shell in [Shell::Fish, Shell::Bash, Shell::Zsh] {
            let text = String::from_utf8(script(shell)).unwrap();
            assert!(text.contains("install-completions"), "{shell:?}: {text}");
        }
    }

    #[test]
    fn fish_script_completes_positional_values() {
        let text = String::from_utf8(script(Shell::Fish)).unwrap();
        assert!(
            text.contains(
                "complete -c gband -n \"__fish_gband_using_subcommand install-completions\" -f -a \
                 \"fish bash zsh\""
            ),
            "{text}"
        );
    }

    #[test]
    fn generation_is_deterministic() {
        for shell in [Shell::Fish, Shell::Bash, Shell::Zsh] {
            assert_eq!(script(shell), script(shell), "{shell:?}");
        }
    }

    #[test]
    fn fish_uses_xdg_config_home() {
        assert_eq!(
            path(Shell::Fish, &[("XDG_CONFIG_HOME", "/c"), ("HOME", "/h")]).unwrap(),
            Path::new("/c/fish/completions/gband.fish")
        );
    }

    #[test]
    fn fish_defaults_under_home() {
        assert_eq!(
            path(Shell::Fish, &[("HOME", "/h")]).unwrap(),
            Path::new("/h/.config/fish/completions/gband.fish")
        );
    }

    #[test]
    fn bash_user_dir_wins_over_xdg_data_home() {
        assert_eq!(
            path(
                Shell::Bash,
                &[("BASH_COMPLETION_USER_DIR", "/b"), ("XDG_DATA_HOME", "/d")]
            )
            .unwrap(),
            Path::new("/b/completions/gband")
        );
    }

    #[test]
    fn bash_uses_xdg_data_home() {
        assert_eq!(
            path(Shell::Bash, &[("XDG_DATA_HOME", "/d"), ("HOME", "/h")]).unwrap(),
            Path::new("/d/bash-completion/completions/gband")
        );
    }

    #[test]
    fn bash_defaults_under_home() {
        assert_eq!(
            path(Shell::Bash, &[("HOME", "/h")]).unwrap(),
            Path::new("/h/.local/share/bash-completion/completions/gband")
        );
    }

    #[test]
    fn zsh_uses_xdg_data_home() {
        assert_eq!(
            path(Shell::Zsh, &[("XDG_DATA_HOME", "/d"), ("HOME", "/h")]).unwrap(),
            Path::new("/d/zsh/site-functions/_gband")
        );
    }

    #[test]
    fn zsh_defaults_under_home() {
        assert_eq!(
            path(Shell::Zsh, &[("HOME", "/h")]).unwrap(),
            Path::new("/h/.local/share/zsh/site-functions/_gband")
        );
    }

    #[test]
    fn relative_and_empty_variables_are_ignored() {
        for value in ["relative", ""] {
            assert_eq!(
                path(Shell::Fish, &[("XDG_CONFIG_HOME", value), ("HOME", "/h")]).unwrap(),
                Path::new("/h/.config/fish/completions/gband.fish"),
                "{value:?}"
            );
        }
    }

    #[test]
    fn missing_home_is_an_error() {
        let message = format!("{:#}", path(Shell::Fish, &[]).unwrap_err());
        assert!(message.contains("XDG_CONFIG_HOME"), "{message}");
        assert!(message.contains("HOME"), "{message}");
    }

    #[test]
    fn install_creates_nested_directories() {
        let root = Scratch::new("completions", "nested");
        let target = root.join("a/b/gband.fish");
        install(Shell::Fish, &mut command(), &target).unwrap();
        assert_eq!(fs::read(&target).unwrap(), script(Shell::Fish));
    }

    #[test]
    fn install_replaces_an_existing_file() {
        let root = Scratch::new("completions", "replace");
        let target = root.join("gband");
        fs::write(&target, "other text").unwrap();
        install(Shell::Bash, &mut command(), &target).unwrap();
        assert_eq!(fs::read(&target).unwrap(), script(Shell::Bash));
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    }

    #[test]
    fn install_into_a_read_only_directory_leaves_nothing() {
        if rustix::process::geteuid().is_root() {
            return;
        }
        let root = Scratch::new("completions", "read-only");
        fs::set_permissions(&root, fs::Permissions::from_mode(0o500)).unwrap();
        let target = root.join("gband.fish");
        let message = format!(
            "{:#}",
            install(Shell::Fish, &mut command(), &target).unwrap_err()
        );
        assert!(message.contains(target.to_str().unwrap()), "{message}");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
    }
}
