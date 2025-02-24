//! Definitions for ELF relocation entries.

use crate::{
    class::{ClassParse, ClassParseBase},
    encoding::EncodingParse,
};

/// An ELF relocation entry without an explicit addend.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rel<C: ClassParse> {
    /// The offset at which to apply the relocation.
    pub offset: C::ClassUsize,
    /// Information necessary to apply the relocation.
    pub info: C::ClassUsize,
}

/// An ELF relocation entry with an explicit addend.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rela<C: ClassParse> {
    /// The offset at which to apply the relocation.
    pub offset: C::ClassUsize,
    /// Information necessary to apply the relocation.
    pub info: C::ClassUsize,
    /// The explicit addend to be used when computing the relocation.
    pub addend: C::ClassIsize,
}

/// A table of [`Rel`] entries.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct RelTable<'slice, C, E> {
    /// The underlying bytes of this [`RelTable`].
    pub(crate) bytes: &'slice [u8],
    /// The number of [`Rel`] entries in this [`RelTable`].
    pub(crate) count: usize,
    /// The [`ClassParseRelocation`] of this [`RelTable`].
    pub(crate) class: C,
    /// The [`EncodingParse`] of this [`RelTable`].
    pub(crate) encoding: E,
}

impl<'slice, C: ClassParse, E: EncodingParse> RelTable<'slice, C, E> {
    /// Creates a new [`RelTable`] from the given `slice`.
    pub fn new(class: C, encoding: E, slice: &'slice [u8], count: usize) -> Option<Self> {
        if count
            .checked_add(class.expected_rel_size())
            .is_none_or(|total_size| slice.len() < total_size)
        {
            return None;
        }

        let table = Self {
            bytes: slice,
            count,
            class,
            encoding,
        };

        Some(table)
    }

    /// Returns the [`Rel`] entry located at `index`.
    pub fn get(&self, index: usize) -> Option<Rel<C>> {
        if index >= self.count {
            return None;
        }

        let rel_bytes = &self.bytes[index * self.class.expected_rel_size()..];
        let rel = Rel {
            offset: self.class.parse_class_usize_at(
                self.encoding,
                self.class.rel_offset_offset(),
                rel_bytes,
            ),
            info: self.class.parse_class_usize_at(
                self.encoding,
                self.class.rel_info_offset(),
                rel_bytes,
            ),
        };

        Some(rel)
    }

    /// Returns the number of [`Rel`] entries in this [`RelTable`].
    pub fn count(&self) -> usize {
        self.count
    }
}

impl<'slice, C: ClassParse, E: EncodingParse> IntoIterator for RelTable<'slice, C, E> {
    type Item = Rel<C>;
    type IntoIter = RelIntoIter<'slice, C, E>;

    fn into_iter(self) -> Self::IntoIter {
        RelIntoIter {
            table: self,
            next: 0,
        }
    }
}

/// An [`Iterator`] over the [`Rel`] structures in a [`RelTable`].
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct RelIntoIter<'slice, C: ClassParse, E: EncodingParse> {
    /// The table to iterate over.
    table: RelTable<'slice, C, E>,
    /// The index in the [`RelTable`].
    next: usize,
}

impl<C: ClassParse, E: EncodingParse> Iterator for RelIntoIter<'_, C, E> {
    type Item = Rel<C>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.table.get(self.next)?;

        self.next += 1;
        Some(item)
    }
}

/// A table of [`Rela`] entries.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct RelaTable<'slice, C, E> {
    /// The underlying bytes of this [`RelaTable`].
    pub(crate) bytes: &'slice [u8],
    /// The number of [`Rela`] entries in this [`RelTable`].
    pub(crate) count: usize,
    /// The [`ClassParseRelocation`] of this [`RelTable`].
    pub(crate) class: C,
    /// The [`EncodingParse`] of this [`RelaTable`].
    pub(crate) encoding: E,
}

impl<'slice, C: ClassParse, E: EncodingParse> RelaTable<'slice, C, E> {
    /// Creates a new [`RelaTable`] from the given `slice`.
    pub fn new(class: C, encoding: E, slice: &'slice [u8], count: usize) -> Option<Self> {
        if count
            .checked_add(class.expected_rel_size())
            .is_none_or(|total_size| slice.len() < total_size)
        {
            return None;
        }

        let table = Self {
            bytes: slice,
            count,
            class,
            encoding,
        };

        Some(table)
    }

    /// Returns the [`Rela`] entry located at `index`.
    pub fn get(&self, index: usize) -> Option<Rela<C>> {
        if index >= self.count {
            return None;
        }

        let rela_bytes = &self.bytes[index * self.class.expected_rel_size()..];
        let rela = Rela {
            offset: self.class.parse_class_usize_at(
                self.encoding,
                self.class.rela_offset_offset(),
                rela_bytes,
            ),
            info: self.class.parse_class_usize_at(
                self.encoding,
                self.class.rela_info_offset(),
                rela_bytes,
            ),
            addend: self.class.parse_class_isize_at(
                self.encoding,
                self.class.rela_addend_offset(),
                rela_bytes,
            ),
        };

        Some(rela)
    }

    /// Returns the number of [`Rela`] entries in this [`RelTable`].
    pub fn count(&self) -> usize {
        self.count
    }
}

impl<'slice, C: ClassParse, E: EncodingParse> IntoIterator for RelaTable<'slice, C, E> {
    type Item = Rela<C>;
    type IntoIter = RelaIntoIter<'slice, C, E>;

    fn into_iter(self) -> Self::IntoIter {
        RelaIntoIter {
            table: self,
            next: 0,
        }
    }
}

/// An [`Iterator`] over the [`Rela`] structures in a [`RelTable`].
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct RelaIntoIter<'slice, C: ClassParse, E: EncodingParse> {
    /// The table to iterate over.
    table: RelaTable<'slice, C, E>,
    /// The index in the [`RelaTable`].
    next: usize,
}

impl<C: ClassParse, E: EncodingParse> Iterator for RelaIntoIter<'_, C, E> {
    type Item = Rela<C>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.table.get(self.next)?;

        self.next += 1;
        Some(item)
    }
}

/// The requirements to implement class aware parsing of ELF relocation entries.
pub trait ClassParseRelocation: ClassParseBase {
    /// Returns the relocation type extracted from `info`.
    fn relocation_type_raw(self, info: Self::ClassUsize) -> u32;
    /// Returns the symbol index extracted from `info`.
    fn symbol_raw(self, info: Self::ClassUsize) -> u32;

    /// The offset of the location at which to apply the relocation.
    fn rel_offset_offset(self) -> usize;
    /// The offset of the information required to apply the relocation.
    fn rel_info_offset(self) -> usize;

    /// The offset of the location at which to apply the relocation.
    fn rela_offset_offset(self) -> usize;
    /// The offset of the information required to apply the relocation.
    fn rela_info_offset(self) -> usize;
    /// The offset of the relocation's addend.
    fn rela_addend_offset(self) -> usize;

    /// The expected size of an ELF rel entry.
    fn expected_rel_size(self) -> usize;
    /// The expected size of an ELF rela entry.
    fn expected_rela_size(self) -> usize;
}
