//! Graphical surface interface for `tvm_loader` crates.
//!
//! The interface is defined in this module and implemented by a platform or system specific crate.

use core::{error, fmt};

/// [`Surface`] defines the basic interface all graphical output devices used by `tvm_loader`
/// should support.
///
/// # Safety
///
/// The values of [`Surface::width()`] and [`Surface::height()`] must not change asynchronously.
///
/// Any [`Surface::read_pixel_unchecked()`] and [`Surface::write_pixel_unchecked()`] that is within
/// the bounds defined by the rectangle defined by [`Surface::width()`] and [`Surface::height()`].
pub unsafe trait Surface {
    /// The with of the [`Surface`] in pixels.
    fn width(&self) -> usize;

    /// The height of the [`Surface`] in pixels.
    fn height(&self) -> usize;

    /// The [`Format`] of a pixel in this [`Surface`].
    fn format(&self) -> Format;

    /// Writes `value` to the pixel at `point`.
    ///
    /// # Safety
    ///
    /// `point` is in the bounds of this [`Surface`]`.
    /// `value` is valid according to [`Surface::format()`].
    unsafe fn write_pixel_unchecked(&mut self, point: Point, value: u64);

    /// Reads the value of the pixel at `point`.
    ///
    /// # Safety
    ///
    /// `point` is in the bounds of this [`Surface`]`.
    unsafe fn read_pixel_unchecked(&self, point: Point) -> u64;

    /// Writes `value` to the pixel at `point`.
    ///
    /// # Errors
    ///
    /// - [`WriteError::OutOfBounds`]: Returned when the specified `point` is out of bounds.
    /// - [`WriteError::InvalidPixelValue`]: Returned when the specified pixel `value` is invalid
    ///   according to [`Surface::format()`].
    fn write_pixel(&mut self, point: Point, value: u64) -> Result<(), WriteError> {
        if !point_in_bounds(point, self.width(), self.height()) {
            return Err(WriteError::OutOfBounds);
        }

        if value & self.format().rgb_mask() != value {
            return Err(WriteError::InvalidPixelValue);
        }

        // SAFETY:
        //
        // `point` was bounds checked before the write.
        unsafe { self.write_pixel_unchecked(point, value) }

        Ok(())
    }

    /// Reads the value of the pixel at `point`.
    ///
    /// # Errors
    ///
    /// [`OutOfBoundsError`]: Returned when the specified `point` is out of bounds.
    fn read_pixel(&self, point: Point) -> Result<u64, OutOfBoundsError> {
        if !point_in_bounds(point, self.width(), self.height()) {
            return Err(OutOfBoundsError);
        }

        // SAFETY:
        //
        // `point` was bounds checked before the read.
        let value = unsafe { self.read_pixel_unchecked(point) };

        Ok(value)
    }

    /// Fills the given `region` with the given pixel `value`.
    ///
    /// # Errors
    ///
    /// - [`WriteError::OutOfBounds`]: Returned when the specified `region` is out of bounds.
    /// - [`WriteError::InvalidPixelValue`]: Returned when the specified pixel `value` is invalid
    ///   according to [`Surface::format()`].
    fn fill(&mut self, region: Region, value: u64) -> Result<(), WriteError> {
        if !region_in_bounds(region, self.width(), self.height()) {
            return Err(WriteError::OutOfBounds);
        }

        if value & self.format().rgb_mask() != value {
            return Err(WriteError::InvalidPixelValue);
        }

        for y_offset in 0..region.height {
            for x_offset in 0..region.width {
                let point = Point {
                    x: region.point.x + x_offset,
                    y: region.point.y + y_offset,
                };

                // SAFETY:
                //
                // The write region was checked before the loop.
                unsafe { self.write_pixel_unchecked(point, value) }
            }
        }

        Ok(())
    }

