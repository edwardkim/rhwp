//! A decoder for CCITT fax-encoded images.
//!
//! This crate implements the CCITT Group 3 and Group 4 fax compression algorithms
//! as defined in ITU-T Recommendations T.4 and T.6. These encodings are commonly
//! used for bi-level (black and white) images in PDF documents and fax transmissions.
//!
//! The main entry point is the [`decode`] function, which takes encoded data, a
//! [`DecoderContext`], and outputs the decoded pixels through a [`Decoder`] trait
//! that can be implemented according to your needs.
//!
//! The crate is `no_std` compatible but requires an allocator to be available.
//!
//! # Safety
//! Unsafe code is forbidden via a crate-level attribute.
//!
//! # License
//! Licensed under either of
//!
//! - Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
//! - MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
//!
//! at your option.
//!
//! [`decode`]: crate::decode
//! [`Decoder`]: crate::Decoder

#![no_std]
#![forbid(unsafe_code)]
#![forbid(missing_docs)]

use rhwp_codec_resource::VecResource as _;
extern crate alloc;

use crate::bit_reader::BitReader;

use crate::decode::{EOFB, Mode};
use alloc::vec;
use alloc::vec::Vec;

mod bit_reader;
mod decode;
mod state_machine;

/// A specialized Result type for CCITT decoding operations.
pub type Result<T> = core::result::Result<T, DecodeError>;

/// An error that can occur during CCITT decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    /// The requesting importer rejected codec resources.
    ResourceLimit,
    /// Unexpected end of input while reading bits.
    UnexpectedEof,
    /// Invalid Huffman code sequence was encountered during decoding.
    InvalidCode,
    /// A scanline didn't have the expected number of pixels.
    LineLengthMismatch,
    /// Arithmetic overflow in run length or position calculation.
    Overflow,
}

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ResourceLimit => write!(f, "codec resource limit"),
            Self::UnexpectedEof => write!(f, "unexpected end of input"),
            Self::InvalidCode => write!(f, "invalid CCITT code sequence"),
            Self::LineLengthMismatch => write!(f, "scanline length mismatch"),
            Self::Overflow => write!(f, "arithmetic overflow in position calculation"),
        }
    }
}

impl core::error::Error for DecodeError {}

/// The encoding mode for CCITT fax decoding.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EncodingMode {
    /// Group 4 (MMR).
    Group4,
    /// Group 3 1D (MH).
    Group3_1D,
    /// Group 3 2D (MR).
    Group3_2D {
        /// The K parameter.
        k: u32,
    },
}

/// Settings to apply during decoding.
#[derive(Copy, Clone, Debug)]
pub struct DecodeSettings {
    /// How many columns the image has (i.e. its width).
    pub columns: u32,
    /// How many rows the image has (i.e. its height).
    ///
    /// In case `end_of_block` has been set to true, decoding will run until
    /// the given number of rows have been decoded, or the `end_of_block` marker
    /// has been encountered, whichever occurs first.
    pub rows: u32,
    /// Whether the stream _MAY_ contain an end-of-block marker
    /// (It doesn't have to. In that case this is set to `true` but there are
    /// no end-of-block markers, hayro-ccitt will still use the value of `rows`
    /// to determine when to stop decoding).
    pub end_of_block: bool,
    /// Whether the stream contains end-of-line markers.
    pub end_of_line: bool,
    /// Whether the data in the stream for each row is aligned to the byte
    /// boundary.
    pub rows_are_byte_aligned: bool,
    /// The encoding mode used by the image.
    pub encoding: EncodingMode,
    /// Whether black and white should be inverted.
    pub invert_black: bool,
}

/// A decoder for CCITT images.
pub trait Decoder {
    /// Push a single pixel with the given color.
    fn push_pixel(&mut self, white: bool);
    /// Push multiple chunks of 8 pixels of the same color.
    ///
    /// The `chunk_count` parameter indicates how many 8-pixel chunks to push.
    /// For example, if this method is called with `white = true` and
    /// `chunk_count = 10`, 80 white pixels are pushed (10 × 8 = 80).
    ///
    /// You can assume that this method is only called if the number of already
    /// pushed pixels is a multiple of 8 (i.e. byte-aligned).
    fn push_pixel_chunk(&mut self, white: bool, chunk_count: u32);
    /// Called when a row has been completed.
    fn next_line(&mut self);
}

