//! The channel definition box (cdef), defined in I.5.3.6.

use alloc::vec::Vec;
use rhwp_codec_resource::VecResource as _;

use crate::error::{FormatError, Result, bail};
use crate::jp2::ImageBoxes;
use crate::reader::BitReader;

pub(crate) fn parse(boxes: &mut ImageBoxes, data: &[u8]) -> Result<()> {
    rhwp_codec_resource::work(1)?;
    let mut reader = BitReader::new(data);
    let count = reader.read_u16().ok_or(FormatError::InvalidBox)? as usize;
    let mut definitions = rhwp_codec_resource::capacity(count)?;

    if count == 0 {
        bail!(FormatError::InvalidBox);
    }

    {
        let (__rhwp_iter_1, __rhwp_iter_1_prepaid) = rhwp_codec_resource::iterations(0..count)?;
        for _ in __rhwp_iter_1 {
            if !__rhwp_iter_1_prepaid {
                rhwp_codec_resource::work(1)?;
            }
            let channel_index = reader.read_u16().ok_or(FormatError::InvalidBox)?;
            let channel_type = reader.read_u16().ok_or(FormatError::InvalidBox)?;
            let association = reader.read_u16().ok_or(FormatError::InvalidBox)?;

            definitions.rhwp_push(ChannelDefinition {
                channel_index,
                channel_type: ChannelType::from_raw(channel_type).ok_or(FormatError::InvalidBox)?,
                _association: ChannelAssociation::from_raw(association)
                    .ok_or(FormatError::InvalidBox)?,
            })?;
        }
    }

    rhwp_codec_resource::sort(&mut definitions, |a, b| {
        a.channel_index.cmp(&b.channel_index)
    })?;

    // Ensure channel indices increases in steps of 1, starting from 0.
    {
        let (__rhwp_iter_2, __rhwp_iter_2_prepaid) =
            rhwp_codec_resource::iterations(definitions.iter().enumerate())?;
        for (idx, def) in __rhwp_iter_2 {
            if !__rhwp_iter_2_prepaid {
                rhwp_codec_resource::work(1)?;
            }
            if def.channel_index as usize != idx {
                bail!(FormatError::InvalidBox);
            }
        }
    }

    boxes.channel_definition = Some(ChannelDefinitionBox {
        channel_definitions: definitions,
    });

    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct ChannelDefinitionBox {
    pub(crate) channel_definitions: Vec<ChannelDefinition>,
}

#[derive(Debug, Clone)]
pub(crate) struct ChannelDefinition {
    pub(crate) channel_index: u16,
    pub(crate) channel_type: ChannelType,
    pub(crate) _association: ChannelAssociation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChannelType {
    Colour,
    Opacity,
}

impl ChannelType {
    fn from_raw(value: u16) -> Option<Self> {
        rhwp_codec_resource::work(1).ok()?;
        match value {
            0 => Some(Self::Colour),
            1 => Some(Self::Opacity),
            // We don't support the others.
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChannelAssociation {
    WholeImage,
    Colour(u16),
}

impl ChannelAssociation {
    fn from_raw(value: u16) -> Option<Self> {
        rhwp_codec_resource::work(1).ok()?;
        match value {
            0 => Some(Self::WholeImage),
            // Unspecified.
            u16::MAX => None,
            v => Some(Self::Colour(v)),
        }
    }
}