    /// Writes data into the given `region` in the [`Surface`] from the given buffer.
    ///
    /// `buffer_stride` is the number of bytes between a row in the `buffer`.
    ///
    /// # Errors
    ///
    /// - [`OutOfBoundsError`]: Returned when the specified `region` is out of bounds or if
    ///   the buffer region is out of bounds.
    fn write_to(
        &mut self,
        region: Region,
        source: Point,
        buffer_stride: usize,
        buffer: &[u8],
    ) -> Result<(), OutOfBoundsError> {
        if !region_in_bounds(region, self.width(), self.height()) {
            return Err(OutOfBoundsError);
        }

        if buffer_stride == 0 {
            return Err(OutOfBoundsError);
        }

        let buffer_height = buffer.len() / buffer_stride;
        let buffer_region = Region {
            point: source,
            width: region.width,
            height: region.height,
        };
        if !region_in_bounds(buffer_region, buffer_stride, buffer_height) {
            return Err(OutOfBoundsError);
        }

        match self.format().bits_per_pixel() {
            8 => {
                let mut offset = source.x + source.y * buffer_stride;
                for y_offset in 0..region.height {
                    for x_offset in 0..region.width {
                        let value = buffer[offset] as u64;
                        let point = Point {
                            x: region.point.x + x_offset,
                            y: region.point.y + y_offset,
                        };

                        // SAFETY:
                        //
                        // The write region was checked before the loop.
                        unsafe { self.write_pixel_unchecked(point, value) }

                        offset += 1;
                    }

                    offset += buffer_stride - region.width;
                }
            }
            16 => {
                let mut offset = source.x * 2 + source.y * buffer_stride;
                for y_offset in 0..region.height {
                    for x_offset in 0..region.width {
                        let value = u16::from_ne_bytes([buffer[offset], buffer[offset + 1]]) as u64;
                        let point = Point {
                            x: region.point.x + x_offset,
                            y: region.point.y + y_offset,
                        };

                        // SAFETY:
                        //
                        // The write region was checked before the loop.
                        unsafe { self.write_pixel_unchecked(point, value) }

                        offset += 2;
                    }

                    offset += buffer_stride - region.width * 2;
                }
            }
            32 => {
                let mut offset = source.x * 4 + source.y * buffer_stride;
                for y_offset in 0..region.height {
                    for x_offset in 0..region.width {
                        let value = u32::from_ne_bytes([
                            buffer[offset],
                            buffer[offset + 1],
                            buffer[offset + 2],
                            buffer[offset + 3],
                        ]) as u64;
                        let point = Point {
                            x: region.point.x + x_offset,
                            y: region.point.y + y_offset,
                        };

                        // SAFETY:
                        //
                        // The write region was checked before the loop.
                        unsafe { self.write_pixel_unchecked(point, value) }

                        offset += 4;
                    }

                    offset += buffer_stride - region.width * 4;
                }
            }
            64 => {
                let mut offset = source.x * 8 + source.y * buffer_stride;
                for y_offset in 0..region.height {
                    for x_offset in 0..region.width {
                        let value = u64::from_ne_bytes([
                            buffer[offset],
                            buffer[offset + 1],
                            buffer[offset + 2],
                            buffer[offset + 3],
                            buffer[offset + 4],
                            buffer[offset + 5],
                            buffer[offset + 6],
                            buffer[offset + 7],
                        ]);
                        let point = Point {
                            x: region.point.x + x_offset,
                            y: region.point.y + y_offset,
                        };

                        // SAFETY:
                        //
                        // The write region was checked before the loop.
                        unsafe { self.write_pixel_unchecked(point, value) }

                        offset += 8;
                    }

                    offset += buffer_stride - region.width * 8;
                }
            }
            bpp if bpp <= 32 => todo!("support {bpp} bits per pixel"),
            _ => unreachable!(),
        }

        Ok(())
    }

