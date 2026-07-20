use std::fs;
use std::path::Path;

use fsc_assembler::Assembler;

use crate::compiler::Compiler;
use crate::{Error, Result};

pub(crate) struct FscCompiler;

impl Compiler for FscCompiler {
    fn compile(&self, input: &Path, output: &Path) -> Result<()> {
        compile(input, output)
    }
}

fn compile(input: &Path, output: &Path) -> Result<()> {
    // TODO: Use the FSC compiler crate's centralized entrypoint once available.
    let source = fs::read_to_string(input).map_err(|source| Error::Io {
        path: input.to_path_buf(),
        source,
    })?;
    let script = fsc_parse::parse(&source).map_err(|error| Error::CompilerFailed {
        compiler: "fsc".to_owned(),
        path: input.to_path_buf(),
        message: format!("parse error: {error:?}"),
    })?;
    let hir = fsc_sema::analyze(&script).map_err(|error| Error::CompilerFailed {
        compiler: "fsc".to_owned(),
        path: input.to_path_buf(),
        message: format!("semantic analysis error: {error}"),
    })?;
    let mut assembler = Assembler::new();
    fsc_codegen::compile(&hir, &mut assembler).map_err(|error| Error::CompilerFailed {
        compiler: "fsc".to_owned(),
        path: input.to_path_buf(),
        message: format!("code generation error: {error}"),
    })?;

    let name = input
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| Error::CompilerFailed {
            compiler: "fsc".to_owned(),
            path: input.to_path_buf(),
            message: "input file has no valid UTF-8 stem".to_owned(),
        })?;
    let binary = assembler
        .finalize(name.to_owned())
        .map_err(|error| Error::CompilerFailed {
            compiler: "fsc".to_owned(),
            path: input.to_path_buf(),
            message: format!("assembly finalization error: {error}"),
        })?;
    let data = binary.serialize().map_err(|error| Error::CompilerFailed {
        compiler: "fsc".to_owned(),
        path: input.to_path_buf(),
        message: format!("serialization error: {error}"),
    })?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|source| Error::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(output, data).map_err(|source| Error::Io {
        path: output.to_path_buf(),
        source,
    })
}
