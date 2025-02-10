//! Virtual memory management interface for `tvm_loader` crates.
//!
//! Provides interfaces intended to allow `tvm_loader` crates to map, unmap, and translate address
//! in an architecture and platform independent manner.

unsafe extern "Rust" {
    static LOADER_ADDRESS_SPACE: &'static dyn LoaderAddressSpace;
}

/// Defines the [`LoaderAddressSpace`].
#[macro_export]
macro_rules! unsafe_loader_address_space {
    ($page_allocator:expr) => {
        #[unsafe(no_mangle)]
        static LOADER_ADDRESS_SPACE: &'static dyn $crate::memory::virt::LoaderAddressSpace =
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

/// Maps the `count` frames starting at `physical_address` with the specified `protection`
/// flags into the loader's address space.
pub fn map(
    physical_address: u64,
    count: usize,
    protection: ProtectionFlags,
) -> Result<usize, MapError> {
    loader_address_space().map(physical_address, count, protection)
}

/// Unmaps `count` pages at starting at `virtual_address`.
///
/// # Safety
///
/// The caller must ensure that the deallocated pages were allocated by a call to [`map()`] and
/// that these pages are not used after this call to [`unmap()`].
pub unsafe fn unmap(virtual_address: usize, count: usize) -> Result<(), ()> {
    unsafe { loader_address_space().unmap(virtual_address, count) }
}

/// Translates the given `virtual_address` to its `physical_address`.
pub fn translate_virt(virtual_address: usize) -> Result<u64, ()> {
    loader_address_space().translate_virt(virtual_address)
}

/// Translates the given `physical_address` to its `virtual_address`.
pub fn translate_phys(physical_address: u64) -> Result<usize, ()> {
    loader_address_space().translate_phys(physical_address)
}

/// Returns the size, in bytes, of a page.
pub fn page_size() -> usize {
    loader_address_space().page_size()
}

/// Returns the maximum address.
pub fn max_address() -> usize {
    loader_address_space().max_address()
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
    fn map(
        &self,
        physical_address: u64,
        count: usize,
        protection: ProtectionFlags,
    ) -> Result<usize, MapError>;

    /// Unmaps `count` pages at starting at `virtual_address`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the deallocated pages were allocated by a call to
    /// [`LoaderAddressSpace::map()`] and that these pages are not used after this call to
    /// [`LoaderAddressSpace::unmap()`].
    unsafe fn unmap(&self, virtual_address: usize, count: usize) -> Result<(), ()>;

    /// Translates the given `virtual_address` to its `physical_address`.
    fn translate_virt(&self, virtual_address: usize) -> Result<u64, ()>;

    /// Translates the given `physical_address` to its `virtual_address`.
    fn translate_phys(&self, physical_address: u64) -> Result<usize, ()>;

    /// Returns the size, in bytes, of a page.
    fn page_size(&self) -> usize;

    /// Returns the maximum address.
    fn max_address(&self) -> usize;
}

/// Interface to manipulate an address space.
///
/// # Safety
///
/// The [`AddressSpace`] trait is an unsafe trait because implementors of this trait must correctly
/// implement the semantics of each method in order to prevent memory corruption.
pub unsafe trait AddressSpace {
    /// Maps the `count` frames starting at `physical_address` to the `count` pages starting at
    /// `virtual_address` with the specified `protection` flags.
    fn map(
        &mut self,
        virtual_address: u64,
        physical_address: u64,
        count: u64,
        protection: ProtectionFlags,
    ) -> Result<(), MapError>;

    /// Unmaps `count` pages at starting at `virtual_address`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the deallocated pages were allocated by a call to
    /// [`AddressSpace::map()`] and that these pages are not used after this call to
    /// [`AddressSpace::unmap()`].
    unsafe fn unmap(&mut self, virtual_address: u64, count: u64) -> Result<(), ()>;

    /// Translates the given `virtual_address` to its `physical_address`.
    fn translate_virt(&self, virtual_address: u64) -> Result<u64, ()>;

    /// Returns the size, in bytes, of a page.
    fn page_size() -> u64;

    /// Returns the maximum address.
    fn max_address() -> u64;
}

/// Various errors that can occur while mapping memory.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum MapError {
    /// Desired mapping would overflow the [`AddressSpace`].
    AddressOverflow,
    /// Requested addresses are not aligned.
    AlignmentError,
    /// An error occurred allocating space for the address space.
    AllocationError,
    /// Requested virtual address was already mapped.
    AlreadyMapped,
    /// An unspecified error occurred.
    GeneralError,
    /// Requested address is not valid.
    InvalidAddress,
    /// Total requested size is larger than address space.
    InvalidSize,
}

impl core::fmt::Display for MapError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AddressOverflow => f.write_str("requested mapping involves overflow"),
            Self::AlignmentError => f.write_str("requested address not aligned"),
            Self::AllocationError => f.write_str("an error occurred allocating memory"),
            Self::AlreadyMapped => f.write_str("requested address is already mapped"),
            Self::GeneralError => f.write_str("an unspecified error occurred"),
            Self::InvalidAddress => f.write_str("requested mapping involves an invalid address"),
            Self::InvalidSize => f.write_str("requested mapping size is too large"),
        }
    }
}

impl core::error::Error for MapError {}

/// Protection settings for a page in an [`AddressSpace`].
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
