//! The `elf` crate provides an interface for reading ELF files.
//!
//! # Capabilities
//!
//! ## Works in `no_std` environments
//!
//! This crate provides an ELF file parsing interface which does not allocate or use any `std`
//! features, so it can be used in `no_std` contexts such as bootloaders, kernels, or hypervisors.
//!
//! ## Endian Awareness
//!
//! This crate handles differences between host and file endianness when parsing the ELF file
//! structures and provides generic implementations intended to support various use cases.
//!
//! ## Class Awareness
//!
//! This crate handles differences between host and file class sizes when parsing the ELF file
//! structures and provides generic implementations intended to support various use cases.
//!
//! ## Zero-Alloc Parsing
//!
//! This crate implements parsing in such a manner that avoids heap allocations. ELF structures are
//! lazily parsed with iterators or tables that only parse the requested structure when required.
//!
//! ## Uses no unsafe code
//!
//! This crate contains zero unsafe blocks of code.

#![no_std]

use core::fmt;

use class::ClassParse;
use encoding::EncodingParse;
use header::{ElfHeader, ParseElfHeaderError, ValidateElfHeaderSpecError};
use program_header::{ProgramHeader, ProgramHeaderTable, ValidateProgramHeaderSpecError};

pub mod class;
pub mod dynamic;
pub mod encoding;
pub mod header;
pub mod ident;
pub mod program_header;
pub mod relocation;

/// An ELF file.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ElfFile<'slice, C, E> {
    /// The underlying bytes of the ELF file.
    pub(crate) bytes: &'slice [u8],
    /// The [`ClassParse`] instance.
    pub(crate) class: C,
    /// The [`EncodingParse`] instance.
    pub(crate) encoding: E,
}

impl<'slice, C: ClassParse, E: EncodingParse> ElfFile<'slice, C, E> {
    /// Creates a new [`ElfFile<MinimalParse>`] from the given `slice`.
    ///
    /// # Errors
    ///
    /// - [`ParseElfFileError::ParseElfHeaderError`]: Returned if an error occurs when parsing the
    ///     [`ElfHeader`] contained in the given `slice`.
    /// - [`ParseElfFileError::ElfHeaderSpecError`]: Returned if an error occurs when validating
    ///     the [`ElfHeader`] follows the ELF specification.
    /// - [`ParseElfFileError::ProgramHeaderTableOutOfBounds`]: Returned if the
    ///     [`ProgramHeaderTable`] is out of the bounds of the given `slice`.
    pub fn new(slice: &'slice [u8]) -> Result<Self, ParseElfFileError> {
        let header = ElfHeader::new(slice)?;
        header.validate_spec()?;

        if header.program_header_count() != 0 {
            let offset = TryInto::<usize>::try_into(header.program_header_offset())
                .map_err(|_| ParseElfFileError::ProgramHeaderTableOutOfBounds)?;

            let total_size = usize::from(
                header
                    .program_header_count()
                    .checked_add(header.program_header_size())
                    .ok_or(ParseElfFileError::ProgramHeaderTableOutOfBounds)?,
            );

            if offset
                .checked_add(total_size)
                .is_none_or(|top| slice.len() < top)
            {
                return Err(ParseElfFileError::ProgramHeaderTableOutOfBounds);
            }
        }

        let file = Self {
            bytes: slice,
            class: header.class,
            encoding: header.encoding,
        };

        if let Some(table) = file.program_header_table() {
            for (index, header) in table.into_iter().enumerate() {
                header
                    .validate_specification()
                    .map_err(|error| ParseElfFileError::ProgramHeaderSpecError { index, error })?;
            }
        }

        Ok(file)
    }
}

impl<'slice, C: ClassParse, E: EncodingParse> ElfFile<'slice, C, E> {
    /// Returns the [`ElfHeader`] of this [`ElfFile`].
    pub fn header(&self) -> ElfHeader<'slice, C, E> {
        ElfHeader {
            bytes: self.bytes,
            class: self.class,
            encoding: self.encoding,
        }
    }

    /// Returns the [`ProgramHeaderTable`] of this [`ElfFile`].
    ///
    /// The presences of a [`ProgramHeaderTable`] is not guaranteed.
    pub fn program_header_table(&self) -> Option<ProgramHeaderTable<'slice, C, E>> {
        if self.header().program_header_count() == 0 {
            return None;
        }

        let Ok(offset) = self.header().program_header_offset().try_into() else {
            unreachable!()
        };

        let table = ProgramHeaderTable {
            bytes: &self.bytes[offset..],
            entry_count: self.header().program_header_count(),
            entry_size: self.header().program_header_size(),
            class: self.class,
            encoding: self.encoding,
        };

        Some(table)
    }

    /// Returns the file data associated with the given [`ProgramHeader`].
    pub fn segment_data(
        &self,
        program_header: ProgramHeader<'slice, C, E>,
    ) -> Option<&'slice [u8]> {
        let start: usize = program_header.file_offset().try_into().ok()?;
        let size = program_header.file_size().try_into().ok()?;
        let end = start.checked_add(size)?;

        self.bytes.get(start..end)
    }
}

impl<C: ClassParse, E: EncodingParse> fmt::Debug for ElfFile<'_, C, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("ElfFile");

        debug_struct.field("header", &self.header());

        if let Some(table) = self.program_header_table() {
            debug_struct.field("program_header_table", &table);
        }

        debug_struct.finish()
    }
}

/// Various errors that can occur while parsing an [`ElfFile`].
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ParseElfFileError {
    /// An error cocured while parsing the [`ElfHeader`].
    ParseElfHeaderError(ParseElfHeaderError),
    /// An error occurred while validating the [`ElfHeader`].
    ElfHeaderSpecError(ValidateElfHeaderSpecError),
    /// The [`ProgramHeaderTable`] is located out of bounds.
    ProgramHeaderTableOutOfBounds,
    /// An error occurred when validing the [`ProgramHeaderTable`].
    ProgramHeaderSpecError {
        /// The index of the [`ProgramHeader`] that fails to conform to the ELF specification.
        index: usize,
        /// The error that occurred.
        error: ValidateProgramHeaderSpecError,
    },
}

impl From<ParseElfHeaderError> for ParseElfFileError {
    fn from(value: ParseElfHeaderError) -> Self {
        Self::ParseElfHeaderError(value)
    }
}

impl From<ValidateElfHeaderSpecError> for ParseElfFileError {
    fn from(value: ValidateElfHeaderSpecError) -> Self {
        Self::ElfHeaderSpecError(value)
    }
}

impl fmt::Display for ParseElfFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseElfHeaderError(error) => {
                write!(f, "error while parsing ELF header: {error}")
            }
            Self::ElfHeaderSpecError(error) => write!(
                f,
                "error validating ELF header specification conformance: {error}"
            ),
            Self::ProgramHeaderTableOutOfBounds => {
                write!(f, "program header table located out of bounds")
            }
            Self::ProgramHeaderSpecError { index, error } => write!(
                f,
                "program header at index {index} does not conform to ELF specification: {error}",
            ),
        }
    }
}
