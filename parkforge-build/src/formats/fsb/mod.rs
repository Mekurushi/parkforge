mod config;
mod diagnostic;
mod error;
mod operation;

pub use error::Error;
pub(crate) use operation::{Compile, Patch};
