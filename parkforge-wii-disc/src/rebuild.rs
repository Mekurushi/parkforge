use std::fs;
use std::io::{Read, Seek, Write};
use std::path::Path;

use disc_riider::builder::build_from_directory;
use tempfile::Builder;

use crate::error::{Error, Result};
use crate::progress::Progress;

pub fn rebuild_to_writer<W, F>(source: &Path, destination: &mut W, mut progress: F) -> Result<()>
where
    W: Read + Write + Seek,
    F: FnMut(Progress),
{
    const TOTAL: u64 = 100;
    let mut completed = 0_u64;
    progress(Progress::new(completed, TOTAL));
    build_from_directory(source, destination, &mut |done_percent| {
        completed = completed.max(u64::from(done_percent));
        progress(Progress::new(completed, TOTAL));
    })
    .map_err(|error| Error::Rebuild {
        path: source.to_path_buf(),
        source: Box::new(error),
    })?;
    if completed < TOTAL {
        progress(Progress::new(TOTAL, TOTAL));
    }
    destination
        .flush()
        .map_err(|source| Error::FlushIso { source })
}

pub fn rebuild_iso<F>(source: &Path, destination: &Path, progress: F) -> Result<()>
where
    F: FnMut(Progress),
{
    let parent = destination
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|source| Error::Io {
        path: parent.to_path_buf(),
        source,
    })?;

    let mut temporary = Builder::new()
        .prefix(".parkforge-rebuild-")
        .suffix(".iso")
        .tempfile_in(parent)
        .map_err(|source| Error::CreateTemporaryIso {
            directory: parent.to_path_buf(),
            source,
        })?;

    rebuild_to_writer(source, temporary.as_file_mut(), progress)?;
    drop(
        temporary
            .persist(destination)
            .map_err(|error| Error::PersistIso {
                destination: destination.to_path_buf(),
                source: error.error,
            })?,
    );
    Ok(())
}