/// Pixel color in a bi-level (black and white) image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Color {
    /// White pixel.
    White,
    /// Black pixel.
    Black,
}

impl Color {
    /// Returns the opposite color.
    #[inline(always)]
    fn opposite(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }

    /// Returns true if this color is white.
    #[inline(always)]
    fn is_white(self) -> bool {
        matches!(self, Self::White)
    }
}

/// Represents a color change at a specific index in a line.
#[derive(Clone, Copy)]
struct ColorChange {
    idx: u32,
    color: Color,
}

/// Decode the given image using the provided decoder context and decoder.
///
/// If decoding was successful, the number of bytes that have been read in total
/// is returned.
///
/// If an error is returned, it means that the file is somehow malformed.
/// However, even if that's the case, it is possible that a number
/// of rows were decoded successfully and written into the decoder, so those
/// can still be used, but the image might be truncated.
pub fn decode(data: &[u8], decoder: &mut impl Decoder, ctx: &mut DecoderContext) -> Result<usize> {
    rhwp_codec_resource::work(1)?;
    ctx.reset();
    let mut reader = BitReader::new(data);

    match ctx.settings.encoding {
        EncodingMode::Group4 => decode_group4(ctx, &mut reader, decoder)?,
        EncodingMode::Group3_1D => decode_group3_1d(ctx, &mut reader, decoder)?,
        EncodingMode::Group3_2D { .. } => decode_group3_2d(ctx, &mut reader, decoder)?,
    }

    reader.align();
    Ok(reader.byte_pos())
}

/// Group 3 1D decoding (T.4 Section 4.1).
fn decode_group3_1d(
    ctx: &mut DecoderContext,
    reader: &mut BitReader<'_>,
    decoder: &mut impl Decoder,
) -> Result<()> {
    rhwp_codec_resource::work(1)?;
    // It seems like PDF producers are a bit sloppy with the `end_of_line` flag,
    // so we just always try to read one.
    let _ = reader.read_eol_if_available();

    loop {
        rhwp_codec_resource::work(1)?;
        decode_1d_line(ctx, reader, decoder)?;
        ctx.next_line(reader, decoder)?;

        if group3_check_eob(ctx, reader) {
            break;
        }
    }

    Ok(())
}

/// Group 3 2D decoding (T.4 Section 4.2).
fn decode_group3_2d(
    ctx: &mut DecoderContext,
    reader: &mut BitReader<'_>,
    decoder: &mut impl Decoder,
) -> Result<()> {
    rhwp_codec_resource::work(1)?;
    // It seems like PDF producers are a bit sloppy with the `end_of_line` flag,
    // so we just always try to read one.
    let _ = reader.read_eol_if_available();

    loop {
        rhwp_codec_resource::work(1)?;
        let tag_bit = reader.read_bit()?;

        if tag_bit == 1 {
            decode_1d_line(ctx, reader, decoder)?;
        } else {
            decode_2d_line(ctx, reader, decoder)?;
        }

        ctx.next_line(reader, decoder)?;

        if group3_check_eob(ctx, reader) {
            break;
        }
    }

    Ok(())
}

/// Check for end-of-block, including RTC (T.4 Section 4.1.4).
fn group3_check_eob(ctx: &mut DecoderContext, reader: &mut BitReader<'_>) -> bool {
    let eol_count = reader.read_eol_if_available();

    // T.4 Section 4.1.4: "The end of a document transmission is indicated by
    // sending six consecutive EOLs."
    // PDFBOX-2778 has 7 EOL, although it should only be 6. Let's be lenient
    // and check with >=.
    if ctx.settings.end_of_block && eol_count >= 6 {
        return true;
    }

    if ctx.decoded_rows == ctx.settings.rows || reader.at_end() {
        return true;
    }

    false
}

fn decode_group4(
    ctx: &mut DecoderContext,
    reader: &mut BitReader<'_>,
    decoder: &mut impl Decoder,
) -> Result<()> {
    rhwp_codec_resource::work(1)?;
    loop {
        rhwp_codec_resource::work(1)?;
        if ctx.settings.end_of_block && reader.peak_bits(24) == Ok(EOFB) {
            reader.read_bits(24)?;
            break;
        }

        if ctx.decoded_rows == ctx.settings.rows || reader.at_end() {
            break;
        }

        decode_2d_line(ctx, reader, decoder)?;
        ctx.next_line(reader, decoder)?;
    }

    Ok(())
}

