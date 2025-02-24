//! Physical memory management interface for `tvm_loader`.
//!
//! The interface is defined in this module and implemented by a platform or system specific crate.

use core::{error, fmt};

unsafe extern "Rust" {
    static FRAME_ALLOCATOR: &'static dyn FrameAllocator;
}

/// Defines the globally available [`FrameAllocator`].
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
    // All `tvm_loader` system crates are required to use a correct implementation of
    // [`FrameAllocator`] and specify its usage through [`unsafe_frame_allocator`].
    unsafe { FRAME_ALLOCATOR }
}

/// Allocates `count` frames aligned to `alignment`.
///
/// `alignment` must be a power of two or zero.
///
/// # Errors
///
/// [`OutOfMemoryError`]: Returned when `count` frames cannot be allocated aligned to `alignment`
/// with the specified `allocation_type`.
pub fn allocate_frame(
    allocation_type: AllocationType,
    count: u64,
    alignment: u64,
) -> Result<u64, OutOfMemoryError> {
    frame_allocator().allocate(allocation_type, count, alignment)
}

/// Deallocates the `count` frames starting at `physical_address`.
///
/// # Safety
///
/// The caller must ensure that the frames were allocated by one or more calls to
/// [`FrameAllocator::allocate()`] and these frames are not used after this call.
pub unsafe fn deallocate_frame(physical_address: u64, count: u64) {
    // SAFETY:
    //
    // The invariants required to call [`FrameAllocator::deallocate()`] are fulfilled by the
    // invariants of [`deallocate_frame()`].
    unsafe { frame_allocator().deallocate(physical_address, count) }
}

/// Returns the size of a frame in bytes.
pub fn frame_size() -> u64 {
    frame_allocator().frame_size()
}

/// A frame region manager capable of allocating usable physical memory.
///
/// # Safety
///
/// The [`FrameAllocator`] is an `unsafe` trait because implementors must ensure that they adhere
/// to these contracts:
///
/// - Allocated frames must point to valid physical memory, and retain their validty until the
///   the frames are deallocated by a call to [`FrameAllocator::deallocate()`].
/// - All methods must have their semantics properly implemented.
pub unsafe trait FrameAllocator: Sync {
    /// Allocates `count` frames aligned to `alignment`.
    ///
    /// `alignment` must be a power of two or zero.
    ///
    /// # Errors
    ///
    /// [`OutOfMemoryError`]: Returned when `count` frames cannot be allocated aligned to
    /// `alignment` with the specified `allocation_type`.
    fn allocate(
        &self,
        allocation_type: AllocationType,
        count: u64,
        alignment: u64,
    ) -> Result<u64, OutOfMemoryError>;

    /// Deallocates the `count` frames starting at `physical_address`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the frames were allocated by one or more calls to
    /// [`FrameAllocator::allocate()`] and these frames are not used after this call.
    unsafe fn deallocate(&self, physical_address: u64, count: u64);

    /// Returns the size of a frame in bytes.
    fn frame_size(&self) -> u64;
}

/// How the frame region should be allocated.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum AllocationType {
    /// Allocate the frames anywhere in memory.
    #[default]
    Any,
    /// Allocate the entire frame region below the given `physical_address`.
    Below {
        /// The physical_address below which the returned frame region must end.
        physical_address: u64,
    },
    /// Allocate the frames starting at `physical_address`.
    At {
        /// The physical address at which the returned frame region must start.
        physical_address: u64,
    },
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
