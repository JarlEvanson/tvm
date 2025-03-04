//! Helper crate for building and testing `tvm`.

use actions::{build_loader, build_tvm, package};
use anyhow::Result;
use cli::Action;

mod actions;
mod cli;
pub(crate) mod loader;
pub(crate) mod tvm;

fn main() -> Result<()> {
    match cli::get_action() {
        Action::BuildLoader(build_config) => {
            let path = build_loader::handle(build_config)?;
            println!("tvm_loader located at \"{}\"", path.display());
        }
        Action::BuildTvm(build_config) => {
            let path = build_tvm::handle(build_config)?;
            println!("tvm located at \"{}\"", path.display());
        }
        Action::Package(package_config) => {
            let path = package::handle(package_config)?;
            println!("tvm packaged at \"{}\"", path.display());
        }
    }

    Ok(())
}
