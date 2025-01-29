//! Definitions for the ELF program headers.

use core::fmt;

use crate::{
    class::{ClassParse, ClassParseBase},
    encoding::EncodingParse,
};

/// View of an ELF program header.
///
/// Structure that describes how to locate, load, and interpret data and configuration relevant to
/// program execution.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ProgramHeader<'slice, C, E> {
    /// The underlying bytes of the [`ProgramHeader`].
    pub(crate) bytes: &'slice [u8],
    /// The [`ClassParseProgramHeader`] of this [`ProgramHeader`].
    pub(crate) class: C,
    /// The [`EncodingParse`] of this [`ProgramHeader`].
    pub(crate) encoding: E,
}

impl<'slice, C: ClassParse, E: EncodingParse> ProgramHeader<'slice, C, E> {
    /// Creates a new [`ProgramHeader<Raw>`] from the given `slice`, returning `None` if the
    /// `slice` is too small to contain a [`ProgramHeader`].
    pub fn new(class: C, encoding: E, slice: &'slice [u8]) -> Option<Self> {
        if slice.len() < class.expected_program_header_size() {
            return None;
        }

        let program_header = Self {
            bytes: slice,
            class,
            encoding,
        };

        Some(program_header)
    }

    /// Validates that this [`ProgramHeader`] matches the ELF specification and is supported by
    /// this crate.
    ///
    /// # Errors
    ///
    /// - [`ValidateProgramHeaderSpecError::InvalidSegmentSizing`]:
    pub fn validate_specification(&self) -> Result<(), ValidateProgramHeaderSpecError> {
        if self.segment_type() == SegmentType::LOAD && self.file_size() > self.memory_size() {
            return Err(ValidateProgramHeaderSpecError::InvalidSegmentSizing);
        }

        let zero = <C::ClassUsize as crate::class::AdditiveIdentity>::ADDITIVE_IDENTITY;
        let one = <C::ClassUsize as crate::class::MultiplicativeIdentity>::MULTIPLICATIVE_IDENTITY;

        if self.alignment() == zero || self.alignment() == one {
        } else {
            let mut current = self.alignment();

            while current % (one + one) == zero {
                current = current / (one + one);
            }
            if current != one {
                return Err(ValidateProgramHeaderSpecError::InvalidAlignment);
            }
        }

        if self.alignment() != zero
            && self.virtual_address() % self.alignment() != self.file_offset() % self.alignment()
        {
            return Err(ValidateProgramHeaderSpecError::BrokenAlignment);
        }

        Ok(())
    }

    /// Returns the [`SegmentType`] of the segment this [`ProgramHeader`] controls.
    pub fn segment_type(&self) -> SegmentType {
        SegmentType(
            self.encoding
                .parse_u32_at(self.class.segment_type_offset(), self.bytes),
        )
    }

    /// Returns the [`SegmentFlags`] associated with this [`ProgramHeader`].
    pub fn flags(&self) -> SegmentFlags {
        SegmentFlags(
            self.encoding
                .parse_u32_at(self.class.segment_flags_offset(), self.bytes),
        )
    }

    /// Returns the offset within the file at which the segment this [`ProgramHeader`] controls
    /// starts.
    pub fn file_offset(&self) -> C::ClassUsize {
        self.class.parse_class_usize_at(
            self.encoding,
            self.class.segment_file_offset_offset(),
            self.bytes,
        )
    }

    /// Returns the number of bytes of the segment when stored within the file.
    pub fn file_size(&self) -> C::ClassUsize {
        self.class.parse_class_usize_at(
            self.encoding,
            self.class.segment_file_size_offset(),
            self.bytes,
        )
    }

    /// Returns the virtual address at which the first byte of the segment should reside in memory
    /// when loaded.
    pub fn virtual_address(&self) -> C::ClassUsize {
        self.class.parse_class_usize_at(
            self.encoding,
            self.class.segment_virtual_address_offset(),
            self.bytes,
        )
    }

    /// Returns the physical address at which the first byte of the segment should reside.
    ///
    /// This field is only relevent on some systems.
    pub fn physical_address(&self) -> C::ClassUsize {
        self.class.parse_class_usize_at(
            self.encoding,
            self.class.segment_physical_address_offset(),
            self.bytes,
        )
    }

