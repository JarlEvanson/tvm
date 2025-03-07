//! Definitions of the supported `tvm` system configurations.

use x86_64_pc::X86_64_PC;
use x86_pc::X86_PC;

use crate::{Arch, loader::Loader};

pub mod x86_64_pc;
pub mod x86_pc;

/// List of all of the `tvm` system crates.
pub static TVM_SYSTEMS: &[&TvmSystem] = &[X86_PC, X86_64_PC];

/// Returns the [`TvmSystem`] associated with the given [`Arch`].
pub fn lookup(arch: Arch) -> Option<&'static TvmSystem> {
    TVM_SYSTEMS
        .iter()
        .copied()
        .find(|tvm_system| tvm_system.arch == arch)
}

/// The description of a `tvm` system crate.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct TvmSystem {
    /// The name of the `tvm` system.
    pub name: &'static str,

    /// The architecture this `tvm` system targets.
    pub arch: Arch,
    /// The target string to be passed to `cargo` when building.
    pub target: &'static str,
    /// The valid features for this `tvm_loader` crate.
    pub features: &'static [&'static str],

    /// The [`Loader`]s that are compatible with this `tvm` system.
    pub compatible_loaders: &'static [&'static Loader],

    /// Additional arguments to pass to `cargo` when building.
    pub additional_build_arguments: &'static [&'static str],
}
