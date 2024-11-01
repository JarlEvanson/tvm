//! Support code for systems booted using `limine`.

#![no_std]

use core::cell::UnsafeCell;

/// The base revision of the `limine` protocol.
pub const BASE_REVISION: u64 = 3;

/// A tag setting the base revision supported by the application.
#[repr(C)]
pub struct BaseRevisionTag {
    /// Magic number.
    magic_0: u64,
    /// The base revision of the `limine` protocol to load the application.
    loaded_revision: UnsafeCell<u64>,
    /// Whether the applications's base revisio is supported by the bootloader.
    supported_revision: UnsafeCell<u64>,
}

impl BaseRevisionTag {
    /// Creates a new [`BaseRevisionTag`] with the latest revision.
    pub const fn new() -> Self {
        Self {
            magic_0: 0xf9562b2d5c95a6c8,
            loaded_revision: UnsafeCell::new(0x6a7b384944536bdc),
            supported_revision: UnsafeCell::new(BASE_REVISION),
        }
    }

    /// Returns `true` if the base revision of the `limine` protocol is supported by the
    /// bootloader.
    pub fn is_supported(&self) -> bool {
        (unsafe { self.supported_revision.get().read_volatile() }) == 0
    }

    /// Returns the actual base revision of the `limine` protocol used to load this application.
    pub fn loaded_revision(&self) -> u64 {
        unsafe { self.loaded_revision.get().read_volatile() }
    }
}

unsafe impl Sync for BaseRevisionTag {}
unsafe impl Send for BaseRevisionTag {}
