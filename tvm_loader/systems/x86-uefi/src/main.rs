//! `tvm_loader` for `x86` systems to boot into `tvm` using UEFI.

#![no_std]
#![no_main]

use alloc::boxed::Box;
use tvm_loader::{
    elf_loader::{Machine, get_machine, load_application},
    log_error, log_info,
};
use tvm_loader_uefi::unsafe_entry_point;
use tvm_loader_x86::paging::pae::PaePageTables;
use x86_common::PagingMode;

extern crate alloc;
extern crate tvm_loader_uefi;

unsafe_entry_point!(main);

fn main() -> Result<(), tvm_loader_uefi::Status> {
    let embedded_image = tvm_loader_uefi::embedded::get_tvm_image();

    log_info!("{:?}", x86_common::current_paging_mode());
    log_info!("{:?}", x86_common::max_supported_paging_mode());

    let machine = get_machine(embedded_image).expect("invalid embedded tvm image");
    match machine {
        Machine::INTEL_386 => {
            let mut page_tables = match x86_common::max_supported_paging_mode() {
                PagingMode::Disabled => unreachable!(),
                PagingMode::Bits32 => todo!(),
                PagingMode::Pae => Box::new(PaePageTables::new_max_supported().unwrap()),
                PagingMode::Level4 | PagingMode::Level5 => unreachable!(),
            };

            let entry_point = load_application(
                embedded_image,
                Machine::INTEL_386,
                page_tables.as_mut(),
                |info| todo!(),
            )
            .unwrap();

            let mut switch_space = match x86_common::current_paging_mode() {
                PagingMode::Disabled => todo!(),
                PagingMode::Bits32 => todo!(),
                PagingMode::Pae => Box::new(PaePageTables::new_current().unwrap()),
                PagingMode::Level4 | PagingMode::Level5 => unreachable!(),
            };
            let result = tvm_loader_x86_common::switch(
                switch_space.as_mut(),
                page_tables.as_mut(),
                entry_point,
            );

            log_info!("{result:?}");

            todo!()
        }
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
