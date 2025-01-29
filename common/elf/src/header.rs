//! Definitions for the ELF file header.

use core::{error, fmt};

use crate::{
    class::{ClassParse, ClassParseBase, UnsupportedClassError},
    encoding::{EncodingParse, UnsupportedEncodingError},
    ident::{ElfIdent, ValidateElfIdentSpecError},
};

/// View of an ELF file header.
///
/// This contains some of the most important metadata of the file.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ElfHeader<'slice, C, E> {
    /// The underlying bytes of the ELF header.
    pub(crate) bytes: &'slice [u8],
    /// The [`ClassParseElfHeader`] of this ELF header.
    pub(crate) class: C,
    /// The [`EncodingParse`] of this ELF header.
    pub(crate) encoding: E,
}

impl<'slice, C: ClassParse, E: EncodingParse> ElfHeader<'slice, C, E> {
    /// Creates a new [`ElfHeader<Raw>`] from the given `slice`.
    ///
    /// # Errors
    ///
    /// - [`ParseElfHeaderError::TooSmall`]: Returned if the given `slice` is too small to
    ///     contain an [`ElfHeader`].
    /// - [`ParseElfHeaderError::UnsupportedClass`]: Returned if the [`Class`][c] of the
    ///     [`ElfHeader`] is not supported.
    /// - [`ParseElfHeaderError::UnsupportedEncoding`]: Returned if the [`Encoding`][e] of the
    ///     [`ElfHeader`] is not supported.
    ///
    /// [c]: crate::ident::Class
    /// [e]: crate::ident::Encoding
    pub fn new(slice: &'slice [u8]) -> Result<Self, ParseElfHeaderError> {
        let ident = ElfIdent::new(slice).ok_or(ParseElfHeaderError::TooSmall)?;
        let class = C::from_elf_class(ident.class())?;
        let encoding = E::from_elf_encoding(ident.encoding())?;

        if slice.len() < class.expected_elf_header_size() {
            return Err(ParseElfHeaderError::TooSmall);
        }

        let header = Self {
            bytes: slice,
            class,
            encoding,
        };

        Ok(header)
    }

    /// Validates that this [`ElfHeader`] matches the ELf specification and is supported by this
    /// crate.
    ///
    /// # Errors
    ///
    /// - [`ValidateElfHeaderSpecError::IdentError`]: Returned if an error occurs while parsing the
    ///     [`ElfIdent`] of this [`ElfHeader`].
    /// - [`ValidateElfHeaderSpecError::InvalidElfHeaderSize`]: Returned if the size of the
    ///     [`ElfHeader`] is smaller than expected.
    pub fn validate_spec(&self) -> Result<(), ValidateElfHeaderSpecError> {
        self.ident().validate_spec()?;

        if (self.header_size() as usize) < self.class.expected_elf_header_size() {
            return Err(ValidateElfHeaderSpecError::InvalidElfHeaderSize);
        }

        Ok(())
    }

    /// Returns the [`ClassParse`] implementation of this [`ElfHeader`].
    pub fn class_parse(&self) -> C {
        self.class
    }

    /// Returns the [`EncodingParse`] implementation of this [`ElfHeader`].
    pub fn encoding_parse(&self) -> E {
        self.encoding
    }

