//! Implementation of 32-bit paging.

use core::{error, fmt};

use tvm_loader::memory::{
    phys::{AllocationType, allocate_frame, deallocate_frame, frame_size},
    virt::{
        FreeRegion, FreeRegionTracker, MapError, NoMapping, NotMapped, ProtectionFlags,
        arch::AddressSpace,
        loader::{map, page_size, translate_phys},
    },
};
use tvm_loader_x86_common::paging::X86CommonAddressSpace;
use x86_common::PagingMode;

/// Bit indicating the translation region is present.
const PRESENT_BIT: u32 = 1 << 0;
/// Bit indicating the translation region is writable.
const WRITE_BIT: u32 = 1 << 1;

/// Mask of the physical address in a page table entry.
const ADDRESS_MASK: u32 = ((1 << 20) - 1) << 12;

/// A set of 32-bit paging tables.
pub struct Bits32PageTables {
    /// Manager of the free virtual regions.
    free_region_tracker: FreeRegionTracker,
    /// The physical address of the top table.
    physical_address: u32,
}

impl Bits32PageTables {
    /// Creates a new `x86` 32-bit page table scheme.
    ///
    /// # Panics
    ///
    /// Panics if the allocation and mapping of the top level page table fails.
    pub fn new() -> Self {
        let (_, physical_address) = atomic_map_new_table().expect("atomic table map failed");

        Self {
            free_region_tracker: FreeRegionTracker::new(&[FreeRegion {
                virtual_address: 0,
                length: u64::from(u32::MAX) + 1,
            }]),

            physical_address,
        }
    }

    /// Maps the physical region starting at `physical_address` and extending `count` pages into
    /// the virtual address space at `virtual_address`.
    ///
    /// # Safety
    ///
    /// The virtual and physical regions are valid and the virtual region has not been mapped.
    unsafe fn map_unchecked(
        &mut self,
        virtual_address: u32,
        physical_address: u32,
        count: u32,
        protection: ProtectionFlags,
    ) {
        let mut translated_flags = 0;
        if protection & ProtectionFlags::READ == ProtectionFlags::READ {
            translated_flags |= PRESENT_BIT;
        }
        if protection & ProtectionFlags::WRITE == ProtectionFlags::WRITE {
            translated_flags |= WRITE_BIT | PRESENT_BIT;
        }
        if protection & ProtectionFlags::EXECUTE == ProtectionFlags::EXECUTE {
            translated_flags |= PRESENT_BIT;
        }

        let pml2_virtual_address =
            translate_phys(u64::from(self.physical_address)).expect("validity constraint violated");
        // SAFETY:
        //
        // The top level page table was allocated and mapped into the loader address space when
        // this object was created.
        let pml2 = unsafe { &mut *(pml2_virtual_address as *mut [u32; 1024]) };

        for index in 0..count {
            let map_address = virtual_address + index * 4096;
            let mapped_address = physical_address + index * 4096;

            let pml2_index = ((map_address >> 22) & 0x2FF) as usize;
            let pml1_index = ((map_address >> 12) & 0x2FF) as usize;

            if pml2[pml2_index] & PRESENT_BIT != PRESENT_BIT {
                let (_, new_table_physical_address) =
                    atomic_map_new_table().expect("failed to atomically map new page table");

                pml2[pml2_index] = new_table_physical_address | WRITE_BIT | PRESENT_BIT;
            }

            let pml1_physical_address = pml2[pml2_index] & ADDRESS_MASK;
            let pml1_virtual_address = translate_phys(u64::from(pml1_physical_address))
                .expect("validity constraint violated");

            // SAFETY:
            //
            // The page tables are properly allocated and mapped in an atomic fashion.
            let pml1 = unsafe { &mut *(pml1_virtual_address as *mut [u32; 1024]) };
            pml1[pml1_index] = mapped_address | translated_flags;
        }
    }
}

impl AddressSpace for Bits32PageTables {
    fn map(
        &mut self,
        virtual_address: u64,
        physical_address: u64,
        count: u64,
        protection: ProtectionFlags,
    ) -> Result<(), MapError> {
        let byte_count = count
            .checked_mul(self.page_size())
            .ok_or(MapError::InvalidSize)?;
        let virtual_end_address = virtual_address.checked_add(byte_count);
        if virtual_end_address
            .is_none_or(|virtual_end_address| virtual_end_address > u64::from(u32::MAX))
        {
            return Err(MapError::InvalidAddress);
        }

        if self
            .free_region_tracker
            .allocate_region(virtual_address, byte_count)
            .is_err()
        {
            return Err(MapError::AlreadyMapped);
        }

        // SAFETY:
        //
        // The invariants required to call `map_unchecked` were just checked.
        unsafe {
            self.map_unchecked(
                virtual_address as u32,
                physical_address as u32,
                count as u32,
                protection,
            )
        }

        Ok(())
    }

