//! Support code for `x86_64` systems booted using `limine`.

#![no_std]
#![no_main]

/// A tag indicating the start of the `limine` requests section of this application.
#[link_section = ".limine_requests_start"]
pub static REQUESTS_START_MARKER: [u64; 4] = limine::REQUESTS_START_MARKER;
/// A tag indicating the end of the `limine` requests section of this application.
#[link_section = ".limine_requests_end"]
pub static REQUESTS_END_MARKER: [u64; 2] = limine::REQUESTS_END_MARKER;

/// A tag indicating that this application uses the `limine` boot protocol and requesting the
/// latest revision of the `limine` protocol.
#[link_section = ".limine_base_revision_tag"]
pub static BASE_REVISION_TAG: limine::BaseRevisionTag = limine::BaseRevisionTag::new();

/// A request for the higher half direct map offset from the bootloader.
#[used]
#[link_section = ".limine_requests"]
pub static HHDM: limine::Request<limine::hhdm::HhdmRequest> =
    limine::Request::new(limine::hhdm::HhdmRequest::new());

/// A request for the memory map from the bootloader.
#[used]
#[link_section = ".limine_requests"]
pub static MEMORY_MAP: limine::Request<limine::memory_map::MemoryMapRequest> =
    limine::Request::new(limine::memory_map::MemoryMapRequest::new());

#[no_mangle]
pub extern "C" fn loader_main() {
    if !BASE_REVISION_TAG.is_supported() {
        loop {}
    }

    loop {}
}

#[panic_handler]
pub fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
