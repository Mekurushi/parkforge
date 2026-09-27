use std::path::Path;

use parkforge_types::BuildConfig;

use crate::build::validate_target_ownership;
use crate::diagnostic::BuildDiagnostic;
use crate::error::{Error, Result};
use crate::operation::Operation;
use crate::paths::absolute_directory;

#[derive(Debug)]
pub struct CheckReport {
    errors: Vec<Error>,
}

impl CheckReport {
    #[must_use]
    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    #[must_use]
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    #[must_use]
    pub fn into_errors(self) -> Vec<Error> {
        self.errors
    }
}

pub fn check<D>(sources: &Path, config: &BuildConfig, mut diagnostics: D) -> Result<CheckReport>
where
    D: for<'a> FnMut(BuildDiagnostic<'a>),
{
    let sources = absolute_directory(sources, false)?;
    let operations = Operation::discover(&sources)?;
    validate_target_ownership(&operations)?;

    let mut errors = Vec::new();
    for operation in &operations {
        if let Err(error) = operation.check(config, &mut diagnostics) {
            errors.push(error);
        }
    }

    Ok(CheckReport { errors })
}
