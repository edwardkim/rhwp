//! The component mapping box (cmap), defined in I.5.3.5.

use alloc::vec::Vec;
use rhwp_codec_resource::VecResource as _;

use crate::error::{FormatError, Result, bail};
use crate::jp2::ImageBoxes;
use crate::reader::BitReader;

pub(crate) fn parse(boxes: &mut ImageBoxes, data: &[u8]) -> Result<()> {
    rhwp_codec_resource::work(1)?;
    let mut reader = BitReader::new(data);
    let mut entries = rhwp_codec_resource::capacity(data.len() / 4)?;

    while !reader.at_end() {
        rhwp_codec_resource::work(1)?;
        let component_index = reader.read_u16().ok_or(FormatError::InvalidBox)?;
        let mapping_type = reader.read_byte().ok_or(FormatError::InvalidBox)?;
        let palette_column = reader.read_byte().ok_or(FormatError::InvalidBox)?;

        let mapping_type = match mapping_type {
            0 => ComponentMappingType::Direct,
            1 => ComponentMappingType::Palette {
                column: palette_column,
            },
            _ => bail!(FormatError::InvalidBox),
        };

        entries.rhwp_push(ComponentMappingEntry {
            component_index,
            mapping_type,
        })?;
    }

    boxes.component_mapping = Some(ComponentMappingBox { entries });

    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct ComponentMappingBox {
    pub(crate) entries: Vec<ComponentMappingEntry>,
}

#[derive(Debug, Clone)]
pub(crate) struct ComponentMappingEntry {
    pub(crate) component_index: u16,
    pub(crate) mapping_type: ComponentMappingType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ComponentMappingType {
    Direct,
    Palette { column: u8 },
}
