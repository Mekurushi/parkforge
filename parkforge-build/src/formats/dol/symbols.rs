use std::collections::BTreeMap;

use serde::Deserialize;

use super::error::{Error, Result};

pub(super) struct CustomSymbols {
    symbols: BTreeMap<String, u32>,
}

impl CustomSymbols {
    pub(super) fn parse(text: &str) -> Result<Self> {
        let file: CustomSymbolsFile =
            serde_yaml_ng::from_str(text).map_err(|source| Error::ParseCustomSymbols { source })?;
        Ok(Self {
            symbols: file.main_dol,
        })
    }

    pub(super) fn get(&self, name: &str) -> Option<u32> {
        self.symbols.get(name).copied()
    }
}

#[derive(Deserialize)]
struct CustomSymbolsFile {
    #[serde(rename = "main.dol")]
    main_dol: BTreeMap<String, u32>,
}
