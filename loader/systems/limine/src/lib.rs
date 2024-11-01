//! Support code for systems booted using `limine`.

#![no_std]

use core::cell::UnsafeCell;

/// The base revision of the `limine` protocol.
pub const BASE_REVISION: u64 = 3;

/// Marks the start of the requests section.
pub const REQUESTS_START_MARKER: [u64; 4] = [
    0xf6b8f4b39de7d1ae,
    0xfab91a6940fcb9cf,
    0x785c6ed015d3e316,
    0x181e920a7852b9d9,
];

/// Marks the end of the requests section.
pub const REQUESTS_END_MARKER: [u64; 2] = [0xadc0e0531bb10d03, 0x9572709f31764c62];

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
