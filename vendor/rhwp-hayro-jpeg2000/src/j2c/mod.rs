mod arithmetic_decoder;
mod bitplane;
mod build;
mod codestream;
mod decode;
mod idwt;
mod mct;
mod progression;
mod rect;
mod segment;
mod tag_tree;
mod tile;

use super::jp2::ImageBoxes;
use super::jp2::colr::{ColorSpace, ColorSpecificationBox, EnumeratedColorspace};
use crate::error::{FormatError, MarkerError, Result, bail};
use crate::j2c::codestream::markers;
use crate::reader::BitReader;
use crate::{DecodeSettings, Image, resolve_alpha_and_color_space};

use crate::math::{SIMD_WIDTH, SimdBuffer};
pub(crate) use codestream::Header;
pub use decode::DecoderContext;
pub(crate) use decode::decode;

pub(crate) struct ParsedCodestream<'a> {
    pub(crate) header: Header<'a>,
    pub(crate) data: &'a [u8],
}

/// Decoded data for one JPEG2000 component.
#[derive(Debug, Clone)]
pub struct ComponentData {
    pub(crate) container: SimdBuffer<{ SIMD_WIDTH }>,
    pub(crate) bit_depth: u8,
}

impl ComponentData {
    /// The bit depth of this component.
    pub fn bit_depth(&self) -> u8 {
        self.bit_depth
    }

    /// The decoded samples of this component.
    pub fn samples(&self) -> &[f32] {
        self.container.truncated()
    }
}

pub(crate) fn parse<'a>(stream: &'a [u8], settings: &DecodeSettings) -> Result<Image<'a>> {
    rhwp_codec_resource::work(1)?;
    let parsed_codestream = parse_raw(stream, settings)?;
    let header = &parsed_codestream.header;
    let mut boxes = ImageBoxes::default();

    // If we are just decoding a raw codestream, we assume greyscale or
    // RGB.
    let cs = if header.component_infos.len() < 3 {
        ColorSpace::Enumerated(EnumeratedColorspace::Greyscale)
    } else {
        ColorSpace::Enumerated(EnumeratedColorspace::Srgb)
    };

    boxes.color_specification = Some(ColorSpecificationBox { color_space: cs });

    let (color_space, has_alpha) =
        resolve_alpha_and_color_space(&boxes, &parsed_codestream.header, settings)?;

    Ok(Image {
        codestream: parsed_codestream.data,
        header: parsed_codestream.header,
        boxes,
        settings: *settings,
        color_space,
        has_alpha,
    })
}

pub(crate) fn parse_raw<'a>(
    stream: &'a [u8],
    settings: &DecodeSettings,
) -> Result<ParsedCodestream<'a>> {
    rhwp_codec_resource::work(1)?;
    let mut reader = BitReader::new(stream);

    let marker = reader.read_marker()?;
    if marker != markers::SOC {
        bail!(MarkerError::Expected("SOC"));
    }

    let header = codestream::read_header(&mut reader, settings)?;
    let code_stream_data = reader.tail().ok_or(FormatError::MissingCodestream)?;

    Ok(ParsedCodestream {
        header,
        data: code_stream_data,
    })
}
