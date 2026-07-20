use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_STAGING_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct StagingDirectory {
    path: PathBuf,
}

impl StagingDirectory {
    pub fn create(parent: &Path, label: &str) -> io::Result<Self> {
        fs::create_dir_all(parent)?;
        for _ in 0..1024 {
            let sequence = NEXT_STAGING_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!(".{label}-{}-{sequence}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not create a unique staging directory",
        ))
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn commit(mut self, target: &Path) -> io::Result<()> {
        fs::rename(&self.path, target)?;
        self.path.clear();
        Ok(())
    }
}

impl Drop for StagingDirectory {
    fn drop(&mut self) {
        if !self.path.as_os_str().is_empty() {
            let _ = fs::remove_dir_all(&self.path);
        }
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

        let staging = StagingDirectory::create(&root, "R8AJ01.extracting")?;
        let path = staging.path().to_path_buf();
        drop(staging);

        assert!(!path.exists());
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
