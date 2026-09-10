use crate::error::{Error, Result};

pub(crate) fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    nlzss11::decompress(data).map_err(|source| Error::DecompressNlzss11 { source })
}
