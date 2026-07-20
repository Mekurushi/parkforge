use std::path::{Path, PathBuf};

use parkforge_model::MANIFEST_FILE_NAME;

#[derive(Debug, Clone)]
pub struct OriginalDir {
    root: PathBuf,
}

impl OriginalDir {
    pub(crate) fn new(root: PathBuf) -> Self {
        Self { root }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn manifest_path(&self) -> PathBuf {
        self.root.join(MANIFEST_FILE_NAME)
    }
}

#[derive(Debug, Clone)]
pub struct SourceDir {
    root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct BuildDir {
    root: PathBuf,
}

impl BuildDir {
    pub(crate) fn new(root: PathBuf) -> Self {
        Self { root }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl SourceDir {
    pub(crate) fn new(root: PathBuf) -> Self {
        Self { root }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
}
