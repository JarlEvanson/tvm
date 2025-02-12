//! Generic ELF loading implementation for `tvm_loader` crates.

use core::fmt;

use elf::{
    ParseElfFileError,
    class::AnyClass,
    dynamic::{ClassParseDynamic, ConstDynamicTag, DynamicTable},
    encoding::AnyEndian,
    header::{ElfType, Machine},
    ident::Encoding,
    program_header::{SegmentFlags, SegmentType},
    relocation::{ClassParseRelocation, RelaTable},
};

use crate::{
    log_debug, log_trace, log_warn,
    memory::{
        phys::{AllocationFlags, allocate_frame, frame_size},
        virt::{AddressSpace, ProtectionFlags, map, page_size, unmap},
    },
};

/// Loads and relocates the ELF application into the provided [`AddressSpace`].
pub fn load_application<A: AddressSpace>(
    application: &[u8],
    machine: Machine,
    application_view: &mut A,
    relocate: fn(&RelocationInformation) -> Result<FinalizedRelocation, ()>,
) -> Result<u64, LoadApplicationError> {
    let elf = elf::ElfFile::<AnyClass, AnyEndian>::new(application)?;
    if elf.header().machine() != machine {
        return Err(LoadApplicationError::MachineMismatch(
            elf.header().machine(),
        ));
    }

    let program_header_table = elf
        .program_header_table()
        .ok_or(LoadApplicationError::MissingHeaderTable)?;

    let slide = match elf.header().elf_type() {
        ElfType::EXECUTABLE => 0,
        ElfType::SHARED => {
            let load_headers = program_header_table
                .into_iter()
                .filter(|header| header.segment_type() == SegmentType::LOAD);

            let mut max_address = u64::MIN;
            let mut min_address = A::max_address();
            let mut max_alignment = A::page_size();

            for header in load_headers {
                max_alignment = max_alignment.max(header.alignment());

                min_address = min_address.min(header.virtual_address());
                max_address = max_address.max(header.virtual_address() + header.memory_size());
            }

            let aligned_min_address = min_address - min_address % A::page_size();
            let aligned_max_address = match max_address.checked_next_multiple_of(A::page_size()) {
                Some(aligned_max_address) if aligned_max_address < A::max_address() => {
                    aligned_max_address
                }
                _ => return Err(LoadApplicationError::ApplicationTooLarge),
            };

            let span_bytes = aligned_max_address - aligned_min_address;
            let base = A::max_address() - span_bytes;
            let base = (base / max_alignment) * max_alignment;

            base
        }
        elf_type => return Err(LoadApplicationError::UnsupportedElfType(elf_type)),
    };
    log_trace!("slide: {slide:#X}");

    for header in program_header_table {
        match header.segment_type() {
            SegmentType::LOAD => {
                let start_address = slide + header.virtual_address();
                let end_address = start_address + header.memory_size();

                // Page aligned addresses and total bytes on mapped pages.
                let aligned_start_address = start_address - start_address % A::page_size();
                let aligned_end_address = end_address.next_multiple_of(A::page_size());
                let page_bytes = aligned_end_address - aligned_start_address;

                // Total number of frames required for page mapping.
                let required_frames = page_bytes.div_ceil(frame_size() as u64);
                let frames = allocate_frame(
                    required_frames as usize,
                    A::page_size() as usize,
                    AllocationFlags::ALLOCATE_ANY,
                    0,
                )
                .unwrap();

                let mut protection = ProtectionFlags::READ;
                if header.flags() & SegmentFlags::WRITE == SegmentFlags::WRITE {
                    protection |= ProtectionFlags::WRITE;
                }
                if header.flags() & SegmentFlags::EXECUTE == SegmentFlags::EXECUTE {
                    protection |= ProtectionFlags::EXECUTE;
                }

                application_view
                    .map(
                        aligned_start_address,
                        frames,
                        page_bytes / A::page_size(),
                        protection,
                    )
                    .unwrap();

                let file_bytes =
                    &application[header.file_offset() as usize..][..header.file_size() as usize];

                let mut operation_address = start_address;
                while operation_address != end_address {
                    let translated = application_view.translate_virt(operation_address).unwrap();

                    let loader_page_address = map(
                        (translated / page_size() as u64) * page_size() as u64,
                        1,
                        ProtectionFlags::WRITE,
                    )
                    .unwrap();
                    let mut loader_address =
                        loader_page_address + translated as usize % page_size();

                    loop {
                        let byte = file_bytes
                            .get((operation_address - start_address) as usize)
                            .copied()
                            .unwrap_or(0);

                        unsafe { *(loader_address as *mut u8) = byte };

                        operation_address += 1;
                        loader_address += 1;
                        if operation_address % page_size() as u64 == 0
                            || operation_address == end_address
                        {
                            break;
                        }
                    }

                    unsafe { unmap(loader_address, 1).unwrap() }
                }
            }
            SegmentType::NULL
            | SegmentType::DYNAMIC
            | SegmentType::INTERPRETER
            | SegmentType::NOTE
            | SegmentType::TLS
            | SegmentType::PHDR => {}
            segment_type => log_warn!("unknown segment type: {segment_type:?}"),
        }
    }

    for header in program_header_table
        .into_iter()
        .filter(|header| header.segment_type() == SegmentType::DYNAMIC)
    {
        let data = &application[header.file_offset() as usize..][..header.file_size() as usize];

        let mut rela_table = None;
        let mut rela_table_size = None;
        let mut rela_entry_size = None;

        let dynamic_table = DynamicTable::new(
            elf.header().class_parse(),
            elf.header().encoding_parse(),
            data,
            data.len() / elf.header().class_parse().expected_dynamic_size(),
        )
        .unwrap();

        log_trace!("reading dynamic table");
        for dynamic_tag in dynamic_table {
            if ClassParseDynamic::dynamic_tag_eq(dynamic_tag.tag, ConstDynamicTag::RELA_TABLE) {
                log_trace!("found rela table offset: {}", dynamic_tag.val);
                rela_table.replace(dynamic_tag.val);
            } else if ClassParseDynamic::dynamic_tag_eq(dynamic_tag.tag, ConstDynamicTag::RELA_SIZE)
            {
                log_trace!("found rela table size: {}", dynamic_tag.val);
                rela_table_size.replace(dynamic_tag.val);
            } else if ClassParseDynamic::dynamic_tag_eq(
                dynamic_tag.tag,
                ConstDynamicTag::RELA_ENTRY_SIZE,
            ) {
                log_trace!("found rela entry size: {}", dynamic_tag.val);
                rela_entry_size.replace(dynamic_tag.val);
            } else if ClassParseDynamic::dynamic_tag_eq(dynamic_tag.tag, ConstDynamicTag::NULL) {
                break;
            }
        }

        let Some(rela_table) = rela_table else {
            crate::log_debug!("dynamic table missing rela table");
            continue;
        };

        let rela_table_size = rela_table_size.ok_or(LoadApplicationError::MissingRelaTableSize)?;
        let rela_entry_size = rela_entry_size.ok_or(LoadApplicationError::MissingRelaEntrySize)?;

        let num_entries = rela_table_size / rela_entry_size;

        log_debug!("executing {num_entries} relocation entries");
        for index in 0..num_entries {
            let mut buffer = [0; 100];
            assert!(rela_entry_size <= buffer.len() as u64);

            for byte_index in 0..rela_entry_size {
                let physical_address = application_view
                    .translate_virt(slide + rela_table + index * rela_entry_size + byte_index)
                    .unwrap();

                let virtual_address = map(physical_address, 1, ProtectionFlags::READ).unwrap();

                buffer[byte_index as usize] = unsafe { *(virtual_address as *const u8) };

                unsafe { unmap(virtual_address, 1).unwrap() }
            }

            let rela = RelaTable::new(
                elf.header().class_parse(),
                elf.header().encoding_parse(),
                &buffer,
                1,
            )
            .unwrap()
            .get(0)
            .unwrap();

            let relocation_info = RelocationInformation {
                relocation_type: elf.header().class_parse().relocation_type_raw(rela.info),
                addend: rela.addend,
                slide,
            };

            let relocation = relocate(&relocation_info).unwrap();
            let address = rela.offset;

            let mut buffer = [0; 8];
            let byte_count;
            match relocation {
                FinalizedRelocation::Bits16(value) => {
                    match elf.header().ident().encoding() {
                        Encoding::LSB2 => buffer[..2].copy_from_slice(&value.to_le_bytes()),
                        Encoding::MSB2 => buffer[..2].copy_from_slice(&value.to_le_bytes()),
                        _ => todo!(),
                    };
                    byte_count = 2;
                }
                FinalizedRelocation::Bits32(value) => {
                    match elf.header().ident().encoding() {
                        Encoding::LSB2 => buffer[..4].copy_from_slice(&value.to_le_bytes()),
                        Encoding::MSB2 => buffer[..4].copy_from_slice(&value.to_le_bytes()),
                        _ => todo!(),
                    }
                    byte_count = 4
                }
                FinalizedRelocation::Bits64(value) => {
                    match elf.header().ident().encoding() {
                        Encoding::LSB2 => buffer.copy_from_slice(&value.to_le_bytes()),
                        Encoding::MSB2 => buffer.copy_from_slice(&value.to_le_bytes()),
                        _ => todo!(),
                    }
                    byte_count = 8;
                }
            }

            for byte_index in 0..byte_count {
                let physical_address = application_view.translate_virt(slide + address).unwrap();

                let virtual_address = map(physical_address, 1, ProtectionFlags::WRITE).unwrap();

                unsafe { *(virtual_address as *mut u8) = buffer[byte_index] };

                unsafe { unmap(virtual_address, 1).unwrap() }
            }
        }
    }

    Ok(slide + elf.header().entry())
}

