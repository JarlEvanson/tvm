//! Helpers for packaging `tvm_loader` and `tvm`.

use std::{fs, path::PathBuf};

use anyhow::Result;

use crate::{
    actions::{build_loader, build_tvm, common::run_cmd},
    cli::{Either, package::PackageConfiguration},
};

/// Packages `tvm_loader` and `tvm` into a single binary, building `tvm_loader` and `tvm` if
/// necessary.
pub fn handle(config: PackageConfiguration) -> Result<PathBuf> {
    let loader_path = match config.loader {
        Either::A(path) => path,
        Either::B(loader) => build_loader::handle(loader)?,
    };

    let tvm_path = match config.tvm {
        Either::A(path) => path,
        Either::B(tvm) => build_tvm::handle(tvm)?,
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
