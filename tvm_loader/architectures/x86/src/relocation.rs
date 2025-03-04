//! ELF relocation code for the `x86` architecture.

use tvm_loader::elf_loader::{FinalizedRelocation, RelocationInformation};

pub fn handle_relocation(info: &RelocationInformation) -> Result<FinalizedRelocation> {
    let relocation = match info.relocation_type {
        8 => FinalizedRelocation::Bits32(
            info.slide
                .checked_add_signed(info.addend)
                .ok_or(())?
                .try_into::<u32>()
                .map_err(|_| ())?,
        ),
        _ => return Err(()),
    };

    Ok(relocation)
}
