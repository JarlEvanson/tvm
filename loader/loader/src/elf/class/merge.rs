//! Implementation of merged [`ClassParse`] implementations.

use crate::elf::{
    class::{ClassParse, ClassParseBase},
    encoding::EncodingParse,
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
