use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

const MAKER_CODE_LENGTH: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct MakerCode(String);

impl MakerCode {
    pub fn new(value: impl Into<String>) -> Result<Self, MakerCodeError> {
        let value = value.into();
        let is_valid = value.len() == MAKER_CODE_LENGTH
            && value
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit());

        if is_valid {
            Ok(Self(value))
        } else {
            Err(MakerCodeError { value })
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for MakerCode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for MakerCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for MakerCode {
    type Err = MakerCodeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for MakerCode {
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
    "{value:?} is not a valid maker code: must contain exactly two uppercase ASCII letters or digits"
)]
pub struct MakerCodeError {
    value: String,
}
