//! ELF relocation code for the `x86` architecture.

use tvm_loader::elf_loader::{FinalizedRelocation, RelocationInformation};

/// Handles execution of ELF relocation entries on `x86_64`.
///
/// # Errors
///
/// Returns `()` if the relocation cannot be performed or is not supported.
#[expect(clippy::result_unit_err)]
pub fn handle_relocation(info: &RelocationInformation) -> Result<FinalizedRelocation, ()> {
    let relocation = match info.relocation_type {
        8 => FinalizedRelocation::Bits32(
            info.slide
                .checked_add_signed(info.addend)
                .ok_or(())?
                .try_into()
                .map_err(|_| ())?,
        ),
        _ => return Err(()),
    };

    Ok(relocation)
}