    /// Returns the [`ElfIdent`] associated with this [`ElfHeader`].
    pub fn ident(&self) -> ElfIdent<'slice> {
        match self.bytes.first_chunk() {
            Some(bytes) => ElfIdent { bytes },
            None => unreachable!(),
        }
    }

    /// Returns the type of this ELf file.
    pub fn elf_type(&self) -> ElfType {
        ElfType(
            self.encoding
                .parse_u16_at(self.class.elf_type_offset(), self.bytes),
        )
    }

    /// Returns the architecture for which this ELF file is targeted.
    pub fn machine(&self) -> Machine {
        Machine(
            self.encoding
                .parse_u16_at(self.class.machine_offset(), self.bytes),
        )
    }

    /// Returns the version of this ELF file.
    pub fn file_version(self) -> u32 {
        self.encoding
            .parse_u32_at(self.class.file_version_offset(), self.bytes)
    }

    /// Returns the processor specific flags associated with the ELF file.
    pub fn flags(&self) -> u32 {
        self.encoding
            .parse_u32_at(self.class.flags_offset(), self.bytes)
    }

    /// Returns the size of the ELF file header in bytes.
    pub fn header_size(&self) -> u16 {
        self.encoding
            .parse_u16_at(self.class.header_size_offset(), self.bytes)
    }

    /// Returns the virtual address of the entry point of this ELF file.
    pub fn entry(&self) -> C::ClassUsize {
        self.class
            .parse_class_usize_at(self.encoding, self.class.entry_offset(), self.bytes)
    }

    /// Returns the program header table's file offset in bytes.
    pub fn program_header_offset(&self) -> C::ClassUsize {
        self.class.parse_class_usize_at(
            self.encoding,
            self.class.program_header_offset_offset(),
            self.bytes,
        )
    }

    /// Returns the number of program headers in the program header table.
    pub fn program_header_count(&self) -> u16 {
        self.encoding
            .parse_u16_at(self.class.program_header_count_offset(), self.bytes)
    }

    /// Return the size of each program header in the program header table.
    pub fn program_header_size(&self) -> u16 {
        self.encoding
            .parse_u16_at(self.class.program_header_size_offset(), self.bytes)
    }

    /// Returns the section header table's file offset in bytes.
    pub fn section_header_offset(&self) -> C::ClassUsize {
        self.class.parse_class_usize_at(
            self.encoding,
            self.class.section_header_offset_offset(),
            self.bytes,
        )
    }

    /// Returns the number of section headers in the section header table.
    pub fn section_header_count(&self) -> u16 {
        self.encoding
            .parse_u16_at(self.class.section_header_count_offset(), self.bytes)
    }

    /// Return the size of each section header in the section header table.
    pub fn section_header_size(&self) -> u16 {
        self.encoding
            .parse_u16_at(self.class.section_header_size_offset(), self.bytes)
    }

    /// Returns the index into the section header table to obtain the section name string table.
    pub fn section_header_string_table_index(&self) -> u16 {
        self.encoding.parse_u16_at(
            self.class.section_header_string_table_index_offset(),
            self.bytes,
        )
    }
}

impl<C: ClassParse, E: EncodingParse> fmt::Debug for ElfHeader<'_, C, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("ElfHeader");

        debug_struct.field("elf_type", &self.elf_type());
        debug_struct.field("machine", &self.machine());
        debug_struct.field("file_version", &self.file_version());
        debug_struct.field("flags", &self.flags());
        debug_struct.field("header_size", &self.header_size());

        debug_struct.field("entry", &self.entry());

        debug_struct.field("program_header_offset", &self.program_header_offset());
        debug_struct.field("program_header_count", &self.program_header_count());
        debug_struct.field("program_header_size", &self.program_header_size());

        debug_struct.field("section_header_offset", &self.section_header_offset());
        debug_struct.field("section_header_count", &self.section_header_count());
        debug_struct.field("section_header_size", &self.section_header_size());

        debug_struct.field(
            "section_header_string_table_index",
            &self.section_header_string_table_index(),
        );

        debug_struct.finish()
    }
}

/// Various errors that can occur while creating an [`ElfHeader`].
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParseElfHeaderError {
    /// The given slice is too small.
    TooSmall,
    /// The [`Class`][c] is unsupported.
    ///
    /// [c]: crate::ident::Class
    UnsupportedClass(UnsupportedClassError),
    /// The [`Encoding`][e] is unsupported.
    ///
    /// [e]: crate::ident::Encoding
    UnsupportedEncoding(UnsupportedEncodingError),
}

impl From<UnsupportedClassError> for ParseElfHeaderError {
    fn from(value: UnsupportedClassError) -> Self {
        Self::UnsupportedClass(value)
    }
}

impl From<UnsupportedEncodingError> for ParseElfHeaderError {
    fn from(value: UnsupportedEncodingError) -> Self {
        Self::UnsupportedEncoding(value)
    }
}

impl fmt::Display for ParseElfHeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooSmall => f.pad("slice too small"),
            Self::UnsupportedClass(error) => fmt::Display::fmt(error, f),
            Self::UnsupportedEncoding(error) => fmt::Display::fmt(error, f),
        }
    }
}

impl error::Error for ParseElfHeaderError {}

