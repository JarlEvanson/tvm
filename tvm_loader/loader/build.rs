//! Build script for `tvm_loader`.

use std::env;

/// The path to the default `GlyphArray` file.
const DEFAULT_GLYPH_ARRAY: &str = "tvm_loader/assets/fonts/Tamsyn8x16r/glyph_array.bin";
/// The path to the default `FontMap` file.
const DEFAULT_FONT_MAP: &str = "tvm_loader/assets/fonts/Tamsyn8x16r/font_map.bin";

fn main() {
    println!("cargo::rerun-if-changed=src");
    println!("cargo::rerun-if-changed=Cargo.toml");

    if env::var_os("CARGO_FEATURE_GRAPHICS").is_some() {
        let glyph_array_set = env::var_os("TVM_LOADER_GLYPH_ARRAY").is_some();
        let font_map_set = env::var_os("TVM_LOADER_FONT_MAP").is_some();

        if !glyph_array_set && !font_map_set {
            println!("cargo::rustc-env=TVM_LOADER_GLYPH_ARRAY={DEFAULT_GLYPH_ARRAY}");
            println!("cargo::rustc-env=TVM_LOADER_FONT_MAP={DEFAULT_FONT_MAP}");
        }
    }
}
