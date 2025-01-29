//! Definitions for ELF dynamic structures.

use crate::{
    class::{ClassParse, ClassParseBase},
    encoding::EncodingParse,
};

/// An ELF dynamic structure.
pub struct Dynamic<C: ClassParse> {
    /// Determinant of how to interpret the [`Dynamic::val`].
    pub tag: DynamicTag<C>,
    /// Value associated with the [`Dynamic`] structure.
    pub val: C::ClassUsize,
}

/// An ELF dynamic tag.
///
/// This identifies the purpose of a [`Dynamic`] structure.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DynamicTag<C: ClassParseDynamic>(pub C::ClassIsize);

impl<C: ClassParseDynamic> PartialEq<ConstDynamicTag> for DynamicTag<C> {
    fn eq(&self, other: &ConstDynamicTag) -> bool {
        C::dynamic_tag_eq(*self, *other)
    }
}

/// Wrapper struct to work around const traits not being available.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConstDynamicTag(pub(crate) i32);

impl ConstDynamicTag {
    /// Marks the end of the ELF dynamic array.
    pub const NULL: Self = Self(0);
    /// Holds the string table offset of a null-terminated string which is the name of a needed
    /// library. This offset is an index into the table recording in the
    /// [`ConstDynamicTag::STRING_TABLE`] entry.
    ///
    /// The dynamic array may contain multiple entries with this type, and the order of these
    /// entries is significant, but only relative to entries of the same type.
    pub const NEEDED: Self = Self(1);
    /// Holds the total size, in bytes, of the relocation entries associated with the procedure
    /// linkage table. If an [`ConstDynamicTag::JMP_REL`] entry is present, this tag must accompany
    /// it.
    pub const PLT_REL_SIZE: Self = Self(2);
    /// Holds an address associated with the procedure linkage table and/or the global offset
    /// table.
    pub const PLT_GOT: Self = Self(3);
    /// Holds the address of the symbol hash table, which refers to the symbol table referenced in
    /// an [`ConstDynamicTag::SYMBOL_TABLE`] entry.
    pub const HASH: Self = Self(4);
    /// Holds the address of the string table.
    pub const STRING_TABLE: Self = Self(5);
    /// Holds the address of the symbol table.
    pub const SYMBOL_TABLE: Self = Self(6);
    /// Holds the address of a relocation table, with explicit addends.
    ///
    /// If this entry is present, the dynamic array must also have [`ConstDynamicTag::RELA_SIZE`] and
    /// [`ConstDynamicTag::RELA_ENTRY_SIZE`] entries.
    pub const RELA_TABLE: Self = Self(7);
    /// Holds the total size, in bytes, of the relocation table pointed to by the [`ConstDynamicTag::RELA_TABLE`] .
    pub const RELA_SIZE: Self = Self(8);
    /// Holds the size, in bytes, of the relocation table pointed to by the
    /// [`ConstDynamicTag::RELA_TABLE`].
    pub const RELA_ENTRY_SIZE: Self = Self(9);
    /// Holds the total size, in bytes, of the string table pointed to by the
    /// [`ConstDynamicTag::STRING_TABLE`] entry.
    pub const STRING_TABLE_SIZE: Self = Self(10);
    /// Holds the size, in bytes, of an entry in the symbol table pointed to by the
    /// [`ConstDynamicTag::SYMBOL_TABLE`] entry.
    pub const SYMBOL_ENTRY_SIZE: Self = Self(11);
    /// Holds the address of the initialization function.
    pub const INIT: Self = Self(12);
    /// Holds the address of the termination function.
    pub const FINI: Self = Self(13);
    /// Holds the string table offset of a null-terminated string giving the name of the shared
    /// object.
    pub const SO_NAME: Self = Self(14);
    /// Holds the string table offset of a null-terminated string giving the library search path
    /// string.
    ///
    /// The use of this has been superseded by [`ConstDynamicTag::RUNPATH`].
    pub const RPATH: Self = Self(15);
    /// Indicates that the dynamic linker's symbol resolution algorithm should start from the
    /// shared object and then if the shared object fails to provided the referenced symbol, then
    /// the linker searches the executable file and other shared objects as usual.
    pub const SYMBOLIC: Self = Self(16);
    /// Holds the address of a relocation table, with implicit addends.
    ///
    /// If this entry is present, the dynamic array must also have [`ConstDynamicTag::REL_SIZE`] and
    /// [`ConstDynamicTag::RELA_ENTRY_SIZE`] entries.
    pub const REL_TABLE: Self = Self(17);
    /// The total size, in bytes, of the relocation table pointed to be the
    /// [`ConstDynamicTag::RELA_TABLE`] entry.
    pub const REL_SIZE: Self = Self(18);
    /// The size, in bytes, of an entry in the relocation table pointed to be the
    /// [`ConstDynamicTag::RELA_TABLE`] entry.
    pub const REL_ENTRY_SIZE: Self = Self(19);
    /// The type of relocation entry to which the prodedure linkage table refers.
    pub const PLT_REL: Self = Self(20);
    /// This member is used for debugging, but its contents are not specified by the ABI.
    pub const DEBUG: Self = Self(21);
    /// Indicates that one or more relocation entries might cause a modification to a non-writable segment.
    ///
    /// The use of this has been superseded by [`ConstDynamicTag::TEXT_REL`].
    pub const TEXT_REL: Self = Self(22);
    /// Holds the address of relocation entries associated solely with the procedure linkage table.
    ///
    /// If this entry is present, the dynamic array must also have [`ConstDynamicTag::PLT_REL`] and
    /// [`ConstDynamicTag::PLT_REL_SIZE`] entries.
    pub const JMP_REL: Self = Self(23);
    /// Indicates that the dynamic linker should process all relocations for the object containing
    /// this entry before transferring control to the program.
    pub const BIND_NOW: Self = Self(24);
    /// Holds the address of the array of pointers to initialization functions.
    pub const INIT_ARRAY: Self = Self(25);
    /// Holds the address of the array of pointers to termination functions.
    pub const FINI_ARRAY: Self = Self(26);
    /// Holds the size, in bytes, of the array of pointers to initialization functions.
    pub const INIT_ARRAY_SIZE: Self = Self(27);
    /// Holds the size, in bytes, of the array of pointers to termination functions.
    pub const FINI_ARRAY_SIZE: Self = Self(28);
    /// Holds the strin table offset of ta null-terminated library search path string.
    pub const RUNPATH: Self = Self(29);
    /// Holds flag values specfic to the object being loaded.
    pub const FLAGS: Self = Self(30);

