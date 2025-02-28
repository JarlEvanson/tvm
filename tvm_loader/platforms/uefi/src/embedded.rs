//! Implementation of PE self-parsing in order to acquire the embedded `tvm` binary to load.

use core::{ffi, mem, ptr, slice};

use uefi::{
    data_types::{Handle, Status},
    protocol::loaded_image::LoadedImageProtocol,
    table::boot::{BootServices1_1, OpenAttributes},
};

use crate::{boot_services_ptr, image_handle};

/// The name of the PE section containing the embedded `tvm` binary.
const SECTION_NAME: [u8; 8] = [b'T', b'V', b'M', b'_', b'B', b'I', b'N', 0];

/// Returns the `tvm` binary embedded in the UEFI application.
///
/// # Panics
///
/// Panics if the image is too small to contain any structures or sections required to find the
/// embedded `tvm` binary.
pub fn get_tvm_image() -> &'static [u8] {
    let (image_base, image_size) = get_image_info();

    assert!(mem::size_of::<DosHeader>() as u64 <= image_size);
    // SAFETY:
    //
    // `image_size` is large enough to contain a [`DosHeader`].
    let dos_header = unsafe { image_base.cast::<DosHeader>().read_unaligned() };
    assert_eq!(dos_header.magic, 0x5A4D, "invalid DOS magic number");

    let size = dos_header.new_exe_header as usize + mem::size_of::<NtHeader64>();
    assert!(size as u64 <= image_size);

    // SAFETY:
    //
    // `image_size` is large enough to contain a [`NtHeader64`] at `dos_header.new_exe_header`
    // offset.
    let nt_header_ptr = unsafe { image_base.byte_add(dos_header.new_exe_header as usize) };
    // SAFETY:
    //
    // `image_size` is large enough to contain a [`NtHeader64`] at `dos_header.new_exe_header`
    // offset.
    let nt_header = unsafe { nt_header_ptr.cast::<NtHeader64>().read_unaligned() };
    assert_eq!(nt_header.signature, 0x00004550);

    let section_header_table_offset = size + nt_header.file_header.size_of_optional_header as usize;
    let size = section_header_table_offset
        + mem::size_of::<SectionHeader>() * nt_header.file_header.number_of_sections as usize;
    assert!(size as u64 <= image_size);

    for section_header_index in 0..nt_header.file_header.number_of_sections {
        let section_header_offset = section_header_table_offset
            + section_header_index as usize * mem::size_of::<SectionHeader>();

        // SAFETY:
        //
        // `image_size` is large enough to contain the entire [`SectionHeader`] table.
        let section_header_ptr = unsafe { image_base.byte_add(section_header_offset) };
        // SAFETY:
        //
        // `image_size` is large enough to contain the entire [`SectionHeader`] table.
        let section_header = unsafe { section_header_ptr.cast::<SectionHeader>().read_unaligned() };

        if section_header.name != SECTION_NAME {
            continue;
        }
        assert!(
            (section_header.virtual_address + section_header.virtual_size) as u64 <= image_size
        );

        // SAFETY:
        //
        // `image_size` is large enough to contain the embedded `tvm` binary section.
        let section_ptr = unsafe { image_base.byte_add(section_header.virtual_address as usize) };
        // SAFETY:
        //
        // `image_size` is large enough to contain the embedded `tvm` binary section.
        return unsafe {
            slice::from_raw_parts(
                section_ptr.cast::<u8>(),
                section_header.virtual_size as usize,
            )
        };
    }

    panic!("tvm binary not embedded")
}

