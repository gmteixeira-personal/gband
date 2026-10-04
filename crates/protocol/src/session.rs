use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

pub const SESSION_NAME_MAX: usize = 64;
pub const DEFAULT_SESSION: &str = "default";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct SessionName(String);

impl SessionName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SessionName {
    fn default() -> Self {
        Self(DEFAULT_SESSION.to_owned())
    }
}

impl fmt::Display for SessionName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for SessionName {
    type Err = String;

    fn from_str(name: &str) -> Result<Self, String> {
        Self::try_from(name.to_owned())
    }
}

impl TryFrom<String> for SessionName {
    type Error = String;

    fn try_from(name: String) -> Result<Self, String> {
        let valid = (1..=SESSION_NAME_MAX).contains(&name.len())
            && name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-');
        if !valid {
            return Err(format!(
                "a session name is 1 to {SESSION_NAME_MAX} ASCII letters, digits, `_` or `-`"
            ));
        }
        Ok(Self(name))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSummary {
    pub name: SessionName,
    pub panes: u32,
    pub clients: u32,
}
