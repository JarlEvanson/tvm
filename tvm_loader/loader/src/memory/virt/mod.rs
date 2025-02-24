//! Virtual memory mangement interfaces for `tvm_loader`.
//!
//! The interfaces are defined in this module and implemented by a platform or system specific
//! crate.

pub mod arch;
pub mod loader;

use core::{error, fmt};

use alloc::vec::Vec;

/// The maximum supported address at which a [`FreeRegion`] can end.
///
/// This address may not be representable in 64 bits.
pub const MAX_REGION_END: u128 = u64::MAX as u128 + 1;

/// Tracks all of the [`FreeRegion`]s in an address space.
pub struct FreeRegionTracker(Vec<FreeRegion>);

impl FreeRegionTracker {
    /// Creates a new [`FreeRegionTracker`].
    ///
    /// `base_regions` should contain all of the free regions that should be tracked.
    ///
    /// # Panics
    ///
    /// Panics if the `base_regions` overlap, are not sorted in ascending order, or the end of the
    /// last [`FreeRegion`] is greater than [`MAX_REGION_END`].
    pub fn new(base_regions: &[FreeRegion]) -> Self {
        let mut end_address = 0u128;

        for region in base_regions {
            assert!(end_address <= region.virtual_address as u128);

            end_address = region.virtual_address as u128 + region.length as u128;
        }
        assert!(end_address <= MAX_REGION_END);

        let mut buffer = Vec::new();
        buffer.extend(base_regions);

        let mut free_region_tracker = Self(buffer);

        let mut index = 0;
        while index < free_region_tracker.0.len() {
            free_region_tracker.merge(index);

            index += 1;
        }

        free_region_tracker
    }

    /// Allocates a region of free virtual memory starting at `virtual_address` and extending for
    /// `length` bytes.
    ///
    /// # Errors
    ///
    /// - [`AllocateRegionError::InvalidRegion`]: Returned when `virtual_address` and `length`
    ///   create an invalid region.
    /// - [`AllocateRegionError::UnavailableRegion`]: Returned when the region specified by
    ///   `virtual_address` and `length` is not entirely contained within a [`FreeRegion`].
    pub fn allocate_region(
        &mut self,
        virtual_address: u64,
        length: u64,
    ) -> Result<(), AllocateRegionError> {
        let end_address = virtual_address as u128 + length as u128;
        if end_address > MAX_REGION_END {
            return Err(AllocateRegionError::InvalidRegion);
        }

        let result = self
            .0
            .binary_search_by_key(&virtual_address, |region| region.virtual_address);

        match result {
            Ok(index) => {
                let region = &mut self.0[index];
                if region.length < length {
                    return Err(AllocateRegionError::UnavailableRegion);
                }

                if region.length == length {
                    self.0.remove(index);
                    return Ok(());
                }

                region.virtual_address += length;
                region.length -= length;
            }
            Err(index) => {
                // The [`FreeRegion`] before the correct insert location is the only possible
                // [`FreeRegion`] to contain the requested region.
                let Some(lower_index) = index.checked_sub(1) else {
                    return Err(AllocateRegionError::UnavailableRegion);
                };

                let region = &mut self.0[lower_index];
                let region_end = region.virtual_address as u128 + region.length as u128;
                if region_end < end_address {
                    return Err(AllocateRegionError::UnavailableRegion);
                }

                let remaining_length = (region_end - end_address) as u64;
                region.length = virtual_address - region.virtual_address;

                if remaining_length > 0 {
                    self.0.insert(
                        index,
                        FreeRegion {
                            virtual_address: end_address as u64,
                            length: remaining_length,
                        },
                    );
                }
            }
        }

        Ok(())
    }

    /// Deallocates a region of virtual memory starting at `virtual_address` and extending for
    /// `length` bytes.
    ///
    /// # Errors
    ///
    /// - [`DeallocateRegionError::InvalidRegion`]: Returned when `virtual_address` and `length`
    ///   create an invalid [`FreeRegion`].
    /// - [`DeallocateRegionError::RegionOverlap`]: Returned when the [`FreeRegion`] created by
    ///   `virtual_address` and `length` over with an existing [`FreeRegion`].
    pub fn deallocate_region(
        &mut self,
        virtual_address: u64,
        length: u64,
    ) -> Result<(), DeallocateRegionError> {
        if virtual_address as u128 + length as u128 > MAX_REGION_END || length == 0 {
            return Err(DeallocateRegionError::InvalidRegion);
        }

        let binary_search = self
            .0
            .binary_search_by_key(&virtual_address, |region| region.virtual_address);
        let index = match binary_search {
            // If `virtual_address` matchs with the start of a [`FreeRegion`] in the
            // [`FreeRegionTracker`], the region overlaps with an already free region.
            Ok(_) => return Err(DeallocateRegionError::RegionOverlap),
            Err(index) => index,
        };

        'lower_region: {
            // If a lower [`FreeRegion`] exists, check that it doesn't overlap with the region we
            // are attempting to free.
            let Some(lower_index) = index.checked_sub(1) else {
                break 'lower_region;
            };

            // The [`FreeRegion`] at `lower_index` is guaranteed to exist.
            let region = self.0[lower_index];

            let region_end = region.virtual_address as u128 + region.length as u128;
            if region_end > virtual_address as u128 {
                return Err(DeallocateRegionError::RegionOverlap);
            }
        }

        'upper_region: {
            // If an upper [`FreeRegion`] exists, check that it doesn't overlap with the region we
            // are attempting to free.
            let Some(region) = self.0.get(index) else {
                break 'upper_region;
            };

