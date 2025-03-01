//! Common `x86` and `x86_64` specific paging-related constants and traits.

use tvm_loader::memory::virt::arch::AddressSpace;
use x86_common::PagingMode;

/// All accesses are uncacheable. Write combining is not allowed. Speculative accesses are not
/// allowed.
pub const UNCACHEABLE: u64 = 0x00;
/// All accesses are uncacheable. Write combining is allowed. Speculative accesses are not allowed.
pub const WRITE_COMBINING: u64 = 0x01;
/// Reads allocate cache lines on a cache miss. Cache lines are not allocated on a write miss.
///
/// Write hits update the cache and main memory.
pub const WRITETHROUGH: u64 = 0x04;
/// Reads allocate cache lines on a cache miss. All writes update main memory.
///
/// Cache lines are not allocated on a write miss. Write hits invalidate the cache line and update
/// main memory.
pub const WRITE_PROTECT: u64 = 0x05;
/// Reads allocate cache lines on a cache miss, and can allocate to the shared, exclusive, or
/// modified state.
///
/// Writes allocate to the modified state on a cache miss.
pub const WRITEBACK: u64 = 0x06;
/// Same as uncacheable, except that this can be overriden by [`WRITE_COMBINING`] MTRRs.
pub const UNCACHED: u64 = 0x07;

/// The layout of the PAT register used for all loader created page tables.
pub const PAT: u64 = WRITEBACK
    | (WRITETHROUGH << 8)
    | (UNCACHED << 16)
    | (UNCACHEABLE << 24)
    | (WRITE_COMBINING << 32)
    | (WRITE_PROTECT << 40)
    | (UNCACHEABLE << 48)
    | (UNCACHEABLE << 56);

/// Interface providing information about `x86` and `x86_64` page table schemes.
pub trait X86CommonAddressSpace: AddressSpace {
    /// The physical address at the top of the page table scheme.
    fn physical_address(&self) -> u64;
    /// The `x86/x86_64` specific paging mode that the [`AddressSpace`] implements.
    fn paging_mode(&self) -> PagingMode;
}
