use super::binary::Dol;
use super::error::{Error, Result};
use serde::Deserialize;

const FREE_SPACE_SECTION_INDEX: usize = 2;
const DEFAULT_THREAD_STACK_SIZE: u32 = 0x1_0000;

pub(super) struct Patchlet {
    pub(super) address: u32,
    pub(super) data: Vec<u8>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) struct PatchMetadata {
    pub(super) free_space_start: u32,
    pub(super) original_dol_size: u32,
    pub(super) original_free_space_ram_address: u32,
    pub(super) default_thread_stack_end_high_instruction: u32,
    pub(super) default_thread_stack_end_low_instruction: u32,
    pub(super) default_thread_stack_base_high_instruction: u32,
    pub(super) default_thread_stack_base_low_instruction: u32,
    pub(super) mem1_arena_low_high_instruction: u32,
    pub(super) mem1_arena_low_low_instruction: u32,
    pub(super) startup_stack_pointer_high_instruction: u32,
    pub(super) startup_stack_pointer_low_instruction: u32,
}

pub(super) fn apply(dol: &mut Dol, patchlets: &[Patchlet], metadata: &PatchMetadata) -> Result<()> {
    for patchlet in patchlets {
        if patchlet.address >= metadata.free_space_start {
            add_free_space_section(dol, &patchlet.data, metadata)?;
        } else {
            dol.write(patchlet.address, &patchlet.data)?;
        }
    }
    Ok(())
}

fn add_free_space_section(dol: &mut Dol, new_bytes: &[u8], metadata: &PatchMetadata) -> Result<()> {
    let patch_length = u32::try_from(new_bytes.len()).map_err(|_| Error::PatchLengthTooLarge {
        length: new_bytes.len(),
    })?;

    let section = dol.section_mut(FREE_SPACE_SECTION_INDEX)?;
    if section.size() != 0 {
        return Err(Error::MultipleFreeSpaceDirectives);
    }
    section.set_offset(metadata.original_dol_size);
    section.set_address(metadata.original_free_space_ram_address);
    section.set_size(patch_length);

    dol.write(metadata.original_free_space_ram_address, new_bytes)?;

    let padded_patch_length = patch_length
        .checked_add(3)
        .ok_or(Error::FreeSpacePointerOverflow)?
        & !3;
    let default_thread_stack_end = metadata
        .original_free_space_ram_address
        .checked_add(padded_patch_length)
        .ok_or(Error::FreeSpacePointerOverflow)?;
    let (high_halfword, low_halfword) = split_pointer(default_thread_stack_end);

    dol.write_u32(
        metadata.default_thread_stack_end_high_instruction,
        0x3CA0_0000 | high_halfword,
    )?;
    dol.write_u32(
        metadata.default_thread_stack_end_low_instruction,
        0x38A5_0000 | low_halfword,
    )?;

    let default_thread_stack_base = default_thread_stack_end
        .checked_add(DEFAULT_THREAD_STACK_SIZE)
        .ok_or(Error::FreeSpacePointerOverflow)?;
    let (high_halfword, low_halfword) = split_pointer(default_thread_stack_base);

    dol.write_u32(
        metadata.default_thread_stack_base_high_instruction,
        0x3C60_0000 | high_halfword,
    )?;
    dol.write_u32(
        metadata.default_thread_stack_base_low_instruction,
        0x3863_0000 | low_halfword,
    )?;
    dol.write_u32(
        metadata.mem1_arena_low_high_instruction,
        0x3C60_0000 | high_halfword,
    )?;
    dol.write_u32(
        metadata.mem1_arena_low_low_instruction,
        0x3863_0000 | low_halfword,
    )?;

    let high_halfword = (default_thread_stack_base & 0xFFFF_0000) >> 16;
    let low_halfword = default_thread_stack_base & 0xFFFF;
    dol.write_u32(
        metadata.startup_stack_pointer_high_instruction,
        0x3C20_0000 | high_halfword,
    )?;
    dol.write_u32(
        metadata.startup_stack_pointer_low_instruction,
        0x6021_0000 | low_halfword,
    )?;

    Ok(())
}

fn split_pointer(pointer: u32) -> (u32, u32) {
    let mut high_halfword = (pointer & 0xFFFF_0000) >> 16;
    let low_halfword = pointer & 0xFFFF;

    if low_halfword >= 0x8000 {
        high_halfword += 1;
    }

    (high_halfword, low_halfword)
}
