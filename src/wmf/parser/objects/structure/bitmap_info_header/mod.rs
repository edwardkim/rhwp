mod core;
mod info;
mod v4;
mod v5;

pub use self::{core::*, info::*, v4::*, v5::*};

#[derive(Clone, Debug)]
pub enum BitmapInfoHeader {
    Core(BitmapInfoHeaderCore),
    Info(BitmapInfoHeaderInfo),
    V4(BitmapInfoHeaderV4),
    V5(BitmapInfoHeaderV5),
}

impl BitmapInfoHeader {
    pub fn parse<R: crate::wmf::Read>(
        buf: &mut R,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let (header_size, mut consumed_bytes) = crate::wmf::parser::read_u32_from_le_bytes(buf)?;

        match header_size {
            0x0000000C => {
                let (header, c) = BitmapInfoHeaderCore::parse(buf, header_size)?;
                consumed_bytes += c;

                Ok((Self::Core(header), consumed_bytes))
            }
            13..=40 => {
                let (header, c) = BitmapInfoHeaderInfo::parse(buf, header_size)?;
                consumed_bytes += c;

                Ok((Self::Info(header), consumed_bytes))
            }
            41..=108 => {
                let (header, c) = BitmapInfoHeaderV4::parse(buf, header_size)?;
                consumed_bytes += c;

                Ok((Self::V4(header), consumed_bytes))
            }
            109..=124 => {
                let (header, c) = BitmapInfoHeaderV5::parse(buf, header_size)?;
                consumed_bytes += c;

                Ok((Self::V5(header), consumed_bytes))
            }
            _ => Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: format!(
                    "The header_size `{header_size:#10X}` field is not match \
                     as any BitmapInfoHeader format"
                ),
            }),
        }
    }

    pub fn header_size(&self) -> u32 {
        match self {
            Self::Core(BitmapInfoHeaderCore { header_size, .. })
            | Self::Info(BitmapInfoHeaderInfo { header_size, .. })
            | Self::V4(BitmapInfoHeaderV4 { header_size, .. })
            | Self::V5(BitmapInfoHeaderV5 { header_size, .. }) => *header_size,
        }
    }

    pub fn bit_count(&self) -> crate::wmf::parser::BitCount {
        match self {
            Self::Core(BitmapInfoHeaderCore { bit_count, .. })
            | Self::Info(BitmapInfoHeaderInfo { bit_count, .. })
            | Self::V4(BitmapInfoHeaderV4 { bit_count, .. })
            | Self::V5(BitmapInfoHeaderV5 { bit_count, .. }) => *bit_count,
        }
    }

    pub fn size(&self) -> usize {
        let size = match self {
            Self::Core(BitmapInfoHeaderCore {
                width,
                height,
                planes,
                bit_count,
                ..
            }) => u32::from((((width * planes * (*bit_count as u16) + 31) & !31) / 8) * height),
            Self::Info(BitmapInfoHeaderInfo {
                width,
                height,
                planes,
                bit_count,
                image_size,
                compression,
                ..
            })
            | Self::V4(BitmapInfoHeaderV4 {
                width,
                height,
                planes,
                bit_count,
                image_size,
                compression,
                ..
            })
            | Self::V5(BitmapInfoHeaderV5 {
                width,
                height,
                planes,
                bit_count,
                image_size,
                compression,
                ..
            }) => match compression {
                crate::wmf::parser::Compression::BI_RGB
                | crate::wmf::parser::Compression::BI_BITFIELDS
                | crate::wmf::parser::Compression::BI_CMYK => {
                    ((((*width as u32) * u32::from(*planes) * (*bit_count as u32) + 31) & !31) / 8)
                        * height.unsigned_abs()
                }
                _ => *image_size,
            },
        };

        size as usize
    }

    pub fn color_used(&self) -> u32 {
        match self {
            Self::Core(BitmapInfoHeaderCore { bit_count, .. }) => 2u32.pow(*bit_count as u32),
            Self::Info(BitmapInfoHeaderInfo {
                bit_count,
                color_used,
                ..
            })
            | Self::V4(BitmapInfoHeaderV4 {
                bit_count,
                color_used,
                ..
            })
            | Self::V5(BitmapInfoHeaderV5 {
                bit_count,
                color_used,
                ..
            }) => {
                if *color_used == 0
                    && matches!(
                        bit_count,
                        crate::wmf::parser::BitCount::BI_BITCOUNT_1
                            | crate::wmf::parser::BitCount::BI_BITCOUNT_2
                            | crate::wmf::parser::BitCount::BI_BITCOUNT_3
                    )
                {
                    2u32.pow(*bit_count as u32)
                } else {
                    *color_used
                }
            }
        }
    }

    pub fn height(&self) -> usize {
        match self {
            Self::Core(BitmapInfoHeaderCore { height, .. }) => usize::from(*height),
            Self::Info(BitmapInfoHeaderInfo { height, .. })
            | Self::V4(BitmapInfoHeaderV4 { height, .. })
            | Self::V5(BitmapInfoHeaderV5 { height, .. }) => *height as usize,
        }
    }

    pub fn width(&self) -> usize {
        match self {
            Self::Core(BitmapInfoHeaderCore { width, .. }) => usize::from(*width),
            Self::Info(BitmapInfoHeaderInfo { width, .. })
            | Self::V4(BitmapInfoHeaderV4 { width, .. })
            | Self::V5(BitmapInfoHeaderV5 { width, .. }) => *width as usize,
        }
    }
}
