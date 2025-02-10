//! Graphics interfaces for `tvm_loader` crates.
//!
//! Provides interfaces and implementations intended to help `tvm_loader` crates carry out
//! graphical output.

pub mod console;
pub mod surface;

pub mod font {
    //! Defines the graphical font interfaces for `tvm_loader`.

    use core::mem;
    use font_map::{FontMap, FontMapEntry};
    use glyph::GlyphArray;

    /// The default font's [`GlyphArray`].
    pub const GLYPH_ARRAY: GlyphArray = {
        const GLYPHS: &[u8] = include_bytes!(concat!(
            "../../../../",
            env!(
                "TVM_LOADER_GLYPH_ARRAY",
                "TVM_LOADER_GLYPH_ARRAY not set when `graphics` feature is enabled"
            )
        ));

        match GlyphArray::from_dump(GLYPHS) {
            Some(glyph_array) => glyph_array,
            None => panic!("TVM_LOADER_GLYPH_ARRAY does not point to a valid dump"),
        }
    };

    /// The default font's [`FontMap`].
    pub const FONT_MAP: FontMap = {
        const FONT_MAP_BYTES: &Aligned<FontMapEntry, [u8]> = &Aligned {
            aligner: [],
            value: *include_bytes!(concat!(
                "../../../../",
                env!(
                    "TVM_LOADER_FONT_MAP",
                    "TVM_LOADER_FONT_MAP not set when `graphics` feature is enabled"
                )
            )),
        };

        #[repr(C)]
        struct Aligned<A, T: ?Sized> {
            aligner: [A; 0],
            value: T,
        }

        let font_map_entries = unsafe {
            core::slice::from_raw_parts(
                FONT_MAP_BYTES.value.as_ptr().cast::<FontMapEntry>(),
                FONT_MAP_BYTES.value.len() / mem::size_of::<FontMapEntry>(),
            )
        };

        FontMap::new(font_map_entries)
    };

    pub use font::*;
}
