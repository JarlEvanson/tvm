//! Virtual memory management interfaces for controlling an architecture's address space.
//!
//! The interface is defined in this module and implemented by an architecture or system specific
//! crate.

use crate::memory::virt::{MapError, NoMapping, NotMapped, ProtectionFlags};

/// Interface to manipulate an address space.
///
/// # Safety
///
/// The [`AddressSpace`] trait is an unsafe trait because implementors of this trait must correctly
/// implement the semantics of each method in order to prevent memory corruption.
pub trait AddressSpace {
    /// Maps the `count` frames starting at `physical_address` and extending `count` frames into
    /// the [`AddressSpace`] starting at `virtual_address`. The frames are mapped with the
    /// specified [`ProtectionFlags`].
    ///
    /// # Errors
    ///
    /// - [`MapError::AddressOverflow`]: Returned when the region described by `physical_address`
    ///   or `virtual_address` overflows.
    /// - [`MapError::AlignmentError`]: Returned when the `physical_address` or the
    ///   `virtual_address` is not aligned to the [`AddressSpace::page_size()`].
    /// - [`MapError::AllocationError`]: Returned when an error allocating memory required to map
    ///   the region occurs.
    /// - [`MapError::AlreadyMapped`]: Returned when the [`AddressSpace`] cannot find a free
    ///   region to map the requested physical region.
    /// - [`MapError::GeneralError`]: Returned when [`AddressSpace::map()`] fails in a manner
    ///   that does not belong to any other [`MapError`] value.
    /// - [`MapError::InvalidAddress`]: Returned when `physical_address` or `virtual_address`
    ///   is not a valid address.
    /// - [`MapError::InvalidSize`]: Returned when the size of the region is too large.
    fn map(
        &mut self,
        virtual_address: u64,
        physical_address: u64,
        count: u64,
        protection: ProtectionFlags,
    ) -> Result<(), MapError>;

    /// Unmaps `count` pages at starting at `virtual_address`.
    ///
    /// # Errors
    ///
    /// [`NotMapped`] is returned if the virtual region to be unmapped was not entirely mapped.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the deallocated pages were allocated by a call to
    /// [`AddressSpace::map()`] and that these pages are not used after this call to
    /// [`AddressSpace::unmap()`].
    unsafe fn unmap(&mut self, virtual_address: u64, count: u64) -> Result<(), NotMapped>;

    /// Translates the given `virtual_address` to its `physical_address`.
    ///
    /// # Errors
    ///
    /// Returns [`NoMapping`] if there exists no mapping from `virtual_address` to a physical
    /// address.
    fn translate_virt(&self, virtual_address: u64) -> Result<u64, NoMapping>;

    /// Returns the size, in bytes, of a page.
    fn page_size(&self) -> u64;

    /// Returns the maximum address.
    fn max_address(&self) -> u64;
}
