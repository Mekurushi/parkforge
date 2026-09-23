use std::ffi::OsStr;
use std::path::Path;

use walkdir::DirEntry;

use crate::error::{Error, Result};

// TODO: collapse Classifications
pub(crate) enum Classification {
    LooseFile(Format),
    PatchDirectory(Format),
}

pub(crate) enum Format {
    Fsb,
    Rlb,
}

pub(crate) fn classify(entry: &DirEntry) -> Result<Option<Classification>> {
    let source = entry.path();
    let unsupported = || Error::UnsupportedSourceFormat(source.to_path_buf());

    match entry {
        entry if is_patch_directory(entry) => {
            let target = source.file_stem().map(Path::new).ok_or_else(unsupported)?;
            match target.extension() {
                Some(extension) if extension == OsStr::new("fsb") => {
                    Ok(Some(Classification::PatchDirectory(Format::Fsb)))
                }
                Some(extension) if extension == OsStr::new("rlb") => {
                    Ok(Some(Classification::PatchDirectory(Format::Rlb)))
                }
                _ => Err(unsupported()),
            }
        }
        entry if entry.file_type().is_file() => match source.extension() {
            Some(extension) if extension == OsStr::new("fsc") => {
                Ok(Some(Classification::LooseFile(Format::Fsb)))
            }
            Some(extension)
                if extension == OsStr::new("toml")
                    && source.file_stem().map(Path::new).and_then(Path::extension)
                        == Some(OsStr::new("rlb")) =>
            {
                Ok(Some(Classification::LooseFile(Format::Rlb)))
            }
            _ => Err(unsupported()),
        },
        entry if entry.file_type().is_dir() => Ok(None),
        _ => Err(unsupported()),
    }
}

pub(crate) fn is_patch_directory(entry: &DirEntry) -> bool {
    entry.file_type().is_dir() && entry.path().extension() == Some(OsStr::new("patch"))
}
