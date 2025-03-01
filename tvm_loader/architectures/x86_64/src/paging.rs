//! Implementation of 4-level and 5-level paging.

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
use x86_common::{
    PagingMode, current_paging_mode, instructions::cpuid::cpuid_unchecked,
    max_supported_paging_mode,
};

/// Architecturally defined maximum physical address.
const ARCHITECTURAL_MAX_PHYSICAL: u64 = (1 << 52) - 1;

/// Bit indicating translation region is present.
const PRESENT_BIT: u64 = 1 << 0;
/// Bit indicating translation region is writable.
const WRITE_BIT: u64 = 1 << 1;
/// Mask of the physical address in a page table entry.
const ADDRESS_MASK: u64 = ((1 << 40) - 1) << 12;

/// Bit indicating translation region is not executable.
const NXE_BIT: u64 = 1 << 63;

/// A set of 4-level or 5-level page tables with supported features.
#[repr(C)]
pub struct X86_64PageTable {
    /// Manager of the free virtual regions.
    free_region_tracker: FreeRegionTracker,
    /// The physical address of the top table.
    physical_address: u64,
    /// If `true`, 5 level paging is supported.
    level_5: bool,
    /// If `true`, NXE is supported.
    nxe: bool,
}

impl X86_64PageTable {
    /// Creates a new `x86_64` 4-level or 5-level page table scheme.
    ///
    /// If `nxe` is true, then the NXE bit is used.
    /// If `level_5` is true, then the page table scheme uses the 5-level paging scheme.
    ///
    /// # Errors
    ///
    /// - [`FeatureNotSupported::Level4`]: Returned if the processor does not support 4-level
    ///   paging.
    /// - [`FeatureNotSupported::Level5`]: Returned if the processor does not support 5-level
    ///   paging and `level_5` was true.
    /// - [`FeatureNotSupported::Nxe`]: Returned if the processor does not support the NXE bit
    ///   and `nxe` was true.
    ///
    /// # Panics
    ///
    /// Panics if the allocation and mapping of the top level page table fails.
    pub fn new(nxe: bool, level_5: bool) -> Result<Self, FeatureNotSupported> {
        match max_supported_paging_mode() {
            PagingMode::Disabled | PagingMode::Bits32 | PagingMode::Pae => {
                return Err(FeatureNotSupported::Level4);
            }
            PagingMode::Level4 if level_5 => return Err(FeatureNotSupported::Level5),
            PagingMode::Level4 | PagingMode::Level5 => {}
        }

        if nxe {
            // SAFETY:
            //
            // The CPUID instruction is available on this processor since 4-level paging is supported.
            let result_80000001 = unsafe { cpuid_unchecked(0x8000_0001, 0).edx };
            if (result_80000001 >> 20) & 1 != 1 {
                // NXE is supported.

                return Err(FeatureNotSupported::Nxe);
            }
        }

        let (_, physical_address) = atomic_map_new_table().expect("atomic table map failed");

        let page_table = Self {
            free_region_tracker: FreeRegionTracker::new(&[
                FreeRegion {
                    virtual_address: 0,
                    length: virtual_region_bounds(level_5).0,
                },
                FreeRegion {
                    virtual_address: virtual_region_bounds(level_5).1 + 1,
                    length: virtual_region_bounds(level_5).0,
                },
            ]),
            physical_address,
            level_5,
            nxe,
        };

        Ok(page_table)
    }

    /// Creates a new `x86_64` 4-level or 5-level page tables.
    ///
    /// # Errors
    ///
    /// Returns `()` if 4-level paging is not supported.
    #[expect(clippy::result_unit_err)]
    pub fn new_max_supported() -> Result<Self, ()> {
        let mut nxe = true;
        let mut level_5 = true;

        loop {
            match Self::new(nxe, level_5) {
                Ok(page_table) => return Ok(page_table),
                Err(FeatureNotSupported::Level4) => return Err(()),
                Err(FeatureNotSupported::Nxe) => nxe = false,
                Err(FeatureNotSupported::Level5) => level_5 = false,
            };
        }
    }

    /// Creates a new `x86_64` page table structure compatible with the currently used page mode.
    ///
    /// # Errors
    ///
    /// Returns `()` if 4-level paging is not supported.
    #[expect(clippy::result_unit_err)]
    #[expect(clippy::missing_panics_doc)]
    pub fn new_current() -> Result<Self, ()> {
        let level_5 = match current_paging_mode() {
            PagingMode::Level4 => false,
            PagingMode::Level5 => true,
            _ => return Err(()),
        };

        Ok(Self::new(false, level_5).unwrap())
    }

    /// Whether NXE should be enabled.
    pub const fn nxe(&self) -> bool {
        self.nxe
    }

