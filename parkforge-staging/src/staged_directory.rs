use std::fs;
use std::io;
use std::path::{self, Path, PathBuf};

use tempfile::{Builder, TempDir};

use crate::error::{Error, Result};

pub struct StagedDirectory {
    directory: TempDir,
    destination: PathBuf,
}

impl StagedDirectory {
    pub fn new(destination: &Path) -> Result<Self> {
        let destination =
            path::absolute(destination).map_err(|source| Error::ResolveDestination {
                path: destination.to_path_buf(),
                source,
            })?;
        match fs::symlink_metadata(&destination) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(Error::DestinationIsSymlink(destination));
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(Error::DestinationIsNotDirectory(destination));
            }
            Ok(_) => {}
            Err(source) if source.kind() == io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(Error::InspectDestination {
                    path: destination,
                    source,
                });
            }
        }
        let parent = destination
            .parent()
            .ok_or_else(|| Error::InvalidDestination(destination.clone()))?;
        let directory = Builder::new()
            .prefix(".parkforge-staging-")
            .tempdir_in(parent)
            .map_err(|source| Error::CreateTemporaryDirectory {
                parent: parent.to_path_buf(),
                source,
            })?;
        Ok(Self {
            directory,
            destination,
        })
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        self.directory.path()
    }

    #[must_use]
    pub fn destination(&self) -> &Path {
        &self.destination
    }

    fn commit_new(self) -> Result<()> {
        fs::rename(self.directory.path(), &self.destination).map_err(|source| Error::Commit {
            destination: self.destination.clone(),
            source,
        })?;
        drop(self.directory.keep());
        Ok(())
    }

    fn replace_existing(self) -> Result<()> {
        let parent = self
            .destination
            .parent()
            .ok_or_else(|| Error::InvalidDestination(self.destination.clone()))?;
        let backup = Builder::new()
            .prefix(".parkforge-staging-backup-")
            .tempdir_in(parent)
            .map_err(|source| Error::CreateTemporaryDirectory {
                parent: parent.to_path_buf(),
                source,
            })?;
        let previous = backup.path().join("previous");

        fs::rename(&self.destination, &previous).map_err(|source| Error::Backup {
            destination: self.destination.clone(),
            backup: previous.clone(),
            source,
        })?;

        if let Err(source) = fs::rename(self.directory.path(), &self.destination) {
            if let Err(rollback_error) = fs::rename(&previous, &self.destination) {
                drop(backup.keep());
                return Err(Error::Rollback {
                    destination: self.destination.clone(),
                    backup: previous,
                    source,
                    rollback_error,
                });
            }
            return Err(Error::Commit {
                destination: self.destination.clone(),
                source,
            });
        }
        drop(self.directory.keep());

        let backup_path = backup.path().to_path_buf();
        backup.close().map_err(|source| Error::RemoveBackup {
            path: backup_path,
            source,
        })
    }

    pub fn commit(self) -> Result<()> {
        match fs::symlink_metadata(&self.destination) {
            Ok(_) => self.replace_existing(),
            Err(source) if source.kind() == io::ErrorKind::NotFound => self.commit_new(),
            Err(source) => Err(Error::InspectDestination {
                path: self.destination.clone(),
                source,
            }),
        }
    }
}
