//! ELF relocation code for the `x86_64` architecture.

use tvm_loader::elf_loader::{FinalizedRelocation, RelocationInformation};

/// Handles execution of ELF relocation entries on `x86_64`.
///
/// # Errors
///
/// Returns `()` if the relocation cannot be performed or is not supported.
#[expect(clippy::result_unit_err)]
pub fn handle_relocation(info: &RelocationInformation) -> Result<FinalizedRelocation, ()> {
    let relocation = match info.relocation_type {
        8 => FinalizedRelocation::Bits64(info.slide.checked_add_signed(info.addend).ok_or(())?),
        _ => return Err(()),
    };

    Ok(relocation)
}