    /// Maps the physical region starting at `physical` and extending `count` pages into the
    /// virtual address space at `virtual_address`.
    ///
    /// # Safety
    ///
    /// The virtual and physical regions are valid and the virtual region has not already been
    /// mapped.
    unsafe fn map_unchecked(
        &mut self,
        virtual_address: u64,
        physical_address: u64,
        count: u64,
        protection: ProtectionFlags,
    ) {
        let pml5 = if self.level_5 {
            let address =
                translate_phys(self.physical_address).expect("validity constraint invalidated");

            // SAFETY:
            //
            // The top level page table was allocated and mapped into the loader address space when
            // this object was created.
            unsafe { &mut *(address as *mut [u64; 512]) }
        } else {
            &mut [self.physical_address | WRITE_BIT | PRESENT_BIT; 512]
        };

        let mut translated_flags = if self.nxe() { NXE_BIT } else { 0 };
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

        let not_present_handler = |entry: &mut u64| -> Result<(), AtomicMapError> {
            let (_, new_table_phys_address) = atomic_map_new_table()?;

            *entry = new_table_phys_address | WRITE_BIT | PRESENT_BIT;

            Ok(())
        };
        for index in 0..count {
            let map_address = virtual_address + index * 4096u64;
            let mapped_address = physical_address + index * 4096u64;

            let pml5_index = ((map_address >> 48) & 0x1FF) as usize;
            let pml4_index = ((map_address >> 39) & 0x1FF) as usize;
            let pml3_index = ((map_address >> 30) & 0x1FF) as usize;
            let pml2_index = ((map_address >> 21) & 0x1FF) as usize;
            let pml1_index = ((map_address >> 12) & 0x1FF) as usize;

            let pml4_virtual_address = next_level_mut(pml5, pml5_index, not_present_handler)
                .expect("failed to atomically new page_table");

            // SAFETY:
            //
            // The page tables are properly allocated and mapped in an atomic fashion.
            let pml4 = unsafe { &mut *(pml4_virtual_address as *mut [u64; 512]) };
            let pml3_virtual_address = next_level_mut(pml4, pml4_index, not_present_handler)
                .expect("failed to atomically new page_table");

            // SAFETY:
            //
            // The page tables are properly allocated and mapped in an atomic fashion.
            let pml3 = unsafe { &mut *(pml3_virtual_address as *mut [u64; 512]) };
            let pml2_virtual_address = next_level_mut(pml3, pml3_index, not_present_handler)
                .expect("failed to atomically new page_table");

            // SAFETY:
            //
            // The page tables are properly allocated and mapped in an atomic fashion.
            let pml2 = unsafe { &mut *(pml2_virtual_address as *mut [u64; 512]) };
            let pml1_virtual_address = next_level_mut(pml2, pml2_index, not_present_handler)
                .expect("failed to atomically new page_table");

            // SAFETY:
            //
            // The page tables are properly allocated and mapped in an atomic fashion.
            let pml1 = unsafe { &mut *(pml1_virtual_address as *mut [u64; 512]) };
            pml1[pml1_index] = mapped_address | translated_flags;
        }
    }
}

impl AddressSpace for X86_64PageTable {
    fn map(
        &mut self,
        virtual_address: u64,
        physical_address: u64,
        count: u64,
        protection: ProtectionFlags,
    ) -> Result<(), MapError> {
        if virtual_address % 4096u64 != 0 || physical_address % 4096u64 != 0 {
            return Err(MapError::AlignmentError);
        }

        let Some(requested_mapping_size) = count.checked_mul(4096u64) else {
            return Err(MapError::InvalidSize);
        };

        if physical_address
            .checked_add(requested_mapping_size)
            .is_none_or(|max_address| max_address > ARCHITECTURAL_MAX_PHYSICAL)
        {
            return Err(MapError::AddressOverflow);
        }

        let virtual_end_address = virtual_address
            .checked_add(requested_mapping_size)
            .ok_or(MapError::AddressOverflow)?;

        let virtual_region = (virtual_address, virtual_end_address);
        let invalid_region = virtual_region_bounds(self.level_5);
        if virtual_region.0 <= invalid_region.1 && invalid_region.0 <= virtual_region.1 {
            return Err(MapError::InvalidAddress);
        }

        if self
            .free_region_tracker
            .allocate_region(virtual_address, count * 4096u64)
            .is_err()
        {
            return Err(MapError::AlreadyMapped);
        }

        // SAFETY:
        //
        // The invariants required to call `map_unchecked()` were just checked.
        unsafe { self.map_unchecked(virtual_address, physical_address, count, protection) }

        Ok(())
    }

    unsafe fn unmap(&mut self, virtual_address: u64, count: u64) -> Result<(), NotMapped> {
        debug_assert_eq!(
            virtual_address % 4096u64,
            0,
            "virtual address not properly aligned"
        );

        let requested_mapping_size = count.checked_mul(4096u64).expect("mapping too large");

        let virtual_end_address = virtual_address
            .checked_add(requested_mapping_size)
            .expect("virtual region too large");

        let virtual_region = (virtual_address, virtual_end_address);
        let invalid_region = virtual_region_bounds(self.level_5);
        assert!(
            virtual_region.0 <= invalid_region.1 && invalid_region.0 <= virtual_region.1,
            "virtual region must not overlap with invalid middle region"
        );

        assert!(
            self.free_region_tracker
                .deallocate_region(virtual_address, requested_mapping_size)
                .is_ok(),
            "virtual region was already unmapped"
        );

        Ok(())
    }

