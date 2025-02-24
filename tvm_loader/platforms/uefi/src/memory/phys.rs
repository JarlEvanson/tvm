//! Implementation of [`FrameAllocator`] that uses UEFI Boot Services to manage physical memory.

use tvm_loader::{
    log_error, log_warn,
    memory::phys::{AllocationType, FrameAllocator, OutOfMemoryError},
    unsafe_frame_allocator,
};
use uefi::{memory::MemoryType, table::boot::AllocateType};

use crate::boot_services_ptr;

unsafe_frame_allocator!(UefiFrameAllocator);

/// Allocator using UEFI Boot Services page allocation services.
struct UefiFrameAllocator;

// SAFETY:
//
// Lifetimes of frame allocations are properly managed according to the UEFI specification for
// UEFI Boot Services.
unsafe impl FrameAllocator for UefiFrameAllocator {
    fn allocate(
        &self,
        allocation_type: AllocationType,
        count: u64,
        alignment: u64,
    ) -> Result<u64, OutOfMemoryError> {
        assert!(alignment.is_power_of_two() || alignment == 0);

        let alignment = alignment.max(self.frame_size());
        let (allocation_type, physical_address) = match allocation_type {
            AllocationType::Any => (AllocateType::ANY_PAGES, 0),
            AllocationType::Below { physical_address } => {
                (AllocateType::MAX_ADDRESS, physical_address)
            }
            AllocationType::At { physical_address } => (AllocateType::ADDRESS, physical_address),
        };

        let Some(boot_services_ptr) = boot_services_ptr() else {
            log_warn!("boot services are not available");
            return Err(OutOfMemoryError);
        };

        // SAFETY:
        //
        // `boot_services_ptr` is not NULL and so according to the UEFI specification, this service
        // should be available.
        let allocate_pages_func = unsafe { (*boot_services_ptr.as_ptr()).allocate_pages };
        // SAFETY:
        //
        // `boot_services_ptr` is not NULL and so according to the UEFI application, this service
        // should be available.
        let deallocate_pages_func = unsafe { (*boot_services_ptr.as_ptr()).free_pages };

        let extra_pages = alignment.div_ceil(self.frame_size()).saturating_sub(1);
        let page_count = usize::try_from(count.checked_add(extra_pages).ok_or(OutOfMemoryError)?)
            .map_err(|_| OutOfMemoryError)?;

        let mut base_address = physical_address;
        // SAFETY:
        //
        // `allocate_pages_func` came from a valid UEFI Boot Services Table, so it should be valid
        // to call.
        let status = unsafe {
            allocate_pages_func(
                allocation_type,
                MemoryType::RUNTIME_SERVICES_DATA,
                page_count,
                &mut base_address,
            )
        };
        if status.error() {
            log_error!("error allocating memory: {status}");
            return Err(OutOfMemoryError);
        }

        let allocated_address = base_address.next_multiple_of(alignment);
        if allocated_address != base_address {
            // SAFETY:
            //
            // `deallocate_pages_func` came from a valid UEFI Boot Services Table, so it should be
            // valid to call.
            let _ = unsafe {
                deallocate_pages_func(
                    base_address,
                    ((allocated_address - base_address) / self.frame_size()) as usize,
                )
            };
        }

        let offset_pages = (allocated_address - base_address) / self.frame_size();
        if extra_pages != 0 && offset_pages != extra_pages {
            // SAFETY:
            //
            // `deallocate_pages_func` came from a valid UEFI Boot Services Table, so it should be
            // valid to call.
            let _ = unsafe {
                deallocate_pages_func(
                    allocated_address + count * self.frame_size(),
                    (extra_pages - offset_pages) as usize,
                )
            };
        }

        Ok(allocated_address)
    }

    unsafe fn deallocate(&self, physical_address: u64, count: u64) {
        let Some(boot_services_ptr) = boot_services_ptr() else {
            log_warn!("boot services are not available");
            return;
        };

        // SAFETY:
        //
        // `boot_services_ptr` is not NULL and so according to the UEFI specification, this service
        // should be available.
        let free_pages_func = unsafe { (*boot_services_ptr.as_ptr()).free_pages };

        // SAFETY:
        //
        // `free_pages_func` came from a valid UEFI Boot Services Table, so it should be valid to
        // call.
        let status = unsafe { free_pages_func(physical_address, count as usize) };
        if status.error() {
            log_warn!("error deallocating memory: {status}");
        }
    }

    fn frame_size(&self) -> u64 {
        // According to the UEFI specification, 4 KiB is the size at which physical pages are
        // allocated.
        4096
    }
}
