mod error;
mod staged_directory;

// TODO: check if it's possible to better abstract and encapsulate the whole
// temporary/staging/complex filesystem so it's more deduplicated with only one source of truth

pub use error::{Error, Result};
pub use staged_directory::StagedDirectory;
