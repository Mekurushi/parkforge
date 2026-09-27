use std::fmt;

use serde::Deserialize;
use serde::de::{Deserializer, MapAccess, Visitor};

use super::error::{Error, Result};
use super::patch::{PatchMetadata, Patchlet};

pub(super) struct Artifact {
    pub(super) patchlets: Vec<Patchlet>,
}

impl Artifact {
    pub(super) fn parse(text: &str) -> Result<Self> {
        let raw: RawArtifact =
            serde_yaml_ng::from_str(text).map_err(|source| Error::ParseArtifact { source })?;
        Ok(Self {
            patchlets: raw.main_dol.0,
        })
    }
}

impl PatchMetadata {
    pub(super) fn parse(text: &str) -> Result<Self> {
        toml::from_str(text).map_err(|source| Error::ParseMetadata { source })
    }
}

#[derive(Deserialize)]
struct RawArtifact {
    #[serde(rename = "main.dol")]
    main_dol: RawPatchlets,
}

struct RawPatchlets(Vec<Patchlet>);

impl<'de> Deserialize<'de> for RawPatchlets {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(RawPatchletsVisitor)
    }
}

struct RawPatchletsVisitor;

impl<'de> Visitor<'de> for RawPatchletsVisitor {
    type Value = RawPatchlets;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a mapping from DOL addresses to patchlets")
    }

    fn visit_map<M>(self, mut map: M) -> std::result::Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut patchlets = Vec::with_capacity(map.size_hint().unwrap_or(0));
        while let Some((address, raw)) = map.next_entry::<u32, RawPatchlet>()? {
            patchlets.push(Patchlet {
                address,
                data: raw.data,
            });
        }
        Ok(RawPatchlets(patchlets))
    }
}

#[derive(Deserialize)]
struct RawPatchlet {
    #[serde(rename = "Data")]
    data: Vec<u8>,
}
