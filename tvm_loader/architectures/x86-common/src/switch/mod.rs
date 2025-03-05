//! Implementation of the cross address space switching and function calls.

use core::{
    arch::{asm, global_asm},
    mem, ptr,
};

use tvm_loader::{
    log_trace,
    memory::{
        phys::{allocate_frame, frame_size, AllocationType},
        virt::{
            loader::{map, page_size},
            ProtectionFlags,
        },
    },
};
use x86::{
    APPLICATION_32_DATA_PTR, APPLICATION_32_END, APPLICATION_32_START, LOADER_DATA_PTR, LOADER_END,
    LOADER_START, LOADER_START_APPLICATION,
};
use x86_common::PagingMode;

use crate::paging::X86CommonAddressSpace;

mod x86;

/// Sets up and executes the switching code, thereby executing the appplication starting at
/// `entry_point`.
///
/// # Errors
///
/// TODO:
pub fn switch(
    switch_space: &mut dyn X86CommonAddressSpace,
    application_space: &mut dyn X86CommonAddressSpace,
    entry_point: u64,
) -> Result<bool, SwitchError> {
    let physical_address = u64::from(u32::MAX) + 1;
    let under_4_gib = AllocationType::Below { physical_address };
    let long_mode = matches!(
        application_space.paging_mode(),
        PagingMode::Level4 | PagingMode::Level5
    );

    let (data_physical_address, data_virtual_address) = {
        let data_size = mem::size_of::<SwitchData>();
        let data_physical_address = allocate_frame(
            under_4_gib,
            (data_size as u64).div_ceil(frame_size()),
            4096u64,
        )
        .unwrap();

        let data_virtual_address = map(
            data_physical_address,
            data_size.div_ceil(page_size()),
            ProtectionFlags::READ | ProtectionFlags::WRITE,
        )
        .unwrap();

        switch_space
            .map(
                data_physical_address,
                data_physical_address,
                (data_size as u64).div_ceil(switch_space.page_size()),
                ProtectionFlags::READ | ProtectionFlags::WRITE,
            )
            .unwrap();

        application_space
            .map(
                data_physical_address,
                data_physical_address,
                (data_size as u64).div_ceil(application_space.page_size()),
                ProtectionFlags::READ | ProtectionFlags::WRITE,
            )
            .unwrap();

        (data_physical_address, data_virtual_address)
    };

    log_trace!(
        "data placed at {data_physical_address:#x} \
        ({data_virtual_address:#x} in loader space)"
    );

    let (code_ptr, code_size, code_physical_address, code_virtual_address, code_data_address) = {
        let code_size = ptr::addr_of!(CODE_END).addr() - ptr::addr_of!(CODE_START).addr();
        let code_physical_address = allocate_frame(
            under_4_gib,
            (code_size as u64).div_ceil(frame_size()),
            4096u64,
        )
        .unwrap();

        let code_virtual_address = map(
            code_physical_address,
            code_size.div_ceil(page_size()),
            ProtectionFlags::READ | ProtectionFlags::WRITE,
        )
        .unwrap();

        switch_space
            .map(
                code_physical_address,
                code_physical_address,
                (code_size as u64).div_ceil(switch_space.page_size()),
                ProtectionFlags::READ | ProtectionFlags::EXECUTE,
            )
            .unwrap();

        application_space
            .map(
                code_physical_address,
                code_physical_address,
                (code_size as u64).div_ceil(application_space.page_size()),
                ProtectionFlags::READ | ProtectionFlags::EXECUTE,
            )
            .unwrap();

        let code_data_address = code_virtual_address
            + (ptr::addr_of!(CODE_DATA_PTR).addr() - ptr::addr_of!(CODE_START).addr());

        (
            ptr::addr_of!(CODE_START),
            code_size,
            code_physical_address,
            code_virtual_address,
            code_data_address,
        )
    };

    log_trace!(
        "code placed at {code_physical_address:#x} \
        ({code_virtual_address:#x} in loader space)"
    );

    let (
        loader_ptr,
        loader_size,
        loader_physical_address,
        loader_virtual_address,
        loader_data_address,
        loader_start_application,
    ) = {
        let loader_size = ptr::addr_of!(LOADER_END).addr() - ptr::addr_of!(LOADER_START).addr();
        let loader_physical_address = allocate_frame(
            under_4_gib,
            (loader_size as u64).div_ceil(frame_size()),
            4096u64,
        )
        .unwrap();

        let loader_virtual_address = map(
            loader_physical_address,
            loader_size.div_ceil(page_size()),
            ProtectionFlags::READ | ProtectionFlags::WRITE | ProtectionFlags::EXECUTE,
        )
        .unwrap();

        switch_space
            .map(
                loader_virtual_address as u64,
                loader_physical_address,
                (loader_size as u64).div_ceil(switch_space.page_size()),
                ProtectionFlags::READ | ProtectionFlags::EXECUTE,
            )
            .unwrap();

        let loader_data_address = loader_virtual_address
            + (ptr::addr_of!(LOADER_DATA_PTR).addr() - ptr::addr_of!(LOADER_START).addr());
        let loader_start_application = loader_virtual_address
            + (ptr::addr_of!(LOADER_START_APPLICATION).addr() - ptr::addr_of!(LOADER_START).addr());

        (
            ptr::addr_of!(LOADER_START),
            loader_size,
            loader_physical_address,
            loader_virtual_address,
            loader_data_address,
            loader_start_application,
        )
    };

    log_trace!(
        "loader code placed at {loader_physical_address:#x} \
         ({loader_virtual_address:#x} in loader space)"
    );

    let (
        application_ptr,
        application_size,
        application_physical_address,
        application_virtual_address,
        application_data_address,
    ) = {
        let application_start_ptr;
        let application_data_address;
        let application_end_ptr;
        if long_mode {
            todo!()
        } else {
            application_start_ptr = ptr::addr_of!(APPLICATION_32_START);
            application_data_address = ptr::addr_of!(APPLICATION_32_DATA_PTR);
            application_end_ptr = ptr::addr_of!(APPLICATION_32_END);
        }

        let application_size = application_end_ptr.addr() - application_start_ptr.addr();
        let application_physical_address = allocate_frame(
            under_4_gib,
            (application_size as u64).div_ceil(frame_size()),
            4096u64,
        )
        .unwrap();

        let application_virtual_address = map(
            application_physical_address,
            application_size.div_ceil(page_size()),
            ProtectionFlags::READ | ProtectionFlags::WRITE,
        )
        .unwrap();

        let application_data_address = application_virtual_address
            + (application_data_address.addr() - application_start_ptr.addr());

        (
            application_start_ptr,
            application_size,
            application_physical_address,
            application_virtual_address,
            application_data_address,
        )
    };

    log_trace!(
        "application code placed at {application_physical_address:#x} \
         ({application_virtual_address:#x} in loader space)"
    );

    unsafe { ptr::copy_nonoverlapping(code_ptr, code_virtual_address as *mut u8, code_size) }
    unsafe { ptr::copy_nonoverlapping(loader_ptr, loader_virtual_address as *mut u8, loader_size) }
    unsafe {
        ptr::copy_nonoverlapping(
            application_ptr,
            application_virtual_address as *mut u8,
            application_size,
        )
    }

    unsafe { *(code_data_address as *mut u32) = data_physical_address as u32 }
    unsafe { *(loader_data_address as *mut usize) = data_virtual_address }
    unsafe { *(application_data_address as *mut u32) = data_physical_address as u32 }

    let switch_data = SwitchData {
        loader: CpuData::default(),
        application: CpuData {
            cr3: application_space.physical_address(),

            cs: 0x18 - (long_mode as u16 * 0x10),
            ds: 0x20 - (long_mode as u16 * 0x10),
            es: 0x20 - (long_mode as u16 * 0x10),

            fs: 0x20 - (long_mode as u16 * 0x10),
            gs: 0x20 - (long_mode as u16 * 0x10),
            ss: 0x20 - (long_mode as u16 * 0x10),
        },

        tmp_storage: TmpStorage::default(),

        // GDT used for the switching code and the application.
        gdt: [
            0x0000_0000_0000_0000, // Null Segment
            0x00AF_9B00_0000_FFFF, // Kernel 64-bit code segment
            0x00CF_9300_0000_FFFF, // Kernel 64-bit data segment
            0x00CF_9B00_0000_FFFF, // Kernel 32-bit code segment
            0x00CF_9300_0000_FFFF, // Kernel 32-bit data segment
        ],
    };

    let result: usize;

    // SAFETY:
    //
    // TODO: Describe safety properties.
    unsafe {
        #[cfg(target_arch = "x86")]
        asm!(
            "5:",
            "jmp 5b",
            "cli", // Disable interrupts.

            "push ebx", // Push first and only argument.
            "call eax", // Call the function to start the application.
            "pop ebx", // ecx is an unused scratch register.

            "sti", // Enable interrupts.
            inout("eax") loader_start_application => result,
            in("ebx") ptr::null_mut::<u8>(),
        );
        #[cfg(target_arch = "x86_64")]
        asm!(
            "cli", // Disable interrupts.

            "call rax", // Call the function to start the application.

            "sti", // Enable interrupts.
            inout("rax") loader_start_application => result,
            in("rdi") ptr::null_mut::<u8>(),
        );
    }

    Ok(result != 0)
}

