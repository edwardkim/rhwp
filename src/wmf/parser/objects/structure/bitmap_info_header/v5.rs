/// The BitmapV5Header Object contains information about the dimensions and
/// color format of a device-independent bitmap (DIB). It is an extension
/// of the BitmapV4Header Object. (Windows NT 3.1, Windows NT 3.5, Windows
/// NT 3.51, Windows 95, and Windows NT 4.0: This structure is not
/// supported.)
#[derive(Clone, Debug)]
pub struct BitmapInfoHeaderV5 {
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
    /// RedMask (4 bytes): A 32-bit unsigned integer that defines the color
    /// mask that specifies the red component of each pixel. If the
    /// Compression value in the BitmapInfoHeader object is not
    /// BI_BITFIELDS, this value MUST be ignored.
    pub red_mask: u32,
    /// GreenMask (4 bytes): A 32-bit unsigned integer that defines the
    /// color mask that specifies the green component of each pixel. If the
    /// Compression value in the BitmapInfoHeader object is not
    /// BI_BITFIELDS, this value MUST be ignored.
    pub green_mask: u32,
    /// BlueMask (4 bytes): A 32-bit unsigned integer that defines the
    /// color mask that specifies the blue component of each pixel. If the
    /// Compression value in the BitmapInfoHeader object is not
    /// BI_BITFIELDS, this value MUST be ignored.
    pub blue_mask: u32,
    /// AlphaMask (4 bytes): A 32-bit unsigned integer that defines the
    /// color mask that specifies the alpha component of each pixel.
    pub alpha_mask: u32,
    /// ColorSpaceType (4 bytes): A 32-bit unsigned integer that defines
    /// the color space of the DeviceIndependentBitmap Object. If this
    /// value is LCS_CALIBRATED_RGB from the LogicalColorSpace Enumeration,
    /// the color values in the DIB are calibrated RGB values, and the
    /// endpoints and gamma values in this structure SHOULD be used to
    /// translate the color values before they are passed to the device.
    ///
    /// See the LogColorSpace and LogColorSpace ObjectW objects for details
    /// concerning a logical color space.
    pub color_space_type: crate::wmf::parser::LogicalColorSpace,
    /// Endpoints (36 bytes): A CIEXYZTriple Object that defines the CIE
    /// chromaticity x, y, and z coordinates of the three colors that
    /// correspond to the red, green, and blue endpoints for the logical
    /// color space associated with the DIB. If the ColorSpaceType field
    /// does not specify LCS_CALIBRATED_RGB, this field MUST be ignored.
    pub endpoints: crate::wmf::parser::CIEXYZTriple,
    /// GammaRed (4 bytes): A 32-bit fixed point value that defines the
    /// toned response curve for red. If the ColorSpaceType field does not
    /// specify LCS_CALIBRATED_RGB, this field MUST be ignored.
    pub gamma_red: u32,
    /// GammaGreen (4 bytes): A 32-bit fixed point value that defines the
    /// toned response curve for green. If the ColorSpaceType field does
    /// not specify LCS_CALIBRATED_RGB, this field MUST be ignored.
    pub gamma_green: u32,
    /// GammaBlue (4 bytes): A 32-bit fixed point value that defines the
    /// toned response curve for blue. If the ColorSpaceType field does not
    /// specify LCS_CALIBRATED_RGB, this field MUST be ignored.
    ///
    /// The gamma value format is an unsigned "8.8" fixed-point integer
    /// that is then left-shifted by 8 bits. "8.8" means "8 integer bits
    /// followed by 8 fraction bits": nnnnnnnnffffffff. Taking the shift
    /// into account, the required format of the 32-bit DWORD is:
    /// 00000000nnnnnnnnffffffff00000000.
    pub gamma_blue: u32,
    /// Intent (4 bytes): A 32-bit unsigned integer that defines the
    /// rendering intent for the DIB. This MUST be a value defined in the
    /// GamutMappingIntent Enumeration.
    pub intent: crate::wmf::parser::GamutMappingIntent,
    /// ProfileData (4 bytes): A 32-bit unsigned integer that defines the
    /// offset, in bytes, from the beginning of this structure to the start
    /// of the color profile data.
    ///
    /// If the color profile is embedded in the DIB, ProfileData is the
    /// offset to the actual color profile; if the color profile is linked,
    /// ProfileData is the offset to the null-terminated file name of the
    /// color profile. This MUST NOT be a Unicode string, but MUST be
    /// composed exclusively of characters from the Windows character set
    /// (code page 1252).
    ///
    /// If the ColorSpaceType field in the BitmapV4Header does not specify
    /// LCS_PROFILE_LINKED or LCS_PROFILE_EMBEDDED, the color profile data
    /// SHOULD be ignored.
    pub profile_data: u32,
    /// ProfileSize (4 bytes): A 32-bit unsigned integer that defines the
    /// size, in bytes, of embedded color profile data.
    pub profile_size: u32,
    /// Reserved (4 bytes): A 32-bit unsigned integer that is undefined and
    /// SHOULD be ignored.
    pub reserved: u32,
}

impl BitmapInfoHeaderV5 {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub fn parse<R: crate::wmf::Read>(
        buf: &mut R,
        header_size: u32,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let (
            (header, header_bytes),
            (intent, intent_bytes),
            (profile_data, profile_data_bytes),
            (profile_size, profile_size_bytes),
            (reserved, reserved_bytes),
        ) = (
            crate::wmf::parser::BitmapInfoHeaderV4::parse(buf, header_size)?,
            crate::wmf::parser::GamutMappingIntent::parse(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
        );
        let consumed_bytes =
            profile_data_bytes + profile_size_bytes + reserved_bytes + header_bytes + intent_bytes;

        let crate::wmf::parser::BitmapInfoHeaderV4 {
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
            mut red_mask,
            mut green_mask,
            mut blue_mask,
            alpha_mask,
            color_space_type,
            endpoints,
            gamma_red,
            gamma_green,
            gamma_blue,
        } = header;

        if matches!(
            bit_count,
            crate::wmf::parser::BitCount::BI_BITCOUNT_4
                | crate::wmf::parser::BitCount::BI_BITCOUNT_6
        ) && matches!(compression, crate::wmf::parser::Compression::BI_RGB)
        {
            // set default bit fields
            match bit_count {
                crate::wmf::parser::BitCount::BI_BITCOUNT_4 => {
                    red_mask = 0x00007C00;
                    green_mask = 0x000003E0;
                    blue_mask = 0x0000001F;
                }
                crate::wmf::parser::BitCount::BI_BITCOUNT_6 => {
                    red_mask = 0x00FF0000;
                    green_mask = 0x0000FF00;
                    blue_mask = 0x000000FF;
                }
                _ => {}
            }
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
                red_mask,
                green_mask,
                blue_mask,
                alpha_mask,
                color_space_type,
                endpoints,
                gamma_red,
                gamma_green,
                gamma_blue,
                intent,
                profile_data,
                profile_size,
                reserved,
            },
            consumed_bytes,
        ))
    }
}
