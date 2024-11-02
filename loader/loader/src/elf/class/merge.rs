//! Implementation of merged [`ClassParse`] implementations.

use crate::elf::{
    class::{ClassParse, ClassParseBase},
    encoding::EncodingParse,
    header::ClassParseElfHeader,
    ident::Class,
};

/// An object used to dispatch the [`ClassParse`] to the two underlying [`ClassParse`]
/// implementations.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Merge<A: ClassParse, B: ClassParse> {
    /// The first [`ClassParse`] implementation.
    A(A),
    /// The second [`ClassParse`] implementation.
    B(B),
}

impl<A: ClassParse, B: ClassParse> ClassParse for Merge<A, B>
where
    B::ClassUsize: From<A::ClassUsize>,
    B::ClassIsize: From<A::ClassIsize>,
{
}

impl<A: ClassParse, B: ClassParse> ClassParseElfHeader for Merge<A, B>
where
    B::ClassUsize: From<A::ClassUsize>,
    B::ClassIsize: From<A::ClassIsize>,
{
    fn elf_type_offset(self) -> usize {
        match self {
            Self::A(a) => a.elf_type_offset(),
            Self::B(b) => b.elf_type_offset(),
        }
    }

    fn machine_offset(self) -> usize {
        match self {
            Self::A(a) => a.machine_offset(),
            Self::B(b) => b.machine_offset(),
        }
    }

    fn file_version_offset(self) -> usize {
        match self {
            Self::A(a) => a.file_version_offset(),
            Self::B(b) => b.file_version_offset(),
        }
    }

    fn entry_offset(self) -> usize {
        match self {
            Self::A(a) => a.entry_offset(),
            Self::B(b) => b.entry_offset(),
        }
    }

    fn flags_offset(self) -> usize {
        match self {
            Self::A(a) => a.flags_offset(),
            Self::B(b) => b.flags_offset(),
        }
    }

    fn header_size_offset(self) -> usize {
        match self {
            Self::A(a) => a.header_size_offset(),
            Self::B(b) => b.header_size_offset(),
        }
    }

    fn program_header_offset_offset(self) -> usize {
        match self {
            Self::A(a) => a.program_header_offset_offset(),
            Self::B(b) => b.program_header_offset_offset(),
        }
    }

    fn program_header_count_offset(self) -> usize {
        match self {
            Self::A(a) => a.program_header_count_offset(),
            Self::B(b) => b.program_header_count_offset(),
        }
    }

    fn program_header_size_offset(self) -> usize {
        match self {
            Self::A(a) => a.program_header_size_offset(),
            Self::B(b) => b.program_header_size_offset(),
        }
    }

    fn section_header_offset_offset(self) -> usize {
        match self {
            Self::A(a) => a.section_header_offset_offset(),
            Self::B(b) => b.section_header_offset_offset(),
        }
    }

    fn section_header_count_offset(self) -> usize {
        match self {
            Self::A(a) => a.section_header_count_offset(),
            Self::B(b) => b.section_header_count_offset(),
        }
    }

    fn section_header_size_offset(self) -> usize {
        match self {
            Self::A(a) => a.section_header_size_offset(),
            Self::B(b) => b.section_header_size_offset(),
        }
    }

    fn section_header_string_table_index_offset(self) -> usize {
        match self {
            Self::A(a) => a.section_header_string_table_index_offset(),
            Self::B(b) => b.section_header_string_table_index_offset(),
        }
    }

    fn expected_elf_header_size(self) -> usize {
        match self {
            Self::A(a) => a.expected_elf_header_size(),
            Self::B(b) => b.expected_elf_header_size(),
        }
    }
}

impl<A: ClassParse, B: ClassParse> ClassParseBase for Merge<A, B>
where
    B::ClassUsize: From<A::ClassUsize>,
    B::ClassIsize: From<A::ClassIsize>,
{
    type ClassUsize = B::ClassUsize;
    type ClassIsize = B::ClassIsize;

    fn from_elf_class(class: Class) -> Result<Self, super::UnsupportedClassError> {
        if let Ok(a) = A::from_elf_class(class) {
            return Ok(Self::A(a));
        }

        B::from_elf_class(class).map(Self::B)
    }

    fn parse_class_usize_at<E: EncodingParse>(
        self,
        encoding: E,
        offset: usize,
        data: &[u8],
    ) -> Self::ClassUsize {
        match self {
            Self::A(a) => B::ClassUsize::from(a.parse_class_usize_at(encoding, offset, data)),
            Self::B(b) => b.parse_class_usize_at(encoding, offset, data),
        }
    }

    fn parse_class_isize_at<E: EncodingParse>(
        self,
        encoding: E,
        offset: usize,
        data: &[u8],
    ) -> Self::ClassIsize {
        match self {
            Self::A(a) => B::ClassIsize::from(a.parse_class_isize_at(encoding, offset, data)),
            Self::B(b) => b.parse_class_isize_at(encoding, offset, data),
        }
    }
}
