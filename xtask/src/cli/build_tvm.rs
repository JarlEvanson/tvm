//! Command line parsing and [`Action::BuildTvm`] construction.

use crate::{
    cli::{Action, Arch, Profile},
    tvm::{TVMS, Tvm},
};

/// Description of what `tvm` system crate to build and the configuration with which to build it.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct BuildTvmConfiguration {
    /// The [`Tvm`] that should be built.
    pub tvm: &'static Tvm,
    /// The [`Profile`] with which to build the [`Tvm`].
    pub profile: Profile,
}

impl BuildTvmConfiguration {
    /// Returns the [`Tvm`] associated with the given [`Arch`].
    pub fn lookup(arch: Arch) -> Option<&'static Tvm> {
        TVMS.iter().copied().find(|tvm| tvm.arch == arch)
    }
}

/// Parses the arguments of the `build-tvm` subcommand.
pub fn parse_arguments(matches: &clap::ArgMatches) -> Action {
    let arch = matches
        .get_one::<Arch>("arch")
        .copied()
        .expect("arch is a required argument");

    let Some(tvm) = BuildTvmConfiguration::lookup(arch) else {
        crate::cli::command_parser()
            .error(
                clap::error::ErrorKind::InvalidValue,
                format!("no tvm system supports {}", arch.as_str()),
            )
            .exit()
    };

    let profile = matches
        .get_one::<Profile>("profile")
        .copied()
        .expect("profile is a required argument");

    Action::BuildTvm(BuildTvmConfiguration { tvm, profile })
}

/// Returns the command parser for a [`Action::BuildTvm`].
pub fn subcommand_parser() -> clap::Command {
    let arch = clap::Arg::new("arch")
        .long("arch")
        .value_parser(clap::builder::EnumValueParser::<Arch>::new())
        .required(true);

    let profile = clap::Arg::new("profile")
        .long("profile")
        .value_parser(clap::builder::EnumValueParser::<Profile>::new())
        .default_value("dev");

    clap::Command::new("build-tvm")
        .about("Builds tvm")
        .arg(arch)
        .arg(profile)
}
