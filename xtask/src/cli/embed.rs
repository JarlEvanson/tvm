//! Command line parsing and [`Action::Embed`][ae] construction.
//!
//! [ae]: crate::cli::Action::Embed

use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, builder::EnumValueParser};

use crate::{
    Arch,
    action::Profile,
    cli::{BuildLoaderConfiguration, BuildTvmConfiguration, Either, build_loader, build_tvm},
    loader::Platform,
};

/// Description of how to embed `tvm` into `tvm_loader`, building if necessary.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EmbedConfiguration {
    /// Either the path to a pre-built `tvm_loader` or a build configuration for `tvm_loader`.
    pub loader: Either<PathBuf, BuildLoaderConfiguration>,
    /// Either the path to a pre-built `tvm` or a build configuration for `tvm`.
    pub tvm: Either<PathBuf, BuildTvmConfiguration>,
    /// The path at which to place the packaged binary.
    pub target_path: PathBuf,
}

/// Parses the arguments of the `embed` subcommand.
#[expect(
    clippy::missing_panics_doc,
    reason = "xtask subcommand guarantees that these are present"
)]
pub fn parse_arguments(matches: &ArgMatches) -> EmbedConfiguration {
    let loader = match matches.get_one::<PathBuf>("loader-path").cloned() {
        Some(path) => Either::A(path),
        None => Either::B(build_loader::parse_arguments(Some("loader"), matches)),
    };

    let tvm = match matches.get_one("tvm-path").cloned() {
        Some(path) => Either::A(path),
        None => Either::B(build_tvm::parse_arguments(Some("tvm"), matches)),
    };

    let target_path = matches
        .get_one::<PathBuf>("target-path")
        .cloned()
        .expect("target-path is required");

    EmbedConfiguration {
        loader,
        tvm,
        target_path,
    }
}

/// Returns the command parser for a [`Action::Embed`][ae].
///
/// [ae]: crate::cli::Action::Embed
pub fn subcommand_parser() -> Command {
    let loader_arch = Arg::new("loader-arch")
        .long("loader-arch")
        .value_parser(EnumValueParser::<Arch>::new())
        .required_unless_present("loader-path");

    let loader_platform = Arg::new("loader-platform")
        .long("loader-platform")
        .value_parser(EnumValueParser::<Platform>::new())
        .required_unless_present("loader-path");

    let loader_profile = Arg::new("loader-profile")
        .long("loader-profile")
        .value_parser(EnumValueParser::<Profile>::new())
        .default_value("dev");

    let loader_features = Arg::new("loader-features")
        .long("loader-features")
        .action(ArgAction::Append);

    let loader_path = Arg::new("loader-path")
        .long("loader-path")
        .value_parser(clap::value_parser!(PathBuf))
        .conflicts_with_all([
            "loader-arch",
            "loader-platform",
            "loader-profile",
            "loader-features",
        ]);

    let tvm_arch = Arg::new("tvm-arch")
        .long("tvm-arch")
        .value_parser(EnumValueParser::<Arch>::new())
        .required(true);

    let tvm_profile = Arg::new("tvm-profile")
        .long("tvm-profile")
        .value_parser(EnumValueParser::<Profile>::new())
        .default_value("dev");

    let tvm_features = Arg::new("tvm-features")
        .long("tvm-features")
        .action(ArgAction::Append);

    let tvm_path = Arg::new("tvm-path")
        .long("tvm-path")
        .value_parser(clap::value_parser!(PathBuf))
        .conflicts_with_all(["tvm-arch", "tvm-profile", "tvm-features"]);

    let target_path = Arg::new("target-path")
        .long("target-path")
        .value_parser(clap::value_parser!(PathBuf))
        .required(true);

    clap::Command::new("embed")
        .about("Embed tvm into tvm_loader to form a stand-alone binary")
        .arg(loader_arch)
        .arg(loader_platform)
        .arg(loader_profile)
        .arg(loader_features)
        .arg(loader_path)
        .arg(tvm_arch)
        .arg(tvm_profile)
        .arg(tvm_path)
        .arg(tvm_features)
        .arg(target_path)
}
