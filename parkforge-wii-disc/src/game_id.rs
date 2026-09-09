use std::fs::File;
use std::io::Read;
use std::path::Path;

use parkforge_types::GameId;

use crate::error::{Error, Result};

const GAME_ID_LENGTH: usize = 6;

pub fn read_game_id(path: &Path) -> Result<GameId> {
    let mut reader = File::open(path).map_err(|source| Error::OpenIso {
        path: path.to_path_buf(),
        source,
    })?;
    let mut raw = [0; GAME_ID_LENGTH];
    reader.read_exact(&mut raw).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse_game_id(raw)
}

pub(crate) fn parse_game_id(raw: [u8; GAME_ID_LENGTH]) -> Result<GameId> {
    let value = std::str::from_utf8(&raw).map_err(|_error| Error::InvalidGameId { raw })?;
    GameId::new(value).map_err(|_error| Error::InvalidGameId { raw })
}
