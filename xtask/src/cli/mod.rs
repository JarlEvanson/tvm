//! Command line parsing and [`Action`] construction.

use clap::Command;

/// Parses `xtask`'s arguments to construct an [`Action`].
pub fn get_action() -> Action {
    let matches = command_parser().get_matches();

    let (subcommand_name, subcommand_matches) =
        matches.subcommand().expect("subcommand is required");
    match subcommand_name {
        _ => unreachable!("unexpected subcommand: {subcommand_name:?}"),
    }
}

/// Returns the command parser for all [`Action`]s.
fn command_parser() -> Command {
    Command::new("xtask")
        .about("Developer utility for running various tasks on tvm_loader and tvm")
        .subcommand_required(true)
        .arg_required_else_help(true)
}

/// The action to carry out.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Action {}
