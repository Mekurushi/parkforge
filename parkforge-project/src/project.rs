use std::collections::BTreeMap;
use std::fs;
use std::path::{self, Path, PathBuf};

use parkforge_types::{GameId, MakerCode};
use serde::Deserialize;

use crate::error::{Error, Result};

// going with hardcoded project structures for simplicity and because there is no real need to make
// this customizable
const CONFIG_FILE_NAME: &str = "project.toml";
const ORIGINAL_DIR_NAME: &str = "original";
const SOURCES_DIR_NAME: &str = "src";
const SHARED_SOURCES_DIR_NAME: &str = "shared";
const BUILD_DIR_NAME: &str = "build";
const DIST_DIR_NAME: &str = "dist";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub project: ProjectMetadata,
    pub games: BTreeMap<GameId, GameRevision>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ProjectMetadata {
    pub name: String,          // name of the mod, probably later used for banner title
    pub maker_code: MakerCode, // target maker code the patched build should have
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct GameRevision {
    pub display_name: Option<String>, // just for easy identification and placeholder for
                                      // revision specific metadata
}

#[derive(Debug)]
pub struct Project {
    root: PathBuf,
    config: ProjectConfig,
}

impl Project {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        // normalizing into absolute path to make handling throughout all crates easier
        let root =
            path::absolute(&root).map_err(|source| Error::ResolveRoot { path: root, source })?;
        let config_path = root.join(CONFIG_FILE_NAME);
        let contents = fs::read_to_string(&config_path).map_err(|source| Error::ReadConfig {
            path: config_path.clone(),
            source,
        })?;
        let config = toml::from_str(&contents).map_err(|source| Error::ParseConfig {
            path: config_path,
            source,
        })?;

        Ok(Self { root, config })
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn config(&self) -> &ProjectConfig {
        &self.config
    }

    #[must_use]
    pub fn config_path(&self) -> PathBuf {
        self.root.join(CONFIG_FILE_NAME)
    }

    #[must_use]
    pub fn original_dir(&self) -> PathBuf {
        self.root.join(ORIGINAL_DIR_NAME)
    }

    #[must_use]
    pub fn sources_dir(&self) -> PathBuf {
        self.root.join(SOURCES_DIR_NAME)
    }

    #[must_use]
    pub fn shared_sources(&self) -> PathBuf {
        self.sources_dir().join(SHARED_SOURCES_DIR_NAME)
    }

    #[must_use]
    pub fn build_dir(&self) -> PathBuf {
        self.root.join(BUILD_DIR_NAME)
    }

    #[must_use]
    pub fn dist_dir(&self) -> PathBuf {
        self.root.join(DIST_DIR_NAME)
    }

    pub fn original_for(&self, game_id: &GameId) -> Result<PathBuf> {
        self.ensure_supported(game_id)?;
        Ok(self.original_dir().join(game_id.as_str()))
    }

    pub fn sources_for(&self, game_id: &GameId) -> Result<PathBuf> {
        self.ensure_supported(game_id)?;
        Ok(self.sources_dir().join(game_id.as_str()))
    }

    pub fn build_for(&self, game_id: &GameId) -> Result<PathBuf> {
        self.ensure_supported(game_id)?;
        Ok(self.build_dir().join(game_id.as_str()))
    }

    fn ensure_supported(&self, game_id: &GameId) -> Result<()> {
        if self.config.games.contains_key(game_id) {
            Ok(())
        } else {
            Err(Error::UnsupportedGameId(game_id.clone()))
        }
    }
}
