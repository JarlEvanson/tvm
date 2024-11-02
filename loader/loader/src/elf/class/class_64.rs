//! Implementation of 64-bit ELF file parsing.

use core::mem;

use crate::elf::{
    class::{ClassParse, ClassParseBase, UnsupportedClassError},
    encoding::EncodingParse,
    header::{ClassParseElfHeader, ElfType, Machine},
    ident::{Class, DefElfIdent},
};

/// A zero-sized object offering methods to safely parse 64-bit ELF files.
#[derive(Clone, Copy, Debug, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Class64;

impl ClassParse for Class64 {}

impl ClassParseElfHeader for Class64 {
    fn elf_type_offset(self) -> usize {
        mem::offset_of!(Elf64Header, elf_type)
    }

    fn machine_offset(self) -> usize {
        mem::offset_of!(Elf64Header, machine)
    }

    fn file_version_offset(self) -> usize {
        mem::offset_of!(Elf64Header, file_version)
    }

    fn entry_offset(self) -> usize {
        mem::offset_of!(Elf64Header, entry)
    }

    fn flags_offset(self) -> usize {
        mem::offset_of!(Elf64Header, flags)
    }

    fn header_size_offset(self) -> usize {
        mem::offset_of!(Elf64Header, header_size)
    }

    fn program_header_offset_offset(self) -> usize {
        mem::offset_of!(Elf64Header, program_header_offset)
    }

    fn program_header_count_offset(self) -> usize {
        mem::offset_of!(Elf64Header, program_header_count)
    }

    fn program_header_size_offset(self) -> usize {
        mem::offset_of!(Elf64Header, program_header_size)
    }

    fn section_header_offset_offset(self) -> usize {
        mem::offset_of!(Elf64Header, section_header_offset)
    }

    fn section_header_count_offset(self) -> usize {
        mem::offset_of!(Elf64Header, section_header_count)
    }

    fn section_header_size_offset(self) -> usize {
        mem::offset_of!(Elf64Header, section_header_size)
    }

    fn section_header_string_table_index_offset(self) -> usize {
        mem::offset_of!(Elf64Header, section_header_string_table_index)
    }

    fn expected_elf_header_size(self) -> usize {
        mem::size_of::<Elf64Header>()
    }
}

#[repr(C)]
#[expect(clippy::missing_docs_in_private_items)]
pub(crate) struct Elf64Header {
    pub identifier: DefElfIdent,

    pub elf_type: ElfType,
    pub machine: Machine,
    pub file_version: u32,
    pub entry: u64,

    pub program_header_offset: u64,
    pub section_header_offset: u64,

    pub flags: u32,
    pub header_size: u16,

    pub program_header_size: u16,
    pub program_header_count: u16,

    pub section_header_size: u16,
    pub section_header_count: u16,

    pub section_header_string_table_index: u16,
}

impl ClassParseBase for Class64 {
    type ClassUsize = u64;
    type ClassIsize = i64;

    fn from_elf_class(class: Class) -> Result<Self, UnsupportedClassError> {
        if class != Class::CLASS64 {
            return Err(UnsupportedClassError(class));
        }

        Ok(Self)
    }

    fn parse_class_usize_at<E: EncodingParse>(
        self,
        encoding: E,
        offset: usize,
        data: &[u8],
    ) -> Self::ClassUsize {
        encoding.parse_u64_at(offset, data)
    }

    fn parse_class_isize_at<E: EncodingParse>(
        self,
        encoding: E,
        offset: usize,
        data: &[u8],
    ) -> Self::ClassIsize {
        encoding.parse_i64_at(offset, data)
    }
}
