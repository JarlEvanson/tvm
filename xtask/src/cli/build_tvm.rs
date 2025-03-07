//! Command line parsing and [`Action::BuildTvm`][abt] construction.
//!
//! [abt]: crate::cli::Action::BuildTvm

use clap::{Arg, ArgAction, ArgMatches, Command, builder::EnumValueParser};

use crate::{
    Arch,
    action::Profile,
    cli::parse_features,
    tvm::{self, TvmSystem},
};

/// Description of what `tvm` system crate to build and the configuration with which to build it.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BuildTvmConfiguration {
    /// The [`TvmSystem`] that should be built.
    pub tvm_system: &'static TvmSystem,

    /// The [`Profile`] with which to build the [`TvmSystem`].
    pub profile: Profile,
    /// The feature flags that should be enabled when building the [`TvmSystem`].
    pub features: Vec<String>,
}

/// Parses the arguments required to produce a valid [`BuildTvmConfiguration`].
#[expect(
    clippy::missing_panics_doc,
    reason = "xtask subcommand guarantees that these are present"
)]
pub fn parse_arguments(prefix: Option<&str>, matches: &ArgMatches) -> BuildTvmConfiguration {
    let arch_arg_name = if let Some(prefix) = prefix {
        &format!("{prefix}-arch")
    } else {
        "arch"
    };

    let arch = matches
        .get_one::<Arch>(arch_arg_name)
        .copied()
        .unwrap_or_else(|| panic!("{arch_arg_name} is a required argument"));

    let Some(tvm_system) = tvm::lookup(arch) else {
        crate::cli::command_parser()
            .error(
                clap::error::ErrorKind::InvalidValue,
                format!("no tvm system supports {}", arch.as_str()),
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

    let features = parse_features(prefix, matches, tvm_system.features);

    BuildTvmConfiguration {
        tvm_system,
        profile,
        features,
    }
}

/// Returns the command parser for an [`Action::BuildLoader`][abt].
///
/// [abt]: crate::cli::Action::BuildTvm
pub fn subcommand_parser() -> Command {
    let arch = Arg::new("arch")
        .long("arch")
        .value_parser(EnumValueParser::<Arch>::new())
        .required(true);

    let profile = Arg::new("profile")
        .long("profile")
        .value_parser(EnumValueParser::<Profile>::new())
        .default_value("dev");

    let features = Arg::new("features")
        .long("features")
        .action(ArgAction::Append);

    Command::new("build-tvm")
        .about("Builds tvm")
        .arg(arch)
        .arg(profile)
        .arg(features)
}
