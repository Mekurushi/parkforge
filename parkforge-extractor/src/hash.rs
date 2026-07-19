use std::fmt;
use std::path::Path;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileHash {
    Sha256([u8; 32]),
}

impl FileHash {
    #[must_use]
    pub fn sha256_of(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        Self::Sha256(hasher.finalize().into())
    }
}

pub fn hash_file(path: &Path) -> Result<(FileHash, u64)> {
    let data = std::fs::read(path).map_err(|e| Error::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    let size = u64::try_from(data.len()).map_err(|_error| Error::ManifestSizeOverflow)?;
    Ok((FileHash::sha256_of(&data), size))
}

impl fmt::Display for FileHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sha256(bytes) => {
                f.write_str("sha256:")?;
                for byte in bytes {
                    write!(f, "{byte:02x}")?;
                }
                Ok(())
            }
        }
    }
}

impl FromStr for FileHash {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let (algorithm, digest) = s.split_once(':').ok_or_else(|| Error::InvalidHash {
            value: s.to_owned(),
        })?;

        match algorithm {
            "sha256" => parse_hex_32(digest)
                .map(Self::Sha256)
                .ok_or_else(|| Error::InvalidHash {
                    value: s.to_owned(),
                }),
            other => Err(Error::UnsupportedHashAlgorithm {
                algorithm: other.to_owned(),
            }),
        }
    }
}

fn parse_hex_32(digest: &str) -> Option<[u8; 32]> {
    if digest.len() != 64 {
        return None;
    }

    let mut bytes = [0u8; 32];
    for (byte, chunk) in bytes.iter_mut().zip(digest.as_bytes().chunks_exact(2)) {
        let chunk = std::str::from_utf8(chunk).ok()?;
        *byte = u8::from_str_radix(chunk, 16).ok()?;
    }
    Some(bytes)
}

impl Serialize for FileHash {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for FileHash {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}
