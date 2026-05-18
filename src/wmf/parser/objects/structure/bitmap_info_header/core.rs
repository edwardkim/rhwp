use crate::wmf::imports::*;

/// The BitmapCoreHeader Object contains information about the dimensions
/// and color format of a device-independent bitmap (DIB). (Although
/// Windows processes BitmapCoreHeader objects in DIBs, it does not
/// write them to WMF metafiles)
///
/// A DIB is specified by a DeviceIndependentBitmap Object.
#[derive(Clone, Debug)]
pub struct BitmapInfoHeaderCore {
    /// HeaderSize (4 bytes): A 32-bit unsigned integer that defines the
    /// size of this object, in bytes.
    pub header_size: u32,
    /// Width (2 bytes): A 16-bit unsigned integer that defines the width
    /// of the DIB, in pixels.
    pub width: u16,
    /// Height (2 bytes): A 16-bit unsigned integer that defines the height
    /// of the DIB, in pixels.
    pub height: u16,
    /// Planes (2 bytes): A 16-bit unsigned integer that defines the number
    /// of planes for the target device. This value MUST be 0x0001.
    pub planes: u16,
    /// BitCount (2 bytes): A 16-bit unsigned integer that defines the
    /// format of each pixel, and the maximum number of colors in the DIB.
    /// This value MUST be in the BitCount Enumeration.
    pub bit_count: crate::wmf::parser::BitCount,
}

impl BitmapInfoHeaderCore {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub(super) fn parse<R: crate::wmf::Read>(
        buf: &mut R,
        header_size: u32,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let (
            (width, width_bytes),
            (height, height_bytes),
            (planes, planes_bytes),
            (bit_count, bit_count_bytes),
        ) = (
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::BitCount::parse(buf)?,
        );
        let consumed_bytes = width_bytes + height_bytes + planes_bytes + bit_count_bytes;

        if planes != 0x0001 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: "The planes field must be 0x01".to_owned(),
            });
        }

        if !matches!(
            bit_count,
            crate::wmf::parser::BitCount::BI_BITCOUNT_1
                | crate::wmf::parser::BitCount::BI_BITCOUNT_2
                | crate::wmf::parser::BitCount::BI_BITCOUNT_3
                | crate::wmf::parser::BitCount::BI_BITCOUNT_5
        ) {
            return Err(crate::wmf::parser::ParseError::UnexpectedEnumValue {
                cause: format!("Invalid BitCount `{}` as Core type.", bit_count as u16),
            });
        }

        Ok((
            Self {
                header_size,
                width,
                height,
                planes,
                bit_count,
            },
            consumed_bytes,
        ))
    }
}
