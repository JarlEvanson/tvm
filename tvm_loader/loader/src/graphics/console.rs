//! Graphical console implementation.
//!
//! Implements an output-only graphical console using a [`Surface`].

use core::fmt;

use crate::graphics::{
    font::{font_map::FontMap, glyph::GlyphArray},
    surface::{Point, Region, Surface},
};

/// Text-based graphical output device.
pub struct Console<'font, S: Surface> {
    /// The x-position in the character coordinate system.
    x: usize,
    /// The y-position in the character coordinate system.
    y: usize,

    /// The width of the [`Console`] in characters.
    text_width: usize,
    /// The height of the [`Console`] in characters.
    text_height: usize,

    /// The color of the foreground (text).
    foreground: u64,
    /// The color of the background.
    background: u64,

    /// The surface the console utilizes.
    surface: S,

    /// The [`GlyphArray`] used for the font this [`Console`] uses.
    glyph_array: GlyphArray<'font>,
    /// The [`FontMap`] used for the font this [`Console`] uses.
    font_map: FontMap<'font>,
}

impl<'font, S: Surface> Console<'font, S> {
    /// Creates a new [`Console`] that prints characters using the given [`GlyphArray`] and
    /// [`FontMap`] onto the given [`Surface`].
    pub fn new(
        surface: S,
        glyph_array: GlyphArray<'font>,
        font_map: FontMap<'font>,
        background: u64,
        foreground: u64,
    ) -> Self {
        let text_width = surface.width() / glyph_array.width() as usize;
        let text_height = surface.height() / glyph_array.height() as usize;

        Self {
            x: 0,
            y: 0,

            text_width,
            text_height,

            foreground,
            background,

            surface,

            glyph_array,
            font_map,
        }
    }

    /// Writes the given [`char`] to the [`Surface`].
    #[expect(
        clippy::missing_panics_doc,
        reason = "bounds checking is performed beforehand"
    )]
    pub fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.new_line(),
            '\r' => self.carriage_return(),
            c => {
                if self.x + 1 >= self.text_width {
                    self.new_line();
                }

                let Some(glyph_index) = self.font_map.get(c) else {
                    self.x += 1;
                    return;
                };
                let Some(glyph) = self.glyph_array.get(glyph_index as usize) else {
                    return;
                };

                let x_base = self.x * self.glyph_array.width() as usize;
                let y_base = self.y * self.glyph_array.height() as usize;

                for (y_offset, row) in glyph.into_iter().enumerate() {
                    for (x_offset, pixel_on) in row.into_iter().enumerate() {
                        let color = if pixel_on {
                            self.foreground
                        } else {
                            self.background
                        };

                        self.surface
                            .write_pixel(
                                Point {
                                    x: x_base + x_offset,
                                    y: y_base + y_offset,
                                },
                                color,
                            )
                            .unwrap();
                    }
                }
            }
        }
    }

    /// Scrolls the [`Console`] up by one line.
    fn scroll(&mut self) {
        let write_height = (self.text_height - 1) * self.glyph_array.height() as usize;

        let source_point = Point {
            x: 0,
            y: self.glyph_array.height() as usize,
        };

        let write_region = Region {
            point: Point { x: 0, y: 0 },
            width: self.surface.width(),
            height: write_height,
        };

        self.surface
            .copy_within(write_region, source_point)
            .unwrap();

        let fill_region = Region {
            point: Point {
                x: 0,
                y: write_height,
            },
            width: self.surface.width(),
            height: self.glyph_array.height() as usize,
        };
        self.surface.fill(fill_region, self.background).unwrap();
    }

    /// Moves the position to the start of the next line, scrolling if required.
    fn new_line(&mut self) {
        self.carriage_return();
        self.y += 1;

        if self.y + 1 >= self.text_height {
            self.y -= 1;
            self.scroll();
        }
    }

    /// Carries out the effects of a carriage return on the [`Console`].
    const fn carriage_return(&mut self) {
        self.x = 0;
    }
}

impl<S: Surface> fmt::Write for Console<'_, S> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }

        Ok(())
    }
}
