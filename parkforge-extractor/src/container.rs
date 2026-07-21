use std::collections::{HashSet, VecDeque};
use std::fs;
use std::path::{Component, Path, PathBuf};

use nlzss11::{compress, decompress};
use u8arc::U8Arc;
use walkdir_minimal::WalkDir;

use crate::error::{Error, Result};
use crate::hash::{FileHash, hash_file};
use crate::manifest::{
    CompressionFormat, ContainerFormat, ContainerId, Manifest, NewContainer, NewFile,
};
use parkforge_model::{GameId, MANIFEST_FILE_NAME, VirtualPath};

#[derive(Debug, Clone, Copy)]
struct ContainerKind {
    format: ContainerFormat,
    compression: CompressionFormat,
}

impl ContainerKind {
    fn detect(path: &Path) -> Option<Self> {
        match path.extension().and_then(|ext| ext.to_str())? {
            "dac" => Some(Self {
                format: ContainerFormat::U8,
                compression: CompressionFormat::Nlzss11,
            }),
            "dan" => Some(Self {
                format: ContainerFormat::U8,
                compression: CompressionFormat::None,
            }),
            _ => None,
        }
    }
}

struct PendingContainer {
    path: PathBuf,
    kind: ContainerKind,
    parent: Option<ContainerId>,
    internal_path: Option<String>,
}

pub fn extract_archives(workspace_root: &Path, game_id: GameId) -> Result<Manifest> {
    let mut manifest = Manifest::new(game_id);

    let mut queue: VecDeque<PendingContainer> = walk_files(workspace_root)?
        .into_iter()
        .filter_map(|path| {
            let kind = ContainerKind::detect(&path)?;
            Some(PendingContainer {
                path,
                kind,
                parent: None,
                internal_path: None,
            })
        })
        .collect();

    while let Some(job) = queue.pop_front() {
        let nested = extract_container(workspace_root, job, &mut manifest)?;
        queue.extend(nested);
    }

    add_loose_files(workspace_root, &mut manifest)?;

    Ok(manifest)
}

pub fn repack_archives(workspace_root: &Path, manifest: &Manifest) -> Result<()> {
    let mut containers: Vec<_> = manifest
        .containers()
        .map(|(_, container)| container)
        .collect();
    containers.sort_by_key(|container| {
        std::cmp::Reverse(container.virtual_path.as_str().matches('/').count())
    });

    for container in containers {
        let path = container.virtual_path.to_path_under(workspace_root);
        if !path.is_dir() {
            return Err(Error::Rebuild {
                path,
                message: "archive container is not an extracted directory".to_owned(),
            });
        }
        let archive = match container.format {
            ContainerFormat::U8 => pack_directory(&path, &path)?,
        };
        let packed = match container.compression {
            CompressionFormat::None => archive,
            CompressionFormat::Nlzss11 => compress(&archive),
        };
        fs::remove_dir_all(&path).map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        fs::write(&path, packed).map_err(|source| Error::Io { path, source })?;
    }
    Ok(())
}

fn pack_directory(directory: &Path, archive_path: &Path) -> Result<Vec<u8>> {
    let mut archive = U8Arc::new();
    add_directory_entries(&mut archive, directory, directory, archive_path)?;
    archive.write_to_vec().map_err(|error| Error::Archive {
        path: archive_path.to_path_buf(),
        message: error.to_string(),
    })
}

fn add_directory_entries(
    archive: &mut U8Arc<'static>,
    root: &Path,
    directory: &Path,
    archive_path: &Path,
) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(directory)
        .map_err(|source| Error::Io {
            path: directory.to_path_buf(),
            source,
        })?
        .collect::<std::result::Result<_, _>>()
        .map_err(|source| Error::Io {
            path: directory.to_path_buf(),
            source,
        })?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    if entries.is_empty() {
        return Err(Error::EmptyArchiveDirectory {
            archive: archive_path.to_path_buf(),
            directory: directory.to_path_buf(),
        });
    }

    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
        if file_type.is_dir() {
            add_directory_entries(archive, root, &path, archive_path)?;
        } else if file_type.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_error| Error::OutsideWorkspace {
                    path: path.clone(),
                    root: root.to_path_buf(),
                })?;
            let name = relative
                .to_str()
                .ok_or_else(|| Error::InvalidUtf8Path { path: path.clone() })?
                .replace(std::path::MAIN_SEPARATOR, "/");
            let data = fs::read(&path).map_err(|source| Error::Io {
                path: path.clone(),
                source,
            })?;
            archive.add_entry_data(&name, data);
        } else {
            return Err(Error::Rebuild {
                path,
                message: "archive contents must contain only regular files and directories"
                    .to_owned(),
            });
        }
    }
    Ok(())
}

