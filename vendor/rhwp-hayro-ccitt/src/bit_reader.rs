//! Bit-level reader for CCITT encoded data streams.

use crate::{DecodeError, Result};
use core::fmt::Debug;

#[derive(Debug, Clone)]
pub(crate) struct BitReader<'a> {
    data: &'a [u8],
    bit_offset: usize,
}

impl<'a> BitReader<'a> {
    #[inline(always)]
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            bit_offset: 0,
        }
    }

    #[inline(always)]
    pub(crate) fn read_bit(&mut self) -> Result<u32> {
        rhwp_codec_resource::work(1)?;
        let byte_pos = self.byte_pos();
        let byte = *self.data.get(byte_pos).ok_or(DecodeError::UnexpectedEof)? as u32;
        let shift = 7 - self.bit_pos();
        self.bit_offset += 1;
        Ok((byte >> shift) & 1)
    }

    #[inline(always)]
    pub(crate) fn read_bits(&mut self, num_bits: usize) -> Result<u32> {
        rhwp_codec_resource::work(1)?;
        let mut result = 0_u32;

        {
            let (__rhwp_iter_1, __rhwp_iter_1_prepaid) =
                rhwp_codec_resource::iterations((0..num_bits).rev())?;
            for i in __rhwp_iter_1 {
                if !__rhwp_iter_1_prepaid {
                    rhwp_codec_resource::work(1)?;
                }
                result |= (self.read_bit()?) << i;
            }
        }

        Ok(result)
    }

    #[inline(always)]
    pub(crate) fn peak_bits(&mut self, num_bits: usize) -> Result<u32> {
        rhwp_codec_resource::work(1)?;
        self.clone().read_bits(num_bits)
    }

    #[inline(always)]
    pub(crate) fn align(&mut self) {
        let bit_pos = self.bit_pos();

        if !bit_pos.is_multiple_of(8) {
            self.bit_offset += 8 - bit_pos;
        }
    }

    #[inline(always)]
    pub(crate) fn at_end(&self) -> bool {
        self.byte_pos() >= self.data.len()
    }

    #[inline(always)]
    pub(crate) fn byte_pos(&self) -> usize {
        self.bit_offset >> 3
    }

    #[inline(always)]
    fn bit_pos(&self) -> usize {
        self.bit_offset & 7
    }
}