/// Returns the image's base and size.
fn get_image_info() -> (*mut ffi::c_void, u64) {
    let boot_services_ptr = boot_services_ptr().expect("UEFI Boot Services have been exited");

    // SAFETY:
    //
    // `boot_services_ptr` is not NULL and so according to the UEFI specification, this header
    // should be available.
    let header = unsafe { (*boot_services_ptr.as_ptr()).header };
    if !(header.revision.major() == 2
        || (header.revision.major() == 1 && header.revision.minor() >= 1))
    {
        panic!("UEFI Boot Services {} is not supported", header.revision);
    }

    // SAFETY:
    //
    // `boot_services_ptr` is not NULL and the UEFI Boot Services Table is revision 1.1 or higher
    // so according to the UEFI specification, this function should be available.
    let open_protocol =
        unsafe { (*boot_services_ptr.as_ptr().cast::<BootServices1_1>()).open_protocol };

    let mut interface = ptr::null_mut();
    // SAFETY:
    //
    // `open_protocol` came from a valid UEFI Boot Services Table of an appropriate revision, so
    // this should be valid to call.
    let status = unsafe {
        open_protocol(
            image_handle(),
            &LoadedImageProtocol::GUID,
            &mut interface,
            image_handle(),
            Handle(ptr::null_mut()),
            OpenAttributes::GET_PROTOCOL,
        )
    };
    assert_eq!(status, Status::SUCCESS);

    // SAFETY:
    //
    // `open_protocol` was successful and so according to the UEFI specification, `interface`
    // should point to a [`LoadedImageProtocol`] structure.
    let interface = unsafe { *interface.cast::<LoadedImageProtocol>() };

    (interface.image_base, interface.image_size)
}

/// The layout of a DOS file header.
///
/// This header appears in the first 64 bytes of a PE file.
#[repr(C)]
struct DosHeader {
    /// The magic numbers identifying a DOS file.
    magic: u16,
    /// The number of bytes on the last page of the file.
    last_page_byte_count: u16,
    /// The number of pages in the file.
    file_page_count: u16,
    /// The number of relocations in the relocation table.
    relocations: u16,
    /// The size of the header in paragraphs.
    header_size_paragraphs: u16,
    /// The minimum number of extra paragraphs required.
    minimum_extra_paragraphs: u16,
    /// The maximum number of extra paragraphs required.
    maximum_extra_paragraphs: u16,
    /// The initial value of the SS register.
    initial_ss_value: u16,
    /// The initial value of the stack pointer.
    initial_sp_value: u16,
    /// The checksum of the header.
    checksum: u16,
    /// The initial value of the instruction pointer.
    initial_ip: u16,
    /// The intial value of the CS register.
    initial_cs: u16,
    /// The offset to the start of the relocation table.
    relocation_table_offset: u16,
    /// The overlay number of this file.
    overlay_number: u16,
    /// Reserved words.
    reserved_0: [u16; 4],
    /// OEM identifier.
    oem_id: u16,
    /// Information specific to the [`DosHeader::oem_id`].
    oem_info: u16,
    /// Reserved words.
    reserved_1: [u16; 10],
    /// The offset to the start of the NT headers.
    new_exe_header: u32,
}

/// Header for PE32+ executable files.
#[repr(C)]
struct NtHeader64 {
    /// The magic numbers that identify the [`NtHeader64`].
    signature: u32,
    /// Structure containing important information about the layout of the PE32+ file.
    file_header: FileHeader,
}

/// Contains important information about the layout of the PE file.
#[repr(C)]
struct FileHeader {
    /// The CPU architecture the executable targets.
    machine: u16,
    /// The number of sections.
    number_of_sections: u16,
    /// A UNIX timestamp that indicates when the file was created.
    time_date_stamp: u32,
    /// The offset to the COFF symbol table.
    pointer_to_symbol_table: u32,
    /// The number of symbols in the COFF symbol table.
    number_of_symbols: u32,
    /// The size of the optional header.
    size_of_optional_header: u16,
    /// Flags that indicate attributes of the file.
    characteristics: u16,
}

/// Contains information about the position and size of a PE section as well as information
/// required to load or debug a PE section.
#[repr(C)]
struct SectionHeader {
    /// The name of the section.
    ///
    /// Must be 8 bytes or less.
    name: [u8; 8],
    /// The total size of the section when loaded into memory.
    virtual_size: u32,
    /// The address of the first byte of the section relative to the image base when loaded in
    /// memory.
    virtual_address: u32,
    /// The size of the section on disk.
    size_of_raw_data: u32,
    /// A pointer to the first page of the section within the file.
    pointer_to_raw_data: u32,
    /// A pointer to the beginning of relocation entries for the section.
    pointer_to_relocations: u32,
    /// A pointer to the beginning of COFF line-number entries for the section.
    pointer_to_line_numbers: u32,
    /// The number of relocation entries for the section.
    number_of_relocations: u16,
    /// The number of COFF line-number entries for the section.
    number_of_line_numbers: u16,
    /// Flags that describe the section.
    characteristics: u32,
}
