mod container;
mod error;
mod hash;
mod manifest;
mod staging_directory;
mod wii_iso_extractor;

pub use error::{Error, Result};
pub use manifest::Manifest;

use crate::container::extract_archives;
use crate::staging_directory::StagingDirectory;
use crate::wii_iso_extractor::WiiIsoExtractor;
use parkforge_model::{GameId, MANIFEST_FILE_NAME};
use std::fs::File;
use std::io::Read;
use std::path::Path;

const GAME_ID_LEN: usize = 6;
const DATA_PARTITION: &str = "DATA";
pub fn read_game_id(iso_path: &Path) -> Result<GameId> {
    let mut file = File::open(iso_path).map_err(|e| Error::Io {
        path: iso_path.to_path_buf(),
        source: e,
    })?;

    let mut raw = [0u8; GAME_ID_LEN];
    file.read_exact(&mut raw).map_err(|e| Error::Io {
        path: iso_path.to_path_buf(),
        source: e,
    })?;

    std::str::from_utf8(&raw)
        .map_err(|_error| Error::InvalidGameId {
            path: iso_path.to_path_buf(),
            reason: "not valid UTF-8",
        })?
        .parse()
        .map_err(|_error| Error::InvalidGameId {
            path: iso_path.to_path_buf(),
            reason: "must be exactly six ASCII alphanumeric characters",
        })
}

pub fn extract(iso_path: &Path, workspace_root: &Path) -> Result<Manifest> {
    let game_id = read_game_id(iso_path)?;
    let game_dir = workspace_root.join(game_id.as_str());
    if game_dir.exists() {
        return Err(Error::ExtractionTargetExists { path: game_dir });
    }

    let staging = StagingDirectory::create(workspace_root, game_id.as_str())?;
    extract_iso_into(iso_path, staging.path())?;
    let manifest = extract_archives(staging.path(), game_id)?;
    manifest.save(&staging.path().join(MANIFEST_FILE_NAME))?;
    staging.commit(&game_dir)?;
    Ok(manifest)
}

fn extract_iso_into(iso_path: &Path, output_dir: &Path) -> Result<()> {
    let mut extractor = WiiIsoExtractor::new(iso_path)?;
    extractor.prepare_extract_section(DATA_PARTITION.to_owned())?;
    extractor.extract_to(output_dir, |progress| println!("progress: {progress}%"))
}