    /// Returns the number of bytes of the segment when loaded into memory.
    pub fn memory_size(&self) -> C::ClassUsize {
        self.class.parse_class_usize_at(
            self.encoding,
            self.class.segment_memory_size_offset(),
            self.bytes,
        )
    }

    /// Returns the alignment of the segment referenced by this [`ProgramHeader`].
    ///
    /// This alignment is both the file offset alignment and the virtual address alignment.
    pub fn alignment(&self) -> C::ClassUsize {
        self.class.parse_class_usize_at(
            self.encoding,
            self.class.segment_alignment_offset(),
            self.bytes,
        )
    }
}

impl<C: ClassParse, E: EncodingParse> fmt::Debug for ProgramHeader<'_, C, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("ProgramHeader");

        debug_struct.field("segment_type", &self.segment_type());
        debug_struct.field("flags", &self.flags());

        debug_struct.field("file_offset", &self.file_offset());
        debug_struct.field("file_size", &self.file_size());

        debug_struct.field("virtual_address", &self.virtual_address());
        debug_struct.field("physical_address", &self.physical_address());
        debug_struct.field("memory_size", &self.memory_size());

        debug_struct.field("alignment", &self.alignment());

        debug_struct.finish()
    }
}

/// Various errors that can occur when validating a [`ProgramHeader`] follows the ELf specification
/// and is supported by this crate.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ValidateProgramHeaderSpecError {
    /// The [`ProgramHeader`] describes a [`SegmentType::LOAD`] segment and the file size is larger
    /// than the memory size.
    InvalidSegmentSizing,
    /// The alignment of a segment described by a [`ProgramHeader`] is not 0 or a power of two.
    InvalidAlignment,
    /// The segment descibed by a [`ProgramHeader`] is not properly aligned.
    BrokenAlignment,
}

impl fmt::Display for ValidateProgramHeaderSpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSegmentSizing => {
                write!(f, "load segment file size is larger than memory size",)
            }
            Self::InvalidAlignment => {
                write!(f, "segment alignment is not 0 or a power of two",)
            }
            Self::BrokenAlignment => write!(
                f,
                "segment virtual address is not equal to file offset modulo alignment"
            ),
        }
    }
}

/// The type of the segment the associated [`ProgramHeader`] contains.
#[repr(transparent)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SegmentType(pub u32);

impl SegmentType {
    /// Unsued [`ProgramHeader`].
    pub const NULL: Self = Self(0);
    /// Loadable segment.
    pub const LOAD: Self = Self(1);
    /// Dynamic linking information.
    pub const DYNAMIC: Self = Self(2);
    /// The program interpreter.
    pub const INTERPRETER: Self = Self(3);
    /// Auxiliary information.
    pub const NOTE: Self = Self(4);
    /// Reserved.
    pub const SHLIB: Self = Self(5);
    /// [`ProgramHeader`] table.
    pub const PHDR: Self = Self(6);
    /// Thread local storage.
    pub const TLS: Self = Self(7);
}

impl fmt::Debug for SegmentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NULL => f.pad("Null"),
            Self::LOAD => f.pad("Load"),
            Self::DYNAMIC => f.pad("Dynamic"),
            Self::INTERPRETER => f.pad("Interpreter"),
            Self::NOTE => f.pad("Note"),
            Self::SHLIB => f.pad("Shlib"),
            Self::PHDR => f.pad("ProgramHeaders"),
            Self::TLS => f.pad("Tls"),
            segment_type => f.debug_tuple("SegmentType").field(&segment_type.0).finish(),
        }
    }
}

/// The permissions of a [`SegmentType::LOAD`] segment.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SegmentFlags(pub u32);

impl SegmentFlags {
    /// The segment should be marked executable.
    pub const EXECUTE: Self = Self(0x1);
    /// The segment should be marked writable.
    pub const WRITE: Self = Self(0x2);
    /// The segment should be marked readable.
    pub const READ: Self = Self(0x4);

    /// Mask of the bits reserved for operating system specific semantics.
    pub const MASK_OS: Self = Self(0x0FF0_FFFF);
    /// Mask of the bits reserved for processor specific semantics.
    pub const MASK_PROCESSOR: Self = Self(0xF000_0000);
}

