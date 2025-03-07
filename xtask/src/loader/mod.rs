//! Definitions of the supported `tvm_loader` system configurations.

use x86_64_uefi::X86_64_UEFI;
use x86_uefi::X86_UEFI;

use crate::Arch;

pub mod x86_64_uefi;
pub mod x86_uefi;

/// List of all of the `tvm_loader` system crates.
pub static LOADERS: &[&Loader] = &[X86_UEFI, X86_64_UEFI];

/// Returns the [`Loader`] associated with the given [`Arch`] and [`Platform`].
pub fn lookup(arch: Arch, platform: Platform) -> Option<&'static Loader> {
    LOADERS
        .iter()
        .copied()
        .find(|loader| loader.arch == arch && loader.platform == platform)
}

/// The description of a `tvm_loader` system crate.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Loader {
    /// The name of the `tvm_loader` system.
    pub name: &'static str,

    /// The architecture this `tvm_loader` system supports.
    pub arch: Arch,
    /// The platform this `tvm_loader` system supports.
    pub platform: Platform,
    /// The target string passed to `cargo` when building.
    ///
    /// This may be a built-in target or a path to custom JSON target file.
    pub target: &'static str,
    /// The valid features for this `tvm_loader` crate.
    pub features: &'static [&'static str],

    /// Additional arguments that must be passed to `cargo` when building.
    pub additional_build_arguments: &'static [&'static str],
}

/// The architectures supported by `tvm_loader`.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Platform {
    /// The UEFI firmware environment.
    Uefi,
}

impl Platform {
    /// Returns the textual representation of the [`Platform`].
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Uefi => "uefi",
        }
    }

    /// Returns the file extension associated with the platform.
    pub fn file_extension(&self) -> Option<&'static str> {
        match self {
            Self::Uefi => Some("efi"),
        }
    }
}

impl clap::ValueEnum for Platform {
    fn value_variants<'a>() -> &'a [Self] {
        static PLATFORMS: &[Platform] = &[Platform::Uefi];

        PLATFORMS
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(clap::builder::PossibleValue::new(self.as_str()))
    }
}
