//! Bitmap representation for JBIG2 decoding.
//!
//! "The variable whose value is the result of this decoding procedure is shown
//! in Table 3." (6.2.3)
//!
//! "GBREG - The decoded region bitmap." (Table 3)

use crate::decode::CombinationOperator;
use crate::error::{OverflowError, Result, bail};
use alloc::vec::Vec;
use rhwp_codec_resource::VecResource as _;

// To guard better against malicious files, we limit bitmap dimensions
// to `u16::MAX`, which should be more than enough.
pub(crate) const MAX_DIMENSION: u32 = u16::MAX as u32;

/// The underlying word type for packed pixel storage.
/// Change this to `u32` to switch back to 32-bit words.
pub(crate) type Word = u32;

pub(crate) const WORD_BITS: u32 = Word::BITS;
pub(crate) const WORD_SHIFT: u32 = WORD_BITS - 1;

/// A decoded bitmap with position information.
///
/// "Pixels decoded by the MMR decoder having the value 'black' shall be treated
/// as having the value 1. Pixels decoded by the MMR decoder having the value
/// 'white' shall be treated as having the value 0." (6.2.6)
#[derive(Debug, Clone, Default)]
pub(crate) struct Bitmap {
    /// Width in pixels.
    pub(crate) width: u32,
    /// Height in pixels.
    pub(crate) height: u32,
    /// Number of words per row.
    pub(crate) stride: u32,
    /// Packed pixel data, one bit per pixel, row-major order.
    /// Each row is padded to a word boundary.
    pub(crate) data: Vec<Word>,
    /// "This four-byte field gives the horizontal offset in pixels of the bitmap
    /// encoded in this segment relative to the page bitmap." (7.4.1.3)
    pub(crate) x_location: u32,
    /// "This four-byte field gives the vertical offset in pixels of the bitmap
    /// encoded in this segment relative to the page bitmap." (7.4.1.4)
    pub(crate) y_location: u32,
}

impl Bitmap {
    pub(crate) fn try_clone(&self) -> Result<Self> {
        Ok(Self {
            width: self.width,
            height: self.height,
            stride: self.stride,
            data: rhwp_codec_resource::copy(&self.data)?,
            x_location: self.x_location,
            y_location: self.y_location,
        })
    }

    /// Create a new bitmap filled with white pixels (false).
    ///
    /// The bitmap is positioned at (0, 0).
    pub(crate) fn new(width: u32, height: u32) -> Result<Self> {
        rhwp_codec_resource::work(1)?;
        Self::new_with(width, height, 0, 0, false)
    }

    /// Create a new bitmap with full configuration.
    pub(crate) fn new_with(
        width: u32,
        height: u32,
        x_location: u32,
        y_location: u32,
        default_pixel: bool,
    ) -> Result<Self> {
        rhwp_codec_resource::work(1)?;
        if width > MAX_DIMENSION || height > MAX_DIMENSION {
            bail!(OverflowError::BitmapDimension);
        }

        let stride = width.div_ceil(WORD_BITS);
        let default_word: Word = if default_pixel { !0 } else { 0 };
        let data = rhwp_codec_resource::filled(
            default_word,
            rhwp_codec_resource::product(&[stride as usize, height as usize])?,
        )?;
        Ok(Self {
            width,
            height,
            stride,
            data,
            x_location,
            y_location,
        })
    }

    pub(crate) fn reinitialize(
        &mut self,
        width: u32,
        height: u32,
        default_pixel: bool,
    ) -> Result<()> {
        rhwp_codec_resource::work(1)?;
        if width > MAX_DIMENSION || height > MAX_DIMENSION {
            return Err(OverflowError::BitmapDimension.into());
        }

        let stride = width.div_ceil(WORD_BITS);
        let default_word: Word = if default_pixel { !0 } else { 0 };
        self.width = width;
        self.height = height;
        self.stride = stride;
        self.x_location = 0;
        self.y_location = 0;
        self.data.clear();
        self.data.rhwp_resize(
            rhwp_codec_resource::product(&[stride as usize, height as usize])?,
            default_word,
        )?;

        Ok(())
    }

