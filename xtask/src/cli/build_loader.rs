//! Command line parsing and [`Action::BuildLoader`][abl] construction.
//!
//! [abl]: crate::cli::Action::BuildLoader

use clap::{Arg, ArgAction, ArgMatches, Command, builder::EnumValueParser};

use crate::{
    Arch,
    action::Profile,
    cli::parse_features,
    loader::{self, Loader, Platform},
};

/// Description of what `tvm_loader` system crate to build and the configuration with which to
/// build it.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BuildLoaderConfiguration {
    /// The [`Loader`] that should be built.
    pub loader: &'static Loader,

    /// The [`Profile`] with which to build the [`Loader`].
    pub profile: Profile,
    /// The feature flags that should be enabled when building the [`Loader`].
    pub features: Vec<String>,
}

/// Parses the arguments required to produce a valid [`BuildLoaderConfiguration`].
#[expect(
    clippy::missing_panics_doc,
    reason = "xtask subcommand guarantees that these are present"
)]
pub fn parse_arguments(prefix: Option<&str>, matches: &ArgMatches) -> BuildLoaderConfiguration {
    let arch_arg_name = if let Some(prefix) = prefix {
        &format!("{prefix}-arch")
    } else {
        "arch"
    };

    let arch = matches
        .get_one::<Arch>(arch_arg_name)
        .copied()
        .unwrap_or_else(|| panic!("{arch_arg_name} is a required argument"));

    let platform_arg_name = if let Some(prefix) = prefix {
        &format!("{prefix}-platform")
    } else {
        "platform"
    };

    let platform = matches
        .get_one::<Platform>(platform_arg_name)
        .copied()
        .unwrap_or_else(|| panic!("{platform_arg_name} is a required argument"));

    let Some(loader) = loader::lookup(arch, platform) else {
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

    let profile_arg_name = if let Some(prefix) = prefix {
        &format!("{prefix}-profile")
    } else {
        "profile"
    };

    let profile = matches
        .get_one::<Profile>(profile_arg_name)
        .copied()
        .unwrap_or_else(|| panic!("{profile_arg_name} is a required argument"));

    let features = parse_features(prefix, matches, loader.features);

    BuildLoaderConfiguration {
        loader,
        profile,
        features,
    }
}

/// Returns the command parser for an [`Action::BuildLoader`][abl].
///
/// [abl]: crate::cli::Action::BuildLoader
pub fn subcommand_parser() -> Command {
    let arch = Arg::new("arch")
        .long("arch")
        .value_parser(EnumValueParser::<Arch>::new())
        .required(true);

    let platform = Arg::new("platform")
        .long("platform")
        .value_parser(EnumValueParser::<Platform>::new())
        .required(true);

    let profile = Arg::new("profile")
        .long("profile")
        .value_parser(EnumValueParser::<Profile>::new())
        .default_value("dev");

    let features = Arg::new("features")
        .long("features")
        .action(ArgAction::Append);

    Command::new("build-loader")
        .about("Builds tvm_loader")
        .arg(arch)
        .arg(platform)
        .arg(profile)
        .arg(features)
}
