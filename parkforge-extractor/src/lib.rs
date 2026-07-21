mod container;
mod error;
mod hash;
mod manifest;
mod wii_iso_extractor;

pub use error::{Error, Result};
pub use manifest::Manifest;

use crate::container::{extract_archives, repack_archives};
use crate::wii_iso_extractor::{WiiIsoExtractor, rebuild_from_directory};
use parkforge_model::{GameId, MANIFEST_FILE_NAME, StagingDirectory};
use std::fs::{self, File};
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

    let staging = StagingDirectory::create(workspace_root, &format!("{game_id}.extracting"))
        .map_err(|source| Error::Io {
            path: workspace_root.to_path_buf(),
            source,
        })?;
    extract_iso_into(iso_path, staging.path())?;
    let manifest = extract_archives(staging.path(), game_id)?;
    manifest.save(&staging.path().join(MANIFEST_FILE_NAME))?;
    staging.commit(&game_dir).map_err(|source| Error::Io {
        path: game_dir,
        source,
    })?;
    Ok(manifest)
}

pub fn rebuild<F>(
    build_root: &Path,
    manifest: &Manifest,
    destination: &Path,
    progress: F,
) -> Result<()>
where
    F: FnMut(u32),
{
    let parent = build_root.parent().ok_or_else(|| Error::Rebuild {
        path: build_root.to_path_buf(),
        message: "build path has no parent directory".to_owned(),
    })?;
    let staging = StagingDirectory::create(parent, &format!("{}.rebuilding", manifest.game_id))
        .map_err(|source| Error::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    copy_tree(build_root, staging.path())?;
    repack_archives(staging.path(), manifest)?;
    rebuild_from_directory(staging.path(), destination, progress)
}

fn extract_iso_into(iso_path: &Path, output_dir: &Path) -> Result<()> {
    let mut extractor = WiiIsoExtractor::new(iso_path)?;
    extractor.prepare_extract_section(DATA_PARTITION.to_owned())?;
    extractor.extract_to(output_dir, |progress| println!("progress: {progress}%"))
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination).map_err(|source_error| Error::Io {
        path: destination.to_path_buf(),
        source: source_error,
    })?;
    for entry in fs::read_dir(source).map_err(|source_error| Error::Io {
        path: source.to_path_buf(),
        source: source_error,
    })? {
        let entry = entry.map_err(|source_error| Error::Io {
            path: source.to_path_buf(),
            source: source_error,
        })?;
        let path = entry.path();
        let output = destination.join(entry.file_name());
        let file_type = entry.file_type().map_err(|source_error| Error::Io {
            path: path.clone(),
            source: source_error,
        })?;
        if file_type.is_dir() {
            copy_tree(&path, &output)?;
        } else if file_type.is_file() {
            fs::copy(&path, &output).map_err(|source_error| Error::Io {
                path: output,
                source: source_error,
            })?;
        } else {
            return Err(Error::Rebuild {
                path,
                message: "build tree must contain only regular files and directories".to_owned(),
            });
        }
    }
    Ok(())
}
