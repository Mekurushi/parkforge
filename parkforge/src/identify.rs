use std::path::{Path, PathBuf};

use parkforge_types::GameId;
use parkforge_wii_disc::read_game_id;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IdentifyError {
    #[error("NKit ISO {input:?} is not supported")]
    UnsupportedNkitIso { input: PathBuf },

    #[error("ISO {input:?} contains an invalid Wii game ID: {raw:?}")]
    InvalidGameId { input: PathBuf, raw: [u8; 6] },

    #[error("failed to read game ID from ISO {input:?}: {source}")]
    ReadGameId {
        input: PathBuf,
        #[source]
        source: Box<parkforge_wii_disc::Error>,
    },
}

pub fn identify(input_iso: &Path) -> Result<GameId, IdentifyError> {
    read_game_id(input_iso).map_err(|source| match source {
        parkforge_wii_disc::Error::UnsupportedNkitIso => IdentifyError::UnsupportedNkitIso {
            input: input_iso.to_path_buf(),
        },
        parkforge_wii_disc::Error::InvalidGameId { raw } => IdentifyError::InvalidGameId {
            input: input_iso.to_path_buf(),
            raw,
        },
        source => IdentifyError::ReadGameId {
            input: input_iso.to_path_buf(),
            source: Box::new(source),
        },
    })
}
