use crate::error::{Error, Result};
use binrw::{BinWrite, BinWriterExt};
use disc_riider::builder::build_from_directory;
use disc_riider::structs::WiiPartType;
use disc_riider::{Fst, FstNode, WiiIsoReader, WiiPartitionReadInfo};
use std::fs;
use std::fs::{OpenOptions, create_dir_all};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
// copied directly from disc_riider; kept exactly the same for now, maybe changing some stuff soon

struct Section {
    part: String,
    fst: Fst,
    partition_reader: WiiPartitionReadInfo,
}

static NEXT_REBUILD_FILE: AtomicUsize = AtomicUsize::new(0);

pub struct WiiIsoExtractor {
    iso: WiiIsoReader<fs::File>,
    sections_to_extract: Vec<Section>,
}

pub fn binrw_write_file(p: &Path, value: &impl for<'a> BinWrite<Args<'a> = ()>) -> Result<()> {
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

impl WiiIsoExtractor {
    pub fn new(path: &Path) -> Result<Self> {
        let iso_file = fs::File::open(path).map_err(|source| Error::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let iso = WiiIsoReader::open(iso_file).map_err(|e| Error::Parse {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;
        Ok(WiiIsoExtractor {
            iso,
            sections_to_extract: vec![],
        })
    }

    pub fn prepare_extract_section(&mut self, mut section: String) -> Result<()> {
        section.make_ascii_uppercase();
        if self.sections_to_extract.iter().any(|s| s.part == section) {
            return Err(Error::DuplicateSection(section));
        }
        let part_type = match section.as_str() {
            "DATA" => WiiPartType::Data,
            "CHANNEL" => WiiPartType::Channel,
            "UPDATE" => WiiPartType::Update,
            _ => return Err(Error::UnknownSection(section)),
        };
        let partition = self
            .iso
            .partitions()
            .iter()
            .find(|p| p.get_type() == part_type)
            .cloned()
            .ok_or_else(|| Error::MissingSection(section.clone()))?;

        let partition_reader = self
            .iso
            .open_partition(partition)
            .map_err(|e| Error::OpenPartition(format!("{e:?}")))?;
        self.sections_to_extract.push(Section {
            part: section,
            fst: partition_reader.get_fst().clone(),
            partition_reader,
        });
        Ok(())
    }
    pub fn extract_to<F>(&mut self, path: &Path, mut progress: F) -> Result<()>
    where
        F: FnMut(u32),
    {
        progress(0);
        let disc_header = self.iso.get_header().clone();
        let region = *self.iso.get_region();
        for mut partition in self.sections_to_extract.drain(..) {
            let section_path = path.join(&partition.part);

            let section_path_disk = section_path.join("disc");
            create_dir_all(&section_path_disk).map_err(|e| Error::Io {
                path: section_path_disk.clone(),
                source: e,
            })?;

            binrw_write_file(&section_path_disk.join("header.bin"), &disc_header)?;
            fs::write(section_path_disk.join("region.bin"), region).map_err(|e| Error::Io {
                path: section_path_disk.clone(),
                source: e,
            })?;

            partition
                .partition_reader
                .extract_system_files(&section_path, &mut self.iso)
                .map_err(|e| Error::BinRw {
                    path: section_path.clone(),
                    source: e,
                })?;
            let mut buffer = vec![0; 0x10_000];
            // count files
            let mut total_bytes = 0usize;
            partition
                .fst
                .callback_all_files::<std::io::Error, _>(&mut |_, node| {
                    if let FstNode::File { length, .. } = node {
                        total_bytes += *length as usize;
                    }

                    Ok(())
                })
                .map_err(|e| Error::Io {
                    path: section_path_disk.clone(),
                    source: e,
                })?;

            let mut done_bytes = 0usize;
            {
                let mut wii_encrypt_reader =
                    partition.partition_reader.get_crypto_reader(&mut self.iso);
                partition
                    .fst
                    .callback_all_files::<std::io::Error, _>(&mut |names, node| {
                        if let FstNode::File { offset, length, .. } = node {
                            let mut filepath = section_path.join("files");
                            for name in names {
                                filepath.push(name);
                            }
                            let parent = filepath.parent().ok_or_else(|| {
                                std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    "ISO file path has no parent directory",
                                )
                            })?;
                            create_dir_all(parent)?;

                            let mut outfile = fs::File::create(&filepath)?;
                            wii_encrypt_reader.seek(SeekFrom::Start(*offset))?;
                            let mut bytes_left = *length as usize;
                            loop {
                                let bytes_to_read = bytes_left.min(buffer.len());
                                let bytes_read =
                                    wii_encrypt_reader.read(&mut buffer[..bytes_to_read])?;
                                if bytes_read == 0 {
                                    break;
                                }

                                outfile.write_all(&buffer[..bytes_read])?;
                                done_bytes += bytes_read;
                                bytes_left -= bytes_read;

                                let percent = done_bytes
                                    .saturating_mul(100)
                                    .checked_div(total_bytes)
                                    .unwrap_or(100)
                                    .min(100);
                                let done_percent = u32::try_from(percent).map_err(|_error| {
                                    std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        "ISO extraction progress is out of range",
                                    )
                                })?;
                                progress(done_percent);
                            }
                        }

                        Ok(())
                    })
                    .map_err(|source| Error::Io {
                        path: section_path.clone(),
                        source,
                    })?;
            }

            let certs = partition
                .partition_reader
                .read_certificates(&mut self.iso)
                .map_err(|e| Error::ReadCertificates(e.to_string()))?;
            binrw_write_file(&section_path.join("cert.bin"), &certs)?;
            let tmd = partition
                .partition_reader
                .read_tmd(&mut self.iso)
                .map_err(|e| Error::ReadTmd(e.to_string()))?;
            binrw_write_file(&section_path.join("tmd.bin"), &tmd)?;
            binrw_write_file(
                &section_path.join("ticket.bin"),
                &partition.partition_reader.get_partition_header().ticket,
            )?;
        }
        Ok(())
    }
}
pub fn rebuild_from_directory<F>(src_dir: &Path, dest_path: &Path, mut progress: F) -> Result<()>
where
    F: FnMut(u32),
{
    let parent = dest_path.parent().ok_or_else(|| Error::Rebuild {
        path: dest_path.to_path_buf(),
        message: "destination ISO path has no parent directory".to_owned(),
    })?;
    fs::create_dir_all(parent).map_err(|source| Error::Io {
        path: parent.to_path_buf(),
        source,
    })?;
    let temporary = temporary_output_path(dest_path);
    let mut dest_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|source| Error::Io {
            path: temporary.clone(),
            source,
        })?;
    if let Err(error) = build_from_directory(src_dir, &mut dest_file, &mut |done_percent| {
        progress(u32::from(done_percent));
    }) {
        let _ = fs::remove_file(&temporary);
        return Err(Error::Rebuild {
            path: src_dir.to_path_buf(),
            message: error.to_string(),
        });
    }
    drop(dest_file);
    fs::rename(&temporary, dest_path).map_err(|source| {
        let _ = fs::remove_file(&temporary);
        Error::Io {
            path: dest_path.to_path_buf(),
            source,
        }
    })?;
    Ok(())
}

fn temporary_output_path(destination: &Path) -> PathBuf {
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("output.iso");
    destination.with_file_name(format!(
        ".{name}.rebuilding-{}-{}",
        std::process::id(),
        NEXT_REBUILD_FILE.fetch_add(1, Ordering::Relaxed)
    ))
}
