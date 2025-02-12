//! Definitions of the boot interface.

#![no_std]

/// Information given to `tvm` and interfaces for allocating and mapping memory
#[repr(C)]
pub struct BootInfo {
    /// List of pointers to utf8 cstrings.
    ///
    /// Ends with a null pointer.
    pub arguments: *const *const u8,

    // System information
    /// Physical address of the ACPI root table.
    pub acpi_root_table: u64,
    /// Physical address of the device tree blob.
    pub device_tree: u64,
    /// Physical address of the entry point for 32-bit SMBIOS.
    pub smbios_entry_32: u64,
    /// Physical address of the entry point for 64-bit SMBIOS.
    pub smbios_entry_64: u64,
    /// Physical address of the UEFI system table.
    pub uefi_system_table: u64,

    // Functions
    /// Outputs the utf8 string located at `ptr` and extending `length` bytes to the bootloader's
    /// output device.
    pub write: extern "C" fn(ptr: *const u8, length: usize),

    /// Allocates `count` frames with the specified `alignment` and returns the starting physical
    /// address of the frame region.
    ///
    /// The zero frame will never be returned from this function.
    pub allocate_frames: extern "C" fn(count: u64, alignment: u64) -> u64,
    /// Deallocates `count` frames starting at `physical_address`.
    pub deallocate_frames: unsafe extern "C" fn(physical_address: u64, count: u64),
    /// Returns the size, in bytes, of a frame in the bootloader.
    pub frame_size: extern "C" fn() -> usize,

    /// Maps the frame region of starting at `physical_address` and extending `page_size() *
    /// count` bytes into `tvm`'s address space with the specified protection flags.
    pub map: extern "C" fn(physical_address: u64, count: usize, flags: ProtectionFlags) -> usize,
    /// Unmaps the page region starting at `virtual_address` and extending `page_size() * `count`
    /// bytes from `tvm`'s address space.
    pub unmap: unsafe extern "C" fn(virtual_address: usize, count: usize),
    /// Returns the size, in bytes, of a page in the bootloader.
    pub page_size: extern "C" fn() -> usize,

    /// Signals to the bootloader that `tvm` is transitioning to control of the computer.
    ///
    /// Much of the behavior of this function is determined by `flags`.
    pub transition: extern "C" fn(flags: TransitionFlags),
}

/// Raw structure used to create raw [`BootInfo`] structs for other address spaces.
#[allow(missing_docs)]
#[repr(C)]
pub struct BootInfoRaw<P, F> {
    pub arguments: P,

    pub acpi_root_table: u64,
    pub device_tree: u64,
    pub smbios_entry_32: u64,
    pub smbios_entry_64: u64,
    pub uefi_system_table: u64,

    pub write: F,

    pub allocate_frames: F,
    pub deallocate_frames: F,
    pub frame_size: F,

    pub map: F,
    pub unmap: F,
    pub page_size: F,

    pub transition: F,
}

/// Protection settings for a page in the `tvm` address space.
#[repr(transparent)]
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

/// Flags controlling the behavior of the takeover function.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransitionFlags(u32);

impl TransitionFlags {}

impl core::ops::BitOr for TransitionFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for TransitionFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl core::ops::BitAnd for TransitionFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl core::ops::BitAndAssign for TransitionFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl core::ops::BitXor for TransitionFlags {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl core::ops::BitXorAssign for TransitionFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl core::ops::Not for TransitionFlags {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}
