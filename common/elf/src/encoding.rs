//! Implementation of encoding aware parsing.

use core::{error, fmt};

use crate::ident::Encoding;

/// An implementation of an encoding aware parser.
pub trait EncodingParse: Clone + Copy {
    /// Returns the [`EncodingParse`] instance that corresponds with the given [`Encoding`].
    ///
    /// # Errors
    ///
    /// Returns [`UnsupportedEncodingError`] if the [`Encoding`] is not supported by this
    /// [`EncodingParse`] type.
    fn from_elf_encoding(encoding: Encoding) -> Result<Self, UnsupportedEncodingError>;

    /// Returns the `u8` at `offset` bytes from the start of the slice.
    ///
    /// # Panics
    ///
    /// Panics if an arithmetic or bounds overflow error occurs.
    fn parse_u8_at(self, offset: usize, data: &[u8]) -> u8;

    /// Returns the `u16` at `offset` bytes from the start of the slice.
    ///
    /// # Panics
    ///
    /// Panics if an arithmetic or bounds overflow error occurs.
    fn parse_u16_at(self, offset: usize, data: &[u8]) -> u16;

    /// Returns the `u32` at `offset` bytes from the start of the slice.
    ///
    /// # Panics
    ///
    /// Panics if an arithmetic or bounds overflow error occurs.
    fn parse_u32_at(self, offset: usize, data: &[u8]) -> u32;

    /// Returns the `64` at `offset` bytes from the start of the slice.
    ///
    /// # Panics
    ///
    /// Panics if an arithmetic or bounds overflow error occurs.
    fn parse_u64_at(self, offset: usize, data: &[u8]) -> u64;

    /// Returns the `i32` at `offset` bytes from the start of the slice.
    ///
    /// # Panics
    ///
    /// Panics if an arithmetic or bounds overflow error occurs.
    fn parse_i32_at(self, offset: usize, data: &[u8]) -> i32;

    /// Returns the `i64` at `offset` bytes from the start of the slice.
    ///
    /// # Panics
    ///
    /// Panics if an arithmetic or bounds overflow error occurs.
    fn parse_i64_at(self, offset: usize, data: &[u8]) -> i64;
}

/// An error that occurs when the code does not support a particular [`Encoding`]
/// object.
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnsupportedEncodingError(Encoding);

impl fmt::Display for UnsupportedEncodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Encoding::NONE => write!(f, "no data encoding ELF parsing not supported"),
            Encoding::LSB2 => write!(f, "two's complement little-endian parsing not supported"),
            Encoding::MSB2 => write!(f, "two's complement big-endian parsing not supported"),
            Encoding(encoding) => write!(f, "unknown data encoding({encoding}) not supported"),
        }
    }
}

impl error::Error for UnsupportedEncodingError {}

/// Generates parsing functions for various encodings.
macro_rules! setup_func {
    ($kind:ident, $func:ident, $convert:ident) => {
        fn $func(self, offset: usize, data: &[u8]) -> $kind {
            let byte_after = offset
                .checked_add(core::mem::size_of::<$kind>())
                .expect("`offset + size` overflowed");
            if byte_after > data.len() {
                if core::mem::size_of::<$kind>() != 1 {
                    panic!(
                        "attempted read of {} bytes at an offset of {} bytes from {} byte buffer",
                        core::mem::size_of::<$kind>(),
                        offset,
                        data.len(),
                    )
                } else {
                    panic!(
                        "attempted read of 1 byte at an offset of {} bytes from {} byte buffer",
                        offset,
                        data.len(),
                    )
                }
            }

            let data = *data[offset..]
                .first_chunk::<{ core::mem::size_of::<$kind>() }>()
                .expect("broken sizing check");
            $kind::$convert(data)
        }
    };
}

/// An object offering methods for safe parsing of unaligned big or little endian integers.
pub type AnyEndian = Merge<LittleEndian, BigEndian>;

/// A zero-sized object offering methods for safe parsing of unaligned little-endian integer
/// parsing.
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LittleEndian;

impl EncodingParse for LittleEndian {
    fn from_elf_encoding(encoding: Encoding) -> Result<Self, UnsupportedEncodingError> {
        if encoding != Encoding::LSB2 {
            return Err(UnsupportedEncodingError(encoding));
        }

        Ok(Self)
    }

    setup_func!(u8, parse_u8_at, from_le_bytes);
    setup_func!(u16, parse_u16_at, from_le_bytes);
    setup_func!(u32, parse_u32_at, from_le_bytes);
    setup_func!(u64, parse_u64_at, from_le_bytes);
    setup_func!(i32, parse_i32_at, from_le_bytes);
    setup_func!(i64, parse_i64_at, from_le_bytes);
}

/// A zero-sized object offering methods for safe parsing of unaligned big-endian integer
/// parsing.
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BigEndian;

impl EncodingParse for BigEndian {
    fn from_elf_encoding(encoding: Encoding) -> Result<Self, UnsupportedEncodingError> {
        if encoding != Encoding::MSB2 {
            return Err(UnsupportedEncodingError(encoding));
        }

        Ok(Self)
    }

    setup_func!(u8, parse_u8_at, from_be_bytes);
    setup_func!(u16, parse_u16_at, from_be_bytes);
    setup_func!(u32, parse_u32_at, from_be_bytes);
    setup_func!(u64, parse_u64_at, from_be_bytes);
    setup_func!(i32, parse_i32_at, from_be_bytes);
    setup_func!(i64, parse_i64_at, from_be_bytes);
}

/// An object used to dispatch the [`EncodingParse`] to the two underlying [`EncodingParse`]
/// implementations.
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Merge<A: EncodingParse, B: EncodingParse> {
    /// The first [`EncodingParse`] implementation.
    A(A),
    /// The second [`EncodingParse`] implementation.
    B(B),
}

impl<A: EncodingParse, B: EncodingParse> EncodingParse for Merge<A, B> {
    fn from_elf_encoding(encoding: Encoding) -> Result<Self, UnsupportedEncodingError> {
        if let Ok(a) = A::from_elf_encoding(encoding) {
            return Ok(Self::A(a));
        }

        B::from_elf_encoding(encoding).map(Self::B)
    }

    fn parse_u8_at(self, offset: usize, data: &[u8]) -> u8 {
        match self {
            Self::A(a) => a.parse_u8_at(offset, data),
            Self::B(b) => b.parse_u8_at(offset, data),
        }
    }

    fn parse_u16_at(self, offset: usize, data: &[u8]) -> u16 {
        match self {
            Self::A(a) => a.parse_u16_at(offset, data),
            Self::B(b) => b.parse_u16_at(offset, data),
        }
    }

    fn parse_u32_at(self, offset: usize, data: &[u8]) -> u32 {
        match self {
            Self::A(a) => a.parse_u32_at(offset, data),
            Self::B(b) => b.parse_u32_at(offset, data),
        }
    }

    fn parse_u64_at(self, offset: usize, data: &[u8]) -> u64 {
        match self {
            Self::A(a) => a.parse_u64_at(offset, data),
            Self::B(b) => b.parse_u64_at(offset, data),
        }
    }

    fn parse_i32_at(self, offset: usize, data: &[u8]) -> i32 {
        match self {
            Self::A(a) => a.parse_i32_at(offset, data),
            Self::B(b) => b.parse_i32_at(offset, data),
        }
    }

    fn parse_i64_at(self, offset: usize, data: &[u8]) -> i64 {
        match self {
            Self::A(a) => a.parse_i64_at(offset, data),
            Self::B(b) => b.parse_i64_at(offset, data),
        }
    }
}
