//! Pattern dictionary segment parsing and decoding (7.4.4, 6.7).

use super::{AdaptiveTemplatePixel, Template, generic};
use crate::ScratchBuffers;
use crate::arithmetic_decoder::{ArithmeticDecoder, ArithmeticDecoderContext};
use crate::bitmap::{Bitmap, WORD_BITS, Word};
use crate::error::{OverflowError, ParseError, Result};
use crate::reader::Reader;
use alloc::vec::Vec;
use rhwp_codec_resource::VecResource as _;

/// Decode a pattern dictionary segment (7.4.4.2, 6.7).
pub(crate) fn decode(
    header: &PatternDictionaryHeader<'_>,
    scratch: &mut ScratchBuffers,
) -> Result<PatternDictionary> {
    rhwp_codec_resource::work(1)?;
    let pattern_width = header.pattern_width as u32;
    let pattern_height = header.pattern_height as u32;
    let num_patterns = header
        .num_patterns
        .checked_add(1)
        .ok_or(OverflowError::BitmapDimension)?;

    // "1) Create a bitmap B_HDC. The height of this bitmap is HDPH. The width
    // of the bitmap is (GRAYMAX + 1) × HDPW. This bitmap contains all the
    // patterns concatenated left to right." (6.7.5)
    let collective_width = num_patterns
        .checked_mul(pattern_width)
        .ok_or(OverflowError::BitmapDimension)?;

    let mut collective_bitmap = Bitmap::new(collective_width, pattern_height)?;

    // "2) Decode the collective bitmap using a generic region decoding procedure
    // as described in 6.2." (6.7.5)
    if header.mmr {
        let _ = generic::decode_bitmap_mmr(&mut collective_bitmap, header.data)?;
    } else {
        let at_pixels = match header.template {
            Template::Template0 => [
                AdaptiveTemplatePixel {
                    x: -(pattern_width as i8),
                    y: 0,
                },
                AdaptiveTemplatePixel { x: -3, y: -1 },
                AdaptiveTemplatePixel { x: 2, y: -2 },
                AdaptiveTemplatePixel { x: -2, y: -2 },
            ],
            Template::Template1 | Template::Template2 | Template::Template3 => [
                AdaptiveTemplatePixel {
                    x: -(pattern_width as i8),
                    y: 0,
                },
                // Unused.
                AdaptiveTemplatePixel::default(),
                AdaptiveTemplatePixel::default(),
                AdaptiveTemplatePixel::default(),
            ],
        };

        let mut decoder = ArithmeticDecoder::new(header.data);
        scratch.contexts.clear();
        scratch.contexts.rhwp_resize(
            1 << header.template.context_bits(),
            ArithmeticDecoderContext::default(),
        )?;

        generic::decode_bitmap_arithmetic_coding(
            &mut collective_bitmap,
            &mut decoder,
            &mut scratch.contexts,
            header.template,
            false,
            &at_pixels,
        )?;
    }

    // "3) Set: GRAY = 0" (6.7.5)
    // "4) While GRAY ≤ GRAYMAX:" (6.7.5)
    let mut patterns = rhwp_codec_resource::capacity(num_patterns as usize)?;

    let src_stride = collective_bitmap.stride;

    {
        let (__rhwp_iter_3, __rhwp_iter_3_prepaid) =
            rhwp_codec_resource::iterations(0..num_patterns)?;
        for gray in __rhwp_iter_3 {
            if !__rhwp_iter_3_prepaid {
                rhwp_codec_resource::work(1)?;
            }
            // "a) Let the subimage of B_HDC consisting of HPH rows and columns
            // HDPW × GRAY through HDPW × (GRAY + 1) − 1 be denoted B_P. Set:
            // HDPATS[GRAY] = B_P" (6.7.5)"
            let start_x = gray * pattern_width;
            let mut pattern = Bitmap::new(pattern_width, pattern_height)?;

            let src_word_idx = start_x / WORD_BITS;
            let src_bit_offset = start_x % WORD_BITS;
            let dst_stride = pattern.stride;

            {
                let (__rhwp_iter_2, __rhwp_iter_2_prepaid) =
                    rhwp_codec_resource::iterations(0..pattern_height)?;
                for y in __rhwp_iter_2 {
                    if !__rhwp_iter_2_prepaid {
                        rhwp_codec_resource::work(1)?;
                    }
                    let src_row = (y * src_stride + src_word_idx) as usize;
                    let dst_row = (y * dst_stride) as usize;

                    {
                        let (__rhwp_iter_1, __rhwp_iter_1_prepaid) =
                            rhwp_codec_resource::iterations(0..dst_stride)?;
                        for w in __rhwp_iter_1 {
                            if !__rhwp_iter_1_prepaid {
                                rhwp_codec_resource::work(1)?;
                            }
                            let s1 = collective_bitmap.data[src_row + w as usize];
                            let s2 = collective_bitmap
                                .data
                                .get(src_row + w as usize + 1)
                                .copied()
                                .unwrap_or(0);

                            let word: Word = if src_bit_offset == 0 {
                                s1
                            } else {
                                (s1 << src_bit_offset) | (s2 >> (WORD_BITS - src_bit_offset))
                            };

                            pattern.data[dst_row + w as usize] = word;
                        }
                    }

                    // Clear trailing bits in the last word beyond `pattern_width`.
                    let trailing = pattern_width % WORD_BITS;
                    if trailing != 0 {
                        let last = dst_row + (dst_stride - 1) as usize;
                        pattern.data[last] &= !0 << (WORD_BITS - trailing);
                    }
                }
            }

            patterns.rhwp_push(pattern)?;
        }
    }

    // It turns out that when rendering patterns, the `Bitmap::combine` operation
    // can often be a huge bottleneck. This is because we need to do a lot of
    // fiddling to compose the patterns in the right position. And since
    // if a page uses such patterns, there's going to be a lot of them,
    // this takes up a lot of time.
    //
    // The key insight for our optimization is that patterns are usually
    // very small. Therefore, we precompute a new version of each pattern
    // for all possible 32-bit alignments, which later on allows us to
    // very easily blit it into the backdrop, significantly speeding up
    // the whole process.
    let shifted_patterns = if pattern_width <= WORD_BITS && pattern_height <= WORD_BITS {
        let h = pattern_height as usize;
        let mut data = rhwp_codec_resource::capacity(rhwp_codec_resource::product(&[
            patterns.len(),
            32,
            h,
            2,
        ])?)?;
        {
            let (__rhwp_iter_6, __rhwp_iter_6_prepaid) =
                rhwp_codec_resource::iterations(&patterns)?;
            for pattern in __rhwp_iter_6 {
                if !__rhwp_iter_6_prepaid {
                    rhwp_codec_resource::work(1)?;
                }
                {
                    let (__rhwp_iter_5, __rhwp_iter_5_prepaid) =
                        rhwp_codec_resource::iterations(0..32_u32)?;
                    for k in __rhwp_iter_5 {
                        if !__rhwp_iter_5_prepaid {
                            rhwp_codec_resource::work(1)?;
                        }
                        {
                            let (__rhwp_iter_4, __rhwp_iter_4_prepaid) =
                                rhwp_codec_resource::iterations(0..h)?;
                            for y in __rhwp_iter_4 {
                                if !__rhwp_iter_4_prepaid {
                                    rhwp_codec_resource::work(1)?;
                                }
                                let word = pattern.data[y];
                                if k == 0 {
                                    data.rhwp_push(word)?;
                                    data.rhwp_push(0)?;
                                } else {
                                    data.rhwp_push(word >> k)?;
                                    data.rhwp_push(word << (WORD_BITS - k))?;
                                }
                            }
                        }
                    }
                }
            }
        }
        data
    } else {
        Vec::new()
    };

    Ok(PatternDictionary {
        patterns,
        pattern_width,
        pattern_height,
        shifted_patterns,
    })
}

