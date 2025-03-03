//! Definition of the `x86_64-uefi` `tvm_loader` system.

use crate::{
    cli::{Arch, BootPlatform},
    loader::Loader,
};

/// [`Loader`] definition for `tvm_loader-x86-uefi`.
pub static X86_64_UEFI: &Loader = &Loader {
    name: "x86_64-uefi",
    arch: Arch::X86_64,
    platform: BootPlatform::Uefi,
    target: "x86_64-unknown-uefi",
    features: &[],
    additional_build_arguments: &[],
};
