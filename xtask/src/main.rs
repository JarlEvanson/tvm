//! Helper crate for building, packaging, and testing `tvm_loader` and `tvm`.

use action::{build_loader::build_tvm_loader, build_tvm::build_tvm, embed::embed};
use anyhow::Result;
use cli::Action;

pub mod action;
pub mod cli;
pub mod loader;
pub mod tvm;

fn main() -> Result<()> {
    match cli::get_action() {
        Action::BuildLoader(config) => {
            let path = build_tvm_loader(config)?;
            println!("tvm_loader located at \"{}\"", path.display());
        }
        Action::BuildTvm(config) => {
            let path = build_tvm(config)?;
            println!("tvm located at \"{}\"", path.display());
        }
        Action::Embed(config) => {
            let path = embed(config)?;
            println!("tvm_loader with embedded tvm at \"{}\"", path.display());
        }
    }

    Ok(())
}

/// The architectures supported by `tvm`.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Arch {
    /// The `x86` architecture.
    X86,
    /// The `x86_64` architecture.
    X86_64,
}

impl Arch {
    /// Returns the textual representation of the [`Arch`].
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::X86 => "x86",
            Self::X86_64 => "x86_64",
        }
    }
}

impl clap::ValueEnum for Arch {
    fn value_variants<'a>() -> &'a [Self] {
        static ARCHITECTURES: &[Arch] = &[Arch::X86, Arch::X86_64];

        ARCHITECTURES
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(clap::builder::PossibleValue::new(self.as_str()))
    }
}
