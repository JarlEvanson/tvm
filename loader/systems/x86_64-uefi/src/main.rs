//! Implementation of `tvm` loader for `x86_64` UEFI systems.

#![no_std]
#![no_main]

/// Entry point to UEFI binary.
#[no_mangle]
pub extern "efiapi" fn efi_main() -> usize {
    0
}

/// Panic handler for `tvm` loader for `x86_64` UEFI systems.
#[panic_handler]
pub fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
