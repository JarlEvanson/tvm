//! Implementation of the cross address space switch.

use core::{
    arch::{asm, global_asm},
    mem::{self, offset_of},
    ptr,
};

use tvm_loader::{
    log_debug,
    memory::{
        phys::{AllocationType, allocate_frame, frame_size},
        virt::{
            ProtectionFlags,
            loader::{map, page_size},
        },
    },
};

#[cfg(target_arch = "x86")]
use x86::*;

#[cfg(target_arch = "x86_64")]
use x86_64::*;

use crate::paging::X86CommonAddressSpace;

#[cfg(target_arch = "x86")]
mod x86;
#[cfg(target_arch = "x86_64")]
mod x86_64;

/// The GDT layout for the switching context.
pub const SWITCH_GDT: [u64; 5] = [
    0x0000_0000_0000_0000, // Null Segment
    0x00AF_9B00_0000_FFFF, // Kernel 64-bit code segment
    0x00CF_9300_0000_FFFF, // Kernel 64-bit data segment
    0x00CF_9B00_0000_FFFF, // Kernel 32-bit code segment
    0x00CF_9300_0000_FFFF, // Kernel 32-bit data segment
];

/// Sets up and executes the switching code, thereby executing the appplication starting at
/// `entry_point`.
pub fn switch(
    switch_space: &mut dyn X86CommonAddressSpace,
    application_space: &mut dyn X86CommonAddressSpace,
    entry_point: u64,
) -> Result<u32, SwitchError> {
    let switch_code_loader_size = ptr::addr_of!(SWITCH_CODE_LOADER_END) as usize
        - ptr::addr_of!(SWITCH_CODE_LOADER_START) as usize;

    let switch_code_loader_physical_address = allocate_frame(
        AllocationType::Any,
        (switch_code_loader_size as u64).div_ceil(frame_size()),
        4096u64,
    )
    .unwrap();

    let loader_switch_virtual_address = map(
        switch_code_loader_physical_address,
        switch_code_loader_size.div_ceil(page_size()),
        ProtectionFlags::READ | ProtectionFlags::EXECUTE,
    )
    .unwrap();

    switch_space
        .map(
            loader_switch_virtual_address as u64,
            switch_code_loader_physical_address,
            (switch_code_loader_size as u64).div_ceil(switch_space.page_size()),
            ProtectionFlags::READ | ProtectionFlags::EXECUTE,
        )
        .unwrap();

    log_debug!(
        "loader switch code located at {switch_code_loader_physical_address:#x} \
        ({loader_switch_virtual_address:#x} in loader space)"
    );

    let identity_data_size = mem::size_of::<IdentityData>();

    let identity_data_address = allocate_frame(
        AllocationType::Any,
        (identity_data_size as u64).div_ceil(frame_size()),
        4096u64,
    )
    .unwrap();

    let loader_identity_data_virtual_address = map(
        identity_data_address,
        identity_data_size.div_ceil(page_size()),
        ProtectionFlags::READ | ProtectionFlags::WRITE,
    )
    .unwrap();

    application_space
        .map(
            identity_data_address,
            identity_data_address,
            (identity_data_size as u64).div_ceil(application_space.page_size()),
            ProtectionFlags::READ | ProtectionFlags::WRITE,
        )
        .unwrap();

    switch_space
        .map(
            identity_data_address,
            identity_data_address,
            (identity_data_size as u64).div_ceil(switch_space.page_size()),
            ProtectionFlags::READ | ProtectionFlags::WRITE,
        )
        .unwrap();

    log_debug!(
        "identity data located at {identity_data_address:#x} \
        ({loader_identity_data_virtual_address:#x} in loader space)"
    );

    let identity_code_size =
        ptr::addr_of!(IDENTITY_CODE_END) as usize - ptr::addr_of!(IDENTITY_CODE_START) as usize;

    let identity_code_address = allocate_frame(
        AllocationType::Any,
        (identity_code_size as u64).div_ceil(frame_size()),
        4096u64,
    )
    .unwrap();

    let loader_identity_code_virtual_address = map(
        identity_code_address,
        identity_code_size.div_ceil(page_size()),
        ProtectionFlags::READ | ProtectionFlags::EXECUTE,
    )
    .unwrap();

    application_space
        .map(
            identity_code_address,
            identity_code_address,
            (identity_code_size as u64).div_ceil(application_space.page_size()),
            ProtectionFlags::READ | ProtectionFlags::EXECUTE,
        )
        .unwrap();

    switch_space
        .map(
            identity_code_address,
            identity_code_address,
            (identity_code_size as u64).div_ceil(switch_space.page_size()),
            ProtectionFlags::READ | ProtectionFlags::EXECUTE,
        )
        .unwrap();

    log_debug!(
        "identity code located at {identity_code_address:#x} \
        ({loader_identity_code_virtual_address:#x} in loader space)"
    );

    unsafe {
        ptr::copy_nonoverlapping(
            ptr::addr_of!(SWITCH_CODE_LOADER_START),
            loader_switch_virtual_address as *mut u8,
            switch_code_loader_size,
        )
    }

    unsafe {
        ptr::copy_nonoverlapping(
            ptr::addr_of!(IDENTITY_CODE_START),
            loader_identity_code_virtual_address as *mut u8,
            identity_code_size,
        )
    }

    let loader_identity_data_address_address = loader_switch_virtual_address
        + (ptr::addr_of!(SWITCH_CODE_LOADER_DATA_ADDRESS) as usize
            - ptr::addr_of!(SWITCH_CODE_LOADER_START) as usize);
    unsafe {
        *(loader_identity_data_address_address as *mut *mut IdentityData) =
            identity_data_address as *mut IdentityData
    }

    let loader_enter_application = loader_switch_virtual_address
        + (ptr::addr_of!(SWITCH_CODE_LOADER_ENTER_APPLICATION) as usize
            - ptr::addr_of!(SWITCH_CODE_LOADER_START) as usize);

    let identity_data = IdentityData {
        loader: CpuData::default(),
        application: CpuData::default(),

        switch_cr3: switch_space.physical_address(),

        bits_32_entry: identity_code_address as u32
            + (ptr::addr_of!(IDENTITY_CODE_ENTRY_32) as usize
                - ptr::addr_of!(IDENTITY_CODE_START) as usize) as u32,
        bits_64_entry: identity_code_address as u32
            + (ptr::addr_of!(IDENTITY_CODE_ENTRY_64) as usize
                - ptr::addr_of!(IDENTITY_CODE_START) as usize) as u32,

        identity_code_address: identity_code_address as u32,
        identity_data_address: identity_data_address as u32,

        tiny_stack: [0; 2],
        stack_top: [0; 0],

        switch_gdt: SWITCH_GDT,
    };

    unsafe { (loader_identity_data_virtual_address as *mut IdentityData).write(identity_data) }

    unsafe {
        #[cfg(target_arch = "x86_64")]
        asm!(
            "5:",
            "jmp 5b",
            "cli",
            "call {enter_application}",
            enter_application = in(reg) loader_enter_application,
        )
    }

    todo!()
}