    unsafe fn unmap(&mut self, virtual_address: u64, count: u64) -> Result<(), NotMapped> {
        debug_assert_eq!(
            virtual_address % self.page_size(),
            0,
            "virtual address not properly aligned"
        );

        let byte_count = count
            .checked_mul(self.page_size())
            .expect("mapping too large");
        let virtual_end_address = virtual_address
            .checked_add(byte_count)
            .expect("virtual region too large");
        assert!(
            virtual_end_address <= u64::from(u32::MAX),
            "virtual region is too large"
        );

        if self
            .free_region_tracker
            .deallocate_region(virtual_address, byte_count)
            .is_err()
        {
            return Err(NotMapped);
        }

        Ok(())
    }

    fn translate_virt(&self, virtual_address: u64) -> Result<u64, NoMapping> {
        if virtual_address > u64::from(u32::MAX) {
            return Err(NoMapping);
        }

        let pml2_index = ((virtual_address >> 22) & 0x2FF) as usize;
        let pml1_index = ((virtual_address >> 12) & 0x2FF) as usize;

        let pml2_virtual_address =
            translate_phys(u64::from(self.physical_address)).expect("validity constraint violated");
        // SAFETY:
        //
        // The top level page table was allocated and mapped into the loader address space when
        // this object was created.
        let pml2 = unsafe { &*(pml2_virtual_address as *const [u32; 1024]) };
        if pml2[pml2_index] & PRESENT_BIT == PRESENT_BIT {
            return Err(NoMapping);
        }

        let pml1_physical_address = pml2[pml2_index] & ADDRESS_MASK;
        let pml1_virtual_address =
            translate_phys(u64::from(pml1_physical_address)).expect("validity constraint violated");

        // SAFETY:
        //
        // The page tables are properly allocated and mapped in an atomic fashion.
        let pml1 = unsafe { &*(pml1_virtual_address as *const [u32; 1024]) };

        Ok(u64::from(
            (pml1[pml1_index] & ADDRESS_MASK) + (virtual_address as u32 & 0xFFF),
        ))
    }

    fn max_address(&self) -> u64 {
        u64::from(u32::MAX)
    }

    fn page_size(&self) -> u64 {
        4096
    }
}

impl X86CommonAddressSpace for Bits32PageTables {
    fn physical_address(&self) -> u64 {
        u64::from(self.physical_address)
    }

    fn paging_mode(&self) -> PagingMode {
        PagingMode::Bits32
    }
}

impl Default for Bits32PageTables {
    fn default() -> Self {
        Self::new()
    }
}

/// Attempts to allocate and map a new page table atomically. If any part of the process fails, it
/// undoes the work.
fn atomic_map_new_table() -> Result<(usize, u32), AtomicMapError> {
    let frame_multiple = 4096u64.div_ceil(frame_size());
    let page_multiple = 4096usize.div_ceil(page_size());
    let frame_alignment = 4096u64;

    let physical_address = allocate_frame(
        AllocationType::Below {
            physical_address: u64::from(u32::MAX),
        },
        frame_multiple,
        frame_alignment,
    )
    .map_err(|_| AtomicMapError::AllocationError)?;
    assert!(physical_address <= u64::from(u32::MAX));

    let map_result = map(
        physical_address,
        page_multiple,
        ProtectionFlags::READ | ProtectionFlags::WRITE,
    );

    match map_result {
        Ok(virtual_address) => {
            // SAFETY:
            //
            // The given virtual_address is properly mapped into the address space and backed by
            // valid physical memory.
            unsafe { core::ptr::write_bytes(virtual_address as *mut u8, 0, 4096) }

            Ok((virtual_address, physical_address as u32))
        }
        Err(_) => {
            // SAFETY:
            //
            // The frame located at `physical_address` was just allocated using `allocate_frame()`
            // and has not exposed yet.
            unsafe { deallocate_frame(physical_address, frame_multiple) }

            Err(AtomicMapError::MapError)
        }
    }
}

/// Various errors that can occur while atomically allocating and mapping a new page table.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum AtomicMapError {
    /// The frame allocation failed.
    AllocationError,
    /// Mapping the frame failed.
    MapError,
}

impl fmt::Display for AtomicMapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AllocationError => f.write_str("failed to allocate page table frame"),
            Self::MapError => f.write_str("failed to map page table frame"),
        }
    }
}

impl error::Error for AtomicMapError {}
