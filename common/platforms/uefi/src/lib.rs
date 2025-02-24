//! Raw interface for working with UEFI.
//!
//! This crate is intended to provide the definitions required to build more sophisticated crates
//! with which end-users should work.

#![no_std]

use core::fmt;

pub mod data_types;
pub mod memory;
pub mod protocol;

/// Forces formatting to be carried out as [`LowerHex`][flh].
///
/// [flh]: fmt::LowerHex
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FmtLowerHex<T: fmt::LowerHex>(T);

impl<T: fmt::LowerHex> fmt::Debug for FmtLowerHex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <T as fmt::LowerHex>::fmt(&self.0, f)
    }
}

impl<T: fmt::LowerHex> fmt::Display for FmtLowerHex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <T as fmt::LowerHex>::fmt(&self.0, f)
    }
}
