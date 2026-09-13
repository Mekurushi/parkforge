use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::error::{Error, Result};
use crate::paths::{absolute_directory, validate_no_overlap};
use crate::staging::Staging;

pub fn build(original: &Path, destination: &Path) -> Result<()> {
    Build::new(original, destination)?.run()
}

struct Build {
    original: PathBuf,
    staging: Staging,
}

impl Build {
    fn new(original: &Path, destination: &Path) -> Result<Self> {
        let original = absolute_directory(original, false)?;
        let staging = Staging::new(destination)?;
        validate_no_overlap(&original, staging.destination())?;

        Ok(Self { original, staging })
    }

    fn run(self) -> Result<()> {
        self.copy_tree()?;
        self.apply_patches()?;
        self.staging.commit()
    }

    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn apply_patches(&self) -> Result<()> {
        // TODO: compile patches/src files onto original
        Ok(())
    }

    fn copy_tree(&self) -> Result<()> {
        let original = self.original.as_path();
        let destination = self.staging.path();
        for entry in WalkDir::new(original) {
            let entry = entry.map_err(|source| Error::WalkOriginal {
                root: original.to_path_buf(),
                source,
            })?;
            let input = entry.path();
            if entry.file_type().is_symlink() {
                return Err(Error::SymbolicLink(input.to_path_buf()));
            }
            let relative = input
                .strip_prefix(original)
                .map_err(|source| Error::RelativeEntry {
                    path: input.to_path_buf(),
                    root: original.to_path_buf(),
                    source,
                })?;
            let output = destination.join(relative);
            if entry.file_type().is_dir() {
                fs::create_dir_all(&output).map_err(|source| Error::CreateDirectory {
                    path: output,
                    source,
                })?;
            } else if entry.file_type().is_file() {
                let _ = fs::copy(input, &output).map_err(|source| Error::CopyFile {
                    input: input.to_path_buf(),
                    output,
                    source,
                })?;
            } else {
                return Err(Error::UnsupportedEntry(input.to_path_buf()));
            }
        }
        Ok(())
    }
}