/// Various errors that can occur while loading an application.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum LoadApplicationError {
    /// An error occurred while parsing the application.
    ElfError(ParseElfFileError),
    /// The desired machine type is unsupported.
    MachineMismatch(Machine),
    /// The application is missing a header table.
    MissingHeaderTable,
    /// The application is too large when loaded.
    ApplicationTooLarge,
    /// The ELF application is of an unsuspported type.
    UnsupportedElfType(ElfType),
    /// [`SegmentType::LOAD`] virtual ranges overlap.
    OverlappingLoadSegments,
    /// A relocation table is present, but the size of the relocation cannot be determined.
    MissingRelaTableSize,
    /// A relocation table is present, but the size of a relocation entry cannot be determined.
    MissingRelaEntrySize,
    /// An unsupported relocation type is present.
    UnsupportedRelocationType(u64),
}

impl From<ParseElfFileError> for LoadApplicationError {
    fn from(error: ParseElfFileError) -> Self {
        Self::ElfError(error)
    }
}

impl fmt::Display for LoadApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ElfError(error) => write!(f, "error ocurred when parsing elf file: {error:?}"),
            Self::MachineMismatch(machine) => write!(f, "application is intended for {machine:?}"),
            Self::MissingHeaderTable => write!(f, "missing elf program header table"),
            Self::ApplicationTooLarge => write!(f, "application is too large when loaded"),
            Self::UnsupportedElfType(elf_type) => {
                write!(f, "elf is of unsupported type {elf_type:?}")
            }
            Self::OverlappingLoadSegments => write!(f, "overlapping load segments"),
            Self::MissingRelaTableSize => write!(f, "missing dynamic rela table size tag"),
            Self::MissingRelaEntrySize => write!(f, "missing dynamic rela entry size tag"),
            Self::UnsupportedRelocationType(rel_type) => {
                write!(f, "unsupported relocation entry type: {rel_type:?}")
            }
        }
    }
}

impl core::error::Error for LoadApplicationError {}

/// Information that might be useful for relocation
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelocationInformation {
    /// The type of the relocation to carry out.
    pub relocation_type: u32,
    /// The value stored for use in a relocation.
    pub addend: i64,

    /// The slide of the relocated executable.
    pub slide: u64,
}

/// The size and value of a resolved relocation.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum FinalizedRelocation {
    /// Write the given 16-bits at the address.
    Bits16(u16),
    /// Write the given 32-bits at the address.
    Bits32(u32),
    /// Write the given 64-bits at the address.
    Bits64(u64),
}