/// A decoded pattern dictionary.
#[derive(Debug, Clone)]
pub(crate) struct PatternDictionary {
    pub(crate) patterns: Vec<Bitmap>,
    pub(crate) pattern_width: u32,
    pub(crate) pattern_height: u32,
    shifted_patterns: Vec<Word>,
}

impl PatternDictionary {
    #[inline(always)]
    pub(crate) fn shifted_pattern(&self, pattern_idx: usize, bit_offset: u32) -> Option<&[Word]> {
        rhwp_codec_resource::work(1).ok()?;
        let h = self.pattern_height as usize;
        let len = h * 2;
        let start = (pattern_idx * 32 + bit_offset as usize) * len;
        self.shifted_patterns.get(start..start + len)
    }
}

/// Parsed pattern dictionary segment header (7.4.4.1).
#[derive(Debug, Clone)]
pub(crate) struct PatternDictionaryHeader<'a> {
    pub(crate) mmr: bool,
    pub(crate) template: Template,
    /// `HDPW`
    pub(crate) pattern_width: u8,
    /// `HDPH`
    pub(crate) pattern_height: u8,
    /// `GRAYMAX`
    pub(crate) num_patterns: u32,
    pub(crate) data: &'a [u8],
}

/// Parse a pattern dictionary segment header (7.4.4.1).
pub(crate) fn parse<'a>(reader: &mut Reader<'a>) -> Result<PatternDictionaryHeader<'a>> {
    rhwp_codec_resource::work(1)?;
    let flags_byte = reader.read_byte().ok_or(ParseError::UnexpectedEof)?;
    let mmr = flags_byte & 0x01 != 0;
    let template = Template::from_byte(flags_byte >> 1);
    let pattern_width = reader
        .read_nonzero_byte()
        .ok_or(ParseError::UnexpectedEof)?;
    let pattern_height = reader
        .read_nonzero_byte()
        .ok_or(ParseError::UnexpectedEof)?;
    let num_patterns = reader.read_u32().ok_or(ParseError::UnexpectedEof)?;
    let data = reader.tail().ok_or(ParseError::UnexpectedEof)?;

    Ok(PatternDictionaryHeader {
        mmr,
        template,
        pattern_width,
        pattern_height,
        num_patterns,
        data,
    })
}