/// Various errors that can occur while executing [`switch()`].
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum SwitchError {}

pub struct SwitchData {
    /// The loader address space's system register values.
    loader: CpuData,
    /// The application address space's system register values.
    application: CpuData,

    /// Temporary storage for the cross address space switching.
    tmp_storage: TmpStorage,

    gdt: [u64; 5],
}

/// Information vital to proper function of the CPU which might be changed between the loader
/// address space and the application address space.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub struct CpuData {
    /// The CR3 value associated with the address space.
    cr3: u64,

    /// The CS value associated with the address space.
    cs: u16,
    /// The DS value associated with the address space.
    ds: u16,
    /// The ES value associated with the address space.
    es: u16,
    /// The size of the GDT.
    gdt_size: u16,
    /// The address of the GDT.
    gdt_address: u64,

    /// The FS value associated with the address space.
    fs: u16,
    /// The GS value associated with the address space.
    gs: u16,
    /// The SS value associated with the address space.
    ss: u16,
    /// The size of the IDT.
    idt_size: u16,
    /// The address of the IDT.
    idt_address: u64,

    /// The [`PagingMode`] of the address space.
    paging_mode: u8,
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub struct TmpStorage {
    /// The identity of the function to call.
    func_id: u16,
    /// The number of arguments that are valid.
    arg_count: u8,

    /// Storage for the 1st argument or the return value.
    arg_1_ret: u64,
    /// Storage for the 2nd argument.
    arg_2: u64,
    /// Storage for the 3rd argument.
    arg_3: u64,
    /// Storage for the 4th argument.
    arg_4: u64,
    /// Storage for the 5th argument.
    arg_5: u64,
}

