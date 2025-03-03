//! Definitions of the supported `tvm_loader` system configurations.

use crate::cli::{Arch, BootPlatform};

pub mod x86_64_uefi;
pub mod x86_uefi;

/// List of all of the `tvm_loader` system crates.
pub static LOADERS: &[&Loader] = &[x86_uefi::X86_UEFI, x86_64_uefi::X86_64_UEFI];

/// The description of a `tvm_loader` system crate.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Loader {
    /// The name of the `tvm_loader` system.
    pub name: &'static str,

    /// The architecture this `tvm_loader` system targets.
    pub arch: Arch,
    /// The platform this `tvm_loader` system targets.
    pub platform: BootPlatform,
    /// The target string to be passed to `cargo` when building.
    pub target: &'static str,
    /// The valid features for this `tvm_loader` crate.
    pub features: &'static [&'static str],

    /// Additional arguments to pass to `cargo` when building.
    pub additional_build_arguments: &'static [&'static str],
}
