use crate::error::{Error, Result};

pub(crate) fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    nlzss11::decompress(data).map_err(|source| Error::DecompressNlzss11 { source })
}

pub(crate) fn compress(data: &[u8]) -> Vec<u8> {
    nlzss11::compress(data)
}
