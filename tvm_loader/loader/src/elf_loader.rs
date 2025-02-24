//! Generic ELF loading implementation for `tvm_loader` crates.

use core::{error, fmt};

use elf::{
    ElfFile, ParseElfFileError,
    class::AnyClass,
    dynamic::{ClassParseDynamic, ConstDynamicTag, DynamicTable},
    encoding::AnyEndian,
    header::ElfType,
    ident::Encoding,
    program_header::{SegmentFlags, SegmentType},
    relocation::{ClassParseRelocation, RelaTable},
};

use crate::{
    log_debug, log_trace, log_warn,
    memory::{
        phys::{AllocationType, allocate_frame, frame_size},
        virt::{
            ProtectionFlags,
            arch::AddressSpace,
            loader::{map, page_size, unmap},
        },
    },
};
pub use elf::header::Machine;

/// Loads the ELF file contained in `application`, verifying that the ELF file's target is
/// `target_machine`.
///
/// The loaded ELF file is relocated with the help of `relocation_handler`.
///
/// # Errors
///
/// - [`LoadApplicationError::InvalidElf`]: Returned when `application` contains an invalid ELF
///   file.
/// - [`LoadApplicationError::MachineMismatch`]: Returned when `application`'s target [`Machine`]
///   is not `target_machine`.
///
#[expect(clippy::missing_panics_doc)]
pub fn load_application(
    application: &[u8],
    target_machine: Machine,
    application_space: &mut dyn AddressSpace,
    relocation_handler: fn(&RelocationInformation) -> Result<FinalizedRelocation, ()>,
) -> Result<u64, LoadApplicationError> {
    let elf = ElfFile::<AnyClass, AnyEndian>::new(application)?;

    let application_machine = elf.header().machine();
    if application_machine != target_machine {
        return Err(LoadApplicationError::MachineMismatch(application_machine));
    }

    let program_header_table = elf
        .program_header_table()
        .ok_or(LoadApplicationError::MissingProgramHeaderTable)?;

    let slide = match elf.header().elf_type() {
        ElfType::EXECUTABLE => 0,
        ElfType::SHARED => {
            let load_headers = program_header_table
                .into_iter()
                .filter(|header| header.segment_type() == SegmentType::LOAD);

            let mut min_address = application_space.max_address();
            let mut max_address = u64::MIN;
            let mut max_alignment = application_space.page_size();

            for header in load_headers {
                max_alignment = max_alignment.max(header.alignment());

                min_address = min_address.min(header.virtual_address());
                max_address = max_address.max(header.virtual_address() + header.memory_size());
            }

            let aligned_min_address = min_address - min_address % application_space.page_size();
            let aligned_max_address =
                match max_address.checked_next_multiple_of(application_space.page_size()) {
                    Some(aligned_max_address)
                        if aligned_max_address < application_space.max_address() =>
                    {
                        aligned_max_address
                    }
                    _ => return Err(LoadApplicationError::ApplicationTooLarge),
                };

            let byte_span = aligned_max_address - aligned_min_address;
            let base = application_space.max_address() - byte_span;

            base - base % max_alignment
        }
        elf_type => return Err(LoadApplicationError::UnsupportedElfType(elf_type)),
    };
    log_trace!("slide: {slide:#x}");

    for header in program_header_table {
        match header.segment_type() {
            SegmentType::NULL
            | SegmentType::DYNAMIC
            | SegmentType::INTERPRETER
            | SegmentType::NOTE
            | SegmentType::TLS
            | SegmentType::PHDR => {}
            SegmentType::LOAD => {
                let start_address = slide + header.virtual_address();
                let end_address = start_address + header.memory_size();

                let aligned_start_address =
                    start_address - start_address % application_space.page_size();
                let aligned_end_address =
                    end_address.next_multiple_of(application_space.page_size());
                let byte_count = aligned_end_address - aligned_start_address;

                let frame_count = byte_count.div_ceil(frame_size());
                let frame_base = allocate_frame(
                    AllocationType::Any,
                    frame_count,
                    application_space.page_size(),
                )
                .expect("allocation failed");

                let mut protection = ProtectionFlags::READ;
                if header.flags() & SegmentFlags::WRITE == SegmentFlags::WRITE {
                    protection |= ProtectionFlags::WRITE;
                }

                if header.flags() & SegmentFlags::EXECUTE == SegmentFlags::EXECUTE {
                    protection |= ProtectionFlags::EXECUTE;
                }

                application_space
                    .map(
                        aligned_start_address,
                        frame_base,
                        byte_count.div_ceil(application_space.page_size()),
                        protection,
                    )
                    .map_err(|_| LoadApplicationError::OverlappingLoadSegments)?;

                let file_bytes =
                    &application[header.file_offset() as usize..][..header.file_size() as usize];

                let mut application_operation_address = start_address;
                while application_operation_address != end_address {
                    let physical_operation_address = application_space
                        .translate_virt(application_operation_address)
                        .expect("misbehaving application address space");

                    let loader_page_address = map(
                        physical_operation_address
                            - physical_operation_address % page_size() as u64,
                        1,
                        ProtectionFlags::READ | ProtectionFlags::WRITE,
                    )
                    .expect("faled to map region");

                    let mut loader_operation_address =
                        loader_page_address + (physical_operation_address as usize % page_size());
                    loop {
                        let byte = file_bytes
                            .get((application_operation_address - start_address) as usize)
                            .copied()
                            .unwrap_or(0);

                        // SAFETY:
                        //
                        // `loader_operation_address` is properly aligned and was just allocated,
                        // thus it is exclusively written.
                        unsafe { *(loader_operation_address as *mut u8) = byte }

                        application_operation_address += 1;
                        loader_operation_address += 1;
                        if application_operation_address == end_address
                            || loader_operation_address % page_size() == 0
                        {
                            break;
                        }
                    }

                    // SAFETY:
                    //
                    // `loader_page_address` was mapped by by a call to [`map()`] and will not be
                    // used after this call.
                    unsafe { unmap(loader_operation_address, 1).expect("failed to unmap region") }
                }
            }

            segment_type => log_warn!("unknown segment type: {segment_type:?}"),
        }
    }

    let dynamic_headers = program_header_table
        .into_iter()
        .filter(|header| header.segment_type() == SegmentType::DYNAMIC);
    for header in dynamic_headers {
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
        .expect("dynamic table too large");

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
            // Read a single Rela entry into a buffer, then extract it.
            let mut buffer = [0; 100];
            assert!(rela_entry_size <= buffer.len() as u64);

            for byte_index in 0..rela_entry_size {
                let physical_address = application_space
                    .translate_virt(slide + rela_table + index * rela_entry_size + byte_index)
                    .unwrap();

                let virtual_address = map(physical_address, 1, ProtectionFlags::READ).unwrap();

                // SAFETY:
                //
                // `virtual_address` is properly aligned and was just mapped, so it has exclusive
                // access to the `virtual_address`.
                buffer[byte_index as usize] = unsafe { *(virtual_address as *const u8) };

                // SAFETY:
                //
                // The mapping at `virtual_address` is not used again.
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
                offset: rela.offset,
                slide,
            };

            let relocation = relocation_handler(&relocation_info).unwrap();
            let address = rela.offset;

            let mut buffer = [0; 8];
            let byte_count;
            match relocation {
                FinalizedRelocation::Bits8(value) => {
                    match elf.header().ident().encoding() {
                        Encoding::LSB2 | Encoding::MSB2 => buffer[0] = value,
                        _ => unimplemented!(),
                    }
                    byte_count = 1;
                }
                FinalizedRelocation::Bits16(value) => {
                    match elf.header().ident().encoding() {
                        Encoding::LSB2 => buffer[..2].copy_from_slice(&value.to_le_bytes()),
                        Encoding::MSB2 => buffer[..2].copy_from_slice(&value.to_le_bytes()),
                        _ => unimplemented!(),
                    };
                    byte_count = 2;
                }
                FinalizedRelocation::Bits32(value) => {
                    match elf.header().ident().encoding() {
                        Encoding::LSB2 => buffer[..4].copy_from_slice(&value.to_le_bytes()),
                        Encoding::MSB2 => buffer[..4].copy_from_slice(&value.to_le_bytes()),
                        _ => unimplemented!(),
                    }
                    byte_count = 4
                }
                FinalizedRelocation::Bits64(value) => {
                    match elf.header().ident().encoding() {
                        Encoding::LSB2 => buffer.copy_from_slice(&value.to_le_bytes()),
                        Encoding::MSB2 => buffer.copy_from_slice(&value.to_le_bytes()),
                        _ => unimplemented!(),
                    }
                    byte_count = 8;
                }
            }

            for byte in buffer.into_iter().take(byte_count) {
                let physical_address = application_space.translate_virt(slide + address).unwrap();

                let virtual_address = map(
                    physical_address,
                    1,
                    ProtectionFlags::READ | ProtectionFlags::WRITE,
                )
                .unwrap();

                // SAFETY:
                //
                // `virtual_address` is properly aligned and was just mapped, so it has exclusive
                // access to the `virtual_address`.
                unsafe { *(virtual_address as *mut u8) = byte };

                // SAFETY:
                //
                // The mapping at `virtual_address` is not used again.
                unsafe { unmap(virtual_address, 1).unwrap() }
            }
        }
    }

    Ok(slide + elf.header().entry())
}

