//! Support code for generic `x86_64` PC-type systems.

#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn tvm_main() {
    loop {}
}

#[panic_handler]
pub fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
