//! Helper function to build `tvm` given a [`BuildTvmConfiguration`].

use std::path::PathBuf;

use anyhow::Result;

use crate::{action::run_cmd, cli::build_tvm::BuildTvmConfiguration};

/// Builds `tvm` as specified by `config`, returning the path to the final binary on success.
///
/// # Errors
///
/// Returns errors when the `cargo build` command fails.
pub fn build_tvm(config: BuildTvmConfiguration) -> Result<PathBuf> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build");

    cmd.arg("--package")
        .arg(format!("tvm-{}", config.tvm_system.name));

    cmd.args(["--target", config.tvm_system.target]);
    cmd.args(["--profile", config.profile.as_str()]);

    if !config.features.is_empty() {
        let features = config
            .features
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(",");

        cmd.args(["--features", &features]);
    }

    cmd.args(config.tvm_system.additional_build_arguments);

    run_cmd(cmd)?;

    let mut target_string = PathBuf::from(config.tvm_system.target);
    target_string.set_extension("");
    let Some(target_string) = target_string.file_name() else {
        unreachable!()
    };

    let mut binary_location = PathBuf::with_capacity(50);
    binary_location.push("target");
    binary_location.push(target_string);
    binary_location.push(config.profile.target_string());

    binary_location.push(format!("tvm-{}", config.tvm_system.name));

    Ok(binary_location)
}
