//! Support code for `x86_64` systems booted using `limine`.

#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn loader_main() {
    loop {}
}

#[panic_handler]
pub fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
