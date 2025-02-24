//! Implementation of [`GlobalAlloc`][aga] that uses UEFI Boot Services pool management functions.
//!
//! [aga]: alloc::GlobalAlloc

use core::{alloc, ffi, ptr};

use tvm_loader::log_warn;
use uefi::{data_types::Status, memory::MemoryType};

use crate::boot_services_ptr;

/// Allocator using UEFI Boot Services pool allocations.
#[global_allocator]
static UEFI_ALLOCATOR: UefiAllocator = UefiAllocator;

/// Allocator using UEFI Boot Services pool allocations.
struct UefiAllocator;

// SAFETY:
//
// The implementation correctly implements [`GlobalAlloc`] using UEFI Boot Services `allocate_pool`
// and `free_pool`.
unsafe impl alloc::GlobalAlloc for UefiAllocator {
    unsafe fn alloc(&self, layout: alloc::Layout) -> *mut u8 {
        assert!(layout.align() <= 8);
        let Some(boot_services_ptr) = boot_services_ptr() else {
            return ptr::null_mut();
        };

        // SAFETY:
        //
        // `boot_services_ptr` is not NULL and so according to the UEFI specification, this
        // service should be available.
        let allocate_pool = unsafe { (*boot_services_ptr.as_ptr()).allocate_pool };

        let mut ptr = ptr::null_mut();
        // SAFETY:
        //
        // `allocate_pool` came from a valid UEFI Boot Services Table, so it should be valid to
        // call.
        let result =
            unsafe { allocate_pool(MemoryType::RUNTIME_SERVICES_DATA, layout.size(), &mut ptr) };
        if result != Status::SUCCESS {
            return ptr::null_mut();
        }

        ptr.cast::<u8>()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _: alloc::Layout) {
        let Some(boot_services_ptr) = boot_services_ptr() else {
            return;
        };

        // SAFETY:
        //
        // `boot_services_ptr` is not NULL and so according to the UEFI specification, this
        // service should be available.
        let free_pool = unsafe { (*boot_services_ptr.as_ptr()).free_pool };

        // SAFETY:
        //
        // `free_pool` came from a valid UEFI Boot Services Table, so it should be valid to call.
        let result = unsafe { free_pool(ptr.cast::<ffi::c_void>()) };
        if result != Status::SUCCESS {
            log_warn!("failed to deallocate: {ptr:p}");
        }
    }
}
