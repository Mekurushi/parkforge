use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

const GAME_ID_LENGTH: usize = 6;

/// A six-character Wii game identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct GameId(String);

impl GameId {
    pub fn new(value: impl Into<String>) -> Result<Self, GameIdError> {
        let value = value.into();
        let is_valid = value.len() == GAME_ID_LENGTH
            && value
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit());

        if is_valid {
            Ok(Self(value))
        } else {
            Err(GameIdError { value })
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for GameId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for GameId {
    type Err = GameIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for GameId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error(
    "{value:?} is not a valid game ID: must contain exactly six uppercase ASCII letters or digits"
)]
pub struct GameIdError {
    value: String,
}