    /// Get a pixel value at (x, y).
    #[inline]
    pub(crate) fn get_pixel(&self, x: u32, y: u32) -> u8 {
        if x >= self.width || y >= self.height {
            return 0;
        }

        let word_idx = (y * self.stride + x / WORD_BITS) as usize;
        let bit_pos = WORD_SHIFT - (x % WORD_BITS);
        ((self.data[word_idx] >> bit_pos) & 1) as u8
    }

    /// Set a pixel value at (x, y).
    #[inline]
    pub(crate) fn set_pixel(&mut self, x: u32, y: u32, value: u8) {
        if x >= self.width || y >= self.height {
            return;
        }

        let word_idx = (y * self.stride + x / WORD_BITS) as usize;
        let bit_pos = WORD_SHIFT - (x % WORD_BITS);
        self.data[word_idx] |= (value as Word) << bit_pos;
    }

    /// Combine another bitmap into this one at a specific location.
    ///
    /// "These operators describe how the segment's bitmap is to be combined with
    /// the page bitmap." (7.4.1.5)
    ///
    /// Pixels outside the destination bitmap are ignored.
    pub(crate) fn combine(&mut self, other: &Self, x: i32, y: i32, operator: CombinationOperator) {
        if rhwp_codec_resource::product(&[other.height as usize, other.stride as usize + 1])
            .and_then(rhwp_codec_resource::work)
            .is_err()
        {
            return;
        }
        // TODO: Figure out how we can optimize this, especially since this is a
        // bottleneck for symbols/patterns.
        let dest_x_start = x.max(0);
        let dest_x_end = (x + other.width as i32).min(self.width as i32);

        if dest_x_start >= dest_x_end {
            return;
        }

        let src_x_start = (dest_x_start - x) as u32;
        let dest_x_start = dest_x_start as u32;
        let dest_x_end = dest_x_end as u32;

        for src_y in 0..other.height {
            let dest_y = y + src_y as i32;
            if dest_y < 0 || dest_y >= self.height as i32 {
                continue;
            }

            let dest_y = dest_y as u32;

            let first_word = dest_x_start / WORD_BITS;
            let last_word = (dest_x_end - 1) / WORD_BITS;

            for word_idx in first_word..=last_word {
                let word_start_x = word_idx * WORD_BITS;
                let word_end_x = word_start_x + WORD_BITS;

                let px_start = dest_x_start.max(word_start_x);
                let px_end = dest_x_end.min(word_end_x);

                let bit_start = px_start - word_start_x;
                let bit_end = px_end - word_start_x;

                let mask: Word = if bit_end == WORD_BITS {
                    !0 >> bit_start
                } else {
                    (!0 >> bit_start) & !(!0 >> bit_end)
                };

                let src_x_for_range = src_x_start + (px_start - dest_x_start);
                let src_word_idx = src_x_for_range / WORD_BITS;
                let src_bit_offset = src_x_for_range % WORD_BITS;

                let src_word1 = other.get_word(src_y, src_word_idx);
                let src_word2 = other.get_word(src_y, src_word_idx + 1);

                let src_raw = if src_bit_offset == 0 {
                    src_word1
                } else {
                    (src_word1 << src_bit_offset) | (src_word2 >> (WORD_BITS - src_bit_offset))
                };
                let src_aligned = src_raw >> bit_start;

                let dest_idx = (dest_y * self.stride + word_idx) as usize;
                let dest_word = self.data[dest_idx];

                let result = match operator {
                    CombinationOperator::Or => dest_word | src_aligned,
                    CombinationOperator::And => dest_word & src_aligned,
                    CombinationOperator::Xor => dest_word ^ src_aligned,
                    CombinationOperator::Xnor => !(dest_word ^ src_aligned),
                    CombinationOperator::Replace => src_aligned,
                };

                self.data[dest_idx] = (dest_word & !mask) | (result & mask);
            }
        }
    }

    #[inline]
    pub(crate) fn get_word(&self, row: u32, word_idx: u32) -> Word {
        if row >= self.height || word_idx >= self.stride {
            return 0;
        }

        self.data[(row * self.stride + word_idx) as usize]
    }
}