fn extract_container(
    workspace_root: &Path,
    job: PendingContainer,
    manifest: &mut Manifest,
) -> Result<Vec<PendingContainer>> {
    let PendingContainer {
        path: container_path,
        kind,
        parent,
        internal_path,
    } = job;

    let raw = fs::read(&container_path).map_err(|e| Error::Io {
        path: container_path.clone(),
        source: e,
    })?;

    let unpacked = match kind.compression {
        CompressionFormat::None => raw,
        CompressionFormat::Nlzss11 => decompress(&raw).map_err(|e| Error::Decompress {
            path: container_path.clone(),
            message: e.to_string(),
        })?,
    };

    let arc = match kind.format {
        ContainerFormat::U8 => U8Arc::read_vec(unpacked).map_err(|e| Error::Archive {
            path: container_path.clone(),
            message: e.to_string(),
        })?,
    };

    let container_virtual_path = virtual_path_from_workspace_path(workspace_root, &container_path)?;
    let container_id = manifest.add_container(NewContainer {
        virtual_path: container_virtual_path.clone(),
        format: kind.format,
        compression: kind.compression,
        parent,
        internal_path,
    })?;

    let staging_dir = staging_path_for(&container_path);
    if staging_dir.exists() {
        fs::remove_dir_all(&staging_dir).map_err(|e| Error::Io {
            path: staging_dir.clone(),
            source: e,
        })?;
    }

    let mut nested = Vec::new();
    for entry_path in arc.get_all_paths() {
        let data = arc
            .get_entry_data(&entry_path)
            .ok_or_else(|| Error::Archive {
                path: container_path.clone(),
                message: format!("entry {entry_path:?} is listed but has no data"),
            })?;

        let relative = validate_archive_entry_path(&container_path, &entry_path)?;
        let out_path = staging_dir.join(&relative);
        if let Some(parent_dir) = out_path.parent() {
            fs::create_dir_all(parent_dir).map_err(|e| Error::Io {
                path: parent_dir.to_path_buf(),
                source: e,
            })?;
        }
        fs::write(&out_path, data).map_err(|e| Error::Io {
            path: out_path.clone(),
            source: e,
        })?;

        if let Some(nested_kind) = ContainerKind::detect(&out_path) {
            nested.push((relative, nested_kind));
        } else {
            let size = u64::try_from(data.len()).map_err(|_error| Error::ManifestSizeOverflow)?;
            manifest.add_file(NewFile {
                virtual_path: container_virtual_path.join(&relative)?,
                container: Some(container_id),
                internal_path: Some(relative),
                hash: FileHash::sha256_of(data),
                size,
            });
        }
    }

    fs::remove_file(&container_path).map_err(|e| Error::Io {
        path: container_path.clone(),
        source: e,
    })?;
    fs::rename(&staging_dir, &container_path).map_err(|e| Error::Io {
        path: container_path.clone(),
        source: e,
    })?;

    let pending = nested
        .into_iter()
        .map(|(relative, nested_kind)| PendingContainer {
            path: container_path.join(&relative),
            kind: nested_kind,
            parent: Some(container_id),
            internal_path: Some(relative),
        })
        .collect();

    Ok(pending)
}

fn staging_path_for(container_path: &Path) -> PathBuf {
    container_path.with_added_extension("staging")
}

fn validate_archive_entry_path(container_path: &Path, entry_path: &str) -> Result<String> {
    // normalizing . because u8arc returns . as segment for the player data
    let relative = entry_path.strip_prefix('/').unwrap_or(entry_path);

    if relative.is_empty() {
        return Err(Error::InvalidArchiveEntryPath {
            path: container_path.to_path_buf(),
            entry: entry_path.to_owned(),
            reason: "must not be empty",
        });
    }

    let mut segments = Vec::new();

    for segment in relative.split('/') {
        if segment.is_empty() {
            return Err(Error::InvalidArchiveEntryPath {
                path: container_path.to_path_buf(),
                entry: entry_path.to_owned(),
                reason: "must not contain empty segments",
            });
        }

        if segment == "." {
            continue;
        }

        if segment == ".." {
            return Err(Error::InvalidArchiveEntryPath {
                path: container_path.to_path_buf(),
                entry: entry_path.to_owned(),
                reason: "must not contain parent directory segments",
            });
        }

        if segment.contains('\\') {
            return Err(Error::InvalidArchiveEntryPath {
                path: container_path.to_path_buf(),
                entry: entry_path.to_owned(),
                reason: "must use forward slashes",
            });
        }

        segments.push(segment);
    }

    if segments.is_empty() {
        return Err(Error::InvalidArchiveEntryPath {
            path: container_path.to_path_buf(),
            entry: entry_path.to_owned(),
            reason: "must not resolve to an empty path",
        });
    }

    Ok(segments.join("/"))
}

