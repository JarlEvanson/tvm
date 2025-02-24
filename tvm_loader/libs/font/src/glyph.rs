//! Interface for interacting with glyphs.

#[cfg(feature = "std")]
use std::io::Write;

/// An array of [`Glyph`]s.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlyphArray<'buffer> {
    /// The width of a [`Glyph`] in pixels.
    width: u8,
    /// The height of a [`Glyph`] in pixels.
    height: u8,
    /// The underlying storage holding the array of [`Glyph`]s.
    buffer: &'buffer [u8],
}

impl<'buffer> GlyphArray<'buffer> {
    /// Creates a new [`GlyphArray`].
    pub const fn new(buffer: &'buffer [u8], width: u8, height: u8) -> Self {
        Self {
            width,
            height,
            buffer,
        }
    }

    /// Creates a new [`GlyphArray`] from a dumped blob.
    #[expect(
        clippy::missing_panics_doc,
        reason = "slice is checked for the proper length before unwrapping"
    )]
    pub const fn from_dump(dump: &'buffer [u8]) -> Option<Self> {
        if dump.len() < 3 {
            return None;
        }

        let glyph_array = Self {
            width: dump[0],
            height: dump[1],
            buffer: dump.split_first().unwrap().1.split_first().unwrap().1,
        };
        Some(glyph_array)
    }

    /// Returns the [`Glyph`] at `index` or `None` if out of bounds.
    pub fn get(&self, index: usize) -> Option<Glyph<'buffer>> {
        if index >= self.glyph_count() {
            return None;
        }

        let row_byte_count = self.width.div_ceil(8) as usize;
        let glyph_byte_count = row_byte_count * self.height as usize;

        let glyph = Glyph {
            width: self.width,
            height: self.height,
            buffer: &self.buffer[index * glyph_byte_count..],
        };
        Some(glyph)
    }

    /// Returns the width of a [`Glyph`] in pixels.
    pub const fn width(&self) -> u8 {
        self.width
    }

    /// Returns the height of a [`Glyph`] in pixels.
    pub const fn height(&self) -> u8 {
        self.height
    }

    /// Returns the number of [`Glyph`]s in this [`GlyphArray`].
    pub const fn glyph_count(&self) -> usize {
        let row_byte_count = self.width.div_ceil(8) as usize;
        let glyph_byte_count = row_byte_count * self.height as usize;

        self.buffer.len() / glyph_byte_count
    }

    /// Dumps the [`GlyphArray`] into the `writer`.
    ///
    /// # Panics
    ///
    /// Panics if dumping the built [`GlyphArray`] into the `writer` fails.
    #[cfg(feature = "std")]
    pub fn dump<W: Write>(&self, mut writer: W) {
        writer.write_all(&[self.width]).unwrap();
        writer.write_all(&[self.height]).unwrap();
        writer.write_all(self.buffer).unwrap();
    }
}

/// Stores the on/off layout of a specific glyph in a font.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Glyph<'buffer> {
    /// The width of a [`Glyph`] in pixels.
    width: u8,
    /// The height of a [`Glyph`] in pixels.
    height: u8,
    /// The bytes holding the [`Glyph`].
    buffer: &'buffer [u8],
}

impl<'buffer> IntoIterator for Glyph<'buffer> {
    type IntoIter = GlyphRowsIter<'buffer>;
    type Item = GlyphRow<'buffer>;

    fn into_iter(self) -> Self::IntoIter {
        GlyphRowsIter {
            width: self.width,
            height: self.height,
            buffer: self.buffer,
            index: 0,
        }
    }
}

/// An [`Iterator`] over the rows of a [`Glyph`].
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlyphRowsIter<'buffer> {
    /// The width of a [`Glyph`] in pixels.
    width: u8,
    /// The height of a [`Glyph`] in pixels.
    height: u8,
    /// The bytes holding the [`Glyph`].
    buffer: &'buffer [u8],
    /// The index of the next [`GlyphRow`] to be returned.
    index: u8,
}

impl<'buffer> Iterator for GlyphRowsIter<'buffer> {
    type Item = GlyphRow<'buffer>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.height {
            return None;
        }

        let row_byte_count = self.width.div_ceil(8) as usize;
        let row_index = row_byte_count * self.index as usize;

        self.index += 1;
        let row = GlyphRow {
            width: self.width,
            buffer: &self.buffer[row_index..],
        };
        Some(row)
    }
}

/// A row in the [`Glyph`].
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlyphRow<'buffer> {
    /// The number of pixels in a [`Glyph`].
    width: u8,
    /// The bytes holding the [`Glyph`]'s row.
    buffer: &'buffer [u8],
}

impl<'buffer> IntoIterator for GlyphRow<'buffer> {
    type Item = bool;
    type IntoIter = GlyphRowIter<'buffer>;

    fn into_iter(self) -> Self::IntoIter {
        GlyphRowIter {
            width: self.width,
            buffer: self.buffer,
            index: 0,
        }
    }
}

/// An [`Iterator`] over the pixels in a [`GlyphRow`].
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GlyphRowIter<'buffer> {
    /// The width of a [`Glyph`] in pixels.
    width: u8,
    /// The bytes holding the [`Glyph`]'s row.
    buffer: &'buffer [u8],
    /// The index of next pixel to be returned.
    index: u8,
}

impl Iterator for GlyphRowIter<'_> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.width {
            return None;
        }

        let byte_index = (self.index / 8) as usize;
        let bit_index = (self.index % 8) as usize;
        let bit = (self.buffer[byte_index] >> bit_index) & 0b1;

        self.index += 1;
        Some(bit == 1)
    }
}
