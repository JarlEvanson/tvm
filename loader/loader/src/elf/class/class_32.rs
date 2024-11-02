//! Implementation of 32-bit ELF file parsing.

use crate::elf::{
    class::{ClassParse, ClassParseBase, UnsupportedClassError},
    encoding::EncodingParse,
    ident::Class,
};

/// A zero-sized object offering methods to safely parse 32-bit ELF files.
#[derive(Clone, Copy, Debug, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Class32;

impl ClassParse for Class32 {}

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
