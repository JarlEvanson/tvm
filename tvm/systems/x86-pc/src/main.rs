//! Implementation of `tvm` for `x86` PC systems.

#![no_std]
#![no_main]

use boot_info::BootInfo;

/// Test message.
static MESSAGE: &str = "Test";

#[unsafe(no_mangle)]
extern "C" fn _start(boot_info: *mut BootInfo) -> u32 {
    unsafe { ((*boot_info).write)(MESSAGE.as_ptr(), MESSAGE.len()) }

    todo!()
}

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
