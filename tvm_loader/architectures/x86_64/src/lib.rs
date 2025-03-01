//! Support code for `tvm_loader` crates for the `x86_64` architecture.

#![no_std]

mod paging;
mod relocation;

pub use paging::{FeatureNotSupported, X86_64PageTable};
pub use relocation::handle_relocation;