    /// Reads data from the given `region` in the [`Surface`] into the given buffer.
    ///
    /// `buffer_stride` is the number of bytes between a row in the `buffer`.
    ///
    /// # Errors
    ///
    /// - [`OutOfBoundsError`]: Returned when the specified `region` is out of bounds or if
    ///   the buffer region is out of bounds.
    fn read_from(
        &self,
        region: Region,
        destination: Point,
        buffer_stride: usize,
        buffer: &mut [u8],
    ) -> Result<(), OutOfBoundsError> {
        if !region_in_bounds(region, self.width(), self.height()) {
            return Err(OutOfBoundsError);
        }

        if buffer_stride == 0 {
            return Err(OutOfBoundsError);
        }

        let buffer_height = buffer.len() / buffer_stride;
        let buffer_region = Region {
            point: destination,
            width: region.width,
            height: region.height,
        };
        if !region_in_bounds(buffer_region, buffer_stride, buffer_height) {
            return Err(OutOfBoundsError);
        }

        match self.format().bits_per_pixel() {
            8 => {
                let mut offset = destination.x + destination.y * buffer_stride;
                for y_offset in 0..region.height {
                    for x_offset in 0..region.width {
                        let point = Point {
                            x: region.point.x + x_offset,
                            y: region.point.y + y_offset,
                        };

                        // SAFETY:
                        //
                        // The read region was checked before the loop.
                        let value = unsafe { self.read_pixel_unchecked(point) } as u8;
                        buffer[offset] = value;

                        offset += 1;
                    }

                    offset += buffer_stride - region.width;
                }
            }
            16 => {
                let mut offset = destination.x * 2 + destination.y * buffer_stride;
                for y_offset in 0..region.height {
                    for x_offset in 0..region.width {
                        let point = Point {
                            x: region.point.x + x_offset,
                            y: region.point.y + y_offset,
                        };

                        // SAFETY:
                        //
                        // The read region was checked before the loop.
                        let value = unsafe { self.read_pixel_unchecked(point) } as u16;
                        buffer[offset] = value.to_ne_bytes()[0];
                        buffer[offset + 1] = value.to_ne_bytes()[1];

                        offset += 2;
                    }

                    offset += buffer_stride - region.width * 2;
                }
            }
            32 => {
                let mut offset = destination.x * 4 + destination.y * buffer_stride;
                for y_offset in 0..region.height {
                    for x_offset in 0..region.width {
                        let point = Point {
                            x: region.point.x + x_offset,
                            y: region.point.y + y_offset,
                        };

                        // SAFETY:
                        //
                        // The read region was checked before the loop.
                        let value = unsafe { self.read_pixel_unchecked(point) } as u32;
                        buffer[offset] = value.to_ne_bytes()[0];
                        buffer[offset + 1] = value.to_ne_bytes()[1];
                        buffer[offset + 2] = value.to_ne_bytes()[2];
                        buffer[offset + 3] = value.to_ne_bytes()[3];

                        offset += 4;
                    }

                    offset += buffer_stride - region.width * 4;
                }
            }
            64 => {
                let mut offset = destination.x * 8 + destination.y * buffer_stride;
                for y_offset in 0..region.height {
                    for x_offset in 0..region.width {
                        let point = Point {
                            x: region.point.x + x_offset,
                            y: region.point.y + y_offset,
                        };

                        // SAFETY:
                        //
                        // The read region was checked before the loop.
                        let value = unsafe { self.read_pixel_unchecked(point) };
                        buffer[offset] = value.to_ne_bytes()[0];
                        buffer[offset + 1] = value.to_ne_bytes()[1];
                        buffer[offset + 2] = value.to_ne_bytes()[2];
                        buffer[offset + 3] = value.to_ne_bytes()[3];
                        buffer[offset + 4] = value.to_ne_bytes()[4];
                        buffer[offset + 5] = value.to_ne_bytes()[5];
                        buffer[offset + 6] = value.to_ne_bytes()[6];
                        buffer[offset + 7] = value.to_ne_bytes()[7];

                        offset += 8;
                    }

                    offset += buffer_stride - region.width * 8;
                }
            }
            bpp if bpp <= 32 => todo!("support {bpp} bits per pixel"),
            _ => unreachable!(),
        }

        Ok(())
    }

