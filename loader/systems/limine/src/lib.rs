//! Support code for systems booted using `limine`.

#![no_std]

use core::cell::UnsafeCell;

pub mod framebuffer;
pub mod memory_map;

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

const COMMON_MAGIC_0: u64 = 0xc7b1dd30df4c8b88;
const COMMON_MAGIC_1: u64 = 0x0a82e883a194f07b;

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

/// The base structure of a `limine` feature's request.
#[repr(C)]
pub struct Request<R: FeatureRequest> {
    id: [u64; 4],
    revision: UnsafeCell<u64>,
    response: UnsafeCell<*mut Response<R::Response>>,
    body: R,
}

impl<R: FeatureRequest> Request<R> {
    /// Creates a new [`Request`] to carry out.
    pub const fn new(body: R) -> Self {
        Self {
            id: R::ID,
            revision: UnsafeCell::new(R::REVISION),
            response: UnsafeCell::new(core::ptr::null_mut()),
            body,
        }
    }

    /// Returns the [`Response<R::Response>`] if the request is supported.
    ///
    /// Otherwise, returns `None`.
    pub fn response(&self) -> Option<&Response<R::Response>> {
        let ptr = unsafe { self.response.get().read_volatile() };
        unsafe { ptr.as_ref() }
    }
}

unsafe impl<R: FeatureRequest> Sync for Request<R> {}
unsafe impl<R: FeatureRequest> Send for Request<R> {}

/// The base structure of a `limine` feature's response.
#[repr(C)]
pub struct Response<R: FeatureResponse> {
    revision: u64,
    body: R,
}

impl<R: FeatureResponse> Response<R> {
    /// Returns the body of this [`Response`] if the revision is supported.
    ///
    /// Otherwise, returns `None`.
    pub fn body(&self) -> Option<&R> {
        if !self.is_supported() {
            return None;
        }

        Some(&self.body)
    }

    /// Returns `true` if the revision of this [`Response`] is supported.
    pub fn is_supported(&self) -> bool {
        self.revision() >= R::REVISION
    }

    /// Returns the revision of this [`Response`].
    pub fn revision(&self) -> u64 {
        self.revision
    }
}

/// Describes the request structure of a `limine` feature.
pub trait FeatureRequest {
    /// The ID used to indicate the feature with which this request is associated.
    const ID: [u64; 4];
    /// The revision of the feature's request.
    const REVISION: u64;

    /// The type of the response.
    type Response: FeatureResponse;
}

/// Describes the response structure of a `limine` feature.
pub trait FeatureResponse {
    /// The revision of the feature's response.
    const REVISION: u64;
}
