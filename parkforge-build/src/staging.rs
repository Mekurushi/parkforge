use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use parkforge_model::StagingDirectory;

use crate::{Error, Result};

static NEXT_BACKUP_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn replace(staging: StagingDirectory, target: &Path) -> Result<()> {
    if !target.exists() {
        return staging.commit(target).map_err(|source| Error::Io {
            path: target.to_path_buf(),
            source,
        });
    }

    let backup = target.with_file_name(format!(
        ".{}.build-backup-{}-{}",
        target
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("build"),
        std::process::id(),
        NEXT_BACKUP_DIRECTORY.fetch_add(1, Ordering::Relaxed)
    ));
    fs::rename(target, &backup).map_err(|source| Error::Io {
        path: target.to_path_buf(),
        source,
    })?;
    if let Err(source) = staging.commit(target) {
        let _ = fs::rename(&backup, target);
        return Err(Error::Io {
            path: target.to_path_buf(),
            source,
        });
    }
    fs::remove_dir_all(&backup).map_err(|source| Error::Io {
        path: backup,
        source,
    })
}
