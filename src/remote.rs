use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::process::{Command as Process, ExitCode, Stdio};
use std::str::FromStr;

use anyhow::{Context, Result, bail};
use clap::Command;
use gband_core::geometry::Size;
use gband_protocol::SessionName;

pub const REMOTE: &str = "remote";
pub const REMOTE_GBAND: &str = "remote_gband";
const DEFAULT_GBAND: &str = "gband";
const SSH_FAILED: i32 = 255;

#[derive(Clone, Debug)]
pub struct Destination(String);

impl FromStr for Destination {
    type Err = String;

    fn from_str(destination: &str) -> Result<Self, String> {
        if destination.is_empty() {
            return Err("the destination is empty".to_owned());
        }
        Ok(Self(destination.to_owned()))
    }
}

impl fmt::Display for Destination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone)]
pub struct Host {
    pub destination: Destination,
    pub gband: Option<String>,
}

impl Host {
    pub fn gband(&self) -> &str {
        self.gband.as_deref().unwrap_or(DEFAULT_GBAND)
    }

    pub fn relay_command(
        &self,
        selection: &[OsString],
        session: &SessionName,
        size: Size,
    ) -> Vec<OsString> {
        let relay = [
            OsString::from("relay"),
            "-s".into(),
            session.as_str().into(),
            "--size".into(),
            format!("{}x{}", size.cols, size.rows).into(),
        ];
        self.command(selection.iter().cloned().chain(relay))
    }

    fn command(&self, arguments: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
        std::iter::once(OsString::from(self.gband()))
            .chain(arguments.into_iter().map(|argument| shell_quote(&argument)))
            .collect()
    }

    pub fn run(&self, arguments: &[OsString], stdin: bool) -> Result<ExitCode> {
        let command = self.command(arguments.iter().cloned());
        tracing::info!(
            "running {} through ssh",
            command.join(OsStr::new(" ")).display()
        );
        let status = Process::new("ssh")
            .arg("-T")
            .arg("--")
            .arg(&self.destination.0)
            .args(&command)
            .stdin(if stdin {
                Stdio::inherit()
            } else {
                Stdio::null()
            })
            .status()
            .context("cannot run ssh")?;
        tracing::info!("ssh finished: {status}");
        match status.code() {
            Some(SSH_FAILED) => bail!("cannot reach {} through ssh", self.destination),
            Some(code) => Ok(ExitCode::from(code as u8)),
            None => Ok(ExitCode::FAILURE),
        }
    }
}

pub fn shell_quote(word: &OsStr) -> OsString {
    let mut quoted = vec![b'\''];
    for &byte in word.as_bytes() {
        if byte == b'\'' {
            quoted.extend_from_slice(b"'\\''");
        } else {
            quoted.push(byte);
        }
    }
    quoted.push(b'\'');
    OsString::from_vec(quoted)
}

pub fn forwarded(arguments: &[OsString], command: &Command) -> Result<Vec<OsString>> {
    let options = Options::of(command);
    let mut walk = Walk::default();
    let kept: Vec<OsString> = arguments
        .iter()
        .skip(1)
        .filter(|argument| walk.keeps(argument, &options, command))
        .cloned()
        .collect();
    let program = arguments.iter().take(1).cloned();
    let matches = command
        .clone()
        .try_get_matches_from(program.chain(kept.iter().cloned()))
        .context("cannot leave the remote options out of the command line")?;
    if [REMOTE, REMOTE_GBAND]
        .into_iter()
        .any(|id| matches.value_source(id).is_some())
    {
        bail!("cannot leave the remote options out of the command line");
    }
    Ok(kept)
}

#[derive(Default)]
struct Options {
    valued_longs: HashSet<String>,
    valued_shorts: HashSet<char>,
    remote_longs: HashSet<String>,
    remote_shorts: HashSet<char>,
}

impl Options {
    fn of(command: &Command) -> Self {
        let mut options = Self::default();
        options.collect(command);
        options
    }

    fn collect(&mut self, command: &Command) {
        for argument in command.get_arguments() {
            if argument.is_positional() || !argument.get_action().takes_values() {
                continue;
            }
            let (longs, shorts) = if [REMOTE, REMOTE_GBAND].contains(&argument.get_id().as_str()) {
                (&mut self.remote_longs, &mut self.remote_shorts)
            } else {
                (&mut self.valued_longs, &mut self.valued_shorts)
            };
            longs.extend(argument.get_long().map(str::to_owned));
            shorts.extend(argument.get_short());
        }
        for subcommand in command.get_subcommands() {
            self.collect(subcommand);
        }
    }
}

#[derive(Default)]
struct Walk<'a> {
    value: Option<bool>,
    values_only: bool,
    subcommand: Option<&'a Command>,
    positionals: usize,
}

