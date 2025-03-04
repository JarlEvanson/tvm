//! `tvm_loader` for `x86` systems to boot into `tvm` using UEFI.

#![no_std]
#![no_main]

use alloc::boxed::Box;
use tvm_loader::{
    elf_loader::{Machine, get_machine, load_application},
    log_error,
};
use tvm_loader_uefi::unsafe_entry_point;
use tvm_loader_x86::paging::{
    bits_32::Bits32PageTables, disabled::DisabledPaging, pae::PaePageTables,
};
use tvm_loader_x86_64::X86_64PageTable;
use tvm_loader_x86_common::paging::X86CommonAddressSpace;
use x86_common::{PagingMode, current_paging_mode, max_supported_paging_mode};

extern crate alloc;
extern crate tvm_loader_uefi;

unsafe_entry_point!(main);

fn main() -> Result<(), tvm_loader_uefi::Status> {
    let mut switch_space: Box<dyn X86CommonAddressSpace> = match current_paging_mode() {
        PagingMode::Disabled => Box::new(DisabledPaging),
        PagingMode::Bits32 => Box::new(Bits32PageTables::new()),
        PagingMode::Pae => {
            Box::new(PaePageTables::new_current().expect("failed to construct switch tables"))
        }
        PagingMode::Level4 | PagingMode::Level5 => {
            unreachable!("4-level and 5-level paging is not a x86 paging mode")
        }
    };

    let embedded_image = tvm_loader_uefi::embedded::get_tvm_image();
    let machine = get_machine(embedded_image).expect("invalid embedded tvm image");
    match machine {
        Machine::INTEL_386 => {
            let mut application_space: Box<dyn X86CommonAddressSpace> =
                match max_supported_paging_mode() {
                    PagingMode::Disabled => {
                        unreachable!("paging is always supported on 32-bit x86 processors")
                    }
                    PagingMode::Bits32 => Box::new(Bits32PageTables::new()),
                    PagingMode::Pae | PagingMode::Level4 | PagingMode::Level5 => Box::new(
                        PaePageTables::new_max_supported()
                            .expect("failed to construct application tables"),
                    ),
                };

            let result = load_application(
                embedded_image,
                Machine::INTEL_386,
                application_space.as_mut(),
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
