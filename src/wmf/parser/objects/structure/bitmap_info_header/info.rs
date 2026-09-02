use crate::wmf::imports::*;

/// The BitmapInfoHeader Object contains information about the dimensions
/// and color format of a device-independent bitmap (DIB).
#[derive(Clone, Debug)]
pub struct BitmapInfoHeaderInfo {
    /// HeaderSize (4 bytes): A 32-bit unsigned integer that defines the
    /// size of this object, in bytes.
    pub header_size: u32,
    /// Width (4 bytes): A 32-bit signed integer that defines the width of
    /// the DIB, in pixels. This value MUST be positive.
    ///
    /// This field SHOULD specify the width of the decompressed image file,
    /// if the Compression value specifies JPEG or PNG format. (Windows NT
    /// 3.1, Windows NT 3.5, Windows NT 3.51, Windows 95, and Windows NT
    /// 4.0: Neither JPEG nor PNG format is supported.)
    pub width: i32,
    /// Height (4 bytes): A 32-bit signed integer that defines the height
    /// of the DIB, in pixels. This value MUST NOT be zero.
    ///
    /// | Value | Meaning |
    /// |-|-|
    /// | 0x00000000 < value | If this value is positive, the DIB is a bottom-up bitmap, and its origin is the lower-left corner. This field SHOULD specify the height of the decompressed image file, if the Compression value specifies JPEG or PNG format. |
    /// | value < 0x00000000 | If this value is negative, the DIB is a top-down bitmap, and its origin is the upper-left corner. Top-down bitmaps do not support compression. |
    pub height: i32,
    /// Planes (2 bytes): A 16-bit unsigned integer that defines the number
    /// of planes for the target device. This value MUST be 0x0001.
    pub planes: u16,
    /// BitCount (2 bytes): A 16-bit unsigned integer that defines the
    /// number of bits that define each pixel and the maximum number of
    /// colors in the DIB. This value MUST be in the BitCount Enumeration.
    pub bit_count: crate::wmf::parser::BitCount,
    /// Compression (4 bytes): A 32-bit unsigned integer that defines the
    /// compression mode of the DIB. This value MUST be in the Compression
    /// Enumeration.
    ///
    /// This value MUST NOT specify a compressed format if the DIB is a
    /// top-down bitmap, as indicated by the Height value.
    pub compression: crate::wmf::parser::Compression,
    /// ImageSize (4 bytes): A 32-bit unsigned integer that defines the
    /// size, in bytes, of the image.
    ///
    /// If the Compression value is BI_RGB, this value SHOULD be zero and
    /// MUST be ignored. (Windows implementations might write a nonzero
    /// value to this field, but it is ignored when the metafile is
    /// parsed.)
    ///
    /// If the Compression value is BI_JPEG or BI_PNG, this value MUST
    /// specify the size of the JPEG or PNG image buffer, respectively.
    pub image_size: u32,
    /// XPelsPerMeter (4 bytes): A 32-bit signed integer that defines the
    /// horizontal resolution, in pixels-per-meter, of the target device
    /// for the DIB.
    pub x_pels_per_meter: i32,
    /// YPelsPerMeter (4 bytes): A 32-bit signed integer that defines the
    /// vertical resolution, in pixels-per-meter, of the target device for
    /// the DIB.
    pub y_pels_per_meter: i32,
    /// ColorUsed (4 bytes): A 32-bit unsigned integer that specifies the
    /// number of indexes in the color table used by the DIB, as follows:
    ///
    /// - If this value is zero, the DIB uses the maximum number of colors that
    ///   correspond to the BitCount value.
    /// - If this value is nonzero and the BitCount value is less than 16, this
    ///   value specifies the number of colors used by the DIB.
    /// - If this value is nonzero and the BitCount value is 16 or greater,
    ///   this value specifies the size of the color table used to optimize
    ///   performance of the system palette.
    ///
    /// Note If this value is nonzero and greater than the maximum possible
    /// size of the color table based on the BitCount value, the maximum
    /// color table size SHOULD be assumed.
    pub color_used: u32,
    /// ColorImportant (4 bytes): A 32-bit unsigned integer that defines
    /// the number of color indexes that are required for displaying the
    /// DIB. If this value is zero, all color indexes are required.
    ///
    /// A DIB is specified by a DeviceIndependentBitmap Object.
    ///
    /// When the array of pixels in the DIB immediately follows the
    /// BitmapInfoHeader, the DIB is a packed bitmap. In a packed bitmap,
    /// the ColorUsed value MUST be either 0x00000000 or the actual size of
    /// the color table.
    pub color_important: u32,
}

