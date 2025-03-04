//! `x86_64`-specific code for the [`switch()`][switch] function.
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
    ".global SWITCH_CODE_LOADER_START",
    "SWITCH_CODE_LOADER_START:",

    ".global SWITCH_CODE_LOADER_DATA_ADDRESS",
    "SWITCH_CODE_LOADER_DATA_ADDRESS:",
    ".8byte 0",

    ".global SWITCH_CODE_LOADER_ENTER_APPLICATION",
    "SWITCH_CODE_LOADER_ENTER_APPLICATION:",
    "mov r10, [rip + SWITCH_CODE_LOADER_DATA_ADDRESS]",

    "mov r11, cr3",
    "mov [r10 + {LOADER_CR3}], r11",

    "mov r11w, cs",
    "mov [r10 + {LOADER_CS}], r11w",
    "mov r11w, ds",
    "mov [r10 + {LOADER_DS}], r11w",
    "mov r11w, es",
    "mov [r10 + {LOADER_ES}], r11w",
    "sgdt [r10 + {LOADER_GDT_LENGTH}]",

    "mov r11w, fs",
    "mov [r10 + {LOADER_FS}], r11w",
    "mov r11w, gs",
    "mov [r10 + {LOADER_GS}], r11w",
    "mov r11w, ss",
    "mov [r10 + {LOADER_SS}], r11w",
    "sidt [r10 + {LOADER_IDT_LENGTH}]",

    "mov [r10 + {LOADER_RAX}], rax",
    "mov [r10 + {LOADER_RBX}], rbx",
    "mov [r10 + {LOADER_RCX}], rcx",
    "mov [r10 + {LOADER_RDX}], rdx",

    "mov [r10 + {LOADER_RSP}], rsp",
    "mov [r10 + {LOADER_RBP}], rbp",
    "mov [r10 + {LOADER_RSI}], rsi",
    "mov [r10 + {LOADER_RDI}], rdi",

    "mov [r10 + {LOADER_R8}], r8",
    "mov [r10 + {LOADER_R9}], r9",
    "mov [r10 + {LOADER_R10}], r10",
    "mov [r10 + {LOADER_R11}], r11",

    "mov [r10 + {LOADER_R12}], r12",
    "mov [r10 + {LOADER_R13}], r13",
    "mov [r10 + {LOADER_R14}], r14",
    "mov [r10 + {LOADER_R15}], r15",

    "xor eax, eax",
    "mov eax, [r10 + {BITS_64_ENTRY}]",
    "mov esi, [r10 + {IDENTITY_CODE_ADDRESS}",
    "mov edi, [r10 + {IDENTITY_DATA_ADDRESS}",
    "lea esp, [rcx + {IDENTITY_STACK_TOP}]",

    "mov r11, [r10 + {SWITCH_CR3}]",
    "mov cr3, r11",

    // Entry point in eax.
    // Identity code address in esi.
    // Identity data address in edi.
    // Tiny stack address in esp.
    "jmp rax",

    // Signals the end of the loader address space's switch code.
    ".global SWITCH_CODE_LOADER_END",
    "SWITCH_CODE_LOADER_END:",

    // Offsets of various fields.
    LOADER_CR3 = const offset_of!(IdentityData, loader.cr3),

    LOADER_CS = const offset_of!(IdentityData, loader.cs),
    LOADER_DS = const offset_of!(IdentityData, loader.ds),
    LOADER_ES = const offset_of!(IdentityData, loader.es),
    LOADER_GDT_LENGTH = const offset_of!(IdentityData, loader.gdt_length),

    LOADER_FS = const offset_of!(IdentityData, loader.fs),
    LOADER_GS = const offset_of!(IdentityData, loader.gs),
    LOADER_SS = const offset_of!(IdentityData, loader.ss),
    LOADER_IDT_LENGTH = const offset_of!(IdentityData, loader.idt_length),

    LOADER_RAX = const offset_of!(IdentityData, loader.rax),
    LOADER_RBX = const offset_of!(IdentityData, loader.rbx),
    LOADER_RCX = const offset_of!(IdentityData, loader.rcx),
    LOADER_RDX = const offset_of!(IdentityData, loader.rdx),

    LOADER_RSP = const offset_of!(IdentityData, loader.rsp),
    LOADER_RBP = const offset_of!(IdentityData, loader.rbp),
    LOADER_RSI = const offset_of!(IdentityData, loader.rsi),
    LOADER_RDI = const offset_of!(IdentityData, loader.rdi),

    LOADER_R8 = const offset_of!(IdentityData, loader.r8),
    LOADER_R9 = const offset_of!(IdentityData, loader.r9),
    LOADER_R10 = const offset_of!(IdentityData, loader.r10),
    LOADER_R11 = const offset_of!(IdentityData, loader.r11),

    LOADER_R12 = const offset_of!(IdentityData, loader.r12),
    LOADER_R13 = const offset_of!(IdentityData, loader.r13),
    LOADER_R14 = const offset_of!(IdentityData, loader.r14),
    LOADER_R15 = const offset_of!(IdentityData, loader.r15),

    BITS_64_ENTRY = const offset_of!(IdentityData, bits_64_entry),
    IDENTITY_DATA_ADDRESS = const offset_of!(IdentityData, identity_data_address),
    IDENTITY_CODE_ADDRESS = const offset_of!(IdentityData, identity_code_address),
    IDENTITY_STACK_TOP = const offset_of!(IdentityData, stack_top),
    SWITCH_CR3 = const offset_of!(IdentityData, switch_cr3),
);
