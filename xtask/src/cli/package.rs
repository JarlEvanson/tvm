//! Command line parsing and [`Action::Package`] construction.

use std::path::PathBuf;

use crate::cli::{
    Action, Arch, BootPlatform, BuildLoaderConfiguration, BuildTvmConfiguration, Either, Profile,
};

/// Description of how to package `tvm_loader` and `tvm`.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PackageConfiguration {
    /// Either the path to a pre-built `tvm_loader` or a build configuration for `tvm_loader`.
    pub loader: Either<PathBuf, BuildLoaderConfiguration>,
    /// Either the path to a pre-built `tvm` or a build configuration for `tvm`.
    pub tvm: Either<PathBuf, BuildTvmConfiguration>,
    /// The path at which to place the packaged binary.
    pub target_path: PathBuf,
}

/// Parses the arguments of the `package` subcommand.
pub fn parse_arguments(matches: &clap::ArgMatches) -> Action {
    let loader = match matches.get_one::<PathBuf>("loader-path").cloned() {
        Some(path) => Either::A(path),
        None => {
            let arch = matches
                .get_one::<Arch>("loader-arch")
                .copied()
                .expect("loader-arch is a required argument");

            let platform = matches
                .get_one::<BootPlatform>("loader-platform")
                .copied()
                .expect("loader-platform is a required argument");

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
                .get_one::<Profile>("loader-profile")
                .copied()
                .expect("loader-profile is a required argument");

            Either::B(BuildLoaderConfiguration { loader, profile })
        }
    };

    let tvm = match matches.get_one("tvm-path").cloned() {
        Some(path) => Either::A(path),
        None => {
            let arch = matches
                .get_one::<Arch>("tvm-arch")
                .copied()
                .expect("tvm-arch is a required argument");

            let Some(tvm) = BuildTvmConfiguration::lookup(arch) else {
                crate::cli::command_parser()
                    .error(
                        clap::error::ErrorKind::InvalidValue,
                        format!("no tvm system supports {}", arch.as_str()),
                    )
                    .exit()
            };

            let profile = matches
                .get_one::<Profile>("tvm-profile")
                .copied()
                .expect("tvm-profile is a required argument");

            Either::B(BuildTvmConfiguration { tvm, profile })
        }
    };

    let target_path = matches
        .get_one::<PathBuf>("target-path")
        .cloned()
        .expect("target-path is required");

    Action::Package(PackageConfiguration {
        loader,
        tvm,
        target_path,
    })
}

/// Returns the command parser for a [`Action::Package`].
pub fn subcommand_parser() -> clap::Command {
    let loader_arch = clap::Arg::new("loader-arch")
        .long("loader-arch")
        .value_parser(clap::builder::EnumValueParser::<Arch>::new())
        .required_unless_present("loader-path");

    let loader_platform = clap::Arg::new("loader-platform")
        .long("loader-platform")
        .value_parser(clap::builder::EnumValueParser::<BootPlatform>::new())
        .required_unless_present("loader-path");

    let loader_profile = clap::Arg::new("loader-profile")
        .long("loader-profile")
        .value_parser(clap::builder::EnumValueParser::<Profile>::new())
        .default_value("dev");

    let loader_path = clap::Arg::new("loader-path")
        .long("loader-path")
        .value_parser(clap::value_parser!(PathBuf))
        .conflicts_with_all(["loader-arch", "loader-platform", "loader-profile"]);

    let tvm_arch = clap::Arg::new("tvm-arch")
        .long("tvm-arch")
        .value_parser(clap::builder::EnumValueParser::<Arch>::new())
        .required(true);

    let tvm_profile = clap::Arg::new("tvm-profile")
        .long("tvm-profile")
        .value_parser(clap::builder::EnumValueParser::<Profile>::new())
        .default_value("dev");

    let tvm_path = clap::Arg::new("tvm-path")
        .long("tvm-path")
        .value_parser(clap::value_parser!(PathBuf))
        .conflicts_with_all(["tvm-arch", "tvm-profile"]);

    let target_path = clap::Arg::new("target-path")
        .long("target-path")
        .value_parser(clap::value_parser!(PathBuf))
        .required(true);

    clap::Command::new("package")
        .about("Packages tvm_loader and tvm into a stand-alone binary")
        .arg(loader_arch)
        .arg(loader_platform)
        .arg(loader_profile)
        .arg(loader_path)
        .arg(tvm_arch)
        .arg(tvm_profile)
        .arg(tvm_path)
        .arg(target_path)
}
