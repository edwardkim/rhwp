use crate::wmf::imports::*;

/// The Bitmap16 Object specifies information about the dimensions and color
/// format of a bitmap.
///
/// Bitmap16 object seems to be Windows DDB.
#[derive(Clone)]
pub struct Bitmap16 {
    /// Type (2 bytes): A 16-bit signed integer that defines the bitmap type.
    pub typ: i16,
    /// Width (2 bytes): A 16-bit signed integer that defines the width of the
    /// bitmap in pixels.
    pub width: i16,
    /// Height (2 bytes): A 16-bit signed integer that defines the height of
    /// the bitmap in scan lines.
    pub height: i16,
    /// WidthBytes (2 bytes): A 16-bit signed integer that defines the number
    /// of bytes per scan line.
    pub width_bytes: i16,
    /// Planes (1 byte): An 8-bit unsigned integer that defines the number of
    /// color planes in the bitmap. The value of this field MUST be 0x01.
    pub planes: u8,
    /// BitsPixel (1 byte): An 8-bit unsigned integer that defines the number
    /// of adjacent color bits on each plane.
    pub bits_pixel: crate::wmf::parser::BitCount,
    /// Bits (variable): A variable length array of bytes that defines the
    /// bitmap pixel data. The length of this field in bytes can be computed as
    /// follows.
    ///
    /// ```text
    /// (((Width * BitsPixel + 15) >> 4) << 1) * Height
    /// ```
    pub bits: Vec<u8>,
}

impl core::fmt::Debug for Bitmap16 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Bitmap16")
            .field("typ", &self.typ)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("width_bytes", &self.width_bytes)
            .field("planes", &self.planes)
            .field("bits_pixel", &self.bits_pixel)
            .field("bits", &format!("[u8; {}]", self.bits.len()))
            .finish()
    }
}

impl Bitmap16 {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub fn parse<R: crate::wmf::Read>(
        buf: &mut R,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let (mut bitmap, mut consumed_bytes) = Self::parse_without_bits(buf)?;
        let (bits, bits_bytes) = crate::wmf::parser::read_variable(buf, bitmap.calc_length())?;

        bitmap.bits = bits;
        consumed_bytes += bits_bytes;

        Ok((bitmap, consumed_bytes))
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub fn parse_without_bits<R: crate::wmf::Read>(
        buf: &mut R,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let (
            (typ, typ_bytes),
            (width, width_consumed_bytes),
            (height, height_bytes),
            (width_bytes, width_bytes_consumed_bytes),
            (planes, planes_bytes),
            (bits_pixel, bits_pixel_bytes),
        ) = (
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u8_from_le_bytes(buf)?,
            crate::wmf::parser::read_u8_from_le_bytes(buf)?,
        );
        let consumed_bytes = typ_bytes
            + width_consumed_bytes
            + height_bytes
            + width_bytes_consumed_bytes
            + planes_bytes
            + bits_pixel_bytes;

        if planes != 0x01 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: "The planes field must be 0x01".to_owned(),
            });
        }

        let (bits_pixel, _) = {
            let u16_bytes = u16::from(bits_pixel).to_le_bytes().to_vec();
            let mut b = &u16_bytes[..];

            crate::wmf::parser::BitCount::parse(&mut b)?
        };

        Ok((
            Self {
                typ,
                width,
                height,
                width_bytes,
                planes,
                bits_pixel,
                bits: vec![],
            },
            consumed_bytes,
        ))
    }

    pub fn calc_length(&self) -> usize {
        ((((self.width * self.bits_pixel as i16 + 15) >> 4) << 1) * self.height) as usize
    }
}

impl From<Bitmap16> for crate::wmf::parser::DeviceIndependentBitmap {
    fn from(v: Bitmap16) -> Self {
        Self {
            dib_header_info: crate::wmf::parser::BitmapInfoHeader::Core(
                crate::wmf::parser::BitmapInfoHeaderCore {
                    header_size: 12,
                    width: v.width as u16,
                    height: v.height as u16,
                    planes: v.planes.into(),
                    bit_count: v.bits_pixel,
                },
            ),
            colors: crate::wmf::parser::Colors::Null,
            bitmap_buffer: crate::wmf::parser::BitmapBuffer {
                undefined_space: vec![],
                a_data: v.bits,
            },
        }
    }
}