/// Decode a single 1D-coded line (T.4 Section 4.1.1, T.6 Section 2.2.4).
#[inline(always)]
fn decode_1d_line(
    ctx: &mut DecoderContext,
    reader: &mut BitReader<'_>,
    decoder: &mut impl Decoder,
) -> Result<()> {
    rhwp_codec_resource::work(1)?;
    while !ctx.at_eol() {
        rhwp_codec_resource::work(1)?;
        let run_length = reader.decode_run(ctx.color)?;
        ctx.push_pixels(decoder, run_length)?;
        ctx.color = ctx.color.opposite();
    }

    Ok(())
}

/// Decode a single 2D-coded line (T.4 Section 4.2, T.6 Section 2.2).
#[inline(always)]
fn decode_2d_line(
    ctx: &mut DecoderContext,
    reader: &mut BitReader<'_>,
    decoder: &mut impl Decoder,
) -> Result<()> {
    rhwp_codec_resource::work(1)?;
    while !ctx.at_eol() {
        rhwp_codec_resource::work(1)?;
        let mode = reader.decode_mode()?;

        match mode {
            // Pass mode (T.4 Section 4.2.1.3.2a, T.6 Section 2.2.3.1).
            Mode::Pass => {
                ctx.push_pixels(decoder, ctx.b2() - ctx.a0().unwrap_or(0))?;
                ctx.update_b();
                // No color change happens in pass mode.
            }
            // Vertical mode (T.4 Section 4.2.1.3.2b, T.6 Section 2.2.3.2).
            Mode::Vertical(i) => {
                let b1 = ctx.b1();
                let a1 = if i >= 0 {
                    b1.checked_add(i as u32).ok_or(DecodeError::Overflow)?
                } else {
                    b1.checked_sub((-i) as u32).ok_or(DecodeError::Overflow)?
                };

                let a0 = ctx.a0().unwrap_or(0);

                ctx.push_pixels(decoder, a1.checked_sub(a0).ok_or(DecodeError::Overflow)?)?;
                ctx.color = ctx.color.opposite();

                ctx.update_b();
            }
            // Horizontal mode (T.4 Section 4.2.1.3.2c, T.6 Section 2.2.3.3).
            Mode::Horizontal => {
                let a0a1 = reader.decode_run(ctx.color)?;
                ctx.push_pixels(decoder, a0a1)?;
                ctx.color = ctx.color.opposite();

                let a1a2 = reader.decode_run(ctx.color)?;
                ctx.push_pixels(decoder, a1a2)?;
                ctx.color = ctx.color.opposite();

                ctx.update_b();
            }
        }
    }

    Ok(())
}

/// A reusable context for decoding CCITT images.
pub struct DecoderContext {
    /// Color changes in the reference line (previous line).
    ref_changes: Vec<ColorChange>,
    /// The minimum index we need to start from when searching for b1.
    ref_pos: u32,
    /// The current index of b1.
    b1_idx: u32,
    /// Color changes in the coding line (current line being decoded).
    coding_changes: Vec<ColorChange>,
    /// Current position in the coding line (number of pixels decoded).
    pixels_decoded: u32,
    /// The width of a line in pixels (i.e. number of columns).
    line_width: u32,
    /// The color of the next run to be decoded.
    color: Color,
    /// How many rows have been decoded so far.
    decoded_rows: u32,
    /// The settings to apply during decoding.
    settings: DecodeSettings,
    /// Whether to invert black and white.
    invert_black: bool,
}

impl DecoderContext {
    /// Creates a new decoder context using the provided decode settings.
    pub fn new(settings: DecodeSettings) -> Self {
        Self {
            ref_changes: vec![],
            ref_pos: 0,
            b1_idx: 0,
            coding_changes: Vec::new(),
            pixels_decoded: 0,
            line_width: settings.columns,
            // Each run starts with an imaginary white pixel on the left.
            color: Color::White,
            decoded_rows: 0,
            settings,
            invert_black: settings.invert_black,
        }
    }

    fn reset(&mut self) {
        self.ref_changes.clear();
        self.ref_pos = 0;
        self.b1_idx = 0;
        self.coding_changes.clear();
        self.pixels_decoded = 0;
        self.line_width = self.settings.columns;
        self.color = Color::White;
        self.decoded_rows = 0;
        self.invert_black = self.settings.invert_black;
    }

