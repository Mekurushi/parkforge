use super::error::{Error, Result};

const TEXT_SECTION_COUNT: usize = 7;
const DATA_SECTION_COUNT: usize = 11;
const SECTION_COUNT: usize = TEXT_SECTION_COUNT + DATA_SECTION_COUNT;
const SECTION_OFFSETS_OFFSET: usize = 0x00;
const SECTION_ADDRESSES_OFFSET: usize = 0x48;
const SECTION_SIZES_OFFSET: usize = 0x90;
const BSS_ADDRESS_OFFSET: usize = 0xD8;
const BSS_SIZE_OFFSET: usize = 0xDC;
const ENTRY_POINT_OFFSET: usize = 0xE0;
const RESERVED_OFFSET: usize = 0xE4;
const RESERVED_WORD_COUNT: usize = 7;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Dol {
    data: Vec<u8>,
    sections: Vec<DolSection>,
    bss_address: u32,
    bss_size: u32,
    entry_point_address: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct DolSection {
    offset: u32,
    address: u32,
    size: u32,
}

impl Dol {
    pub(super) fn parse(data: Vec<u8>) -> Result<Self> {
        let mut sections = Vec::with_capacity(SECTION_COUNT);
        for index in 0..SECTION_COUNT {
            sections.push(DolSection {
                offset: read_u32(&data, SECTION_OFFSETS_OFFSET + index * 4)?,
                address: read_u32(&data, SECTION_ADDRESSES_OFFSET + index * 4)?,
                size: read_u32(&data, SECTION_SIZES_OFFSET + index * 4)?,
            });
        }

        let bss_address = read_u32(&data, BSS_ADDRESS_OFFSET)?;
        let bss_size = read_u32(&data, BSS_SIZE_OFFSET)?;
        let entry_point_address = read_u32(&data, ENTRY_POINT_OFFSET)?;

        for index in 0..RESERVED_WORD_COUNT {
            let offset = RESERVED_OFFSET + index * 4;
            let value = read_u32(&data, offset)?;
            if value != 0 {
                return Err(Error::NonzeroReservedHeader { offset, value });
            }
        }

        Ok(Self {
            data,
            sections,
            bss_address,
            bss_size,
            entry_point_address,
        })
    }

    #[must_use]
    pub(super) fn sections(&self) -> &[DolSection] {
        &self.sections
    }

    pub(super) fn section_mut(&mut self, index: usize) -> Result<&mut DolSection> {
        self.sections
            .get_mut(index)
            .ok_or(Error::InvalidSectionIndex { index })
    }

    #[must_use]
    pub(super) const fn bss_address(&self) -> u32 {
        self.bss_address
    }

    #[must_use]
    pub(super) const fn bss_size(&self) -> u32 {
        self.bss_size
    }

    #[must_use]
    pub(super) const fn entry_point_address(&self) -> u32 {
        self.entry_point_address
    }

    #[must_use]
    pub(super) fn address_to_offset(&self, address: u32) -> Option<u32> {
        self.sections.iter().find_map(|section| {
            section
                .contains_address(address)
                .then(|| address - section.address + section.offset)
        })
    }

    #[must_use]
    pub(super) fn offset_to_address(&self, offset: u32) -> Option<u32> {
        self.sections.iter().find_map(|section| {
            section
                .contains_offset(offset)
                .then(|| offset - section.offset + section.address)
        })
    }

    #[must_use]
    pub(super) fn offset_to_section_index(&self, offset: u32) -> Option<usize> {
        self.sections
            .iter()
            .position(|section| section.contains_offset(offset))
    }

    pub(super) fn read(&self, address: u32, length: usize) -> Result<&[u8]> {
        let offset = self
            .address_to_offset(address)
            .ok_or(Error::UnmappedAddress { address })?;
        let offset = usize::try_from(offset).map_err(|_| Error::InvalidFileOffset { offset })?;
        let end = offset.checked_add(length).ok_or(Error::ReadOutsideFile {
            offset,
            length,
            file_length: self.data.len(),
        })?;
        self.data.get(offset..end).ok_or(Error::ReadOutsideFile {
            offset,
            length,
            file_length: self.data.len(),
        })
    }

    pub(super) fn write(&mut self, address: u32, bytes: &[u8]) -> Result<()> {
        let offset = self
            .address_to_offset(address)
            .ok_or(Error::UnmappedAddress { address })?;
        let offset = usize::try_from(offset).map_err(|_| Error::InvalidFileOffset { offset })?;
        let end = offset
            .checked_add(bytes.len())
            .ok_or(Error::WriteTooLarge { offset })?;
        if end > self.data.len() {
            self.data.resize(end, 0);
        }
        self.data[offset..end].copy_from_slice(bytes);
        Ok(())
    }

    pub(super) fn read_u32(&self, address: u32) -> Result<u32> {
        let bytes = self.read(address, 4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub(super) fn write_u32(&mut self, address: u32, value: u32) -> Result<()> {
        self.write(address, &value.to_be_bytes())
    }

    pub(super) fn into_bytes(mut self) -> Vec<u8> {
        self.write_header();
        self.data
    }

    fn write_header(&mut self) {
        for (index, section) in self.sections.iter().enumerate() {
            write_u32(
                &mut self.data,
                SECTION_OFFSETS_OFFSET + index * 4,
                section.offset,
            );
            write_u32(
                &mut self.data,
                SECTION_ADDRESSES_OFFSET + index * 4,
                section.address,
            );
            write_u32(
                &mut self.data,
                SECTION_SIZES_OFFSET + index * 4,
                section.size,
            );
        }
        write_u32(&mut self.data, BSS_ADDRESS_OFFSET, self.bss_address);
        write_u32(&mut self.data, BSS_SIZE_OFFSET, self.bss_size);
        write_u32(&mut self.data, ENTRY_POINT_OFFSET, self.entry_point_address);
    }
}

impl DolSection {
    #[must_use]
    pub(super) const fn offset(self) -> u32 {
        self.offset
    }

    #[must_use]
    pub(super) const fn address(self) -> u32 {
        self.address
    }

    #[must_use]
    pub(super) const fn size(self) -> u32 {
        self.size
    }

    pub(super) fn set_offset(&mut self, offset: u32) {
        self.offset = offset;
    }

    pub(super) fn set_address(&mut self, address: u32) {
        self.address = address;
    }

    pub(super) fn set_size(&mut self, size: u32) {
        self.size = size;
    }

    #[must_use]
    pub(super) fn contains_address(self, address: u32) -> bool {
        u64::from(self.address) <= u64::from(address)
            && u64::from(address) < u64::from(self.address) + u64::from(self.size)
    }

    #[must_use]
    pub(super) fn contains_offset(self, offset: u32) -> bool {
        u64::from(self.offset) <= u64::from(offset)
            && u64::from(offset) < u64::from(self.offset) + u64::from(self.size)
    }
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32> {
    let bytes = data
        .get(offset..offset + 4)
        .ok_or(Error::ReadU32 { offset })?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn write_u32(data: &mut [u8], offset: usize, value: u32) {
    data[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}