/// Various errors that can occur when validating an [`ElfHeader`] follows the ELF specification
/// and is supported by this crate.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidateElfHeaderSpecError {
    /// An error occured while validating the contained [`ElfIdent`].
    IdentError(ValidateElfIdentSpecError),
    /// The size of the [`ElfHeader`] is given as smaller than expected.
    InvalidElfHeaderSize,
}

impl From<ValidateElfIdentSpecError> for ValidateElfHeaderSpecError {
    fn from(value: ValidateElfIdentSpecError) -> Self {
        Self::IdentError(value)
    }
}

impl fmt::Display for ValidateElfHeaderSpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdentError(error) => write!(f, "error while parsing ELF identifier: {error}"),
            Self::InvalidElfHeaderSize => {
                write!(f, "given ELF header size is smaller than expected")
            }
        }
    }
}

impl error::Error for ValidateElfHeaderSpecError {}

/// The type of the ELF file.
#[repr(transparent)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ElfType(pub u16);

impl ElfType {
    /// No type.
    pub const NONE: Self = Self(0);
    /// Relocatable ELF file.
    pub const RELOCATABLE: Self = Self(1);
    /// Executable ELF file.
    pub const EXECUTABLE: Self = Self(2);
    /// Shared object ELF file.
    pub const SHARED: Self = Self(3);
    /// Core ELF file.
    pub const CORE: Self = Self(4);
}

impl fmt::Debug for ElfType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NONE => f.pad("None"),
            Self::RELOCATABLE => f.pad("Relocatable"),
            Self::EXECUTABLE => f.pad("Executable"),
            Self::SHARED => f.pad("SharedObject"),
            Self::CORE => f.pad("Core"),
            elf_type => f.debug_tuple("ElfType").field(&elf_type.0).finish(),
        }
    }
}

/// The architecture of the ELF file.
#[repr(transparent)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Machine(pub u16);

impl Machine {
    /// No required machine.
    pub const NONE: Self = Self(0);
    /// ELF file requires the Intel 80386 architecture.
    pub const INTEL_386: Self = Self(3);
    /// ELF file requires the AArch32 architecture.
    pub const ARM: Self = Self(40);
    /// ELF file requires the AMD x86_64 architecture.
    pub const X86_64: Self = Self(62);
    /// ELF file requires the AArch64 architecture.
    pub const AARCH64: Self = Self(183);
}

impl fmt::Debug for Machine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NONE => f.pad("None"),
            Self::INTEL_386 => f.pad("Intel386"),
            Self::ARM => f.pad("Aarch32"),
            Self::X86_64 => f.pad("x86_64"),
            Self::AARCH64 => f.pad("Aarch64"),
            machine => f.debug_tuple("Machine").field(&machine.0).finish(),
        }
    }
}

/// The information required to implement class aware parsing of an ELF file header.
pub trait ClassParseElfHeader: ClassParseBase {
    /// The offset of the [`ElfType`].
    fn elf_type_offset(self) -> usize;
    /// The offset of the [`Machine`].
    fn machine_offset(self) -> usize;
    /// The offset of version of the ELF header.
    fn file_version_offset(self) -> usize;
    /// The offset of the entry point to the loaded ELF file.
    fn entry_offset(self) -> usize;

    /// The offset of the ELF file flags.
    fn flags_offset(self) -> usize;
    /// The offset of the size of the ELF file header.
    fn header_size_offset(self) -> usize;

    /// The offset of the offset of the program header table.
    fn program_header_offset_offset(self) -> usize;
    /// The offset of the number of entries in the program header table.
    fn program_header_count_offset(self) -> usize;
    /// The offset of the size of an entry in the program header table.
    fn program_header_size_offset(self) -> usize;

    /// The offset of the offset of the section header table.
    fn section_header_offset_offset(self) -> usize;
    /// The offset of the number of entries in the section header table.
    fn section_header_count_offset(self) -> usize;
    /// The offset of the size of an entry in the section header table.
    fn section_header_size_offset(self) -> usize;

    /// The offset of the index into the section header table to obtain the section name string
    /// table.
    fn section_header_string_table_index_offset(self) -> usize;

    /// The expected size of the ELF file header.
    fn expected_elf_header_size(self) -> usize;
}
