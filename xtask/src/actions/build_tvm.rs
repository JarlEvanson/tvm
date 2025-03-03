//! `tvm` build helpers.

use std::path::PathBuf;

use anyhow::Result;

use crate::{actions::common::run_cmd, cli::build_tvm::BuildTvmConfiguration};

/// Builds `tvm` as specified by `config`, returning the path to the final binary on success.
pub fn handle(config: BuildTvmConfiguration) -> Result<PathBuf> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build");

    cmd.arg("--package").arg(format!("tvm-{}", config.tvm.name));

    cmd.args(["--target", config.tvm.target]);
    cmd.args(["--profile", config.profile.as_str()]);

    /*
    // TODO: Support features
    if !arguments.features.is_empty() {
        let features = arguments
            .features
            .iter()
            .map(Feature::as_str)
            .collect::<Vec<_>>()
            .join(",");

        cmd.args(["--features", &features]);
    }
    */

    cmd.args(config.tvm.additional_build_arguments);

    run_cmd(cmd)?;

    let mut target_string = PathBuf::from(config.tvm.target);
    target_string.set_extension("");
    let target_string = target_string.file_name().unwrap();

    let mut binary_location = PathBuf::with_capacity(50);
    binary_location.push("target");
    binary_location.push(target_string);
    binary_location.push(config.profile.target_string());

    binary_location.push(format!("tvm-{}", config.tvm.name));

    Ok(binary_location)
}
