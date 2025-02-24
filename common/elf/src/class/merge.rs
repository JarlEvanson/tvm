//! Implementation of merged [`ClassParse`] implementations.

use crate::{
    class::{ClassParse, ClassParseBase},
    dynamic::ClassParseDynamic,
    header::ClassParseElfHeader,
    program_header::ClassParseProgramHeader,
    relocation::ClassParseRelocation,
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
    A::ClassUsize: TryFrom<B::ClassUsize>,
{
}

impl<A: ClassParse, B: ClassParse> ClassParseElfHeader for Merge<A, B>
where
    B::ClassUsize: From<A::ClassUsize>,
    B::ClassIsize: From<A::ClassIsize>,
    A::ClassUsize: TryFrom<B::ClassUsize>,
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

impl<A: ClassParse, B: ClassParse> ClassParseProgramHeader for Merge<A, B>
where
    B::ClassUsize: From<A::ClassUsize>,
    B::ClassIsize: From<A::ClassIsize>,
    A::ClassUsize: TryFrom<B::ClassUsize>,
{
    fn segment_type_offset(self) -> usize {
        match self {
            Self::A(a) => a.segment_type_offset(),
            Self::B(b) => b.segment_type_offset(),
        }
    }

    fn segment_flags_offset(self) -> usize {
        match self {
            Self::A(a) => a.segment_flags_offset(),
            Self::B(b) => b.segment_flags_offset(),
        }
    }

    fn segment_file_offset_offset(self) -> usize {
        match self {
            Self::A(a) => a.segment_file_offset_offset(),
            Self::B(b) => b.segment_file_offset_offset(),
        }
    }

    fn segment_file_size_offset(self) -> usize {
        match self {
            Self::A(a) => a.segment_file_size_offset(),
            Self::B(b) => b.segment_file_size_offset(),
        }
    }

    fn segment_virtual_address_offset(self) -> usize {
        match self {
            Self::A(a) => a.segment_virtual_address_offset(),
            Self::B(b) => b.segment_virtual_address_offset(),
        }
    }

    fn segment_physical_address_offset(self) -> usize {
        match self {
            Self::A(a) => a.segment_physical_address_offset(),
            Self::B(b) => b.segment_physical_address_offset(),
        }
    }

    fn segment_memory_size_offset(self) -> usize {
        match self {
            Self::A(a) => a.segment_memory_size_offset(),
            Self::B(b) => b.segment_memory_size_offset(),
        }
    }

    fn segment_alignment_offset(self) -> usize {
        match self {
            Self::A(a) => a.segment_alignment_offset(),
            Self::B(b) => b.segment_alignment_offset(),
        }
    }

    fn expected_program_header_size(self) -> usize {
        match self {
            Self::A(a) => a.expected_program_header_size(),
            Self::B(b) => b.expected_program_header_size(),
        }
    }
}

impl<A: ClassParse, B: ClassParse> ClassParseDynamic for Merge<A, B>
where
    B::ClassUsize: From<A::ClassUsize>,
    B::ClassIsize: From<A::ClassIsize>,
    A::ClassUsize: TryFrom<B::ClassUsize>,
{
    fn dynamic_tag_eq(
        tag: crate::dynamic::DynamicTag<Self>,
        const_tag: crate::dynamic::ConstDynamicTag,
    ) -> bool {
        B::dynamic_tag_eq(crate::dynamic::DynamicTag::<B>(tag.0), const_tag)
    }

    fn dynamic_tag_offset(self) -> usize {
        match self {
            Self::A(a) => a.dynamic_tag_offset(),
            Self::B(b) => b.dynamic_tag_offset(),
        }
    }

    fn dynamic_value_offset(self) -> usize {
        match self {
            Self::A(a) => a.dynamic_value_offset(),
            Self::B(b) => b.dynamic_value_offset(),
        }
    }

    fn expected_dynamic_size(self) -> usize {
        match self {
            Self::A(a) => a.expected_dynamic_size(),
            Self::B(b) => b.expected_dynamic_size(),
        }
    }
}

impl<A: ClassParse, B: ClassParse> ClassParseRelocation for Merge<A, B>
where
    B::ClassUsize: From<A::ClassUsize>,
    B::ClassIsize: From<A::ClassIsize>,
    A::ClassUsize: TryFrom<B::ClassUsize>,
{
    fn relocation_type_raw(self, info: Self::ClassUsize) -> u32 {
        match self {
            Self::A(a) => {
                a.relocation_type_raw(A::ClassUsize::try_from(info).map_err(|_| ()).unwrap())
            }
            Self::B(b) => b.relocation_type_raw(info),
        }
    }

    fn symbol_raw(self, info: Self::ClassUsize) -> u32 {
        match self {
            Self::A(a) => a.symbol_raw(A::ClassUsize::try_from(info).map_err(|_| ()).unwrap()),
            Self::B(b) => b.symbol_raw(info),
        }
    }

    fn rel_offset_offset(self) -> usize {
        match self {
            Self::A(a) => a.rel_offset_offset(),
            Self::B(b) => b.rel_offset_offset(),
        }
    }

    fn rel_info_offset(self) -> usize {
        match self {
            Self::A(a) => a.rel_info_offset(),
            Self::B(b) => b.rel_info_offset(),
        }
    }

    fn rela_offset_offset(self) -> usize {
        match self {
            Self::A(a) => a.rela_offset_offset(),
            Self::B(b) => b.rela_offset_offset(),
        }
    }

    fn rela_info_offset(self) -> usize {
        match self {
            Self::A(a) => a.rela_info_offset(),
            Self::B(b) => b.rela_info_offset(),
        }
    }

    fn rela_addend_offset(self) -> usize {
        match self {
            Self::A(a) => a.rela_addend_offset(),
            Self::B(b) => b.rela_addend_offset(),
        }
    }

    fn expected_rel_size(self) -> usize {
        match self {
            Self::A(a) => a.expected_rel_size(),
            Self::B(b) => b.expected_rel_size(),
        }
    }

    fn expected_rela_size(self) -> usize {
        match self {
            Self::A(a) => a.expected_rela_size(),
            Self::B(b) => b.expected_rela_size(),
        }
    }
}

impl<A: ClassParse, B: ClassParse> ClassParseBase for Merge<A, B>
where
    B::ClassUsize: From<A::ClassUsize>,
    B::ClassIsize: From<A::ClassIsize>,
    A::ClassUsize: TryFrom<B::ClassUsize>,
{
    type ClassUsize = B::ClassUsize;
    type ClassIsize = B::ClassIsize;

    fn from_elf_class(class: crate::ident::Class) -> Result<Self, super::UnsupportedClassError> {
        if let Ok(a) = A::from_elf_class(class) {
            return Ok(Self::A(a));
        }

        B::from_elf_class(class).map(Self::B)
    }

    fn parse_class_usize_at<E: crate::encoding::EncodingParse>(
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

    fn parse_class_isize_at<E: crate::encoding::EncodingParse>(
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
