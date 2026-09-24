use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use parkforge_types::BuildConfig;
use rlb_domain::{RLBFile, Row};
use serde::Deserialize;

use super::value::SourceValue;
use crate::diagnostic::BuildDiagnostic;
use crate::error::{Error, Result, RlbValueLocation};

pub(crate) struct Patch {
    directory: PathBuf,
    source: PathBuf,
    target: PathBuf,
}

impl Patch {
    pub(crate) fn new(directory: PathBuf, relative_directory: &Path) -> Result<Self> {
        let target = relative_directory.with_extension("");
        let target_name = target
            .file_name()
            .ok_or_else(|| Error::UnsupportedSourceFormat(directory.clone()))?;
        let source = directory.join(Path::new(target_name).with_extension("toml"));
        Ok(Self {
            directory,
            source,
            target,
        })
    }

    pub(crate) fn directory(&self) -> &Path {
        &self.directory
    }

    pub(crate) fn target(&self) -> &Path {
        &self.target
    }

    pub(crate) fn check(
        &self,
        config: &BuildConfig,
        _diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
    ) -> Result<()> {
        PatchDocument::read(&self.source)?.check(&self.source, config)
    }

    pub(crate) fn process(
        &self,
        staging_root: &Path,
        config: &BuildConfig,
        _diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
    ) -> Result<()> {
        let target = staging_root.join(&self.target);
        let source = PatchDocument::read(&self.source)?;
        let bytes = fs::read(&target).map_err(|source| Error::ReadRlb {
            path: target.clone(),
            source,
        })?;
        let mut file = RLBFile::parse(&bytes).map_err(|source| Error::ParseRlb {
            path: target.clone(),
            source,
        })?;

        source.apply(&self.source, &target, config, &mut file)?;

        let bytes = file.write().map_err(|source| Error::SerializeRlb {
            path: target.clone(),
            source,
        })?;
        fs::write(&target, bytes).map_err(|source| Error::WriteRlb {
            path: target,
            source,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PatchDocument {
    #[serde(default)]
    edits: Vec<Edit>,
    #[serde(default)]
    removes: Vec<Remove>,
    #[serde(default)]
    appends: Vec<Append>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Edit {
    table: String,
    row: usize,
    #[serde(default)]
    set: BTreeMap<String, SourceValue>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Remove {
    table: String,
    row: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Append {
    table: String,
    #[serde(default)]
    row: BTreeMap<String, SourceValue>,
}

impl PatchDocument {
    pub(crate) fn read(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path).map_err(|source| Error::ReadRlbSource {
            path: path.to_path_buf(),
            source,
        })?;
        toml::from_str(&text).map_err(|source| Error::ParseRlbSource {
            path: path.to_path_buf(),
            source,
        })
    }

    pub(crate) fn check(&self, path: &Path, config: &BuildConfig) -> Result<()> {
        let mut assigned = HashSet::new();
        for edit in &self.edits {
            for (field, value) in &edit.set {
                if !assigned.insert((edit.table.as_str(), edit.row, field.as_str())) {
                    return Err(Error::DuplicateRlbAssignment {
                        path: path.to_path_buf(),
                        table: edit.table.clone(),
                        row: edit.row,
                        field: field.clone(),
                    });
                }
                drop(value.resolve(config).map_err(|error| {
                    error.into_build_error(
                        path,
                        &edit.table,
                        RlbValueLocation::Row(edit.row),
                        field,
                    )
                })?);
            }
        }

        let mut removed = HashSet::new();
        for remove in &self.removes {
            if !removed.insert((remove.table.as_str(), remove.row)) {
                return Err(Error::DuplicateRlbRemoval {
                    path: path.to_path_buf(),
                    table: remove.table.clone(),
                    row: remove.row,
                });
            }
        }

        for (append_index, append) in self.appends.iter().enumerate() {
            for (field, value) in &append.row {
                drop(value.resolve(config).map_err(|error| {
                    error.into_build_error(
                        path,
                        &append.table,
                        RlbValueLocation::Append(append_index),
                        field,
                    )
                })?);
            }
        }
        Ok(())
    }

    pub(crate) fn apply(
        &self,
        path: &Path,
        target: &Path,
        config: &BuildConfig,
        file: &mut RLBFile,
    ) -> Result<()> {
        for edit in &self.edits {
            if edit.set.is_empty() {
                continue;
            }

            let table = file
                .tables()
                .map_err(|source| Error::InspectRlb {
                    path: target.to_path_buf(),
                    source,
                })?
                .into_iter()
                .find(|table| table.label == edit.table)
                .ok_or_else(|| Error::MissingRlbTable {
                    path: path.to_path_buf(),
                    target: target.to_path_buf(),
                    table: edit.table.clone(),
                })?;
            let table_id = table.id;

            // rlb patches are only based on the source; for that we're doing all edits first;
            // afterward all removes in descending order and only then appending new rows

            for (field, source_value) in &edit.set {
                let value = source_value.resolve(config).map_err(|error| {
                    error.into_build_error(
                        path,
                        &edit.table,
                        RlbValueLocation::Row(edit.row),
                        field,
                    )
                })?;
                file.set_field(table_id, edit.row, field, value)
                    .map_err(|source| Error::PatchRlb {
                        path: path.to_path_buf(),
                        target: target.to_path_buf(),
                        table: edit.table.clone(),
                        row: edit.row,
                        field: field.clone(),
                        source,
                    })?;
            }
        }

        let mut remove_tables = Vec::new();
        for remove in &self.removes {
            if !remove_tables.contains(&remove.table.as_str()) {
                remove_tables.push(remove.table.as_str());
            }
        }

        for table_label in remove_tables {
            let table = file
                .tables()
                .map_err(|source| Error::InspectRlb {
                    path: target.to_path_buf(),
                    source,
                })?
                .into_iter()
                .find(|table| table.label == table_label)
                .ok_or_else(|| Error::MissingRlbTable {
                    path: path.to_path_buf(),
                    target: target.to_path_buf(),
                    table: table_label.to_owned(),
                })?;
            let table_id = table.id;

            let mut rows = self
                .removes
                .iter()
                .filter(|remove| remove.table == table_label)
                .map(|remove| remove.row)
                .collect::<Vec<_>>();
            rows.sort_unstable_by(|left, right| right.cmp(left));

            for row in rows {
                file.remove_row(table_id, row)
                    .map_err(|source| Error::RemoveRlbRow {
                        path: path.to_path_buf(),
                        target: target.to_path_buf(),
                        table: table_label.to_owned(),
                        row,
                        source,
                    })?;
            }
        }

        for (append_index, append) in self.appends.iter().enumerate() {
            let table = file
                .tables()
                .map_err(|source| Error::InspectRlb {
                    path: target.to_path_buf(),
                    source,
                })?
                .into_iter()
                .find(|table| table.label == append.table)
                .ok_or_else(|| Error::MissingRlbTable {
                    path: path.to_path_buf(),
                    target: target.to_path_buf(),
                    table: append.table.clone(),
                })?;
            let table_id = table.id;

            let mut row = Row::new();
            for (field, source_value) in &append.row {
                let value = source_value.resolve(config).map_err(|error| {
                    error.into_build_error(
                        path,
                        &append.table,
                        RlbValueLocation::Append(append_index),
                        field,
                    )
                })?;
                drop(row.insert(field, value));
            }

            let _row_index =
                file.append_row(table_id, row)
                    .map_err(|source| Error::AppendRlbRow {
                        path: path.to_path_buf(),
                        target: target.to_path_buf(),
                        table: append.table.clone(),
                        append: append_index,
                        source,
                    })?;
        }
        Ok(())
    }
}
