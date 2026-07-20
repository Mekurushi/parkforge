mod compiler;
mod config;
mod error;
mod fsc;
mod pipeline;
mod staging;

use std::path::Path;

pub use config::{BuildConfig, BuildRule};
pub use error::{Error, Result};

pub struct BuildRequest<'a> {
    pub game_id: &'a str,
    pub original_root: &'a Path,
    pub source_root: &'a Path,
    pub build_root: &'a Path,
    pub config: &'a BuildConfig,
}

pub fn build(request: &BuildRequest<'_>) -> Result<()> {
    pipeline::build(request)
}