impl<'a> Walk<'a> {
    fn keeps(&mut self, argument: &OsStr, options: &Options, command: &'a Command) -> bool {
        if let Some(remote) = self.value.take() {
            return !remote;
        }
        if self.values_only {
            return true;
        }
        let token = argument.to_string_lossy();
        if token == "--" {
            self.values_only = true;
            return true;
        }
        if let Some(long) = token.strip_prefix("--") {
            let (name, attached) = match long.split_once('=') {
                Some((name, _)) => (name, true),
                None => (long, false),
            };
            let remote = options.remote_longs.contains(name);
            if !attached && (remote || options.valued_longs.contains(name)) {
                self.value = Some(remote);
            }
            return !remote;
        }
        if let Some(shorts) = token.strip_prefix('-').filter(|shorts| !shorts.is_empty()) {
            for (at, short) in shorts.char_indices() {
                let remote = options.remote_shorts.contains(&short);
                if remote || options.valued_shorts.contains(&short) {
                    if at + short.len_utf8() == shorts.len() {
                        self.value = Some(remote);
                    }
                    return !remote;
                }
            }
            return true;
        }
        match self.subcommand {
            None => self.subcommand = command.find_subcommand(argument),
            Some(subcommand) => {
                let filled = subcommand.get_positionals().nth(self.positionals);
                self.positionals += 1;
                self.values_only = filled.is_some_and(|positional| {
                    positional.is_trailing_var_arg_set() || subcommand.is_trailing_var_arg_set()
                });
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;
    use crate::Cli;

    fn quoted(word: &str) -> String {
        shell_quote(OsStr::new(word)).into_string().unwrap()
    }

    fn forwarded_from(line: &[&str]) -> Vec<String> {
        let arguments: Vec<OsString> = std::iter::once("gband")
            .chain(line.iter().copied())
            .map(OsString::from)
            .collect();
        forwarded(&arguments, &Cli::command())
            .unwrap()
            .into_iter()
            .map(|argument| argument.into_string().unwrap())
            .collect()
    }

    #[test]
    fn plain_word_is_quoted() {
        assert_eq!(quoted("list-sessions"), "'list-sessions'");
    }

    #[test]
    fn space_stays_inside_the_quotes() {
        assert_eq!(quoted("/tmp/my socket"), "'/tmp/my socket'");
    }

    #[test]
    fn single_quote_is_closed_escaped_and_reopened() {
        assert_eq!(quoted("it's"), r"'it'\''s'");
    }

    #[test]
    fn empty_string_is_two_quotes() {
        assert_eq!(quoted(""), "''");
    }

    #[test]
    fn separate_short_remote_is_dropped() {
        assert_eq!(
            forwarded_from(&["-r", "host", "-S", "feature", "list-sessions"]),
            ["-S", "feature", "list-sessions"]
        );
    }

    #[test]
    fn attached_short_remote_is_dropped() {
        assert_eq!(
            forwarded_from(&["-rhost", "list-sessions"]),
            ["list-sessions"]
        );
    }

    #[test]
    fn separate_long_remote_is_dropped() {
        assert_eq!(
            forwarded_from(&["-s", "work", "kill-session", "--remote", "host"]),
            ["-s", "work", "kill-session"]
        );
    }

    #[test]
    fn attached_long_remote_is_dropped() {
        assert_eq!(
            forwarded_from(&["--remote=host", "errors", "--json"]),
            ["errors", "--json"]
        );
    }

    #[test]
    fn separate_remote_gband_is_dropped() {
        assert_eq!(
            forwarded_from(&["-r", "host", "--remote-gband", "/opt/gband", "reload"]),
            ["reload"]
        );
    }

    #[test]
    fn attached_remote_gband_is_dropped() {
        assert_eq!(
            forwarded_from(&["reload", "--remote-gband=/opt/gband", "-rhost"]),
            ["reload"]
        );
    }

    #[test]
    fn option_after_the_chunk_source_is_dropped() {
        assert_eq!(
            forwarded_from(&["eval", "return ...", "-r", "host"]),
            ["eval", "return ..."]
        );
    }

    #[test]
    fn chunk_arguments_that_look_like_the_option_are_kept() {
        assert_eq!(
            forwarded_from(&["-r", "host", "eval", "return ...", "a", "-r", "x"]),
            ["eval", "return ...", "a", "-r", "x"]
        );
    }

    #[test]
    fn arguments_after_the_separator_are_kept() {
        assert_eq!(
            forwarded_from(&["-r", "host", "eval", "--", "-r"]),
            ["eval", "--", "-r"]
        );
    }

    #[test]
    fn relay_command_quotes_every_word_after_the_gband_command() {
        let host = Host {
            destination: "u@h".parse().unwrap(),
            gband: Some("~/.cargo/bin/gband".to_owned()),
        };
        let command = host.relay_command(
            &["-S".into(), "feature".into()],
            &"work".parse().unwrap(),
            Size::new(79, 24),
        );
        assert_eq!(
            command.join(OsStr::new(" ")),
            "~/.cargo/bin/gband '-S' 'feature' 'relay' '-s' 'work' '--size' '79x24'"
        );
    }
}
