//! Virtual memory management interfaces for controlling the virtual memory space of `tvm_loader`.
//!
//! The interface is defined in this module and implemented by a platform or system specific crate.

use crate::memory::virt::{MapError, NoMapping, NotMapped, ProtectionFlags};

unsafe extern "Rust" {
    static LOADER_ADDRESS_SPACE: &'static dyn LoaderAddressSpace;
}

/// Defines the [`LoaderAddressSpace`].
#[macro_export]
macro_rules! unsafe_loader_address_space {
    ($page_allocator:expr) => {
        #[unsafe(no_mangle)]
        static LOADER_ADDRESS_SPACE: &'static dyn $crate::memory::virt::loader::LoaderAddressSpace =
            &$page_allocator;
    };
}

/// Returns the [`LoaderAddressSpace`] for the system.
#[inline]
fn loader_address_space() -> &'static dyn LoaderAddressSpace {
    // SAFETY:
    //
    // All `tvm_loader` system crates are required to implement [`LoaderAddressSpace`] and define
    // this symbol.
    unsafe { LOADER_ADDRESS_SPACE }
}

/// Maps the `count` frames starting at `physical_address` with the specified `protection` flags
/// into the loader's address space.
///
/// # Errors
///
/// - [`MapError::AddressOverflow`]: Returned when the region described by `physical_address`
///   overflows.
/// - [`MapError::AlignmentError`]: Returned when the `physical_address` is not aligned to the
///   [`LoaderAddressSpace::page_size()`].
/// - [`MapError::AllocationError`]: Returned when an error allocating memory required to map
///   the region occurs.
/// - [`MapError::AlreadyMapped`]: Returned when the [`LoaderAddressSpace`] cannot find a free
///   region to map the requested physical region.
/// - [`MapError::GeneralError`]: Returned when [`LoaderAddressSpace::map()`] fails in a manner
///   that does not belong to any other [`MapError`] value.
/// - [`MapError::InvalidAddress`]: Returned when `physical_address` is not a valid address.
/// - [`MapError::InvalidSize`]: Returned when the size of the region is too large.
pub fn map(
    physical_address: u64,
    count: usize,
    protection: ProtectionFlags,
) -> Result<usize, MapError> {
    loader_address_space().map(physical_address, count, protection)
}

/// Unmaps `count` pages at starting at `virtual_address`.
///
/// # Errors
///
/// [`NotMapped`] is returned if the virtual region to be unmapped was not entirely mapped.
///
/// # Safety
///
/// The caller must ensure that the deallocated pages were allocated by a call to
/// [`LoaderAddressSpace::map()`] and that these pages are not used after this call to
/// [`LoaderAddressSpace::unmap()`].
pub unsafe fn unmap(virtual_address: usize, count: usize) -> Result<(), NotMapped> {
    // SAFETY:
    //
    // The invariants required to call [`LoaderAddressSpace::unmap()`] are fulfilled by the
    // invariants required to call [`unmap()`].
    unsafe { loader_address_space().unmap(virtual_address, count) }
}

/// Translates the given `virtual_address` to its `physical_address`.
///
/// # Errors
///
/// Returns [`NoMapping`] if there exists no mapping from `virtual_address` to a physical address.
pub fn translate_virt(virtual_address: usize) -> Result<u64, NoMapping> {
    loader_address_space().translate_virt(virtual_address)
}

/// Translates the given `physical_address` to its `virtual_address`.
///
/// # Errors
///
/// Returns [`NoMapping`] if there exists no mapping from `physical_address` to a virtual address.
pub fn translate_phys(physical_address: u64) -> Result<usize, NoMapping> {
    loader_address_space().translate_phys(physical_address)
}

/// Returns the size, in bytes, of a page.
pub fn page_size() -> usize {
    loader_address_space().page_size()
}

/// Interface to manipulate `tvm_loader`'s address space.
///
/// # Safety
///
/// The [`LoaderAddressSpace`] trait is an unsafe trait because implementors of this trait must
/// correctly implement the semantics of each method in order to prevent memory corruption.
pub unsafe trait LoaderAddressSpace: Sync {
    /// Maps the `count` frames starting at `physical_address` with the specified `protection`
    /// flags into the loader's address space.
    ///
    /// # Errors
    ///
    /// - [`MapError::AddressOverflow`]: Returned when the region described by `physical_address`
    ///   overflows.
    /// - [`MapError::AlignmentError`]: Returned when the `physical_address` is not aligned to the
    ///   [`LoaderAddressSpace::page_size()`].
    /// - [`MapError::AllocationError`]: Returned when an error allocating memory required to map
    ///   the region occurs.
    /// - [`MapError::AlreadyMapped`]: Returned when the [`LoaderAddressSpace`] cannot find a free
    ///   region to map the requested physical region.
    /// - [`MapError::GeneralError`]: Returned when [`LoaderAddressSpace::map()`] fails in a manner
    ///   that does not belong to any other [`MapError`] value.
    /// - [`MapError::InvalidAddress`]: Returned when `physical_address` is not a valid address.
    /// - [`MapError::InvalidSize`]: Returned when the size of the region is too large.
    fn map(
        &self,
        physical_address: u64,
        count: usize,
        protection: ProtectionFlags,
    ) -> Result<usize, MapError>;

    /// Unmaps `count` pages at starting at `virtual_address`.
    ///
    /// # Errors
    ///
    /// [`NotMapped`] is returned if the virtual region to be unmapped was not entirely mapped.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the deallocated pages were allocated by a call to
    /// [`LoaderAddressSpace::map()`] and that these pages are not used after this call to
    /// [`LoaderAddressSpace::unmap()`].
    unsafe fn unmap(&self, virtual_address: usize, count: usize) -> Result<(), NotMapped>;

    /// Translates the given `virtual_address` to its `physical_address`.
    ///
    /// # Errors
    ///
    /// Returns [`NoMapping`] if there exists no mapping from `virtual_address` to a physical
    /// address.
    fn translate_virt(&self, virtual_address: usize) -> Result<u64, NoMapping>;

    /// Translates the given `physical_address` to its `virtual_address`.
    ///
    /// # Errors
    ///
    /// Returns [`NoMapping`] if there exists no mapping from `physical_address` to a virtual
    /// address.
    fn translate_phys(&self, physical_address: u64) -> Result<usize, NoMapping>;

    /// Returns the size, in bytes, of a page.
    fn page_size(&self) -> usize;
}
