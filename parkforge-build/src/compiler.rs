use std::fs;
use std::path::Path;

use crate::{Error, Result, fsc};

pub(crate) trait Compiler: Sync {
    fn compile(&self, input: &Path, output: &Path) -> Result<()>;
}

pub(crate) fn find(name: &str) -> Option<&'static dyn Compiler> {
    match name {
        "copy" => Some(&CopyCompiler),
        "fsc" => Some(&fsc::FscCompiler),
        _ => None,
    }
}

struct CopyCompiler;

impl Compiler for CopyCompiler {
    fn compile(&self, input: &Path, output: &Path) -> Result<()> {
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).map_err(|source| Error::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::copy(input, output).map_err(|source| Error::Io {
            path: output.to_path_buf(),
            source,
        })?;
        Ok(())
    }
}
