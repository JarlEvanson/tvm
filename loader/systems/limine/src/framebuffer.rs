//! Implementation of the `limine` framebuffer feature.

use crate::{FeatureRequest, FeatureResponse, COMMON_MAGIC_0, COMMON_MAGIC_1};

/// Request for the memory map from the bootloader.
pub struct FramebufferRequest;

impl FramebufferRequest {
    /// Creates a new [`FramebufferRequest`].
    pub const fn new() -> Self {
        Self
    }
}

impl FeatureRequest for FramebufferRequest {
    const ID: [u64; 4] = [
        COMMON_MAGIC_0,
        COMMON_MAGIC_1,
        0x9d5827dcd881dd75,
        0xa3148604f6fab11b,
    ];
    const REVISION: u64 = 0;

    type Response = FramebufferResponse;
}

/// Response to the [`FramebufferRequest`] from the bootloader.
///
/// This contains a list of [`Framebuffer`]s that describe the framebuffers available on the
/// system.
#[repr(C)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FramebufferResponse {
    framebuffer_count: u64,
    framebuffers: *mut *mut Framebuffer,
}

impl FramebufferResponse {
    /// Returns a slice of [`Framebuffer`]s, which describes the framebuffers available on the
    /// system.
    pub fn as_slice(&self) -> &'static [&'static Framebuffer] {
        assert!(!self.framebuffers.is_null());
        let slice = unsafe {
            core::slice::from_raw_parts(self.framebuffers, self.framebuffer_count as usize)
        };
        for entry in slice {
            assert!(!entry.is_null());
        }

        unsafe {
            core::slice::from_raw_parts(
                self.framebuffers.cast::<&Framebuffer>(),
                self.framebuffer_count as usize,
            )
        }
    }
}

impl FeatureResponse for FramebufferResponse {
    const REVISION: u64 = 1;
}

/// A framebuffer on the current system.
#[repr(C)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Framebuffer {
    /// The base address of the framebuffer.
    pub address: *mut u8,
    /// The width, in pixels, of the screen.
    pub width: u64,
    /// The height, in pixels, of the screen.
    pub height: u64,
    /// The stride, in pixels, of the screen.
    pub pitch: u64,
    /// The number of bits per pixel of the screen.
    pub bpp: u16,
    /// The memory model of the screen.
    pub memory_model: u8,
    /// The number of bits to represent red.
    pub red_mask_size: u8,
    /// The offset at which the red bits start.
    pub red_mask_shift: u8,
    /// The number of bits to represent green,
    pub green_mask_shift: u8,
    /// The offset at which the greeen bits start.
    pub green_mask_size: u8,
    /// The number of bits to represent blue.
    pub blue_mask_size: u8,
    /// The offset at which the blue bits start.
    pub blue_mask_shift: u8,
    unused: [u8; 7],

    /// The size of the EDID blob in bytes.
    pub edid_size: u64,
    /// Pointer to the EDID blob.
    ///
    /// This may be NULL.
    pub edid: *mut u8,

    mode_count: u64,
    modes: *mut *mut VideoMode,
}

impl Framebuffer {
    /// Returns a slice of the [`VideoMode`]s available to the [`Framebuffer`].
    pub fn modes(&self) -> &'static [&'static VideoMode] {
        assert!(!self.modes.is_null());
        let slice = unsafe { core::slice::from_raw_parts(self.modes, self.mode_count as usize) };
        for entry in slice {
            assert!(!entry.is_null());
        }

        unsafe {
            core::slice::from_raw_parts(self.modes.cast::<&VideoMode>(), self.mode_count as usize)
        }
    }
}

/// Description of a video mode of a framebuffer.
#[repr(C)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct VideoMode {
    /// The stride, in pixels, of the screen.
    pub pitch: u64,
    /// The width, in pixels, of the screen.
    pub width: u64,
    /// The height, in pixels, of the screen.
    pub height: u64,
    /// The number of bits per pixel of the screen.
    pub bpp: u16,
    /// The memory model of the screen.
    pub memory_model: u8,
    /// The number of bits to represent red.
    pub red_mask_size: u8,
    /// The offset at which the red bits start.
    pub red_mask_shift: u8,
    /// The number of bits to represent green,
    pub green_mask_shift: u8,
    /// The offset at which the greeen bits start.
    pub green_mask_size: u8,
    /// The number of bits to represent blue.
    pub blue_mask_size: u8,
    /// The offset at which the blue bits start.
    pub blue_mask_shift: u8,
}
