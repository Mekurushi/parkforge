use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use parkforge_types::BuildConfig;
use rlb_domain::{RLBFile, Row};
use serde::Deserialize;

use super::error::{Error, Result, RlbValueLocation};
use super::value::SourceValue;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CreateDocument {
    #[serde(default)]
    tables: Vec<Table>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Table {
    label: String,
    #[serde(default)]
    rows: Vec<BTreeMap<String, SourceValue>>,
}

impl CreateDocument {
    pub(super) fn read(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path).map_err(|source| Error::ReadRlbSource {
            path: path.to_path_buf(),
            source,
        })?;
        toml::from_str(&text).map_err(|source| Error::ParseRlbSource {
            path: path.to_path_buf(),
            source,
        })
    }

    pub(super) fn check(&self, path: &Path, config: &BuildConfig) -> Result<()> {
        for table in &self.tables {
            for (row_index, row) in table.rows.iter().enumerate() {
                for (field, value) in row {
                    drop(value.resolve(config).map_err(|error| {
                        error.into_build_error(
                            path,
                            &table.label,
                            RlbValueLocation::Row(row_index),
                            field,
                        )
                    })?);
                }
            }
        }
        Ok(())
    }

    pub(super) fn create(
        &self,
        path: &Path,
        target: &Path,
        config: &BuildConfig,
    ) -> Result<RLBFile> {
        // TODO: chech whether min 1 table should be enforced
        let mut file = RLBFile::new();
        for table in &self.tables {
            let mut rows = Vec::with_capacity(table.rows.len());
            for (row_index, source_row) in table.rows.iter().enumerate() {
                let mut row = Row::new();
                for (field, source_value) in source_row {
                    let value = source_value.resolve(config).map_err(|error| {
                        error.into_build_error(
                            path,
                            &table.label,
                            RlbValueLocation::Row(row_index),
                            field,
                        )
                    })?;
                    drop(row.insert(field, value));
                }
                rows.push(row);
            }
            let _table_id =
                file.create_table(&table.label, rows)
                    .map_err(|source| Error::CreateRlbTable {
                        path: path.to_path_buf(),
                        target: target.to_path_buf(),
                        table: table.label.clone(),
                        source,
                    })?;
        }
        Ok(file)
    }
}
