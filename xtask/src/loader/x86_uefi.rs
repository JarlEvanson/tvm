//! Definition of the `x86-uefi` `tvm_loader` system.

use crate::{
    Arch,
    loader::{Loader, Platform},
};

/// [`Loader`] definition for `tvm_loader-x86-uefi`.
pub static X86_UEFI: &Loader = &Loader {
    name: "x86-uefi",
    arch: Arch::X86,
    platform: Platform::Uefi,
    target: "i686-unknown-uefi",
    features: &[],
    additional_build_arguments: &[],
};