impl core::ops::BitOr for SegmentFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for SegmentFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl core::ops::BitAnd for SegmentFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl core::ops::BitAndAssign for SegmentFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl core::ops::BitXor for SegmentFlags {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl core::ops::BitXorAssign for SegmentFlags {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

/// A table of [`ProgramHeader`]s.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProgramHeaderTable<'slice, C, E> {
    /// The underlying bytes of the ELF program header table.
    pub(crate) bytes: &'slice [u8],
    /// The number of entries in the [`ProgramHeaderTable`].
    pub(crate) entry_count: u16,
    /// The stride of each [`ProgramHeader`] in the [`ProgramHeaderTable`].
    pub(crate) entry_size: u16,
    /// The [`ClassParse`] of this [`ProgramHeaderTable`].
    pub(crate) class: C,
    /// The [`EncodingParse`] of this [`ProgramHeaderTable`].
    pub(crate) encoding: E,
}

impl<'slice, C: ClassParse, E: EncodingParse> ProgramHeaderTable<'slice, C, E> {
    /// Creates a new [`ProgramHeaderTable`] from the given `slice`.
    ///
    /// The generated [`ProgramHeaderTable`] has `count` [`ProgramHeader`]s.
    pub fn new(class: C, encoding: E, slice: &'slice [u8], count: u16, size: u16) -> Option<Self> {
        if usize::from(size) < class.expected_program_header_size() {
            return None;
        }

        if usize::from(count)
            .checked_mul(usize::from(size))
            .is_none_or(|total_size| slice.len() < total_size)
        {
            return None;
        }

        let table = Self {
            bytes: slice,
            entry_count: count,
            entry_size: size,
            class,
            encoding,
        };

        Some(table)
    }

    /// Returns the [`ProgramHeader`] located at `index`.
    pub fn get(&self, index: u16) -> Option<ProgramHeader<'slice, C, E>> {
        if index >= self.entry_count {
            return None;
        }

        let program_header = ProgramHeader {
            bytes: &self.bytes[usize::from(index * self.entry_size)..],
            class: self.class,
            encoding: self.encoding,
        };

        Some(program_header)
    }

    /// Returns the number of [`ProgramHeader`]s in this [`ProgramHeaderTable`].
    pub fn count(&self) -> u16 {
        self.entry_count
    }
}

impl<'slice, C: ClassParse, E: EncodingParse> IntoIterator for ProgramHeaderTable<'slice, C, E> {
    type Item = ProgramHeader<'slice, C, E>;
    type IntoIter = IntoIter<'slice, C, E>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            table: self,
            next: 0,
        }
    }
}

impl<C: ClassParse, E: EncodingParse> fmt::Debug for ProgramHeaderTable<'_, C, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(*self).finish()
    }
}

/// An [`Iterator`] over the [`ProgramHeader`]s in a [`ProgramHeaderTable`].
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct IntoIter<'slice, C: ClassParse, E: EncodingParse> {
    /// The table to iterate over.
    table: ProgramHeaderTable<'slice, C, E>,
    /// The index in the [`ProgramHeaderTable`].
    next: u16,
}

impl<'slice, C: ClassParse, E: EncodingParse> Iterator for IntoIter<'slice, C, E> {
    type Item = ProgramHeader<'slice, C, E>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.table.get(self.next)?;

        self.next += 1;
        Some(item)
    }
}

/// The information required to implement class aware parsing of an ELF program header.
pub trait ClassParseProgramHeader: ClassParseBase {
    /// The offset of the [`SegmentType`].
    fn segment_type_offset(self) -> usize;
    /// The offset of the segment flags.
    fn segment_flags_offset(self) -> usize;

    /// The offset of the file offset of the segment.
    fn segment_file_offset_offset(self) -> usize;
    /// The offset of the number of bytes in the file's view of the segment.
    fn segment_file_size_offset(self) -> usize;

    /// The offset of the virtual address of the loaded segment.
    fn segment_virtual_address_offset(self) -> usize;
    /// The offset of the physical address of the loaded segment.
    fn segment_physical_address_offset(self) -> usize;
    /// The offset of the number of bytes in the loaded segment.
    fn segment_memory_size_offset(self) -> usize;

    /// The offset of the alignment of the segment.
    fn segment_alignment_offset(self) -> usize;

    /// The expected size of an ELF program header.
    fn expected_program_header_size(self) -> usize;
}