    /// Copies the pixels from `source` to the given `write` region.
    ///
    /// # Errors
    ///
    /// [`OutOfBoundsError`] is returned whenever the given `write` region or the given `source`
    /// region is not within the bounds of this [`Surface`].
    fn copy_within(&mut self, write: Region, source: Point) -> Result<(), OutOfBoundsError> {
        if !region_in_bounds(write, self.width(), self.height()) {
            return Err(OutOfBoundsError);
        }

        let read_region = Region {
            point: source,
            width: write.width,
            height: write.height,
        };
        if !region_in_bounds(read_region, self.width(), self.height()) {
            return Err(OutOfBoundsError);
        }

        let (write_base_y, source_base_y, y_offset_addend) = if write.point.y >= source.y {
            (write.point.y, source.y, 1)
        } else {
            (
                write.point.y + write.height - 1,
                source.y + write.height - 1,
                -1,
            )
        };
        let (write_base_x, source_base_x, x_offset_addend) = if write.point.x >= source.x {
            (write.point.x, source.x, 1)
        } else {
            (
                write.point.x + write.width - 1,
                source.x + write.width - 1,
                -1,
            )
        };

        let mut y_offset = 0;
        for _ in 0..write.height {
            let mut x_offset = 0;
            for _ in 0..write.width {
                // SAFETY:
                //
                // The reads were bounds checked before the loop.
                let value = unsafe {
                    self.read_pixel_unchecked(Point {
                        x: source_base_x.wrapping_add_signed(x_offset),
                        y: source_base_y.wrapping_add_signed(y_offset),
                    })
                };

                // SAFETY:
                //
                // The writes were bounds checked before the loop.
                unsafe {
                    self.write_pixel_unchecked(
                        Point {
                            x: write_base_x.wrapping_add_signed(x_offset),
                            y: write_base_y.wrapping_add_signed(y_offset),
                        },
                        value,
                    )
                }

                x_offset += x_offset_addend;
            }
            y_offset += y_offset_addend;
        }

        Ok(())
    }
}

/// Various errors that can occur when writing to a [`Surface`].
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum WriteError {
    /// The requested operation is out of bounds.
    OutOfBounds,
    /// The specified pixel value is invalid according to the [`Format`].
    InvalidPixelValue,
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfBounds => f.write_str("write out of bounds"),
            Self::InvalidPixelValue => f.write_str("invalid pixel value"),
        }
    }
}

impl error::Error for WriteError {}

/// A requested operation would have been out of bounds.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct OutOfBoundsError;

impl fmt::Display for OutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "operation out of bounds".fmt(f)
    }
}

/// Returns `true` if the given `point` is within the given bounds.
pub const fn point_in_bounds(point: Point, width: usize, height: usize) -> bool {
    point.x < width && point.y < height
}

/// Returns `true` if the given `region` is within the given bounds.
pub fn region_in_bounds(region: Region, width: usize, height: usize) -> bool {
    let end_x = region.point.x.checked_add(region.width);
    let end_y = region.point.y.checked_add(region.height);
    if let Some((end_x, end_y)) = end_x.zip(end_y) {
        end_x <= width && end_y <= height
    } else {
        false
    }
}

/// The format of a pixel.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Format {
    /// The bit count of the red mask.
    red_size: u8,
    /// The bit offset of the red mask.
    red_shift: u8,
    /// The bit count of the green mask.
    green_size: u8,
    /// The bit offset of the green mask.
    green_shift: u8,
    /// The bit count of the blue mask.
    blue_size: u8,
    /// The bit offset of the blue mask.
    blue_shift: u8,
    /// The number of bits in a pixel of this [`Format`].
    bits_per_pixel: u8,
}

