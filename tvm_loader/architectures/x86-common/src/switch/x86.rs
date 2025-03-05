//! `x86`-specific code for the [`switch()`][switch] function.
//!
//! [switch]: crate::switch::switch

use core::arch::global_asm;

unsafe extern "C" {
    /// The start of the loader code.
    pub static LOADER_START: u8;

    /// The pointer to the data page in the loader's address space.
    pub static LOADER_DATA_PTR: u8;

    /// The address that starts the application.
    pub static LOADER_START_APPLICATION: u8;

    /// The end of the loader code.
    pub static LOADER_END: u8;

    /// The start of the code for 32-bit applications.
    pub static APPLICATION_32_START: u8;

    /// The pointer to the data page in the application's address space.
    pub static APPLICATION_32_DATA_PTR: u8;

    /// The end of the code for 32-bit applications.
    pub static APPLICATION_32_END: u8;
}

// 32-bit loader code.
global_asm!(
    // Signals the start of the loader code.
    ".global _LOADER_START",
    "_LOADER_START:",

    ".global _LOADER_DATA_PTR",
    "_LOADER_DATA_PTR:",
    ".4byte 0",

    ".global _LOADER_START_APPLICATION",
    "_LOADER_START_APPLICATION:",

    "5:",
    "jmp 5b",

    // Signals the end of the loader code.
    ".global _LOADER_END",
    "_LOADER_END:",
);

// 32-bit application code.
global_asm!(
    // Signals the start of the loader code.
    ".global _APPLICATION_32_START",
    "_APPLICATION_32_START:",
    ".global _APPLICATION_32_DATA_PTR",
    "_APPLICATION_32_DATA_PTR:",
    ".4byte 0",
    // Signals the end of the loader code.
    ".global _APPLICATION_32_END",
    "_APPLICATION_32_END:",
);
