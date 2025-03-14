//! Interface for defining a mapping between characters and indices into a [`GlyphArray`][ga].
//!
//! [ga]: crate::glyph::GlyphArray

#[cfg(feature = "std")]
use std::{error, fmt, io::Write};

/// The [`u32`] value that indicates that the [`FontMapEntry`] is empty.
const EMPTY_VALUE: u32 = 0x110000;

/// A mapping between a [`char`] and indices into a [`GlyphArray`][ga].
///
/// [ga]: crate::glyph::GlyphArray
pub struct FontMap<'buffer> {
    /// The buffer that is the underlying storage behind the [`FontMap`].
    buffer: &'buffer [FontMapEntry],
}

impl<'buffer> FontMap<'buffer> {
    /// Creates a new [`FontMap`] using the `buffer`.
    pub const fn new(buffer: &'buffer [FontMapEntry]) -> Self {
        let mut i = 0;
        while i < buffer.len() {
            let entry = buffer[i];

            assert!(
                char::from_u32(entry.c).is_some() || entry.c == EMPTY_VALUE,
                "invalid character mapping found"
            );

            i += 1;
        }

        Self { buffer }
    }

    /// Returns the index of the glyph associated with `c`. If `c` is not in the [`FontMap`], then
    /// `None` is returned.
    pub fn get(&self, c: char) -> Option<u32> {
        for index in ProbeIter::new(c, self.buffer.len()) {
            let entry = self.buffer[index];
            if entry.c == c as u32 {
                return Some(entry.glyph_index);
            } else if entry.c == EMPTY_VALUE {
                return None;
            }
        }

        unreachable!()
    }
}

/// Builder for a valid [`FontMap`].
#[cfg(feature = "std")]
#[derive(Clone, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FontMapBuilder {
    /// The buffer that acts as the underlying mutable storage behind the [`FontMapBuilder`].
    buffer: Box<[FontMapEntry]>,
    /// The number of [`FontMapEntry`] currently inserted into the [`FontMapBuilder`].
    count: usize,
}

#[cfg(feature = "std")]
impl FontMapBuilder {
    /// Creates a new [`FontMapBuilder`] with `size` slots.
    pub fn new(size: usize) -> Self {
        Self {
            buffer: vec![
                FontMapEntry {
                    c: EMPTY_VALUE,
                    glyph_index: 0
                };
                size
            ]
            .into_boxed_slice(),
            count: 0,
        }
    }

    /// Inserts the provided [`char`] to `glyph_index` mapping. If the [`char`] already exists in
    /// the [`FontMap`], then the mapping is updated to point to `glyph_index`.
    ///
    /// # Errors
    ///
    /// Returns [`InsertionFailure`] when the [`FontMapBuilder`] is full.
    pub fn insert(&mut self, c: char, glyph_index: u32) -> Result<(), InsertionFailure> {
        if self.count == self.buffer.len() {
            return Err(InsertionFailure);
        }

        self.count += 1;
        for index in ProbeIter::new(c, self.buffer.len()) {
            if self.buffer[index].c == EMPTY_VALUE {
                self.buffer[index] = FontMapEntry {
                    c: c as u32,
                    glyph_index,
                };
                return Ok(());
            }
        }

        unreachable!()
    }

    /// Returns the underlying [`FontMap`].
    pub fn font_map(&self) -> FontMap {
        FontMap::new(self.buffer.as_ref())
    }

    /// Dumps the built [`FontMap`] into the `writer`.
    ///
    /// # Panics
    ///
    /// Panics if dumping the built [`FontMap`] into the `writer` fails.
    pub fn dump<W: Write>(&self, mut writer: W, little_endian: bool) {
        if little_endian {
            for entry in self.buffer.as_ref() {
                writer.write_all(&entry.c.to_le_bytes()).unwrap();
                writer.write_all(&entry.glyph_index.to_le_bytes()).unwrap();
            }
        } else {
            for entry in self.buffer.as_ref() {
                writer.write_all(&entry.c.to_be_bytes()).unwrap();
                writer.write_all(&entry.glyph_index.to_be_bytes()).unwrap();
            }
        }
    }
}

/// An error that occurs when
#[cfg(feature = "std")]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct InsertionFailure;

#[cfg(feature = "std")]
impl fmt::Display for InsertionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

#[cfg(feature = "std")]
impl error::Error for InsertionFailure {}

/// An entry in a [`FontMap`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct FontMapEntry {
    /// The key of the [`FontMapEntry`].
    pub c: u32,
    /// The value of the [`FontMapEntry`].
    pub glyph_index: u32,
}

struct ProbeIter {
    /// The last index to be probed.
    index: usize,
    /// The number of probes made.
    try_count: usize,
    /// The bit mask to wrap the index correctly.
    index_mask: usize,
    /// The actual size of the underlying storage the [`FontMap`] utilizes.
    size: usize,
}

impl ProbeIter {
    /// Creates a new [`ProbeIter`], which returns the sequence of hash indices to attempt probing.
    fn new(c: char, size: usize) -> Self {
        let index = hash(c as u32) as usize;
        let index_mask = size
            .checked_next_power_of_two()
            .unwrap_or(0)
            .wrapping_sub(1);

        Self {
            index,
            try_count: 0,
            index_mask,
            size,
        }
    }
}

impl Iterator for ProbeIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.try_count = self.try_count.wrapping_add(1);
            self.index = self.index.wrapping_add(self.try_count) & self.index_mask;
            if self.index < self.size {
                return Some(self.index);
            }
        }
    }
}

/// A simple integer hash.
fn hash(mut value: u32) -> u32 {
    value = (value ^ 61) ^ (value >> 16);
    value = value.wrapping_add(value << 3);
    value = value ^ (value >> 4);
    value = value.wrapping_mul(0x27d4eb2d);
    value ^ (value >> 15)
}
