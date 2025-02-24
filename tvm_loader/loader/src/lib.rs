//! Core implementation of `tvm_loader`.
//!
//! This crate implements the core features and interfaces of `tvm_loader` and system-independent
//! code that many other architecture, platform, and system crates utilize.

#![no_std]

extern crate alloc;

pub mod elf_loader;
#[cfg(feature = "graphics")]
pub mod graphics;
pub mod logging;
pub mod memory;