impl Format {
    /// Creates a [`Format`] from the size-shift representation of a pixel format.
    pub const fn from_size_shift(
        red_size: u8,
        red_shift: u8,
        green_size: u8,
        green_shift: u8,
        blue_size: u8,
        blue_shift: u8,
        bits_per_pixel: u8,
    ) -> Option<Self> {
        match red_shift.checked_add(red_size) {
            Some(red_max) if red_max <= 64 => {}
            _ => return None,
        }

        match green_shift.checked_add(green_size) {
            Some(green_max) if green_max <= 64 => {}
            _ => return None,
        }

        match blue_shift.checked_add(blue_size) {
            Some(blue_max) if blue_max <= 64 => {}
            _ => return None,
        }

        let format = Self {
            red_size,
            red_shift,
            green_size,
            green_shift,
            blue_size,
            blue_shift,
            bits_per_pixel,
        };

        if format.red_mask() & format.green_mask() != 0
            || format.blue_mask() & format.reserved_mask() != 0
        {
            return None;
        }

        let red_green_mask = format.red_mask() | format.green_mask();
        let blue_reserved_mask = format.blue_mask() | format.reserved_mask();
        if red_green_mask & blue_reserved_mask != 0 {
            return None;
        }

        Some(format)
    }

    /// Returns the size of the red bitmask.
    pub const fn red_size(&self) -> u8 {
        self.red_size
    }

    /// Returns the offset of the red bitmask.
    pub const fn red_shift(&self) -> u8 {
        self.red_shift
    }

    /// Returns the red bitmask.
    pub const fn red_mask(&self) -> u64 {
        Self::compute_mask(self.red_size, self.red_shift)
    }

    /// Returns the size of the green bitmask.
    pub const fn green_size(&self) -> u8 {
        self.green_size
    }

    /// Returns the offset of the green bitmask.
    pub const fn green_shift(&self) -> u8 {
        self.green_shift
    }

    /// Returns the green bitmask.
    pub const fn green_mask(&self) -> u64 {
        Self::compute_mask(self.green_size, self.green_shift)
    }

    /// Returns the size of the blue bitmask.
    pub const fn blue_size(&self) -> u8 {
        self.blue_size
    }

    /// Returns the offset of the blue bitmask.
    pub const fn blue_shift(&self) -> u8 {
        self.blue_shift
    }

    /// Returns the blue bitmask.
    pub const fn blue_mask(&self) -> u64 {
        Self::compute_mask(self.blue_size, self.blue_shift)
    }

    /// Returns the red, green, and blue bitmasks ORed together.
    pub const fn rgb_mask(&self) -> u64 {
        self.red_mask() | self.green_mask() | self.blue_mask()
    }

    /// Returns the reserved bitmask.
    pub const fn reserved_mask(&self) -> u64 {
        let pixel_mask = Self::compute_mask(self.bits_per_pixel, 0);

        !(self.rgb_mask() & pixel_mask)
    }

    /// Returns the total amount of bits in the pixel.
    pub const fn bits_per_pixel(&self) -> u8 {
        self.bits_per_pixel
    }

    /// Helper function that computes the mask specified by `size` and `shift`.
    const fn compute_mask(size: u8, shift: u8) -> u64 {
        1u64.wrapping_shl(size as u32)
            .wrapping_sub(1)
            .wrapping_shl(shift as u32)
    }
}

/// A point in a [`Surface`].
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point {
    /// The x-coordinate of the pixel.
    pub x: usize,
    /// The y-coordinate of the pixel.
    pub y: usize,
}

/// A region of pixels.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Region {
    /// The upper left corner of the region.
    pub point: Point,
    /// The width of the region in pixels.
    pub width: usize,
    /// The height of the region in pixels.
    pub height: usize,
}
