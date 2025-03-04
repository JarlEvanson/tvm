//! `x86`-specific code for the [`switch()`][switch] function.
//!
//! [switch]: crate::switch::switch

use core::{arch::global_asm, mem::offset_of};

use crate::switch::IdentityData;

unsafe extern "C" {
    /// The start of the loader address space's switch code.
    pub static SWITCH_CODE_LOADER_START: u8;

    /// The address of the identity data when viewed in the loader's address space.
    pub static SWITCH_CODE_LOADER_DATA_ADDRESS: u8;
    /// The entry point to the application switch code.
    pub static SWITCH_CODE_LOADER_ENTER_APPLICATION: u8;

    /// The end of the loader address space's switch code.
    pub static SWITCH_CODE_LOADER_END: u8;
}

global_asm!(
    // Signals the start of the loader address space's switch code.
    ".global _SWITCH_CODE_LOADER_START",
    "_SWITCH_CODE_LOADER_START:",

    ".global _SWITCH_CODE_LOADER_DATA_ADDRESS",
    "_SWITCH_CODE_LOADER_DATA_ADDRESS:",
    ".4byte 0",

    ".global _SWITCH_CODE_LOADER_ENTER_APPLICATION",
    "_SWITCH_CODE_LOADER_ENTER_APPLICATION:",

    // Acquire current address.
    "call 5f",
    "5:",
    "mov eax, [esp]",
    "ret",

    "add eax, 5b",
    "sub eax, _SWITCH_CODE_LOADER_START",

    "mov eax, {LOADER_CR3}",



    // Signals the end of the loader address space's switch code.
    ".global _SWITCH_CODE_LOADER_END",
    "_SWITCH_CODE_LOADER_END:",

    LOADER_CR3 = const offset_of!(IdentityData, loader.cr3),
);
