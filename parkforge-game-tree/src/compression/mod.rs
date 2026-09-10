mod nlzss11;

pub(crate) use nlzss11::{compress, decompress};
#[derive(Copy, Clone)]
pub(crate) enum CompressionFormat {
    None,
    Nlzss11,
}
