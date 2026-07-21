use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::ProjectConfig;
use crate::error::{Error, Result};
use crate::layout::{BuildDir, DistDir, OriginalDir, SourceDir};
use parkforge_model::{GameId, VirtualPath};

const PROJECT_CONFIG_FILE_NAME: &str = "project.toml";
const GITIGNORE_FILE_NAME: &str = ".gitignore";
const ORIGINAL_DIR_NAME: &str = "original";
const SOURCE_DIR_NAME: &str = "src";
const BUILD_DIR_NAME: &str = "build";
const DIST_DIR_NAME: &str = "dist";

const DEFAULT_GITIGNORE: &str = "/original/\n/build/\n/dist/\n*.fsb\n*.rlb\n";

#[derive(Debug, Clone)]
pub struct Project {
    root: PathBuf,
    config: ProjectConfig,
}

impl Project {
    pub fn create(root: &Path, config: ProjectConfig) -> Result<Self> {
        if root.exists() {
            return Err(Error::ProjectAlreadyExists {
                path: root.to_path_buf(),
            });
        }

        let project = Self {
            root: root.to_path_buf(),
            config,
        };
        project.validate()?;

        fs::create_dir_all(root).map_err(|source| Error::Io {
            path: root.to_path_buf(),
            source,
        })?;
        for dir_name in [
            ORIGINAL_DIR_NAME,
            SOURCE_DIR_NAME,
            BUILD_DIR_NAME,
            DIST_DIR_NAME,
        ] {
            let dir = root.join(dir_name);
            fs::create_dir_all(&dir).map_err(|source| Error::Io { path: dir, source })?;
        }
        project.write_config(&project.config)?;

        let gitignore_path = root.join(GITIGNORE_FILE_NAME);
        fs::write(&gitignore_path, DEFAULT_GITIGNORE).map_err(|source| Error::Io {
            path: gitignore_path,
            source,
        })?;
        Ok(project)
    }

    pub fn open(root: &Path) -> Result<Self> {
        let config_path = root.join(PROJECT_CONFIG_FILE_NAME);
        if !config_path.exists() {
            return Err(Error::NotAProject {
                path: root.to_path_buf(),
            });
        }
        let data = fs::read_to_string(&config_path).map_err(|source| Error::Io {
            path: config_path.clone(),
            source,
        })?;
        let config = toml::from_str(&data).map_err(|error| Error::Parse {
            path: config_path,
            message: error.to_string(),
        })?;
        let project = Self {
            root: root.to_path_buf(),
            config,
        };
        project.validate()?;
        Ok(project)
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn validate(&self) -> Result<()> {
        self.config.build.validate()?;
        let declared: HashSet<GameId> = self
            .config
            .games
            .iter()
            .map(|game| game.game_id.clone())
            .collect();
        if declared.len() != self.config.games.len() {
            let mut seen = HashSet::new();
            for game in &self.config.games {
                if !seen.insert(game.game_id.clone()) {
                    return Err(Error::DuplicateGameId {
                        game_id: game.game_id.clone(),
                    });
                }
            }
        }

        for game_id in Self::game_directories(&self.original_root())? {
            if !declared.contains(&game_id) {
                return Err(Error::UnregisteredGameDirectory {
                    path: self.original_root().join(game_id.as_str()),
                    game_id,
                });
            }
        }
        Ok(())
    }

    fn game_directories(original_root: &Path) -> Result<HashSet<GameId>> {
        if !original_root.exists() {
            return Ok(HashSet::new());
        }
        let entries = fs::read_dir(original_root).map_err(|source| Error::Io {
            path: original_root.to_path_buf(),
            source,
        })?;
        let mut game_ids = HashSet::new();
        for entry in entries {
            let entry = entry.map_err(|source| Error::Io {
                path: original_root.to_path_buf(),
                source,
            })?;
            if !entry
                .file_type()
                .map_err(|source| Error::Io {
                    path: entry.path(),
                    source,
                })?
                .is_dir()
            {
                continue;
            }
            let path = entry.path();
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_error| Error::InvalidUtf8Path { path })?;
            game_ids.insert(GameId::new(name)?);
        }
        Ok(game_ids)
    }

    pub fn games(&self) -> impl Iterator<Item = &GameId> {
        self.config.games.iter().map(|game| &game.game_id)
    }

