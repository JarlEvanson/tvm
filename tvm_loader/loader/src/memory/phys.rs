//! Physical memory management interface for `tvm_loader` crates.
//!
//! Provides an interface intended to allocate physical memory in an architecture and platform
//! independent manner.

use core::{error, fmt, ops};

unsafe extern "Rust" {
    static FRAME_ALLOCATOR: &'static dyn FrameAllocator;
}

/// Defines the [`FrameAllocator`].
#[macro_export]
macro_rules! unsafe_frame_allocator {
    ($frame_allocator:expr) => {
        #[unsafe(no_mangle)]
        static FRAME_ALLOCATOR: &'static dyn $crate::memory::phys::FrameAllocator =
            &$frame_allocator;
    };
}

/// Returns the [`FrameAllocator`] for the system.
#[inline]
fn frame_allocator() -> &'static dyn FrameAllocator {
    // SAFETY:
    //
    // All `tvm_loader` system crates are required to implement [`FrameAllocator`] and define
    // this symbol.
    unsafe { FRAME_ALLOCATOR }
}

/// Allocates `count` frames with a minimum `alignment`.
///
/// `alignment` must be a power of two or zero.
/// `flags` determines the manner in which physical memory is allocated.
/// `physical_address`'s effect depends on the value of `flags`.
///
/// # Errors
///
/// Returns [`OutOfMemoryError`] when `count` frames in accordance with the given `flags`
/// cannot be allocated.
pub fn allocate_frame(
    count: usize,
    alignment: usize,
    flags: AllocationFlags,
    physical_address: u64,
) -> Result<u64, OutOfMemoryError> {
    frame_allocator().allocate(count, alignment, flags, physical_address)
}

/// Deallocates the `count` frames located at `base`.
///
/// # Safety
///
/// The caller must ensure that these frames were allocated by a call to
/// [`FrameAllocator::allocate()`] and these frames are not used after this call to
/// [`FrameAllocator::deallocate()`].
pub unsafe fn deallocate_frame(base: u64, count: usize) {
    unsafe { frame_allocator().deallocate(base, count) }
}

/// Returns the size, in bytes, of a frame.
pub fn frame_size() -> u64 {
    frame_allocator().frame_size()
}

/// A physical memory manager capable of allocating usable frames.
///
/// # Safety
///
/// The [`FrameAllocator`] trait is an `unsafe` trait because implementors of this trait must
/// only return frames that they own, and must ensure that the lifetime between
/// [`FrameAllocator::allocate()`] and [`FrameAllocator::deallocate()`] is not
/// disturbed.
pub unsafe trait FrameAllocator: Sync {
    /// Allocates `count` frames with a minimum `alignment`.
    ///
    /// `alignment` must be a power of two or zero.
    /// `flags` determines the manner in which physical memory is allocated.
    /// `physical_address`'s effect depends on the value of `flags`.
    ///
    /// # Errors
    ///
    /// Returns [`OutOfMemoryError`] when `count` frames in accordance with the given `flags`
    /// cannot be allocated.
    fn allocate(
        &self,
        count: usize,
        alignment: usize,
        flags: AllocationFlags,
        physical_address: u64,
    ) -> Result<u64, OutOfMemoryError>;

    /// Deallocates the `count` frames located at `base`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that these frames were allocated by a call to
    /// [`FrameAllocator::allocate()`] and these frames are not used after this call to
    /// [`FrameAllocator::deallocate()`].
    unsafe fn deallocate(&self, base: u64, count: usize);

    /// Returns the size, in bytes, of a frame.
    fn frame_size(&self) -> u64;
}

/// Flags affecting the allocation of physical memory.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct AllocationFlags(pub u64);

impl AllocationFlags {
    /// Allocate the physical memory anywhere in memory.
    ///
    /// Disregards the given `physical_address`.
    ///
    /// Incompatible with [`AllocationFlags::ALLOCATE_BELOW`] and
    /// [`AllocationFlags::ALLOCATE_AT`].
    pub const ALLOCATE_ANY: Self = Self(0);
    /// Allocate the entire physical memory region below the given `physical_address`.
    ///
    /// Incompatible with [`AllocationFlags::ALLOCATE_ANY`] and
    /// [`AllocationFlags::ALLOCATE_AT`].
    pub const ALLOCATE_BELOW: Self = Self(1);
    /// Allocate the physical memory region beginning directly at the given `physical_address`.
    ///
    /// Incompatible with [`AllocationFlags::ALLOCATE_ANY`] and
    /// [`AllocationFlags::ALLOCATE_BELOW`].
    pub const ALLOCATE_AT: Self = Self(2);
}

impl ops::BitOr for AllocationFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl ops::BitOrAssign for AllocationFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl ops::BitAnd for AllocationFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl ops::BitAndAssign for AllocationFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl ops::BitXor for AllocationFlags {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl ops::BitXorAssign for AllocationFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl ops::Not for AllocationFlags {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

/// The error returned from [`FrameAllocator::allocate()`] when the platform is out of memory to allocate.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct OutOfMemoryError;

impl fmt::Display for OutOfMemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "frame allocation failed")
    }
}

impl error::Error for OutOfMemoryError {}