    /// Holds the address of the array of pointers to pre-initialization functions.
    ///
    /// This is processed only in an executable file.
    pub const PREINIT_ARRAY: Self = Self(32);
    /// Holds the size, in bytes, of the array of pointers to pre-initialization functions.
    pub const PREINIT_ARRAY_SIZE: Self = Self(33);
    /// Holds the address of the SHT_SYMTAB_SHNDX section associated with the dynamic symbol
    /// table referenced by the [`ConstDynamicTag::SYMBOL_TABLE`] element.
    pub const SYMBOL_TABLE_SECTION_INDEX: Self = Self(34);
}

/// A table of [`Dynamic`] structures.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct DynamicTable<'slice, C, E> {
    /// The underlying bytes of this [`DynamicTable`].
    pub(crate) bytes: &'slice [u8],
    /// The number of [`Dynamic`] structures in this [`DynamicTable`].
    pub(crate) count: usize,
    /// The [`ClassParseDynamic`] of this [`DynamicTable`].
    pub(crate) class: C,
    /// The [`EncodingParse`] of this [`DynamicTable`].
    pub(crate) encoding: E,
}

impl<'slice, C: ClassParse, E: EncodingParse> DynamicTable<'slice, C, E> {
    /// Creates a new [`DynamicTable`] from the given `slice`.
    pub fn new(class: C, encoding: E, slice: &'slice [u8], count: usize) -> Option<Self> {
        if count
            .checked_add(class.expected_dynamic_size())
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

    /// Returns the [`Dynamic`] structure located at `index`.
    pub fn get(&self, index: usize) -> Option<Dynamic<C>> {
        if index >= self.count {
            return None;
        }

        let dynamic_bytes = &self.bytes[index * self.class.expected_dynamic_size()..];
        let dynamic = Dynamic {
            tag: DynamicTag(self.class.parse_class_isize_at(
                self.encoding,
                self.class.dynamic_tag_offset(),
                dynamic_bytes,
            )),
            val: self.class.parse_class_usize_at(
                self.encoding,
                self.class.dynamic_value_offset(),
                dynamic_bytes,
            ),
        };

        Some(dynamic)
    }

    /// Returns the number of [`Dynamic`] structures in this [`DynamicTable`].
    pub fn count(&self) -> usize {
        self.count
    }
}

impl<'slice, C: ClassParse, E: EncodingParse> IntoIterator for DynamicTable<'slice, C, E> {
    type Item = Dynamic<C>;
    type IntoIter = IntoIter<'slice, C, E>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            table: self,
            next: 0,
        }
    }
}

/// An [`Iterator`] over the [`Dynamic`] structures in a [`DynamicTable`].
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct IntoIter<'slice, C: ClassParse, E: EncodingParse> {
    /// The table to iterate over.
    table: DynamicTable<'slice, C, E>,
    /// The index in the [`DynamicTable`].
    next: usize,
}

impl<C: ClassParse, E: EncodingParse> Iterator for IntoIter<'_, C, E> {
    type Item = Dynamic<C>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.table.get(self.next)?;

        self.next += 1;
        Some(item)
    }
}

/// The requirements to implement class aware parsing of ELF dynamic structures.
pub trait ClassParseDynamic: ClassParseBase {
    /// Returns whether the given [`DynamicTag<Self>`] is equal to the given [`ConstDynamicTag`].
    fn dynamic_tag_eq(tag: DynamicTag<Self>, const_tag: ConstDynamicTag) -> bool;

    /// The offset of the tag of the ELF dynamic structure.
    fn dynamic_tag_offset(self) -> usize;
    /// The offset of the value of the ELF dynamic structure.
    fn dynamic_value_offset(self) -> usize;

    /// The expected size of an ELF dynamic structure.
    fn expected_dynamic_size(self) -> usize;
}
