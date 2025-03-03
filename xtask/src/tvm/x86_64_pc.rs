//! Definition of the `x86_64-pc` `tvm` system.

use crate::{
    cli::Arch,
    loader::{x86_64_uefi::X86_64_UEFI, x86_uefi::X86_UEFI},
    tvm::Tvm,
};

/// [`Tvm`] definition for `tvm-x86_64-pc`.
pub static X86_64_PC: &Tvm = &Tvm {
    name: "x86_64-pc",

    arch: Arch::X86_64,
    target: "x86_64-unknown-none",
    features: &[],

    compatible_loaders: &[X86_UEFI, X86_64_UEFI],

    additional_build_arguments: &[],
};
