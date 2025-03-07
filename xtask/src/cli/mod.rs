//! Command line parsing and [`Action`] construction.

use std::collections::HashSet;

use build_loader::BuildLoaderConfiguration;
use clap::{ArgMatches, Command, error::ErrorKind};

pub mod build_loader;

/// Parses `xtask`'s arguments to construct an [`Action`].
#[expect(
    clippy::missing_panics_doc,
    reason = "xtask guarantees that a subcommand is present"
)]
pub fn get_action() -> Action {
    let matches = command_parser().get_matches();

    let (subcommand_name, subcommand_matches) =
        matches.subcommand().expect("subcommand is required");
    match subcommand_name {
        "build-loader" => {
            Action::BuildLoader(build_loader::parse_arguments(None, subcommand_matches))
        }
        _ => unreachable!("unexpected subcommand: {subcommand_name:?}"),
    }
}

/// Returns the command parser for all [`Action`]s.
fn command_parser() -> Command {
    Command::new("xtask")
        .about("Developer utility for running various tasks on tvm_loader and tvm")
        .subcommand(build_loader::subcommand_parser())
        .subcommand_required(true)
        .arg_required_else_help(true)
}

/// The action to carry out.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Action {
    /// Build `tvm_loader` with a specific configuration.
    BuildLoader(BuildLoaderConfiguration),
}

/// Parses arguments that specify the features of a build.
pub fn parse_features(
    prefix: Option<&str>,
    matches: &ArgMatches,
    valid_features: &[&str],
) -> Vec<String> {
    let feature_arg_name = if let Some(prefix) = prefix {
        &format!("{prefix}-features")
    } else {
        "features"
    };

    let mut features = HashSet::new();
    for feature in matches
        .get_many(feature_arg_name)
        .into_iter()
        .flatten()
        .map(String::as_str)
        .flat_map(parse_feature)
    {
        let Some(feature) = valid_features
            .iter()
            .find(|&&valid_feature| feature == valid_feature)
        else {
            crate::cli::command_parser()
                .error(
                    ErrorKind::InvalidValue,
                    format!("feature {feature} is not supported",),
                )
                .exit()
        };

        features.insert(feature.to_string());
    }

    features.into_iter().collect::<Vec<String>>()
}

/// Parses feature names from a comma seperated list.
fn parse_feature(feature: &str) -> impl Iterator<Item = &str> + '_ {
    feature
        .split_whitespace()
        .flat_map(|s| s.split(','))
        .filter(|s| !s.is_empty())
}
