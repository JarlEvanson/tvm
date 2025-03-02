//! Implementation of PAE paging.

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
use x86_common::{PagingMode, instructions::cpuid::cpuid_unchecked, max_supported_paging_mode};

/// Architecturally defined maximum physical address.
const MAX_PHYSICAL_ADDRESS: u64 = (1 << 52) - 1;

/// Bit indicating that the translation region is present.
const PRESENT_BIT: u64 = 1 << 0;
/// Bit indicating that the translation region is writable.
const WRITE_BIT: u64 = 1 << 1;
/// Mask of the physical address in a page table entry.
const ADDRESS_MASK: u64 = ((1 << 40) - 1) << 12;
/// Bit indicating that the translation region is not executable.
const NXE_BIT: u64 = 1 << 63;

/// A set of PAE page tables.
pub struct PaePageTables {
    /// Manager of the free virtual regions.
    free_region_tracker: FreeRegionTracker,
    /// The physical address of the top table.
    physical_address: u32,
    /// If `true`, NXE is supported.
    nxe: bool,
}

impl PaePageTables {
    /// Creates a new PAE page table scheme.
    ///
    /// If `nxe` is true, then the NXE bit is used.
    ///
    /// # Errors
    ///
    /// - [`FeatureNotSupported::Pae`]: Returned if the processor does not support PAE paging.
    /// - [`FeatureNotSupported::Nxe`]: Returned if the processor does not support the NXE bit and
    ///   `nxe` was true.
    ///
    /// # Panics
    ///
    /// Panics if the allocation and mapping of the top level page table fails.
    pub fn new(nxe: bool) -> Result<Self, FeatureNotSupported> {
        match max_supported_paging_mode() {
            PagingMode::Pae | PagingMode::Level4 | PagingMode::Level5 => {}
            PagingMode::Disabled | PagingMode::Bits32 => return Err(FeatureNotSupported::Pae),
        }

        if nxe {
            // SAFETY:
            //
            // The CPUID instruction is available on this processor since PAE paging is supported.
            let result_80000001 = unsafe { cpuid_unchecked(0x8000_0001, 0).edx };
            if (result_80000001 >> 20) & 1 != 1 {
                // NXE is supported.

                return Err(FeatureNotSupported::Nxe);
            }
        }

        let frame_multiple = 32u64.div_ceil(frame_size());
        let page_multiple = 32usize.div_ceil(page_size());
        let frame_alignment = 32u64;

        let physical_address = allocate_frame(
            AllocationType::Below {
                physical_address: u32::MAX as u64 + 1,
            },
            frame_multiple,
            frame_alignment,
        )
        .expect("failed to atomically map root page table");

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
            }
            Err(_) => {
                // SAFETY:
                //
                // The frame located at `physical_address` was just allocated using `allocate_frame()`
                // and has not exposed yet.
                unsafe { deallocate_frame(physical_address, frame_multiple) }

                panic!("failed to atomically map root page table");
            }
        }

        let page_table = Self {
            free_region_tracker: FreeRegionTracker::new(&[FreeRegion {
                virtual_address: 0,
                length: u32::MAX as u64 + 1,
            }]),
            physical_address: physical_address as u32,
            nxe,
        };

        Ok(page_table)
    }

    /// Creates a new PAE page table scheme with the most features supported.
    ///
    /// # Errors
    ///
    /// Returns `()` if PAE paging is not supported.
    #[expect(clippy::result_unit_err)]
    pub fn new_max_supported() -> Result<Self, ()> {
        let mut nxe = true;

        loop {
            match Self::new(nxe) {
                Ok(page_table) => return Ok(page_table),
                Err(FeatureNotSupported::Pae) => return Err(()),
                Err(FeatureNotSupported::Nxe) => nxe = false,
            }
        }
    }

    /// Creates a new PAE page table scheme compatible with the currently used paging mode.
    ///
    /// # Errors
    ///
    /// Returns `()` if PAE paging is not supported.
    #[expect(clippy::result_unit_err)]
    #[expect(clippy::missing_panics_doc)]
    pub fn new_current() -> Result<Self, ()> {
        Ok(Self::new(false).unwrap())
    }

    /// Whether NXE must be enabled for this page table to be valid.
    pub const fn nxe(&self) -> bool {
        self.nxe
    }

    /// Maps the physical region starting at `virtual_address` and extending `count` pages into the
    /// virtual address space at `virtual_address`.
    ///
    /// # Safety
    ///
    /// The virtual and physical regions are valid and the virtual region has not already been
    /// mapped.
    unsafe fn map_unchecked(
        &mut self,
        virtual_address: u32,
        physical_address: u64,
        count: u32,
        protection: ProtectionFlags,
    ) {
        let mut translated_flags = if self.nxe { NXE_BIT } else { 0 };
        if protection & ProtectionFlags::READ == ProtectionFlags::READ {
            translated_flags |= PRESENT_BIT;
        }
        if protection & ProtectionFlags::WRITE == ProtectionFlags::WRITE {
            translated_flags |= WRITE_BIT | PRESENT_BIT;
        }
        if protection & ProtectionFlags::EXECUTE == ProtectionFlags::EXECUTE {
            translated_flags |= PRESENT_BIT;
            translated_flags &= !NXE_BIT;
        }

        let pml3_virtual_address =
            translate_phys(self.physical_address as u64).expect("validity constraint invalidated");
        // SAFETY:
        //
        // The top level page table was allocated and mapped into the loader address space when
        // this [`PaePageTables`] object was created.
        let pml3 = unsafe { &mut *(pml3_virtual_address as *mut [u64; 4]) };

        for index in 0..count {
            let map_address = virtual_address + index * self.page_size() as u32;
            let mapped_address = physical_address + index as u64 * self.page_size();

            let pml3_index = ((map_address >> 30) & 0b11) as usize;
            let pml2_index = ((map_address >> 21) & 0x1FF) as usize;
            let pml1_index = ((map_address >> 12) & 0x1FF) as usize;

            if pml3[pml3_index] & PRESENT_BIT != PRESENT_BIT {
                let (_, new_table_physical_address) =
                    atomic_map_new_table().expect("failed to atomically map new page table");

                pml3[pml3_index] = new_table_physical_address | PRESENT_BIT;
            }

            let pml2_physical_address = pml3[pml3_index] & ADDRESS_MASK;
            let pml2_virtual_address =
                translate_phys(pml2_physical_address).expect("validity constraint invalidated");

            // SAFETY:
            //
            // The page tables are properly allocated and mapped in an atomic fashion.
            let pml2 = unsafe { &mut *(pml2_virtual_address as *mut [u64; 512]) };
            if pml2[pml2_index] & PRESENT_BIT != PRESENT_BIT {
                let (_, new_table_physical_address) =
                    atomic_map_new_table().expect("failed to atomically map new page table");

                pml2[pml2_index] = new_table_physical_address | WRITE_BIT | PRESENT_BIT;
            }

            let pml1_physical_address = pml2[pml2_index] & ADDRESS_MASK;
            let pml1_virtual_address =
                translate_phys(pml1_physical_address).expect("validity constraint invalidated");

            // SAFETY:
            //
            // The page tables are properly allocated and mapped in an atomic fashion.
            let pml1 = unsafe { &mut *(pml1_virtual_address as *mut [u64; 512]) };
            pml1[pml1_index] = mapped_address | translated_flags;
        }
    }
}