/// Various errors that can occur while loading and relocating an ELF file.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum LoadApplicationError {
    /// The provided `application` is not a valid ELF.
    InvalidElf(ParseElfFileError),
    /// The target machine of the provided `application` is not `target_machine`.
    MachineMismatch(Machine),
    /// The provided `application` does not contain program headers.
    ///
    /// This indicates that the `application` is not an executable file.
    MissingProgramHeaderTable,
    /// The provided `application` is of an unsupported [`ElfType`].
    UnsupportedElfType(ElfType),
    /// The application is too large for the [`AddressSpace`].
    ApplicationTooLarge,
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
        Self::InvalidElf(error)
    }
}

impl fmt::Display for LoadApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidElf(error) => write!(f, "error ocurred when parsing elf file: {error:?}"),
            Self::MachineMismatch(machine) => write!(f, "application is intended for {machine:?}"),
            Self::MissingProgramHeaderTable => write!(f, "missing elf program header table"),
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

impl error::Error for LoadApplicationError {}

/// Returns the target [`Machine`] for the given `application` as specified by the ELF file.
///
/// # Errors
///
/// - [`ParseElfFileError`]: Returned if an error occurs while parsing the ELF file.
pub fn get_machine(application: &[u8]) -> Result<Machine, ParseElfFileError> {
    Ok(ElfFile::<AnyClass, AnyEndian>::new(application)?
        .header()
        .machine())
}

/// Information that might be useful for relocation
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelocationInformation {
    /// The type of the relocation to carry out.
    pub relocation_type: u32,
    /// The offset at which the relocation should be applied.
    pub offset: u64,
    /// The value stored for use in a relocation.
    pub addend: i64,

    /// The slide of the relocated executable.
    pub slide: u64,
}

/// The size and value of a resolved relocation.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum FinalizedRelocation {
    /// Writes the given bits at the address.
    Bits8(u8),
    /// Write the given bits at the address.
    Bits16(u16),
    /// Write the given bits at the address.
    Bits32(u32),
    /// Write the given bits at the address.
    Bits64(u64),
}
