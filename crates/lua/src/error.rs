use std::fmt;
use std::path::PathBuf;

use mlua::Lua;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigError {
    pub plugin: Option<String>,
    pub location: Option<(PathBuf, u32)>,
    pub message: String,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(plugin) = &self.plugin {
            write!(f, "{plugin}: ")?;
        }
        match &self.location {
            Some((path, line)) => write!(f, "{}:{line}: {}", path.display(), self.message),
            None => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for ConfigError {}

impl ConfigError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            plugin: None,
            location: None,
            message: message.into(),
        }
    }

    pub fn at_caller(lua: &Lua, message: impl Into<String>) -> Self {
        Self {
            plugin: None,
            location: caller(lua),
            message: message.into(),
        }
    }

    pub fn raise(lua: &Lua, message: impl Into<String>) -> mlua::Error {
        mlua::Error::external(Self::at_caller(lua, message))
    }

    pub fn from_lua(error: &mlua::Error, sources: &[PathBuf]) -> Self {
        match error {
            mlua::Error::CallbackError { cause, .. } | mlua::Error::WithContext { cause, .. } => {
                Self::from_lua(cause, sources)
            }
            mlua::Error::ExternalError(external) => match external.downcast_ref::<ConfigError>() {
                Some(error) => error.clone(),
                None => Self::new(external.to_string()),
            },
            mlua::Error::SyntaxError { message, .. } => located(message, sources),
            mlua::Error::RuntimeError(message) => {
                let message = message
                    .split_once("\nstack traceback:")
                    .map_or(message.as_str(), |(message, _)| message);
                located(message, sources)
            }
            other => Self::new(other.to_string()),
        }
    }
}

pub(crate) fn caller(lua: &Lua) -> Option<(PathBuf, u32)> {
    lua.inspect_stack(1, |debug| {
        let source = debug.source().source?;
        let path = source.strip_prefix('@')?;
        let line = u32::try_from(debug.current_line()?).ok()?;
        Some((PathBuf::from(path), line))
    })
    .flatten()
}

fn located(message: &str, sources: &[PathBuf]) -> ConfigError {
    let split = message.match_indices(':').find_map(|(colon, _)| {
        let rest = &message[colon + 1..];
        let digits = rest.find(|c: char| !c.is_ascii_digit())?;
        let line: u32 = rest[..digits].parse().ok()?;
        let text = rest[digits..].strip_prefix(": ")?;
        Some((&message[..colon], line, text))
    });
    let Some((source, line, text)) = split else {
        return ConfigError::new(message);
    };
    let path = match source.strip_prefix("...") {
        Some(tail) => sources
            .iter()
            .find(|path| path.to_string_lossy().ends_with(tail))
            .map_or_else(|| PathBuf::from(source), Clone::clone),
        None => PathBuf::from(source),
    };
    ConfigError {
        plugin: None,
        location: Some((path, line)),
        message: text.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_names_the_location() {
        let error = ConfigError {
            plugin: None,
            location: Some((PathBuf::from("/home/u/.config/gband/user/init.lua"), 12)),
            message: "unexpected symbol".to_owned(),
        };
        assert_eq!(
            error.to_string(),
            "/home/u/.config/gband/user/init.lua:12: unexpected symbol"
        );
        assert_eq!(ConfigError::new("plain").to_string(), "plain");
    }

    #[test]
    fn display_names_the_plugin() {
        let error = ConfigError {
            plugin: Some("hello".to_owned()),
            location: Some((PathBuf::from("/p/x.lua"), 3)),
            message: "boom".to_owned(),
        };
        assert_eq!(error.to_string(), "hello: /p/x.lua:3: boom");
    }

    #[test]
    fn truncated_sources_expand_to_a_loaded_path() {
        let config = PathBuf::from("/a/very/long/path/gband/user/init.lua");
        let plugin = PathBuf::from("/a/very/long/path/plugins/hello/plugin/hello.lua");
        let sources = [config.clone(), plugin.clone()];
        let error = located("...ng/path/gband/user/init.lua:3: boom", &sources);
        assert_eq!(error.location, Some((config, 3)));
        assert_eq!(error.message, "boom");
        let error = located("...plugins/hello/plugin/hello.lua:7: bang", &sources);
        assert_eq!(error.location, Some((plugin, 7)));
    }
}
