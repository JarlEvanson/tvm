//! Implementation of 64-bit ELF file parsing.

use crate::elf::{
    class::{ClassParse, ClassParseBase, UnsupportedClassError},
    encoding::EncodingParse,
    ident::Class,
};

/// A zero-sized object offering methods to safely parse 64-bit ELF files.
#[derive(Clone, Copy, Debug, Hash, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Class64;

impl ClassParse for Class64 {}

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
