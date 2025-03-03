//! Command line parsing and [`Action`] construction.

use build_loader::BuildLoaderConfiguration;
use build_tvm::BuildTvmConfiguration;

pub mod build_loader;
pub mod build_tvm;

/// Parses the executable's arguments to construct an [`Action`].
pub fn get_action() -> Action {
    let matches = command_parser().get_matches();
    let (subcommand_name, subcommand_matches) = matches.subcommand().expect("subcommand required");
    match subcommand_name {
        "build-loader" => build_loader::parse_arguments(subcommand_matches),
        "build-tvm" => build_tvm::parse_arguments(subcommand_matches),
        "run-qemu" => todo!(),
        _ => unreachable!("unexpected subcommand: {subcommand_name:?}"),
    }
}

/// Returns the command parser for all [`Action`]s.
fn command_parser() -> clap::Command {
    clap::Command::new("xtask")
        .about("Developer utility for running various tasks on tvm and tvm loader")
        .subcommand(build_loader::subcommand_parser())
        .subcommand(build_tvm::subcommand_parser())
        .subcommand_required(true)
        .arg_required_else_help(true)
}

/// The action to carry out.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Action {
    /// Build `tvm_loader` with a specific configuration.
    BuildLoader(BuildLoaderConfiguration),
    /// Builds `tvm` with a specific configuration.
    BuildTvm(BuildTvmConfiguration),
}

/// The architectures supported by `tvm`.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Arch {
    /// The `x86` architecture.
    X86,
    /// The `x86_64` architecture.
    X86_64,
}

impl Arch {
    /// Returns the textual representation of the [`Arch`].
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::X86 => "x86",
            Self::X86_64 => "x86_64",
        }
    }
}

impl clap::ValueEnum for Arch {
    fn value_variants<'a>() -> &'a [Self] {
        static ARCHITECTURES: &[Arch] = &[Arch::X86, Arch::X86_64];

        ARCHITECTURES
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(clap::builder::PossibleValue::new(self.as_str()))
    }
}

/// The architectures supported by `tvm_loader`.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum BootPlatform {
    /// The UEFI firmware environment.
    Uefi,
}

impl BootPlatform {
    /// Returns the textual representation of the [`BootPlatform`].
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

impl clap::ValueEnum for BootPlatform {
    fn value_variants<'a>() -> &'a [Self] {
        static PLATFORMS: &[BootPlatform] = &[BootPlatform::Uefi];

        PLATFORMS
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(clap::builder::PossibleValue::new(self.as_str()))
    }
}

/// A `cargo` profile.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub enum Profile {
    /// The `dev` cargo profile.
    #[default]
    Dev,
    /// The `release` cargo profile.
    Release,
}

impl Profile {
    /// Returns the textual representation of the [`Profile`].
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Release => "release",
        }
    }

    /// Returns the textual representation of the [`Profile`]'s target path component.
    pub fn target_string(&self) -> &'static str {
        match self {
            Self::Dev => "debug",
            Self::Release => "release",
        }
    }
}

impl clap::ValueEnum for Profile {
    fn value_variants<'a>() -> &'a [Self] {
        static PROFILES: &[Profile] = &[Profile::Dev, Profile::Release];

        PROFILES
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(clap::builder::PossibleValue::new(self.as_str()))
    }
}
