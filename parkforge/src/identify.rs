use std::path::Path;

use parkforge_types::GameId;
use parkforge_wii_disc::read_game_id;

use crate::error::{Error, Result};

pub fn identify(input_iso: &Path) -> Result<GameId> {
    read_game_id(input_iso).map_err(|source| Error::ReadGameId {
        input: input_iso.to_path_buf(),
        source: Box::new(source),
    })
}
