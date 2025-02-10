//! Tool for converting various bitmap font formats into the custom [`GlyphArray`] and
//! [`FontMapBuilder`] dump formats.

use std::{env::args, fs::File, path::Path};

use font::{font_map::FontMapBuilder, glyph::GlyphArray};
use simple_psf::Psf;

fn main() {
    let font_path = args().nth(1).expect("expected font path");
    let dir_path = args().nth(2).expect("expected output directory path");

    let font = std::fs::read(font_path).expect("error reading provided font");

    convert_psf(&font, dir_path.as_ref());
}

fn convert_psf(font: &[u8], dir: &Path) {
    let font = Psf::parse(font).unwrap();

    let single_char_mappings = font
        .iter_unicode_entries()
        .unwrap()
        .map(|(_, str)| match str {
            Ok(str) if str.chars().count() == 1 => 1,
            _ => 0,
        })
        .sum::<usize>();

    let glyph_array = GlyphArray::new(font.glyphs, font.glyph_width as u8, font.glyph_height as u8);
    let mut font_map = FontMapBuilder::new((single_char_mappings * 3) / 2);
    for (glyph_index, string) in font.iter_unicode_entries().unwrap() {
        let Ok(string) = string else {
            continue;
        };

        if string.chars().count() == 1 {
            font_map
                .insert(string.chars().next().unwrap(), glyph_index as u32)
                .unwrap();
        }
    }

    let mut glyph_array_file = File::create(dir.join("glyph_array.bin")).unwrap();
    glyph_array.dump(&mut glyph_array_file);

    let mut font_map_file = File::create(dir.join("font_map.bin")).unwrap();
    font_map.dump(&mut font_map_file, true);
}
