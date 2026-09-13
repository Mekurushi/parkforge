use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::{self, Path, PathBuf};

use parkforge_types::{GameId, MakerCode};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::error::{Error, Result};
use crate::overlay;

// going with hardcoded project structures for simplicity and because there is no real need to make
// this customizable
const CONFIG_FILE_NAME: &str = "project.toml";
const ORIGINAL_DIR_NAME: &str = "original";
const SOURCES_DIR_NAME: &str = "src";
const SHARED_SOURCES_DIR_NAME: &str = "shared";
const BUILD_DIR_NAME: &str = "build";
const DIST_DIR_NAME: &str = "dist";
const GITIGNORE_FILE_NAME: &str = ".gitignore";
const INITIAL_GITIGNORE: &str = "/original/\n/build/\n/dist/\n";

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub project: ProjectMetadata,
    pub games: BTreeMap<GameId, GameRevision>,
}

impl ProjectConfig {
    pub fn new(name: impl Into<String>, game_id: Option<GameId>) -> Self {
        let mut games = BTreeMap::new();
        if let Some(game_id) = game_id {
            // the right now created map can't contain the game_id already so we're saving the
            // error handling
            drop(games.insert(game_id, GameRevision { display_name: None }));
        }

        Self {
            project: ProjectMetadata {
                name: name.into(),
                version: None,
                maker_code: None,
            },
            games,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ProjectMetadata {
    pub name: String, // name of the mod, probably later used for banner title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>, // free string field for now, planned to be used for banner title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maker_code: Option<MakerCode>, // target maker code the patched build should have
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct GameRevision {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>, // just for easy identification and placeholder for
                                      // revision specific metadata
}

#[derive(Debug)]
pub struct Project {
    root: PathBuf,
    config: ProjectConfig,
}

impl Project {
    pub fn init(
        root: impl Into<PathBuf>,
        name: impl Into<String>,
        game_id: Option<GameId>,
    ) -> Result<Self> {
        let root = root.into();
        if !root.is_dir() {
            return Err(Error::RootNotDirectory(root));
        }
        let root =
            path::absolute(&root).map_err(|source| Error::ResolveRoot { path: root, source })?;
        let project = Self {
            root,
            config: ProjectConfig::new(name, game_id),
        };
        let config_path = project.config_path();
        let contents =
            toml::to_string(&project.config).map_err(|source| Error::SerializeConfig {
                path: config_path.clone(),
                source,
            })?;
        let mut config_file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&config_path)
            .map_err(|source| Error::CreateConfig {
                path: config_path.clone(),
                source,
            })?;
        config_file
            .write_all(contents.as_bytes())
            .map_err(|source| Error::WriteConfig {
                path: config_path,
                source,
            })?;

        for directory in [
            project.original_dir(),
            project.shared_sources(),
            project.build_dir(),
            project.dist_dir(),
        ] {
            fs::create_dir_all(&directory).map_err(|source| Error::CreateDirectory {
                path: directory,
                source,
            })?;
        }
        for game_id in project.config.games.keys() {
            let directory = project.sources_for(game_id)?;
            fs::create_dir_all(&directory).map_err(|source| Error::CreateDirectory {
                path: directory,
                source,
            })?;
        }
        project.create_gitignore()?;
        Ok(project)
    }

    fn create_gitignore(&self) -> Result<()> {
        let path = self.root.join(GITIGNORE_FILE_NAME);
        let mut file = match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => file,
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => return Ok(()),
            Err(source) => return Err(Error::CreateGitignore { path, source }),
        };
        file.write_all(INITIAL_GITIGNORE.as_bytes())
            .map_err(|source| Error::WriteGitignore { path, source })
    }

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

    pub fn create_source_overlay(&self, game_id: &GameId) -> Result<()> {
        let original = self.original_for(game_id)?;
        if !original.is_dir() {
            return Err(Error::OriginalNotDirectory(original));
        }
        let sources = self.sources_for(game_id)?;

        for entry in WalkDir::new(&original) {
            let entry = entry.map_err(|source| Error::WalkOriginal {
                root: original.clone(),
                source,
            })?;
            if !entry.file_type().is_dir() {
                continue;
            }
            let relative = entry.path().strip_prefix(&original).map_err(|source| {
                Error::RelativeOriginalPath {
                    path: entry.path().to_path_buf(),
                    root: original.clone(),
                    source,
                }
            })?;
            let directory = sources.join(relative);
            fs::create_dir_all(&directory).map_err(|source| Error::CreateDirectory {
                path: directory,
                source,
            })?;
        }
        Ok(())
    }

    pub fn merge_sources(&self, game_id: &GameId, destination: &Path) -> Result<()> {
        let revision = self.sources_for(game_id)?;
        overlay::merge_sources(&self.shared_sources(), &revision, destination)
    }

    fn ensure_supported(&self, game_id: &GameId) -> Result<()> {
        if self.config.games.contains_key(game_id) {
            Ok(())
        } else {
            Err(Error::UnsupportedGameId(game_id.clone()))
        }
    }
}
