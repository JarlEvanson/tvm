//! Helpers for embedding `tvm` into `tvm_loader`.

use std::{fs, path::PathBuf};

use anyhow::Result;

use crate::{
    action::{build_loader::build_tvm_loader, build_tvm::build_tvm, run_cmd},
    cli::{Either, embed::EmbedConfiguration},
};

/// Embeds `tvm` into `tvm_loader`, forming a single stand-alone binary. This action will build
/// `tvm_loader` and `tvm` if necessary.
///
/// # Errors
///
/// Returns errors when [`build_tvm_loader()`], [`build_tvm()`], or the `llvm_objcopy` command fails.
pub fn embed(config: EmbedConfiguration) -> Result<PathBuf> {
    let loader_path = match config.loader {
        Either::A(path) => path,
        Either::B(loader) => build_tvm_loader(loader)?,
    };

    let tvm_path = match config.tvm {
        Either::A(path) => path,
        Either::B(tvm) => build_tvm(tvm)?,
    };

    if let Some(folder) = config.target_path.parent() {
        fs::create_dir_all(folder)?;
    }

    let mut cmd = std::process::Command::new("llvm-objcopy");

    cmd.arg("--add-section")
        .arg(format!("TVM_BIN={}", tvm_path.display()));
    cmd.arg("--set-section-flags")
        .arg("TVM_BIN=alloc,readonly,data");
    cmd.arg(&loader_path).arg(&config.target_path);

    run_cmd(cmd)?;

    Ok(config.target_path)
}