    fn translate_virt(&self, virtual_address: u64) -> Result<u64, NoMapping> {
        let pml5_index = ((virtual_address >> 48) & 0x1FF) as usize;
        let pml4_index = ((virtual_address >> 39) & 0x1FF) as usize;
        let pml3_index = ((virtual_address >> 30) & 0x1FF) as usize;
        let pml2_index = ((virtual_address >> 21) & 0x1FF) as usize;
        let pml1_index = ((virtual_address >> 12) & 0x1FF) as usize;

        let pml5 = if self.level_5 {
            let address =
                translate_phys(self.physical_address).expect("validity constraint invalidated");

            // SAFETY:
            //
            // The top level page table was allocated and mapped into the loader address space when
            // this object was created.
            unsafe { &*(address as *const [u64; 512]) }
        } else {
            &[self.physical_address | WRITE_BIT | PRESENT_BIT; 512]
        };

        let pml4_virtual_address = next_level(pml5, pml5_index, |_| NoMapping)?;

        // SAFETY:
        //
        // The page tables are properly allocated and mapped in an atomic fashion.
        let pml4 = unsafe { &*(pml4_virtual_address as *const [u64; 512]) };
        let pml3_virtual_address = next_level(pml4, pml4_index, |_| NoMapping)?;

        // SAFETY:
        //
        // The page tables are properly allocated and mapped in an atomic fashion.
        let pml3 = unsafe { &*(pml3_virtual_address as *const [u64; 512]) };
        let pml2_virtual_address = next_level(pml3, pml3_index, |_| NoMapping)?;

        // SAFETY:
        //
        // The page tables are properly allocated and mapped in an atomic fashion.
        let pml2 = unsafe { &*(pml2_virtual_address as *const [u64; 512]) };
        let pml1_virtual_address = next_level(pml2, pml2_index, |_| NoMapping)?;

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
        u64::MAX
    }

    fn page_size(&self) -> u64 {
        4096
    }
}

impl X86CommonAddressSpace for X86_64PageTable {
    fn physical_address(&self) -> u64 {
        self.physical_address
    }

    fn paging_mode(&self) -> PagingMode {
        if self.level_5 {
            PagingMode::Level5
        } else {
            PagingMode::Level4
        }
    }
}

/// A required CPU feature was not supported.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum FeatureNotSupported {
    /// 4-level paging is not supported.
    Level4,
    /// 5-level paging is not supported and it was requested.
    Level5,
    /// The NXE bit is not supported and it was requested.
    Nxe,
}

impl fmt::Display for FeatureNotSupported {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Level4 => f.write_str("4-level paging is not supported"),
            Self::Level5 => f.write_str("5-level paging is not supported"),
            Self::Nxe => f.write_str("the NXE bit is not supported"),
        }
    }
}

impl error::Error for FeatureNotSupported {}

/// Descends to the next level of the page table tree.
fn next_level<E>(
    table: &[u64; 512],
    index: usize,
    not_present_handler: fn(u64) -> E,
) -> Result<usize, E> {
    if table[index] & PRESENT_BIT != PRESENT_BIT {
        return Err(not_present_handler(table[index]));
    }

    let lower_table_phys_address = table[index] & ADDRESS_MASK;
    let virtual_address = translate_phys(lower_table_phys_address).expect("malformed table");

    Ok(virtual_address)
}

/// Descends to the next level of the page table tree.
fn next_level_mut<E>(
    table: &mut [u64; 512],
    index: usize,
    not_present_handler: fn(&mut u64) -> Result<(), E>,
) -> Result<usize, E> {
    if table[index] & PRESENT_BIT != PRESENT_BIT {
        not_present_handler(&mut table[index])?;
    }

    let lower_table_phys_address = table[index] & ADDRESS_MASK;
    let virtual_address = translate_phys(lower_table_phys_address).expect("malformed table");

    Ok(virtual_address)
}

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

impl fmt::Display for AtomicMapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AllocationError => f.write_str("failed to allocate page table frame"),
            Self::MapError => f.write_str("failed to map page table frame"),
        }
    }
}

impl error::Error for AtomicMapError {}

/// Returns the bounds of the invalid region in the middle of the address space.
const fn virtual_region_bounds(level_5: bool) -> (u64, u64) {
    if level_5 {
        (0x0100_0000_0000_0000, 0xFEFF_FFFF_FFFF_FFFF)
    } else {
        (0x0000_8000_0000_0000, 0xFFFF_7FFF_FFFF_FFFF)
    }
}