fn add_loose_files(workspace_root: &Path, manifest: &mut Manifest) -> Result<()> {
    let known: HashSet<String> = manifest
        .files()
        .map(|file| file.virtual_path.as_str().to_owned())
        .collect();

    for path in walk_files(workspace_root)? {
        if path.file_name().and_then(|name| name.to_str()) == Some(MANIFEST_FILE_NAME) {
            continue;
        }

        let virtual_path = virtual_path_from_workspace_path(workspace_root, &path)?;
        if known.contains(virtual_path.as_str()) {
            continue;
        }

        let (hash, size) = hash_file(&path)?;
        manifest.add_file(NewFile {
            virtual_path,
            container: None,
            internal_path: None,
            hash,
            size,
        });
    }

    Ok(())
}

fn virtual_path_from_workspace_path(workspace_root: &Path, path: &Path) -> Result<VirtualPath> {
    let relative = path
        .strip_prefix(workspace_root)
        .map_err(|_error| Error::OutsideWorkspace {
            path: path.to_path_buf(),
            root: workspace_root.to_path_buf(),
        })?;

    let mut segments = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(part) => segments.push(
                part.to_str()
                    .ok_or_else(|| Error::InvalidUtf8Path {
                        path: path.to_path_buf(),
                    })?
                    .to_owned(),
            ),
            Component::CurDir => {}
            _ => {
                return Err(Error::OutsideWorkspace {
                    path: path.to_path_buf(),
                    root: workspace_root.to_path_buf(),
                });
            }
        }
    }

    VirtualPath::new(segments.join("/")).map_err(Error::from)
}

fn walk_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let walker = WalkDir::new(dir).map_err(|e| Error::WalkDir {
        path: dir.to_path_buf(),
        message: e.to_string(),
    })?;

    let mut files = Vec::new();
    for entry in walker {
        let entry = entry.map_err(|e| Error::WalkDir {
            path: dir.to_path_buf(),
            message: e.to_string(),
        })?;

        let is_file = entry
            .file_type()
            .map_err(|e| Error::WalkDir {
                path: entry.path().to_path_buf(),
                message: e.to_string(),
            })?
            .is_file();

        if is_file {
            files.push(entry.path().to_path_buf());
        }
    }

    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use nlzss11::decompress;
    use u8arc::U8Arc;

    use super::repack_archives;
    use crate::manifest::{CompressionFormat, ContainerFormat, Manifest, NewContainer};

    static NEXT_TEST_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

    fn test_directory() -> PathBuf {
        std::env::temp_dir().join(format!(
            "parkforge-container-test-{}-{}",
            std::process::id(),
            NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn repack_archives_packs_nested_containers_before_their_parents()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = test_directory();
        let outer_path = root.join("DATA/files/outer.dan");
        let nested_path = outer_path.join("nested.dac");
        fs::create_dir_all(&nested_path)?;
        fs::write(outer_path.join("top.bin"), b"top")?;
        fs::write(nested_path.join("inner.bin"), b"inner")?;

        let mut manifest = Manifest::new("R8AJ01".parse()?);
        let outer = manifest.add_container(NewContainer {
            virtual_path: "DATA/files/outer.dan".parse()?,
            format: ContainerFormat::U8,
            compression: CompressionFormat::None,
            parent: None,
            internal_path: None,
        })?;
        manifest.add_container(NewContainer {
            virtual_path: "DATA/files/outer.dan/nested.dac".parse()?,
            format: ContainerFormat::U8,
            compression: CompressionFormat::Nlzss11,
            parent: Some(outer),
            internal_path: Some("nested.dac".to_owned()),
        })?;

        repack_archives(&root, &manifest)?;

        assert!(outer_path.is_file());
        let outer = U8Arc::read_vec(fs::read(&outer_path)?)?;
        assert_eq!(outer.get_entry_data("top.bin"), Some(&b"top"[..]));
        let nested = decompress(outer.get_entry_data("nested.dac").ok_or("missing nested")?)?;
        let nested = U8Arc::read_vec(nested)?;
        assert_eq!(nested.get_entry_data("inner.bin"), Some(&b"inner"[..]));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn repack_archives_rejects_empty_directories() -> Result<(), Box<dyn std::error::Error>> {
        let root = test_directory();
        let archive_path = root.join("DATA/files/empty.dan");
        fs::create_dir_all(&archive_path)?;
        let mut manifest = Manifest::new("R8AJ01".parse()?);
        manifest.add_container(NewContainer {
            virtual_path: "DATA/files/empty.dan".parse()?,
            format: ContainerFormat::U8,
            compression: CompressionFormat::None,
            parent: None,
            internal_path: None,
        })?;

        assert!(repack_archives(&root, &manifest).is_err());
        assert!(archive_path.is_dir());

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
