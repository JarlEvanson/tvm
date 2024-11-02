//! Implementation of 32-bit ELF file parsing.

use core::mem;

use crate::elf::{
    class::{ClassParse, ClassParseBase, UnsupportedClassError},
    dynamic::{ClassParseDynamic, ConstDynamicTag, DynamicTag},
    encoding::EncodingParse,
    header::{ClassParseElfHeader, ElfType, Machine},
    ident::{Class, DefElfIdent},
    program_header::{ClassParseProgramHeader, SegmentFlags, SegmentType},
};

/// A zero-sized object offering methods to safely parse 32-bit ELF files.
#[derive(Clone, Copy, Debug, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Class32;

impl ClassParse for Class32 {}

impl ClassParseElfHeader for Class32 {
    fn elf_type_offset(self) -> usize {
        mem::offset_of!(Elf32Header, elf_type)
    }

    fn machine_offset(self) -> usize {
        mem::offset_of!(Elf32Header, machine)
    }

    fn file_version_offset(self) -> usize {
        mem::offset_of!(Elf32Header, file_version)
    }

    fn entry_offset(self) -> usize {
        mem::offset_of!(Elf32Header, entry)
    }

    fn flags_offset(self) -> usize {
        mem::offset_of!(Elf32Header, flags)
    }

    fn header_size_offset(self) -> usize {
        mem::offset_of!(Elf32Header, header_size)
    }

    fn program_header_offset_offset(self) -> usize {
        mem::offset_of!(Elf32Header, program_header_offset)
    }

    fn program_header_count_offset(self) -> usize {
        mem::offset_of!(Elf32Header, program_header_count)
    }

    fn program_header_size_offset(self) -> usize {
        mem::offset_of!(Elf32Header, program_header_size)
    }

    fn section_header_offset_offset(self) -> usize {
        mem::offset_of!(Elf32Header, section_header_offset)
    }

    fn section_header_count_offset(self) -> usize {
        mem::offset_of!(Elf32Header, section_header_count)
    }

    fn section_header_size_offset(self) -> usize {
        mem::offset_of!(Elf32Header, section_header_size)
    }

    fn section_header_string_table_index_offset(self) -> usize {
        mem::offset_of!(Elf32Header, section_header_string_table_index)
    }

    fn expected_elf_header_size(self) -> usize {
        mem::size_of::<Elf32Header>()
    }
}

#[repr(C)]
#[expect(clippy::missing_docs_in_private_items)]
pub(crate) struct Elf32Header {
    pub identifier: DefElfIdent,

    pub elf_type: ElfType,
    pub machine: Machine,
    pub file_version: u32,
    pub entry: u32,

    pub program_header_offset: u32,
    pub section_header_offset: u32,

    pub flags: u32,
    pub header_size: u16,

    pub program_header_size: u16,
    pub program_header_count: u16,

    pub section_header_size: u16,
    pub section_header_count: u16,

    pub section_header_string_table_index: u16,
}

impl ClassParseProgramHeader for Class32 {
    fn segment_type_offset(self) -> usize {
        mem::offset_of!(Elf32ProgramHeader, segment_type)
    }

    fn segment_flags_offset(self) -> usize {
        mem::offset_of!(Elf32ProgramHeader, flags)
    }

    fn segment_file_offset_offset(self) -> usize {
        mem::offset_of!(Elf32ProgramHeader, file_offset)
    }

    fn segment_file_size_offset(self) -> usize {
        mem::offset_of!(Elf32ProgramHeader, file_size)
    }

    fn segment_virtual_address_offset(self) -> usize {
        mem::offset_of!(Elf32ProgramHeader, virtual_address)
    }

    fn segment_physical_address_offset(self) -> usize {
        mem::offset_of!(Elf32ProgramHeader, physical_address)
    }

    fn segment_memory_size_offset(self) -> usize {
        mem::offset_of!(Elf32ProgramHeader, memory_size)
    }

    fn segment_alignment_offset(self) -> usize {
        mem::offset_of!(Elf32ProgramHeader, alignment)
    }

    fn expected_program_header_size(self) -> usize {
        mem::size_of::<Elf32ProgramHeader>()
    }
}

#[repr(C)]
#[expect(clippy::missing_docs_in_private_items)]
pub(crate) struct Elf32ProgramHeader {
    pub segment_type: SegmentType,
    pub file_offset: u32,
    pub virtual_address: u32,
    pub physical_address: u32,
    pub file_size: u32,
    pub memory_size: u32,
    pub flags: SegmentFlags,
    pub alignment: u32,
}

impl ClassParseDynamic for Class32 {
    fn dynamic_tag_eq(tag: DynamicTag<Self>, const_tag: ConstDynamicTag) -> bool {
        tag.0 == const_tag.0
    }

    fn dynamic_tag_offset(self) -> usize {
        mem::offset_of!(Elf32Dynamic, tag)
    }

    fn dynamic_value_offset(self) -> usize {
        mem::offset_of!(Elf32Dynamic, value)
    }

    fn expected_dynamic_size(self) -> usize {
        mem::size_of::<Elf32Dynamic>()
    }
}

#[repr(C)]
#[expect(clippy::missing_docs_in_private_items)]
pub(crate) struct Elf32Dynamic {
    pub tag: i32,
    pub value: u32,
}

impl ClassParseBase for Class32 {
    type ClassUsize = u32;
    type ClassIsize = i32;

    fn from_elf_class(class: Class) -> Result<Self, UnsupportedClassError> {
        if class != Class::CLASS32 {
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
        encoding.parse_u32_at(offset, data)
    }

    fn parse_class_isize_at<E: EncodingParse>(
        self,
        encoding: E,
        offset: usize,
        data: &[u8],
    ) -> Self::ClassIsize {
        encoding.parse_i32_at(offset, data)
    }
}
