//! Definitions of the supported `tvm` system configurations.

use crate::{cli::Arch, loader::Loader};

pub mod x86_64_pc;
pub mod x86_pc;

/// List of all the `tvm` system crates.
pub static TVMS: &[&Tvm] = &[x86_pc::X86_PC, x86_64_pc::X86_64_PC];

/// The description of a `tvm` system crate.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Tvm {
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
