//! `tvm_loader` for `x86` systems to boot into `tvm` using UEFI.

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
        Machine::X86_64 => todo!(),
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