impl AddressSpace for PaePageTables {
    fn map(
        &mut self,
        virtual_address: u64,
        physical_address: u64,
        count: u64,
        protection: ProtectionFlags,
    ) -> Result<(), MapError> {
        if virtual_address % self.page_size() != 0 || physical_address % self.page_size() != 0 {
            return Err(MapError::AlignmentError);
        }

        let Some(requested_mapping_size) = count.checked_mul(self.page_size()) else {
            return Err(MapError::InvalidSize);
        };

        if physical_address
            .checked_add(requested_mapping_size)
            .is_none_or(|max_address| max_address > MAX_PHYSICAL_ADDRESS)
        {
            return Err(MapError::AddressOverflow);
        }

        let virtual_end_address = virtual_address
            .checked_add(requested_mapping_size)
            .ok_or(MapError::AddressOverflow)?;

        if virtual_end_address > u64::from(u32::MAX) {
            return Err(MapError::InvalidAddress);
        }

        if self
            .free_region_tracker
            .allocate_region(virtual_address, requested_mapping_size)
            .is_err()
        {
            return Err(MapError::AlreadyMapped);
        }

        // SAFETY:
        //
        // THe validity constraints have been checked as much as possible.
        unsafe {
            self.map_unchecked(
                virtual_address as u32,
                physical_address,
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
            "virtual address not properly aligned",
        );

        let requested_size = count
            .checked_mul(self.page_size())
            .expect("mapping too large");

        let virtual_end_address = virtual_address
            .checked_add(requested_size)
            .expect("virtual region too large");

        assert!(
            virtual_end_address > u64::from(u32::MAX),
            "invalid virtual region"
        );

        assert!(
            self.free_region_tracker
                .deallocate_region(virtual_address, requested_size)
                .is_ok(),
            "virtual region was already unmapped",
        );

        Ok(())
    }

    fn translate_virt(&self, virtual_address: u64) -> Result<u64, NoMapping> {
        if virtual_address > u64::from(u32::MAX) {
            return Err(NoMapping);
        }

        let pml3_index = ((virtual_address >> 30) & 0b11) as usize;
        let pml2_index = ((virtual_address >> 21) & 0x1FF) as usize;
        let pml1_index = ((virtual_address >> 12) & 0x1FF) as usize;

        let pml3_virtual_address =
            translate_phys(self.physical_address as u64).expect("validity constraint invalidated");

        // SAFETY:
        //
        // The top level page table was allocated and mapped into the loader address space when
        // this [`PaePageTables`] object was created.
        let pml3 = unsafe { &*(pml3_virtual_address as *const [u64; 4]) };
        if pml3[pml3_index] & PRESENT_BIT != PRESENT_BIT {
            return Err(NoMapping);
        }

        let pml2_physical_address = pml3[pml3_index] & ADDRESS_MASK;
        let pml2_virtual_address =
            translate_phys(pml2_physical_address).expect("validity constraint invalidated");

        // SAFETY:
        //
        // The page tables are properly allocated and mapped in an atomic fashion.
        let pml2 = unsafe { &*(pml2_virtual_address as *const [u64; 512]) };
        if pml2[pml2_index] & PRESENT_BIT != PRESENT_BIT {
            return Err(NoMapping);
        }

        let pml1_physical_address = pml2[pml2_index] & ADDRESS_MASK;
        let pml1_virtual_address =
            translate_phys(pml1_physical_address).expect("validity constraint invalidated");

        // SAFETY:
        //
        // The page tables are properly allocated and mapped in an atomic fashion.
        let pml1 = unsafe { &*(pml1_virtual_address as *const [u64; 512]) };
        if pml1[pml1_index] & PRESENT_BIT != PRESENT_BIT {
            return Err(NoMapping);
        }

        Ok((pml1[pml1_index] & ADDRESS_MASK) + (virtual_address & 0xFFF))
    }

    fn max_address(&self) -> u64 {
        u64::from(u32::MAX)
    }

    fn page_size(&self) -> u64 {
        4096
    }
}

impl X86CommonAddressSpace for PaePageTables {
    fn physical_address(&self) -> u64 {
        u64::from(self.physical_address)
    }

    fn paging_mode(&self) -> PagingMode {
        PagingMode::Pae
    }
}

/// A required CPU feature was not supported.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum FeatureNotSupported {
    /// PAE paging is not supported.
    Pae,
    /// The NXE bit is not supported and it was requested.
    Nxe,
}

impl fmt::Display for FeatureNotSupported {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pae => f.write_str("PAE paging is not supported"),
            Self::Nxe => f.write_str("the NXE bit is not supported"),
        }
    }
}

impl error::Error for FeatureNotSupported {}

/// Attempts to allocate and map a new page table atomically. If any part of the process fails, it
/// undoes the work.
fn atomic_map_new_table() -> Result<(usize, u64), AtomicMapError> {
    let frame_multiple = 4096u64.div_ceil(frame_size());
    let page_multiple = 4096usize.div_ceil(page_size());
    let frame_alignment = 4096u64;

    let physical_address = allocate_frame(AllocationType::Any, frame_multiple, frame_alignment)
        .map_err(|_| AtomicMapError::AllocationError)?;

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

            Ok((virtual_address, physical_address))
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
