//! Implementation of the `limine` memory map feature.

use crate::{FeatureRequest, FeatureResponse, COMMON_MAGIC_0, COMMON_MAGIC_1};

/// Request for the memory map from the bootloader.
pub struct MemoryMapRequest;

impl MemoryMapRequest {
    /// Creates a new [`MemoryMapRequest`].
    pub const fn new() -> Self {
        Self
    }
}

impl FeatureRequest for MemoryMapRequest {
    const ID: [u64; 4] = [
        COMMON_MAGIC_0,
        COMMON_MAGIC_1,
        0x67cf3d9d378a806f,
        0xe304acdfc50c3c62,
    ];
    const REVISION: u64 = 0;

    type Response = MemoryMapResponse;
}

/// Response to the [`MemoryMapRequest`] from the bootloader.
///
/// This contains a list of [`MemoryMapEntry`]s that describe the memory map upon entry into the
/// application.
#[repr(C)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryMapResponse {
    entry_count: u64,
    entries: *mut *mut MemoryMapEntry,
}

impl MemoryMapResponse {
    /// Returns a slice of [`MemoryMapEntry`]s, which describes the layout of memory upon entry
    /// into the application.
    pub fn as_slice(&self) -> &'static [&'static MemoryMapEntry] {
        assert!(!self.entries.is_null());
        let slice = unsafe { core::slice::from_raw_parts(self.entries, self.entry_count as usize) };
        for entry in slice {
            assert!(!entry.is_null());
        }

        unsafe {
            core::slice::from_raw_parts(
                self.entries.cast::<&MemoryMapEntry>(),
                self.entry_count as usize,
            )
        }
    }
}

impl FeatureResponse for MemoryMapResponse {
    const REVISION: u64 = 0;
}

/// An entry in the memory map.
#[repr(C)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryMapEntry {
    /// The base address of the entry's memory region.
    pub base: u64,
    /// The length of the the entry's memory region in bytes.
    pub length: u64,
    /// The type of the entry's memory region.
    pub mem_type: MemoryMapEntryType,
}

/// The type of a memory region.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryMapEntryType(u64);

impl MemoryMapEntryType {
    /// The memory region contains usable memory.
    pub const USABLE: Self = Self(0);
    /// The memory region is reserved.
    pub const RESERVED: Self = Self(1);
    /// The memory region contains ACPI data and can be reclaimed.
    pub const ACPI_RECLAIMABLE: Self = Self(2);
    /// The memory region contains ACPI data and is nonvolatile.
    pub const ACPI_NVS: Self = Self(3);
    /// The memory region has bad memory.
    pub const BAD_MEMORY: Self = Self(4);
    /// The memory region contains memory used by the bootloader.
    pub const BOOTLOADER_RECLAIMABLE: Self = Self(5);
    /// The memory region contains the kernel and modules.
    pub const KERNEL_AND_MODULES: Self = Self(6);
    /// The memory region contains a framebuffer.
    pub const FRAMEBUFFER: Self = Self(7);
}
