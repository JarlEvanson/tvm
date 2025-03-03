//! Definition of the `x86-uefi` `tvm_loader` system.

use crate::{
    cli::{Arch, BootPlatform},
    loader::Loader,
};

/// [`Loader`] definition for `tvm_loader-x86-uefi`.
pub static X86_UEFI: &Loader = &Loader {
    name: "x86-uefi",
    arch: Arch::X86,
    platform: BootPlatform::Uefi,
    target: "i686-unknown-uefi",
    features: &[],
    additional_build_arguments: &[],
};
