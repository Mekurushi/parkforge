use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::hash::FileHash;
use parkforge_model::{GameId, VirtualPath};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) struct ContainerId(u32);

impl ContainerId {
    fn from_index(index: usize) -> Result<Self> {
        u32::try_from(index)
            .map(Self)
            .map_err(|_error| Error::ManifestSizeOverflow)
    }

    fn as_index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ContainerFormat {
    U8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum CompressionFormat {
    None,
    Nlzss11,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Container {
    pub(crate) id: ContainerId,
    pub(crate) virtual_path: VirtualPath,
    pub(crate) format: ContainerFormat,
    pub(crate) compression: CompressionFormat,
    pub(crate) parent: Option<ContainerId>,
    pub(crate) internal_path: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct NewContainer {
    pub(crate) virtual_path: VirtualPath,
    pub(crate) format: ContainerFormat,
    pub(crate) compression: CompressionFormat,
    pub(crate) parent: Option<ContainerId>,
    pub(crate) internal_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ManifestFile {
    pub(crate) virtual_path: VirtualPath,
    pub(crate) container: Option<ContainerId>,
    pub(crate) internal_path: Option<String>,
    pub(crate) hash: FileHash,
    pub(crate) size: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct NewFile {
    pub(crate) virtual_path: VirtualPath,
    pub(crate) container: Option<ContainerId>,
    pub(crate) internal_path: Option<String>,
    pub(crate) hash: FileHash,
    pub(crate) size: u64,
}

/// TODO: veryifing it's possible to rebuild out-of-the-box with current manifest metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub game_id: GameId,
    containers: Vec<Container>,
    files: Vec<ManifestFile>,
}

impl Manifest {
    #[must_use]
    pub(crate) fn new(game_id: GameId) -> Self {
        Self {
            game_id,
            containers: Vec::new(),
            files: Vec::new(),
        }
    }

    pub(crate) fn add_container(&mut self, container: NewContainer) -> Result<ContainerId> {
        let id = ContainerId::from_index(self.containers.len())?;
        self.containers.push(Container {
            id,
            virtual_path: container.virtual_path,
            format: container.format,
            compression: container.compression,
            parent: container.parent,
            internal_path: container.internal_path,
        });
        Ok(id)
    }

    pub(crate) fn add_file(&mut self, file: NewFile) {
        self.files.push(ManifestFile {
            virtual_path: file.virtual_path,
            container: file.container,
            internal_path: file.internal_path,
            hash: file.hash,
            size: file.size,
        });
    }

    fn container(&self, id: ContainerId) -> Option<&Container> {
        self.containers.get(id.as_index())
    }

    pub(crate) fn containers(&self) -> impl Iterator<Item = (ContainerId, &Container)> {
        self.containers
            .iter()
            .map(|container| (container.id, container))
    }

    pub(crate) fn files(&self) -> impl Iterator<Item = &ManifestFile> {
        self.files.iter()
    }
    fn validate(&self) -> Result<()> {
        self.validate_containers()?;
        self.validate_files()
    }

    fn validate_containers(&self) -> Result<()> {
        let mut paths = HashSet::new();
        for (index, container) in self.containers.iter().enumerate() {
            if container.id != ContainerId::from_index(index)? {
                return Err(Error::InvalidManifest {
                    virtual_path: container.virtual_path.to_string(),
                    message: "container ID does not match its serialized position".to_owned(),
                });
            }
            if !paths.insert(&container.virtual_path) {
                return Err(Error::InvalidManifest {
                    virtual_path: container.virtual_path.to_string(),
                    message: "container virtual paths must be unique".to_owned(),
                });
            }
            if let Some(parent) = container.parent
                && self.container(parent).is_none()
            {
                return Err(Error::UnknownContainer {
                    virtual_path: container.virtual_path.to_string(),
                    container: parent.0,
                });
            }

            if container.parent.is_some() != container.internal_path.is_some() {
                return Err(Error::InvalidManifest {
                    virtual_path: container.virtual_path.to_string(),
                    message: "a container must have an internal_path exactly when it has a parent"
                        .to_owned(),
                });
            }

            if let (Some(parent), Some(internal_path)) =
                (container.parent, &container.internal_path)
            {
                let parent = self
                    .container(parent)
                    .ok_or_else(|| Error::UnknownContainer {
                        virtual_path: container.virtual_path.to_string(),
                        container: parent.0,
                    })?;
                if parent.virtual_path.join(internal_path)? != container.virtual_path {
                    return Err(Error::InvalidManifest {
                        virtual_path: container.virtual_path.to_string(),
                        message: "container path does not match its parent and internal_path"
                            .to_owned(),
                    });
                }
            }

            self.check_no_parent_cycle(container.id)?;
        }
        Ok(())
    }

    fn check_no_parent_cycle(&self, start: ContainerId) -> Result<()> {
        let mut current = start;
        for _ in 0..=self.containers.len() {
            let Some(container) = self.container(current) else {
                return Ok(());
            };
            let Some(parent) = container.parent else {
                return Ok(());
            };
            current = parent;
        }
        Err(Error::InvalidManifest {
            virtual_path: self
                .container(start)
                .map_or_else(String::new, |c| c.virtual_path.to_string()),
            message: "container parent chain contains a cycle".to_owned(),
        })
    }

    fn validate_files(&self) -> Result<()> {
        let mut paths = HashSet::new();
        for file in &self.files {
            if !paths.insert(&file.virtual_path) {
                return Err(Error::InvalidManifest {
                    virtual_path: file.virtual_path.to_string(),
                    message: "file virtual paths must be unique".to_owned(),
                });
            }
            if let Some(container) = file.container
                && self.container(container).is_none()
            {
                return Err(Error::UnknownContainer {
                    virtual_path: file.virtual_path.to_string(),
                    container: container.0,
                });
            }
            if file.container.is_some() != file.internal_path.is_some() {
                return Err(Error::InvalidManifest {
                    virtual_path: file.virtual_path.to_string(),
                    message: "a file must have an internal_path exactly when it has a container"
                        .to_owned(),
                });
            }
            if let (Some(container), Some(internal_path)) = (file.container, &file.internal_path) {
                let container =
                    self.container(container)
                        .ok_or_else(|| Error::UnknownContainer {
                            virtual_path: file.virtual_path.to_string(),
                            container: container.0,
                        })?;
                if container.virtual_path.join(internal_path)? != file.virtual_path {
                    return Err(Error::InvalidManifest {
                        virtual_path: file.virtual_path.to_string(),
                        message: "file path does not match its container and internal_path"
                            .to_owned(),
                    });
                }
            }
        }
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        let data = fs::read_to_string(path).map_err(|e| Error::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        let manifest: Self = serde_json::from_str(&data).map_err(|e| Error::Parse {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub(crate) fn save(&self, path: &Path) -> Result<()> {
        let data = serde_json::to_string_pretty(self).map_err(|e| Error::Parse {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;
        fs::write(path, data).map_err(|e| Error::Io {
            path: path.to_path_buf(),
            source: e,
        })
    }

    #[must_use]
    pub fn directories(&self) -> BTreeSet<VirtualPath> {
        let mut directories = BTreeSet::new();

        for (_, container) in self.containers() {
            directories.insert(container.virtual_path.clone());
        }

        for file in self.files() {
            if let Some(parent) = file.virtual_path.parent() {
                directories.insert(parent);
            }
        }

        directories
    }
}

#[cfg(test)]
mod tests {
    use super::Manifest;

    #[test]
    fn new_manifest_keeps_its_game_id() -> Result<(), Box<dyn std::error::Error>> {
        let manifest = Manifest::new("RMGE01".parse()?);

        assert_eq!(manifest.game_id.as_str(), "RMGE01");
        Ok(())
    }
}
