use crate::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) struct StagingDirectory {
    path: PathBuf,
}

impl StagingDirectory {
    pub(crate) fn create(parent: &Path, game_id: &str) -> crate::Result<Self> {
        fs::create_dir_all(parent).map_err(|source| Error::Io {
            path: parent.to_path_buf(),
            source,
        })?;

        let process_id = std::process::id();
        for attempt in 0..1024_u16 {
            let path = parent.join(format!(".{game_id}.extracting-{process_id}-{attempt}"));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(source) => return Err(Error::Io { path, source }),
            }
        }

        Err(Error::StagingDirectoryUnavailable {
            path: parent.to_path_buf(),
        })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn commit(self, target: &Path) -> crate::Result<()> {
        if target.exists() {
            return Err(Error::ExtractionTargetExists {
                path: target.to_path_buf(),
            });
        }
        fs::rename(&self.path, target).map_err(|source| Error::Io {
            path: target.to_path_buf(),
            source,
        })
    }
}

impl Drop for StagingDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::StagingDirectory;

    #[test]
    fn uncommitted_staging_directory_is_removed() -> Result<(), Box<dyn std::error::Error>> {
        let root =
            std::env::temp_dir().join(format!("parkforge-staging-test-{}", std::process::id()));
        fs::create_dir_all(&root)?;

        let staging = StagingDirectory::create(&root, "R8AJ01")?;
        let staging_path = staging.path().to_path_buf();
        drop(staging);

        assert!(!staging_path.exists());
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
