//! Implementation of the `limine` HHDM (High Half Direct Map) feature.

use crate::{FeatureRequest, FeatureResponse, COMMON_MAGIC_0, COMMON_MAGIC_1};

/// Request for the higher half direct map from the bootloader.
pub struct HhdmRequest;

impl HhdmRequest {
    /// Creates a new [`HhdmRequest`].
    pub const fn new() -> Self {
        Self
    }
}

impl FeatureRequest for HhdmRequest {
    const ID: [u64; 4] = [
        COMMON_MAGIC_0,
        COMMON_MAGIC_1,
        0x48dcf1cb8ad2b852,
        0x63984e959a98244b,
    ];
    const REVISION: u64 = 0;

    type Response = HhdmResponse;
}

/// Response to the [`HhdmRequest`] from the bootloader.
///
/// This contains the offset of the higher half direct map.
#[repr(C)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct HhdmResponse {
    /// The offset of the higher half direct map.
    pub offset: u64,
}

impl FeatureResponse for HhdmResponse {
    const REVISION: u64 = 0;
}
