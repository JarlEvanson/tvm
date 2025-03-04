//! `tvm_loader` for `x86_64` systems to boot into `tvm` using UEFI.

#![no_std]
#![no_main]

use tvm_loader::{
    elf_loader::{Machine, get_machine},
    log_error,
};
use tvm_loader_uefi::unsafe_entry_point;

extern crate tvm_loader_uefi;

unsafe_entry_point!(main);

fn main() -> Result<(), tvm_loader_uefi::Status> {
    let embedded_image = tvm_loader_uefi::embedded::get_tvm_image();

    let machine = get_machine(embedded_image).expect("invalid embedded tvm image");
    match machine {
        Machine::INTEL_386 => todo!(),
        Machine::X86_64 => {
            let mut application_space = X86_64PageTable::new_max_supported().unwrap();

            let entry_point = load_application(
                embedded_image,
                Machine::X86_64,
                &mut application_space,
                tvm_loader_x86_64::handle_relocation,
            )
            .expect("entry point");

            let mut switch_space = X86_64PageTable::new_current().unwrap();
            let result = tvm_loader_x86_common::switch(
                &mut switch_space,
                &mut application_space,
                entry_point,
            );

            log_info!("{result:?}");
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
