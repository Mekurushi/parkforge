use crate::error::{Error, Result};
use crate::game_id::parse_game_id;
use crate::partition::PartitionKind;
use crate::progress::Progress;
use binrw::{BinWrite, BinWriterExt};
use disc_riider::{Fst, FstNode, WiiIsoReader, WiiPartitionReadInfo};
use std::fs;
use std::fs::create_dir_all;
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;

struct PreparedPartition {
    kind: PartitionKind,
    fst: Fst,
    partition_reader: WiiPartitionReadInfo,
}


pub struct WiiIsoExtractor<R: Read + Seek> {
    iso: WiiIsoReader<R>,
    partitions_to_extract: Vec<PreparedPartition>,
}

fn binrw_write_file(p: &Path, value: &impl for<'a> BinWrite<Args<'a> = ()>) -> Result<()> {
    let mut f = fs::File::create(p).map_err(|e| Error::Io {
        path: p.to_path_buf(),
        source: e,
    })?;
    f.write_be(value).map_err(|e| Error::BinRw {
        path: p.to_path_buf(),
        source: e,
    })?;
    Ok(())
}

impl<R: Read + Seek> WiiIsoExtractor<R> {
    pub fn from_reader(reader: R) -> Result<Self> {
        let iso = WiiIsoReader::open(reader).map_err(|source| Error::ParseIso { source })?;
        Ok(WiiIsoExtractor {
            iso,
            partitions_to_extract: vec![],
        })
    }

    pub fn game_id(&self) -> Result<parkforge_types::GameId> {
        parse_game_id(self.iso.get_header().game_id)
    }

    pub fn prepare_partition(&mut self, kind: PartitionKind) -> Result<()> {
        if self
            .partitions_to_extract
            .iter()
            .any(|partition| partition.kind == kind)
        {
            return Err(Error::DuplicatePartition(kind));
        }
        let partition = self
            .iso
            .partitions()
            .iter()
            .find(|partition| partition.get_type() == kind.as_disc_riider_type())
            .cloned()
            .ok_or(Error::MissingPartition(kind))?;

        let partition_reader =
            self.iso
                .open_partition(partition)
                .map_err(|source| Error::OpenPartition {
                    partition: kind,
                    source,
                })?;
        self.partitions_to_extract.push(PreparedPartition {
            kind,
            fst: partition_reader.get_fst().clone(),
            partition_reader,
        });
        Ok(())
    }
    pub fn extract_to<F>(&mut self, path: &Path, mut progress: F) -> Result<()>
    where
        F: FnMut(Progress),
    {
        prepare_destination(path)?;
        let total_bytes = total_file_bytes(&self.partitions_to_extract);
        progress(Progress::new(0, total_bytes));
        let disc_header = self.iso.get_header().clone();
        let region = *self.iso.get_region();
        let mut done_bytes = 0_u64;
        for mut partition in self.partitions_to_extract.drain(..) {
            let partition_path = path.join(partition.kind.as_str());

            let partition_path_disk = partition_path.join("disc");
            create_dir_all(&partition_path_disk).map_err(|e| Error::Io {
                path: partition_path_disk.clone(),
                source: e,
            })?;

            binrw_write_file(&partition_path_disk.join("header.bin"), &disc_header)?;
            fs::write(partition_path_disk.join("region.bin"), region).map_err(|e| Error::Io {
                path: partition_path_disk.clone(),
                source: e,
            })?;

            partition
                .partition_reader
                .extract_system_files(&partition_path, &mut self.iso)
                .map_err(|e| Error::BinRw {
                    path: partition_path.clone(),
                    source: e,
                })?;
            let mut buffer = vec![0; 0x10_000];
            {
                let mut wii_encrypt_reader =
                    partition.partition_reader.get_crypto_reader(&mut self.iso);
                partition
                    .fst
                    .callback_all_files::<std::io::Error, _>(&mut |names, node| {
                        if let FstNode::File { offset, length, .. } = node {
                            let mut filepath = partition_path.join("files");
                            for name in names {
                                filepath.push(name);
                            }
                            let parent = filepath.parent().ok_or_else(|| {
                                std::io::Error::new(
                                    ErrorKind::InvalidData,
                                    "ISO file path has no parent directory",
                                )
                            })?;
                            create_dir_all(parent)?;

                            let mut outfile = fs::File::create(&filepath)?;
                            let _position = wii_encrypt_reader.seek(SeekFrom::Start(*offset))?;
                            let mut bytes_left = *length as usize;
                            loop {
                                let bytes_to_read = bytes_left.min(buffer.len());
                                let bytes_read =
                                    wii_encrypt_reader.read(&mut buffer[..bytes_to_read])?;
                                if bytes_read == 0 {
                                    break;
                                }

                                outfile.write_all(&buffer[..bytes_read])?;
                                done_bytes += u64::try_from(bytes_read).map_err(|_error| {
                                    std::io::Error::new(
                                        ErrorKind::InvalidData,
                                        "ISO read size does not fit into progress",
                                    )
                                })?;
                                bytes_left -= bytes_read;
                                progress(Progress::new(done_bytes, total_bytes));
                            }
                        }

                        Ok(())
                    })
                    .map_err(|source| Error::Io {
                        path: partition_path.clone(),
                        source,
                    })?;
            }

            let certs = partition
                .partition_reader
                .read_certificates(&mut self.iso)
                .map_err(|source| Error::ReadCertificates {
                    partition: partition.kind,
                    source,
                })?;
            binrw_write_file(&partition_path.join("cert.bin"), &certs)?;
            let tmd = partition
                .partition_reader
                .read_tmd(&mut self.iso)
                .map_err(|source| Error::ReadTmd {
                    partition: partition.kind,
                    source,
                })?;
            binrw_write_file(&partition_path.join("tmd.bin"), &tmd)?;
            binrw_write_file(
                &partition_path.join("ticket.bin"),
                &partition.partition_reader.get_partition_header().ticket,
            )?;
        }
        Ok(())
    }
}

fn total_file_bytes(partitions: &[PreparedPartition]) -> u64 {
    partitions
        .iter()
        .map(|partition| total_node_bytes(partition.fst.get_entries()))
        .sum()
}

fn total_node_bytes(nodes: &[FstNode]) -> u64 {
    nodes
        .iter()
        .map(|node| match node {
            FstNode::File { length, .. } => u64::from(*length),
            FstNode::Directory { files, .. } => total_node_bytes(files),
        })
        .sum()
}

impl WiiIsoExtractor<fs::File> {
    pub fn open(path: &Path) -> Result<Self> {
        let reader = fs::File::open(path).map_err(|source| Error::OpenIso {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_reader(reader)
    }
}


fn prepare_destination(path: &Path) -> Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == ErrorKind::NotFound => {
            return create_dir_all(path).map_err(|source| Error::Io {
                path: path.to_path_buf(),
                source,
            });
        }
        Err(source) => {
            return Err(Error::Io {
                path: path.to_path_buf(),
                source,
            });
        }
    };

    if metadata.file_type().is_symlink() {
        return Err(Error::DestinationIsSymlink(path.to_path_buf()));
    }
    if !metadata.is_dir() {
        return Err(Error::DestinationIsNotDirectory(path.to_path_buf()));
    }

    let mut entries = fs::read_dir(path).map_err(|source| Error::Io {
        path: path.to_path_buf(),
        source,
    })?;
    match entries.next() {
        Some(Ok(_)) => Err(Error::DestinationIsNotEmpty(path.to_path_buf())),
        Some(Err(source)) => Err(Error::Io {
            path: path.to_path_buf(),
            source,
        }),
        None => Ok(()),
    }
}