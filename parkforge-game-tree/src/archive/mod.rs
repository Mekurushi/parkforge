mod u8arc;

pub(crate) use u8arc::unpack_archive;
#[derive(Copy, Clone)]
pub(crate) enum ArchiveFormat {
    U8,
}