/// Various errors that can occur while executing [`switch()`].
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SwitchError {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IdentityData {
    loader: CpuData,
    application: CpuData,

    switch_cr3: u64,

    bits_32_entry: u32,
    bits_64_entry: u32,

    identity_code_address: u32,
    identity_data_address: u32,

    tiny_stack: [u64; 2],
    stack_top: [u8; 0],

    switch_gdt: [u64; 5],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CpuData {
    // System registers.
    cr3: u64,

    cs: u16,
    ds: u16,
    es: u16,
    gdt_length: u16,
    gdt_offset: u64,

    fs: u16,
    gs: u16,
    ss: u16,
    idt_length: u16,
    idt_offset: u64,

    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,

    rsp: u64,
    rbp: u64,
    rsi: u64,
    rdi: u64,

    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,

    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
}

// 0-255 are the interrupt tables delivering to the loader application.
const ENTER_APPLICATION: u16 = 256;
const RETURN_TO_LOADER: u16 = 257;
const RETURN_TO_APPLICATION: u16 = 258;

unsafe extern "C" {
    /// The start of the identity-mapped code.
    #[cfg_attr(target_arch = "x86_64", link_name = "_IDENTITY_CODE_START")]
    static IDENTITY_CODE_START: u8;

    #[cfg_attr(target_arch = "x86_64", link_name = "_IDENTITY_CODE_ENTRY_32")]
    static IDENTITY_CODE_ENTRY_32: u8;
    #[cfg_attr(target_arch = "x86_64", link_name = "_IDENTITY_CODE_ENTRY_64")]
    static IDENTITY_CODE_ENTRY_64: u8;

    /// The end of the identity-mapped code.
    #[cfg_attr(target_arch = "x86_64", link_name = "_IDENTITY_CODE_END")]
    static IDENTITY_CODE_END: u8;
}

global_asm!(
    // Signals the start of the identity-mapped code.
    ".global _IDENTITY_CODE_START",
    "_IDENTITY_CODE_START:",

    // The 32-bit entry point.
    ".global _IDENTITY_CODE_ENTRY_32",
    "_IDENTITY_CODE_ENTRY_32:",
    ".code32",
    "5:",
    "jmp 5b",

    // The 64-bit entry point.
    //
    // Entry point in eax.
    //
    // Function code in ebx.
    // Argument 0 in ecx.
    // Argument 1 in edx.
    // Argument 2 in ebp.
    //
    // Identity code address in esi.
    // Identity data address in edi.
    // Tiny stack address in esp.
    ".global _IDENTITY_CODE_ENTRY_64",
    "_IDENTITY_CODE_ENTRY_64:",

    ".code64",

    "lea eax, [ecx + {SWITCH_GDT_OFFSET}]",
    "push rax",
    "mov rax, {SWITCH_GDT_SIZE}",
    "push ax",

    "lgdt [esp]",

    "pop ax",
    "pop rax",

    // Build far-return stack to switch to a 32-bit code segment.
    "mov rax, 0x18",
    "push rax",
    "lea rax, [rip + 5f]",
    "push rax",

    // Execute the far-return.
    //
    // This puts us into compatibility mode.
    "retfq",
    "5:",

    ".code32",
    "mov ax, 0x20",
    "mov ds, ax",
    "mov es, ax",
    "mov fs, ax",
    "mov gs, ax",
    "mov ss, ax",

    "mov eax, cr0",
    "and eax, 0x7fffffff",
    "mov cr0, eax",

    "",

    // Signals the end of the identity-mapped code.
    ".global _IDENTITY_CODE_END",
    "_IDENTITY_CODE_END:",

    SWITCH_GDT_OFFSET = const offset_of!(IdentityData, switch_gdt),
    SWITCH_GDT_SIZE = const mem::size_of_val(&SWITCH_GDT) - 1,
);
