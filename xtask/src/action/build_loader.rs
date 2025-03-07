//! Helper function to build `tvm_loader` given a [`BuildLoaderConfiguration`].

use std::path::PathBuf;

use anyhow::Result;

use crate::{action::run_cmd, cli::build_loader::BuildLoaderConfiguration};

/// Builds `tvm_loader` as specified by `config`, returning the path to the final binary on
/// success.
///
/// # Errors
///
/// Returns errors when the `cargo build` command fails.
pub fn build_tvm_loader(config: BuildLoaderConfiguration) -> Result<PathBuf> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build");

    cmd.arg("--package")
        .arg(format!("tvm_loader-{}", config.loader.name));

    cmd.args(["--target", config.loader.target]);
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

    cmd.args(config.loader.additional_build_arguments);

    run_cmd(cmd)?;

    let mut target_string = PathBuf::from(config.loader.target);
    target_string.set_extension("");
    let Some(target_string) = target_string.file_name() else {
        unreachable!()
    };

    let mut binary_location = PathBuf::with_capacity(50);
    binary_location.push("target");
    binary_location.push(target_string);
    binary_location.push(config.profile.target_string());

    binary_location.push(format!("tvm_loader-{}", config.loader.name));
    if let Some(extension) = config.loader.platform.file_extension() {
        binary_location.set_extension(extension);
    }

    Ok(binary_location)
}