    /// `a0` refers to the first changing element on the current line.
    #[inline(always)]
    fn a0(&self) -> Option<u32> {
        rhwp_codec_resource::work(1).ok()?;
        if self.pixels_decoded == 0 {
            // If we haven't coded anything yet, a0 conceptually points at the
            // index -1. This is a bit of an edge case, and we therefore require
            // callers of this method to handle the case themselves.
            None
        } else {
            // Otherwise, the index points to the next element to be decoded.
            Some(self.pixels_decoded)
        }
    }

    /// "The first changing element on the reference line to the right of a0 and
    /// of opposite color to a0."
    #[inline(always)]
    fn b1(&self) -> u32 {
        self.ref_changes
            .get(self.b1_idx as usize)
            .map_or(self.line_width, |c| c.idx)
    }

    /// "The next changing element to the right of b1, on the reference line."
    #[inline(always)]
    fn b2(&self) -> u32 {
        self.ref_changes
            .get(self.b1_idx as usize + 1)
            .map_or(self.line_width, |c| c.idx)
    }

    /// Compute the new position of b1 (and implicitly b2).
    #[inline(always)]
    fn update_b(&mut self) {
        // b1 refers to an element of the opposite color.
        let target_color = self.color.opposite();
        // b1 must be strictly greater than a0.
        let min_idx = self.a0().map_or(0, |a| a + 1);

        self.b1_idx = self.line_width;

        for i in self.ref_pos..self.ref_changes.len() as u32 {
            let change = &self.ref_changes[i as usize];

            if change.idx < min_idx {
                self.ref_pos = i + 1;
                continue;
            }

            if change.color == target_color {
                self.b1_idx = i;
                break;
            }
        }
    }

    #[inline(always)]
    fn push_pixels(&mut self, decoder: &mut impl Decoder, count: u32) -> Result<()> {
        rhwp_codec_resource::work(count as usize)?;
        // Make sure we don't have too many pixels (for invalid files).
        let count = count.min(self.line_width - self.pixels_decoded);
        let white = self.color.is_white() ^ self.invert_black;
        let mut remaining = count;

        // Push individual pixels until we reach an 8-pixel boundary.
        let pixels_to_boundary = (8 - (self.pixels_decoded % 8)) % 8;
        let unaligned_pixels = remaining.min(pixels_to_boundary);
        for _ in 0..unaligned_pixels {
            decoder.push_pixel(white);
            remaining -= 1;
        }

        // Push full chunks of 8 pixels.
        let full_chunks = remaining / 8;
        if full_chunks > 0 {
            decoder.push_pixel_chunk(white, full_chunks);
            remaining %= 8;
        }

        // Push remaining individual pixels.
        for _ in 0..remaining {
            decoder.push_pixel(white);
        }

        // Track the color change:
        // - At start of line (no previous changes): only add if color differs from
        //   imaginary white, i.e., only add if black.
        // - Mid-line: only add if color differs from previous.
        if count > 0 {
            let is_change = self
                .coding_changes
                .last()
                .map_or(!self.color.is_white(), |last| last.color != self.color);
            if is_change {
                self.coding_changes.rhwp_push(ColorChange {
                    idx: self.pixels_decoded,
                    color: self.color,
                })?;
            }
            self.pixels_decoded += count;
        }
        Ok(())
    }

    #[inline(always)]
    fn at_eol(&self) -> bool {
        self.a0().unwrap_or(0) == self.line_width
    }

    #[inline(always)]
    fn next_line(&mut self, reader: &mut BitReader<'_>, decoder: &mut impl Decoder) -> Result<()> {
        rhwp_codec_resource::work(1)?;
        if self.pixels_decoded != self.settings.columns {
            return Err(DecodeError::LineLengthMismatch);
        }

        core::mem::swap(&mut self.ref_changes, &mut self.coding_changes);
        self.coding_changes.clear();
        self.pixels_decoded = 0;
        self.ref_pos = 0;
        self.b1_idx = 0;
        self.color = Color::White;
        self.decoded_rows += 1;
        decoder.next_line();

        if self.settings.rows_are_byte_aligned {
            reader.align();
        }

        self.update_b();

        Ok(())
    }
}

impl From<rhwp_codec_resource::ResourceError> for DecodeError {
    fn from(_: rhwp_codec_resource::ResourceError) -> Self {
        Self::ResourceLimit
    }
}