unsafe extern "C" {
    /// The start of the identity-mapped code.
    #[cfg_attr(target_arch = "x86_64", link_name = "_CODE_START")]
    static CODE_START: u8;

    #[cfg_attr(target_arch = "x86_64", link_name = "_CODE_DATA_PTR")]
    static CODE_DATA_PTR: u8;

    #[cfg_attr(target_arch = "x86_64", link_name = "_CODE_ENTRY_32")]
    static CODE_ENTRY_32: u8;
    #[cfg_attr(target_arch = "x86_64", link_name = "_CODE_ENTRY_64")]
    static CODE_ENTRY_64: u8;

    /// The end of the identity-mapped code.
    #[cfg_attr(target_arch = "x86_64", link_name = "_CODE_END")]
    static CODE_END: u8;
}

const ENTER_APPLICATION: u16 = 0;

// Shared code.
global_asm!(
    // Signals the start of the shared code.
    ".global _CODE_START",
    "_CODE_START:",
    ".global _CODE_DATA_PTR",
    "_CODE_DATA_PTR:",
    ".4byte 0",
    ".global _CODE_ENTRY_32",
    "_CODE_ENTRY_32:",
    ".global _CODE_ENTRY_64",
    "_CODE_ENTRY_64:",
    // Signals the end of the shared code.
    ".global _CODE_END",
    "_CODE_END:",
);