impl BitmapInfoHeaderInfo {
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
            (compression, compression_bytes),
            (image_size, image_size_bytes),
            (x_pels_per_meter, x_pels_per_meter_bytes),
            (y_pels_per_meter, y_pels_per_meter_bytes),
            (color_used, color_used_bytes),
            (color_important, color_important_bytes),
        ) = (
            crate::wmf::parser::read_i32_from_le_bytes(buf)?,
            crate::wmf::parser::read_i32_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::BitCount::parse(buf)?,
            crate::wmf::parser::Compression::parse(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
            crate::wmf::parser::read_i32_from_le_bytes(buf)?,
            crate::wmf::parser::read_i32_from_le_bytes(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
        );
        let consumed_bytes = width_bytes
            + height_bytes
            + planes_bytes
            + image_size_bytes
            + x_pels_per_meter_bytes
            + y_pels_per_meter_bytes
            + color_used_bytes
            + color_important_bytes
            + bit_count_bytes
            + compression_bytes;

        if width <= 0 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: "The width field must be positive".to_owned(),
            });
        }

        if height == 0 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: "The height field must not be zero".to_owned(),
            });
        }

        if planes != 0x0001 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: "The planes field must be 0x01".to_owned(),
            });
        }

        Ok((
            Self {
                header_size,
                width,
                height,
                planes,
                bit_count,
                compression,
                image_size,
                x_pels_per_meter,
                y_pels_per_meter,
                color_used,
                color_important,
            },
            consumed_bytes,
        ))
    }
}

/// Computes the DIB pixel-buffer byte count from header dimensions using the
/// MS-WMF formula `(((Width * Planes * BitCount + 31) & !31) / 8) * abs(Height)`.
///
/// Width/Height/BitCount are attacker-controlled; evaluating the formula in the
/// header's narrow integer types (`u16` for a core header, `u32` for
/// info/v4/v5) overflows on crafted dimensions and panics under debug overflow
/// checks (DoS). Evaluate in `u64` with saturating arithmetic and clamp into
/// `usize`; the caller (`read_variable`) bounds the actual allocation.
pub(super) fn dib_image_size(width: u64, height: u64, planes: u64, bit_count: u64) -> usize {
    let stride = (width
        .saturating_mul(planes)
        .saturating_mul(bit_count)
        .saturating_add(31)
        & !31)
        / 8;

    usize::try_from(stride.saturating_mul(height)).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::dib_image_size;
    use crate::wmf::parser::{BitCount, BitmapInfoHeader, BitmapInfoHeaderCore};

    /// 유효 치수에서는 기존 좁은-정수 공식과 동일한 바이트 수를 낸다(무회귀).
    #[test]
    fn dib_image_size_matches_formula_on_valid_dimensions() {
        // (((640 * 1 * 24 + 31) & !31) / 8) * 480 = 921_600
        assert_eq!(dib_image_size(640, 480, 1, 24), 921_600);
    }

    /// 거대한 치수(u32::MAX)에서도 u64 포화로 패닉하지 않고 유한한 값을 낸다.
    #[test]
    fn dib_image_size_saturates_on_huge_dimensions() {
        let v = dib_image_size(u64::from(u32::MAX), u64::from(u32::MAX), 1, 32);
        assert!(v > 0);
    }

    /// Core 헤더 `size()`는 손상된 치수에서 u16 산술 오버플로로 패닉했었다.
    /// 이제 포화 산술로 유한 값을 돌려준다.
    #[test]
    fn core_header_size_does_not_overflow() {
        let header = BitmapInfoHeader::Core(BitmapInfoHeaderCore {
            header_size: 12,
            width: u16::MAX,
            height: u16::MAX,
            planes: 1,
            bit_count: BitCount::BI_BITCOUNT_5,
        });
        assert!(header.size() > 0);
    }
}
