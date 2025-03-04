//! `tvm_loader` for `x86_64` systems to boot into `tvm` using UEFI.

#![no_std]
#![no_main]

use tvm_loader::{
    elf_loader::{Machine, get_machine, load_application},
    log_error,
};
use tvm_loader_uefi::unsafe_entry_point;
use tvm_loader_x86::paging::pae::PaePageTables;
use tvm_loader_x86_64::X86_64PageTable;

extern crate tvm_loader_uefi;

unsafe_entry_point!(main);

fn main() -> Result<(), tvm_loader_uefi::Status> {
    let mut switch_space =
        X86_64PageTable::new_current().expect("failed to construct switch tables");

    let embedded_image = tvm_loader_uefi::embedded::get_tvm_image();
    let machine = get_machine(embedded_image).expect("invalid embedded tvm image");
    match machine {
        Machine::INTEL_386 => {
            let mut application_space =
                PaePageTables::new_max_supported().expect("failed to construct application tables");

            let result = load_application(
                embedded_image,
                Machine::INTEL_386,
                &mut application_space,
                tvm_loader_x86::handle_relocation,
            );

            let entry_point = result.expect("failed to load tvm image");
        }
        Machine::X86_64 => {
            let mut application_space = X86_64PageTable::new_max_supported()
                .expect("failed to construct application tables");

            let result = load_application(
                embedded_image,
                Machine::X86_64,
                &mut application_space,
                tvm_loader_x86_64::handle_relocation,
            );

            let entry_point = result.expect("failed to load tvm image");
        }
        machine => unimplemented!("{machine:?} is not supported"),
    }

    Ok(())
}

/// Handles all panics that occur.
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    log_error!("{info}");

    loop {}
}
