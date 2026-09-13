use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use tempfile::{Builder, TempDir};

use crate::error::{Error, Result};
use crate::paths::absolute_directory;

pub(crate) struct Staging {
    directory: TempDir,
    destination: PathBuf,
}

impl Staging {
    pub(crate) fn new(destination: &Path) -> Result<Self> {
        let destination = absolute_directory(destination, true)?;
        let parent = destination
            .parent()
            .ok_or_else(|| Error::InvalidDestination(destination.clone()))?;
        let directory = Builder::new()
            .prefix(".parkforge-build-")
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

    pub(crate) fn path(&self) -> &Path {
        self.directory.path()
    }

    pub(crate) fn destination(&self) -> &Path {
        &self.destination
    }

    fn parent(&self) -> Result<&Path> {
        self.destination
            .parent()
            .ok_or_else(|| Error::InvalidDestination(self.destination.clone()))
    }

    fn commit_new(self) -> Result<()> {
        fs::rename(self.directory.path(), &self.destination).map_err(|source| {
            Error::CommitBuild {
                destination: self.destination.clone(),
                source,
            }
        })?;
        drop(self.directory.keep());
        Ok(())
    }
    fn replace_existing(self) -> Result<()> {
        let parent = self.parent()?;
        let backup = Builder::new()
            .prefix(".parkforge-build-backup-")
            .tempdir_in(parent)
            .map_err(|source| Error::CreateTemporaryDirectory {
                parent: parent.to_path_buf(),
                source,
            })?;
        let previous = backup.path().join("previous");

        fs::rename(&self.destination, &previous).map_err(|source| Error::BackupBuild {
            destination: self.destination.clone(),
            backup: previous.clone(),
            source,
        })?;

        if let Err(source) = fs::rename(self.directory.path(), &self.destination) {
            if let Err(rollback_error) = fs::rename(&previous, &self.destination) {
                drop(backup.keep());
                return Err(Error::RollbackBuild {
                    destination: self.destination.clone(),
                    backup: previous,
                    source,
                    rollback_error,
                });
            }
            return Err(Error::CommitBuild {
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

    pub(crate) fn commit(self) -> Result<()> {
        let destination_exists = match fs::symlink_metadata(&self.destination) {
            Ok(_) => true,
            Err(source) if source.kind() == io::ErrorKind::NotFound => false,
            Err(source) => {
                return Err(Error::InspectPath {
                    path: self.destination.clone(),
                    source,
                });
            }
        };
        if destination_exists {
            self.replace_existing()
        } else {
            self.commit_new()
        }
    }
}
