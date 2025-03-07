//! Definition of the `x86_64-uefi` `tvm_loader` system.

use crate::{
    Arch,
    loader::{Loader, Platform},
};

/// [`Loader`] definition for `tvm_loader-x86-uefi`.
pub static X86_64_UEFI: &Loader = &Loader {
    name: "x86_64-uefi",
    arch: Arch::X86_64,
    platform: Platform::Uefi,
    target: "x86_64-unknown-uefi",
    features: &[],
    additional_build_arguments: &[],
};
