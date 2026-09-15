use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::diagnostic::BuildDiagnostic;
use crate::error::{Error, Result};
use crate::operation::Operation;
use crate::paths::{absolute_directory, validate_no_overlap};
use crate::staging::Staging;

pub enum BuildProgress {
    CopyingOriginal,
    Operations { completed: usize, total: usize },
    Committing,
}

pub fn build<P, D>(
    original: &Path,
    sources: &Path,
    destination: &Path,
    mut progress: P,
    mut diagnostics: D,
) -> Result<()>
where
    P: FnMut(BuildProgress),
    D: for<'a> FnMut(BuildDiagnostic<'a>),
{
    Build::new(original, sources, destination)?.run(&mut progress, &mut diagnostics)
}

struct Build {
    original: PathBuf,
    operations: Vec<Operation>,
    staging: Staging,
}

impl Build {
    fn new(original: &Path, sources: &Path, destination: &Path) -> Result<Self> {
        let original = absolute_directory(original, false)?;
        let sources = absolute_directory(sources, false)?;
        let operations = Operation::discover(&sources)?;
        validate_target_ownership(&operations)?;
        let staging = Staging::new(destination)?;
        validate_no_overlap(&original, staging.destination())?;
        validate_no_overlap(&sources, staging.destination())?;

        Ok(Self {
            original,
            operations,
            staging,
        })
    }

    fn run(
        self,
        progress: &mut impl FnMut(BuildProgress),
        diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
    ) -> Result<()> {
        progress(BuildProgress::CopyingOriginal);
        self.copy_tree()?;
        self.apply_operations(progress, diagnostics)?;
        progress(BuildProgress::Committing);
        self.staging.commit()
    }

    fn apply_operations(
        &self,
        progress: &mut impl FnMut(BuildProgress),
        diagnostics: &mut impl for<'a> FnMut(BuildDiagnostic<'a>),
    ) -> Result<()> {
        let total = self.operations.len();
        progress(BuildProgress::Operations {
            completed: 0,
            total,
        });
        for (index, operation) in self.operations.iter().enumerate() {
            operation.process(self.staging.path(), diagnostics)?;
            progress(BuildProgress::Operations {
                completed: index + 1,
                total,
            });
        }
        Ok(())
    }

    fn copy_tree(&self) -> Result<()> {
        let original = self.original.as_path();
        let destination = self.staging.path();
        for entry in WalkDir::new(original) {
            let entry = entry.map_err(|source| Error::WalkOriginal {
                root: original.to_path_buf(),
                source,
            })?;
            let input = entry.path();
            if entry.file_type().is_symlink() {
                return Err(Error::SymbolicLink(input.to_path_buf()));
            }
            let relative = input
                .strip_prefix(original)
                .map_err(|source| Error::RelativeEntry {
                    path: input.to_path_buf(),
                    root: original.to_path_buf(),
                    source,
                })?;
            let output = destination.join(relative);
            if entry.file_type().is_dir() {
                fs::create_dir_all(&output).map_err(|source| Error::CreateDirectory {
                    path: output,
                    source,
                })?;
            } else if entry.file_type().is_file() {
                let _ = fs::copy(input, &output).map_err(|source| Error::CopyFile {
                    input: input.to_path_buf(),
                    output,
                    source,
                })?;
            } else {
                return Err(Error::UnsupportedEntry(input.to_path_buf()));
            }
        }
        Ok(())
    }
}

fn validate_target_ownership(operations: &[Operation]) -> Result<()> {
    let mut owners = HashMap::new();
    for operation in operations {
        if let Some(first) = owners.insert(operation.target(), operation.source()) {
            return Err(Error::ConflictingTarget {
                target: operation.target().to_path_buf(),
                first: first.to_path_buf(),
                second: operation.source().to_path_buf(),
            });
        }
    }
    Ok(())
}
