//! Support code for `tvm_loader` crates for the `x86` architecture.

#![no_std]

pub mod paging;
mod relocation;

pub use relocation::handle_relocation;
