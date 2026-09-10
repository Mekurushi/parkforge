mod u8arc;

pub(crate) use u8arc::{pack_archive, unpack_archive};
#[derive(Copy, Clone)]
pub(crate) enum ArchiveFormat {
    U8,
}
