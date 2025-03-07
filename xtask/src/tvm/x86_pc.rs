//! Definition of the `x86-pc` `tvm` system.

use crate::{
    Arch,
    loader::{x86_64_uefi::X86_64_UEFI, x86_uefi::X86_UEFI},
    tvm::TvmSystem,
};

/// [`TvmSystem`] definition for `tvm-x86-pc`.
pub static X86_PC: &TvmSystem = &TvmSystem {
    name: "x86-pc",

    arch: Arch::X86,
    target: "target_specs/i686-unknown-none.json",
    features: &[],

    compatible_loaders: &[X86_UEFI, X86_64_UEFI],

    additional_build_arguments: &["-Zbuild-std=core"],
};
