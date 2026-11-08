//! Combined byte and bit reader utilities.

use crate::error::{MarkerError, Result, bail};
use core::fmt::Debug;

#[derive(Debug, Clone)]
pub(crate) struct BitReader<'a> {
    data: &'a [u8],
    cur_pos: usize,
}

impl<'a> BitReader<'a> {
    #[inline]
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self { data, cur_pos: 0 }
    }

    #[inline]
    pub(crate) fn align(&mut self) {
        let bit_pos = self.bit_pos();

        if !bit_pos.is_multiple_of(8) {
            self.cur_pos += 8 - bit_pos;
        }
    }

    #[inline]
    pub(crate) fn at_end(&self) -> bool {
        self.byte_pos() >= self.data.len()
    }

    #[inline]
    pub(crate) fn jump_to_end(&mut self) {
        self.cur_pos = self.data.len() * 8;
    }

    #[inline]
    pub(crate) fn tail(&self) -> Option<&'a [u8]> {
        rhwp_codec_resource::work(1).ok()?;
        self.data.get(self.byte_pos()..)
    }

    #[inline]
    pub(crate) fn offset(&self) -> usize {
        self.byte_pos()
    }

    #[inline]
    pub(crate) fn read_bytes(&mut self, len: usize) -> Option<&'a [u8]> {
        rhwp_codec_resource::work(1).ok()?;
        debug_assert_eq!(self.bit_pos(), 0);

        let bytes = self.peek_bytes(len)?;
        self.cur_pos += len * 8;

        Some(bytes)
    }

    #[inline]
    pub(crate) fn read_byte(&mut self) -> Option<u8> {
        rhwp_codec_resource::work(1).ok()?;
        debug_assert_eq!(self.bit_pos(), 0);

        let byte = self.peek_byte()?;
        self.cur_pos += 8;

        Some(byte)
    }

    #[inline]
    pub(crate) fn skip_bytes(&mut self, len: usize) -> Option<()> {
        rhwp_codec_resource::work(1).ok()?;
        self.read_bytes(len).map(|_| ())
    }

    #[inline]
    pub(crate) fn peek_bytes(&self, len: usize) -> Option<&'a [u8]> {
        rhwp_codec_resource::work(1).ok()?;
        let start = self.byte_pos();
        let end = start.checked_add(len)?;
        self.data.get(start..end)
    }

    #[inline]
    pub(crate) fn peek_byte(&self) -> Option<u8> {
        rhwp_codec_resource::work(1).ok()?;
        self.data.get(self.byte_pos()).copied()
    }

    #[inline]
    pub(crate) fn read_u16(&mut self) -> Option<u16> {
        rhwp_codec_resource::work(1).ok()?;
        let bytes = self.read_bytes(2)?;

        Some(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    #[inline]
    pub(crate) fn read_u32(&mut self) -> Option<u32> {
        rhwp_codec_resource::work(1).ok()?;
        let bytes = self.read_bytes(4)?;

        Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    #[inline]
    pub(crate) fn read_u64(&mut self) -> Option<u64> {
        rhwp_codec_resource::work(1).ok()?;
        let bytes = self.read_bytes(8)?;

        Some(u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    #[inline(always)]
    pub(crate) fn read_bit(&mut self) -> Option<u32> {
        rhwp_codec_resource::work(1).ok()?;
        let byte_pos = self.byte_pos();
        let byte = *self.data.get(byte_pos)? as u32;
        let shift = 7 - self.bit_pos();
        self.cur_pos += 1;
        Some((byte >> shift) & 1)
    }

    #[inline]
    pub(crate) fn byte_pos(&self) -> usize {
        self.cur_pos / 8
    }

    #[inline]
    pub(crate) fn bit_pos(&self) -> usize {
        self.cur_pos % 8
    }

    /// Like the normal `read_bits` method, but accounts for stuffing bits
    /// in addition.
    #[inline]
    pub(crate) fn read_bits_with_stuffing(&mut self, bit_size: u8) -> Option<u32> {
        rhwp_codec_resource::work(1).ok()?;
        let mut bit = 0;

        {
            let (__rhwp_iter_1, __rhwp_iter_1_prepaid) =
                rhwp_codec_resource::iterations(0..bit_size).ok()?;
            for _ in __rhwp_iter_1 {
                if !__rhwp_iter_1_prepaid {
                    rhwp_codec_resource::work(1).ok()?;
                }
                let needs_stuff_bit = self.needs_to_read_stuff_bit();

                bit = (bit << 1) | self.read_bit()?;

                if needs_stuff_bit {
                    self.read_stuff_bit()?;
                }
            }
        }

        Some(bit)
    }

    pub(crate) fn needs_to_read_stuff_bit(&mut self) -> bool {
        // B.10.1: "If the value of the byte is 0xFF, the next byte includes an extra zero bit
        // stuffed into the MSB."
        self.bit_pos() == 7 && self.data[self.byte_pos()] == 0xff
    }

    #[inline]
    pub(crate) fn read_stuff_bit(&mut self) -> Option<()> {
        rhwp_codec_resource::work(1).ok()?;
        let stuff_bit = self.read_bit()?;

        if stuff_bit != 0 {
            return None;
        }

        Some(())
    }

    #[inline]
    pub(crate) fn peak_bits_with_stuffing(&mut self, bit_size: u8) -> Option<u32> {
        rhwp_codec_resource::work(1).ok()?;
        self.clone().read_bits_with_stuffing(bit_size)
    }

    #[inline]
    pub(crate) fn read_marker(&mut self) -> Result<u8> {
        rhwp_codec_resource::work(1)?;
        if self.peek_byte().ok_or(MarkerError::Invalid)? != 0xFF {
            bail!(MarkerError::Invalid);
        }

        self.read_byte().unwrap();
        Ok(self.read_byte().ok_or(MarkerError::Invalid)?)
    }

    #[inline]
    pub(crate) fn peek_marker(&mut self) -> Option<u8> {
        rhwp_codec_resource::work(1).ok()?;
        self.clone().read_marker().ok()
    }
}