            let new_region_end = virtual_address as u128 + length as u128;
            if new_region_end > region.virtual_address as u128 {
                return Err(DeallocateRegionError::RegionOverlap);
            }
        }

        self.0.insert(
            index,
            FreeRegion {
                virtual_address,
                length,
            },
        );

        // If a lower [`FreeRegion`] exists, attempt to merge it.
        'merge_lower: {
            let Some(merge_index) = index.checked_sub(1) else {
                break 'merge_lower;
            };

            self.merge(merge_index);
        }

        // Attempt to merge the newly inserted [`FreeRegion`] with the [`FreeRegion`] above it.
        self.merge(index);

        Ok(())
    }

    /// Merges the [`FreeRegion`]s that are contiguous with the [`FreeRegion`] contained at
    /// `index` and have higher `virtual_address` bases than that [`FreeRegion`].
    fn merge(&mut self, index: usize) {
        // Get the base [`FreeRegion`] of the merge.
        let Some(mut merge_region) = self.0.get(index).copied() else {
            return;
        };

        // Calculate the next index to test with.
        let Some(test_index) = index.checked_add(1) else {
            return;
        };

        loop {
            // Check if a [`FreeRegion] above the base region exists.
            let Some(test_region) = self.0.get(test_index) else {
                break;
            };

            // Calculate end of the base region.
            let merge_end_address =
                merge_region.virtual_address as u128 + merge_region.length as u128;
            if merge_end_address != test_region.virtual_address as u128 {
                break;
            }

            merge_region.length += test_region.length;
            self.0.remove(test_index);
        }

        self.0[index] = merge_region;
    }

    /// Returns a slice containing the [FreeRegion`]s.
    pub fn free_regions(&self) -> &[FreeRegion] {
        self.0.as_slice()
    }
}

/// Various errors that can occur when calling [`FreeRegionTracker::allocate_region()`].
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum AllocateRegionError {
    /// The provided `virtual_address` and `length` create an invalid [`FreeRegion`].
    InvalidRegion,
    /// The provided `virtual_address` and `length` create a [`FreeRegion`] that does not overlap
    /// with an existing [`FreeRegion`] in [`FreeRegionTracker`].
    UnavailableRegion,
}

impl fmt::Display for AllocateRegionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRegion => "region is invalid".fmt(f),
            Self::UnavailableRegion => "region is not free for allocation".fmt(f),
        }
    }
}

impl error::Error for AllocateRegionError {}

/// Various errors that can occur when calling [`FreeRegionTracker::deallocate_region()`].
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeallocateRegionError {
    /// The provided `virtual_address` and `length` create an invalid [`FreeRegion`].
    InvalidRegion,
    /// The provided `virtual_address` and `length` create a [`FreeRegion`] that overlaps with an
    /// existing [`FreeRegion`] in [`FreeRegionTracker`].
    RegionOverlap,
}

impl fmt::Display for DeallocateRegionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRegion => "region is invalid".fmt(f),
            Self::RegionOverlap => "region overlaps with already freed regions".fmt(f),
        }
    }
}

impl error::Error for DeallocateRegionError {}

/// A region of free virtual memory.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FreeRegion {
    /// The virtual address at the start of the free region.
    pub virtual_address: u64,
    /// The size, in bytes, of the free region.
    pub length: u64,
}

/// Protection settings for a page in an address space.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProtectionFlags(u32);

impl ProtectionFlags {
    /// The page should be readable.
    pub const READ: Self = Self(0x1);
    /// The page should be writable.
    pub const WRITE: Self = Self(0x2);
    /// The page should be executable.
    pub const EXECUTE: Self = Self(0x4);
}

impl core::ops::BitOr for ProtectionFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for ProtectionFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl core::ops::BitAnd for ProtectionFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl core::ops::BitAndAssign for ProtectionFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl core::ops::BitXor for ProtectionFlags {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl core::ops::BitXorAssign for ProtectionFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl core::ops::Not for ProtectionFlags {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

/// Various errors that can occur when mapping a physical region into an address space.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum MapError {
    /// The requested mapping would overflow in the target address space.
    AddressOverflow,
    /// The requested mapping starts at an invalid address.
    AlignmentError,
    /// An error occurred when allocating memory required to fulfill the requested mapping.
    AllocationError,
    /// The virtual region is already mapped.
    AlreadyMapped,
    /// An unspecified error occurred.
    GeneralError,
    /// A provided address is invalid.
    InvalidAddress,
    /// The size of the requested mapping is invalid.
    InvalidSize,
}

impl fmt::Display for MapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AddressOverflow => "requested mapping involves overflow".fmt(f),
            Self::AlignmentError => "requested mapping starts at an invalid alignment".fmt(f),
            Self::AllocationError => "requested mapping had a required allocation fail".fmt(f),
            Self::AlreadyMapped => "requested virtual region is already in use".fmt(f),
            Self::GeneralError => "an unspecified error occurred".fmt(f),
            Self::InvalidAddress => "requested mapping involves an invalid address".fmt(f),
            Self::InvalidSize => "requested mapping is too large".fmt(f),
        }
    }
}

impl error::Error for MapError {}

/// An error obtained when attempting to unmap a virtual region that is already not mapped.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NotMapped;

impl fmt::Display for NotMapped {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "requested unmapping region was not mapped".fmt(f)
    }
}

impl error::Error for NotMapped {}

/// An error obtained when attempting to translate a physical or virtual address using an address
/// space.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NoMapping;

impl fmt::Display for NoMapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "no mapping exists to facilitate the translation".fmt(f)
    }
}

impl error::Error for NoMapping {}
