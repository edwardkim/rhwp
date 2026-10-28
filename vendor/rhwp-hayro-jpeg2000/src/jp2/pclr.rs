//! The palette box (pclr), defined in I.5.3.4.

use alloc::vec::Vec;
use rhwp_codec_resource::VecResource as _;

use crate::error::{FormatError, Result, bail};
use crate::jp2::ImageBoxes;
use crate::reader::BitReader;

pub(crate) fn parse(boxes: &mut ImageBoxes, data: &[u8]) -> Result<()> {
    rhwp_codec_resource::work(1)?;
    let mut reader = BitReader::new(data);
    let num_entries = reader.read_u16().ok_or(FormatError::InvalidBox)? as usize;
    let num_components = reader.read_byte().ok_or(FormatError::InvalidBox)? as usize;

    if num_entries == 0 || num_components == 0 {
        bail!(FormatError::InvalidBox);
    }

    let mut columns = rhwp_codec_resource::capacity(num_components)?;
    {
        let (__rhwp_iter_1, __rhwp_iter_1_prepaid) =
            rhwp_codec_resource::iterations(0..num_components)?;
        for _ in __rhwp_iter_1 {
            if !__rhwp_iter_1_prepaid {
                rhwp_codec_resource::work(1)?;
            }
            let descriptor = reader.read_byte().ok_or(FormatError::InvalidBox)?;
            let bit_depth = (descriptor & 0x7F)
                .checked_add(1)
                .ok_or(FormatError::InvalidBox)?;
            let is_signed = (descriptor & 0x80) != 0;

            if is_signed {
                bail!(FormatError::InvalidBox);
            }

            columns.rhwp_push(PaletteColumn { bit_depth })?;
        }
    }

    let mut entries = rhwp_codec_resource::capacity(num_entries)?;

    {
        let (__rhwp_iter_4, __rhwp_iter_4_prepaid) =
            rhwp_codec_resource::iterations(0..num_entries)?;
        for _ in __rhwp_iter_4 {
            if !__rhwp_iter_4_prepaid {
                rhwp_codec_resource::work(1)?;
            }
            let mut row = rhwp_codec_resource::capacity(num_components)?;

            {
                let (__rhwp_iter_3, __rhwp_iter_3_prepaid) =
                    rhwp_codec_resource::iterations(&columns)?;
                for column in __rhwp_iter_3 {
                    if !__rhwp_iter_3_prepaid {
                        rhwp_codec_resource::work(1)?;
                    }
                    let num_bytes = (column.bit_depth as usize).div_ceil(8).max(1);
                    let raw_bytes = reader
                        .read_bytes(num_bytes)
                        .ok_or(FormatError::InvalidBox)?;
                    let mut raw_value = 0_u64;
                    {
                        let (__rhwp_iter_2, __rhwp_iter_2_prepaid) =
                            rhwp_codec_resource::iterations(raw_bytes)?;
                        for &byte in __rhwp_iter_2 {
                            if !__rhwp_iter_2_prepaid {
                                rhwp_codec_resource::work(1)?;
                            }
                            raw_value = (raw_value << 8) | byte as u64;
                        }
                    }

                    row.rhwp_push(raw_value)?;
                }
            }

            entries.rhwp_push(row)?;
        }
    }

    boxes.palette = Some(PaletteBox { entries, columns });

    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct PaletteBox {
    pub(crate) entries: Vec<Vec<u64>>,
    pub(crate) columns: Vec<PaletteColumn>,
}

impl PaletteBox {
    #[inline(always)]
    pub(crate) fn map(&self, entry: usize, column: usize) -> Option<u64> {
        rhwp_codec_resource::work(1).ok()?;
        self.entries
            .get(entry)
            .and_then(|row| row.get(column))
            .copied()
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PaletteColumn {
    pub(crate) bit_depth: u8,
}
