use crate::error::{Error, Result};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use u8arc::{Entry, FileEntry, U8Arc};
use walkdir::WalkDir;

pub(crate) fn unpack_archive(data: Vec<u8>, destination: &Path) -> Result<()> {
    let archive = U8Arc::read_vec(data).map_err(|source| Error::ParseU8 {
        destination: destination.to_path_buf(),
        source,
    })?;

    unpack_entries(&archive, archive.get_root_entry(), destination)
}

fn unpack_entries(archive: &U8Arc<'_>, entries: &[Entry], destination: &Path) -> Result<()> {
    for entry in entries {
        let output_path = destination.join(entry.get_name());

        match entry {
            Entry::DirEntry { files, .. } => {
                fs::create_dir_all(&output_path).map_err(|source| {
                    Error::CreateArchiveDirectory {
                        path: output_path.clone(),
                        source,
                    }
                })?;
                unpack_entries(archive, files, &output_path)?;
            }

            Entry::FileEntry { data, .. } => {
                let contents = entry_data(archive, data, &output_path)?;

                let mut output = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&output_path)
                    .map_err(|source| Error::WriteArchiveEntry {
                        path: output_path.clone(),
                        source,
                    })?;

                output
                    .write_all(contents)
                    .map_err(|source| Error::WriteArchiveEntry {
                        path: output_path.clone(),
                        source,
                    })?;
            }
        }
    }

    Ok(())
}

fn entry_data<'a>(archive: &'a U8Arc<'_>, data: &'a FileEntry, path: &Path) -> Result<&'a [u8]> {
    match data {
        FileEntry::Data(data) => Ok(data),

        FileEntry::Ref { offset, length } => {
            let start = usize::try_from(*offset).map_err(|_| Error::InvalidArchiveData {
                path: path.to_path_buf(),
            })?;
            let length = usize::try_from(*length).map_err(|_| Error::InvalidArchiveData {
                path: path.to_path_buf(),
            })?;
            let end = start
                .checked_add(length)
                .ok_or_else(|| Error::InvalidArchiveData {
                    path: path.to_path_buf(),
                })?;

            archive
                .get_data()
                .get(start..end)
                .ok_or_else(|| Error::InvalidArchiveData {
                    path: path.to_path_buf(),
                })
        }
    }
}

pub(crate) fn pack_archive(source: &Path) -> Result<Vec<u8>> {
    let mut archive = U8Arc::new();

    for entry in WalkDir::new(source) {
        let entry = entry.map_err(|source_error| Error::Walk {
            root: source.to_path_buf(),
            source: source_error,
        })?;
        if !entry.file_type().is_file() {
            continue;
        }

        let relative_path = entry
            .path()
            .strip_prefix(source)
            .map_err(|_source| Error::InvalidArchiveEntryPath(entry.path().to_path_buf()))?;
        let archive_path = relative_path
            .to_str()
            .ok_or_else(|| Error::InvalidArchiveEntryPath(entry.path().to_path_buf()))?
            .replace('\\', "/");
        let data = fs::read(entry.path()).map_err(|source| Error::ReadArchiveEntry {
            path: entry.path().to_path_buf(),
            source,
        })?;

        drop(
            archive
                .add_entry_data(&archive_path, data)
                .ok_or_else(|| Error::InvalidArchiveEntryPath(entry.path().to_path_buf()))?,
        );
    }

    archive
        .write_to_vec()
        .map_err(|source_error| Error::PackU8 {
            path: source.to_path_buf(),
            source: Box::new(source_error),
        })
}
