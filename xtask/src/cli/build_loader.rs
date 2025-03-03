//! Command line parsing and [`Action::BuildLoader`] construction.

use crate::{
    cli::{Action, Arch, BootPlatform, Profile},
    loader::{LOADERS, Loader},
};

/// Description of what `tvm_loader` system crate to build and the configuration with which to
/// build it.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct BuildLoaderConfiguration {
    /// The [`Loader`] that should be built.
    pub loader: &'static Loader,
    /// The [`Profile`] with which to build the [`Loader`].
    pub profile: Profile,
}

impl BuildLoaderConfiguration {
    /// Returns the [`Loader`] associated with the given [`Arch`] and [`BootPlatform`].
    pub fn lookup(arch: Arch, platform: BootPlatform) -> Option<&'static Loader> {
        LOADERS
            .iter()
            .copied()
            .find(|loader| loader.arch == arch && loader.platform == platform)
    }
}

/// Parses the arguments of the `build-loader` subcommand.
pub fn parse_arguments(matches: &clap::ArgMatches) -> Action {
    let arch = matches
        .get_one::<Arch>("arch")
        .copied()
        .expect("arch is a required argument");

    let platform = matches
        .get_one::<BootPlatform>("platform")
        .copied()
        .expect("platform is a required argument");

    let Some(loader) = BuildLoaderConfiguration::lookup(arch, platform) else {
        crate::cli::command_parser()
            .error(
                clap::error::ErrorKind::InvalidValue,
                format!(
                    "no tvm_loader system supports {}-{}",
                    arch.as_str(),
                    platform.as_str()
                ),
            )
            .exit();
    };

    let profile = matches
        .get_one::<Profile>("profile")
        .copied()
        .expect("profile is a required argument");

    Action::BuildLoader(BuildLoaderConfiguration { loader, profile })
}

/// Returns the command parser for a [`Action::BuildLoader`].
pub fn subcommand_parser() -> clap::Command {
    let arch = clap::Arg::new("arch")
        .long("arch")
        .value_parser(clap::builder::EnumValueParser::<Arch>::new())
        .required(true);

    let platform = clap::Arg::new("platform")
        .long("platform")
        .value_parser(clap::builder::EnumValueParser::<BootPlatform>::new())
        .required(true);

    let profile = clap::Arg::new("profile")
        .long("profile")
        .value_parser(clap::builder::EnumValueParser::<Profile>::new())
        .default_value("dev");

    clap::Command::new("build-loader")
        .about("Builds tvm_loader")
        .arg(arch)
        .arg(platform)
        .arg(profile)
}