    pub fn add_game(&mut self, game_id: GameId, label: Option<String>) -> Result<()> {
        if self.games().any(|registered| registered == &game_id) {
            return Err(Error::GameAlreadyRegistered { game_id });
        }
        let mut config = self.config.clone();
        config
            .games
            .push(crate::config::GameConfig { game_id, label });
        self.write_config(&config)?;
        self.config = config;
        Ok(())
    }

    pub fn require_registered_game(&self, game_id: &GameId) -> Result<()> {
        if self.games().any(|registered| registered == game_id) {
            Ok(())
        } else {
            Err(Error::GameNotRegistered {
                game_id: game_id.clone(),
            })
        }
    }

    pub fn require_original(&self, game_id: &GameId) -> Result<OriginalDir> {
        self.require_registered_game(game_id)?;
        let original = self.original(game_id);
        if !original.root().is_dir() {
            return Err(Error::MissingOriginalDirectory {
                game_id: game_id.clone(),
                path: original.root().to_path_buf(),
            });
        }
        Ok(original)
    }

    pub fn require_build(&self, game_id: &GameId) -> Result<BuildDir> {
        self.require_registered_game(game_id)?;
        let build = self.build(game_id);
        if !build.root().is_dir() {
            return Err(Error::MissingBuildDirectory {
                game_id: game_id.clone(),
                path: build.root().to_path_buf(),
            });
        }
        Ok(build)
    }

    #[must_use]
    pub fn original_root(&self) -> PathBuf {
        self.root.join(ORIGINAL_DIR_NAME)
    }

    #[must_use]
    pub fn original(&self, game_id: &GameId) -> OriginalDir {
        OriginalDir::new(self.original_root().join(game_id.as_str()))
    }

    #[must_use]
    pub fn source(&self, game_id: &GameId) -> SourceDir {
        SourceDir::new(self.root.join(SOURCE_DIR_NAME).join(game_id.as_str()))
    }

    #[must_use]
    pub fn build(&self, game_id: &GameId) -> BuildDir {
        BuildDir::new(self.root.join(BUILD_DIR_NAME).join(game_id.as_str()))
    }

    #[must_use]
    pub fn dist(&self, game_id: &GameId) -> DistDir {
        DistDir::new(self.root.join(DIST_DIR_NAME).join(game_id.as_str()))
    }

    #[must_use]
    pub fn build_config(&self) -> &parkforge_build::BuildConfig {
        &self.config.build
    }

    fn write_config(&self, config: &ProjectConfig) -> Result<()> {
        let config_path = self.root.join(PROJECT_CONFIG_FILE_NAME);
        let data = toml::to_string_pretty(config).map_err(|error| Error::Parse {
            path: config_path.clone(),
            message: error.to_string(),
        })?;
        fs::write(&config_path, data).map_err(|source| Error::Io {
            path: config_path,
            source,
        })
    }

    pub fn create_source_overlay(
        &self,
        game_id: &GameId,
        directories: impl IntoIterator<Item = VirtualPath>,
    ) -> Result<()> {
        self.require_registered_game(game_id)?;
        let source_root = self.source(game_id).root().to_path_buf();
        for directory in directories {
            let target = directory.to_path_under(&source_root);
            fs::create_dir_all(&target).map_err(|source| Error::Io {
                path: target,
                source,
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::Project;
    use crate::{BuildConfig, Error, ProjectConfig, ProjectMetadata};

    static NEXT_TEST_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

    fn test_directory() -> PathBuf {
        let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "parkforge-project-test-{}-{sequence}",
            std::process::id()
        ))
    }

    fn config() -> ProjectConfig {
        ProjectConfig {
            project: ProjectMetadata {
                name: "test".to_owned(),
                version: "0.1.0".to_owned(),
            },
            games: Vec::new(),
            build: BuildConfig::default(),
        }
    }

    #[test]
    fn create_writes_a_project_that_can_be_opened() -> Result<(), Box<dyn std::error::Error>> {
        let root = test_directory();
        Project::create(&root, config())?;
        assert!(root.join("project.toml").is_file());
        assert!(root.join(".gitignore").is_file());
        assert!(root.join("original").is_dir());
        assert!(root.join("src").is_dir());
        assert!(root.join("build").is_dir());
        assert!(root.join("dist").is_dir());
        Project::open(&root)?;
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn create_rejects_an_existing_directory() -> Result<(), Box<dyn std::error::Error>> {
        let root = test_directory();
        fs::create_dir_all(&root)?;
        assert!(matches!(
            Project::create(&root, config()),
            Err(Error::ProjectAlreadyExists { .. })
        ));
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
