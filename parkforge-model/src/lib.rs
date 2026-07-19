use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

pub const MANIFEST_FILE_NAME: &str = "manifest.json";

#[derive(Debug, Error)]
#[error("{value:?} is not a valid game ID: {reason}")]
pub struct GameIdError {
    value: String,
    reason: &'static str,
}

#[derive(Debug, Error)]
#[error("{value:?} is not a valid virtual path: {reason}")]
pub struct VirtualPathError {
    value: String,
    reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GameId(String);

impl GameId {
    pub fn new(value: impl Into<String>) -> Result<Self, GameIdError> {
        let value = value.into();
        let reason = (!((value.len() == 6)
            && value.bytes().all(|byte| byte.is_ascii_alphanumeric())))
        .then_some("must contain exactly six ASCII alphanumeric characters");

        match reason {
            Some(reason) => Err(GameIdError { value, reason }),
            None => Ok(Self(value)),
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for GameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for GameId {
    type Err = GameIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl Serialize for GameId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for GameId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VirtualPath(String);

impl VirtualPath {
    pub fn new(value: impl Into<String>) -> Result<Self, VirtualPathError> {
        let value = value.into();
        if value.is_empty() {
            return Err(VirtualPathError {
                value,
                reason: "must not be empty",
            });
        }

        for segment in value.split('/') {
            let reason = if segment.is_empty() {
                Some("must not contain empty segments")
            } else if matches!(segment, "." | "..") {
                Some("must not contain relative path segments")
            } else if segment.contains('\\') {
                Some("must use forward slashes")
            } else {
                None
            };
            if let Some(reason) = reason {
                return Err(VirtualPathError { value, reason });
            }
        }

        Ok(Self(value))
    }

    pub fn join(&self, path: &str) -> Result<Self, VirtualPathError> {
        Self::new(format!("{}/{path}", self.0))
    }

    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        self.0
            .rsplit_once('/')
            .map(|(parent, _)| Self(parent.to_owned()))
    }

    #[must_use]
    pub fn to_path_under(&self, root: &Path) -> PathBuf {
        let mut path = root.to_path_buf();
        path.extend(self.0.split('/'));
        path
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for VirtualPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for VirtualPath {
    type Err = VirtualPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl Serialize for VirtualPath {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for VirtualPath {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::VirtualPath;

    #[test]
    fn virtual_paths_reject_traversal() {
        assert!("../outside".parse::<VirtualPath>().is_err());
    }

    #[test]
    fn virtual_path_join_and_parent_are_portable() -> Result<(), Box<dyn std::error::Error>> {
        let path: VirtualPath = "DATA/files".parse()?;

        assert_eq!(path.join("player.bin")?.as_str(), "DATA/files/player.bin");
        assert_eq!(
            path.parent().as_ref().map(VirtualPath::as_str),
            Some("DATA")
        );
        Ok(())
    }
}
