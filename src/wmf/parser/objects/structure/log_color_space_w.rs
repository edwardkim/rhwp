use crate::wmf::imports::*;

/// The LogColorSpaceW Object specifies a logical color space, which can be
/// defined by a color profile file with a name consisting of Unicode 16-bit
/// characters.
#[derive(Clone, Debug)]
pub struct LogColorSpaceW {
    /// A 32-bit unsigned integer that specifies the signature of color space
    /// objects. This MUST be set to the value 0x50534F43, which is the ASCII
    /// encoding of the string "PSOC".
    pub signature: u32,
    /// Version (4 bytes): A 32-bit unsigned integer that defines a version
    /// number; it MUST be 0x00000400.
    pub version: u32,
    /// Size (4 bytes): A 32-bit unsigned integer that defines the size of this
    /// object, in bytes.
    pub size: u32,
    /// ColorSpaceType (4 bytes): A 32-bit signed integer that specifies the
    /// color space type. It MUST be defined in the LogicalColorSpace
    /// Enumeration. If this value is LCS_sRGB or LCS_WINDOWS_COLOR_SPACE, the
    /// sRGB color space MUST be used.
    pub color_space_type: crate::wmf::parser::LogicalColorSpace,
    /// Intent (4 bytes): A 32-bit signed integer that defines the gamut
    /// mapping intent. It MUST be defined in the GamutMappingIntent
    /// Enumeration.
    pub intent: crate::wmf::parser::GamutMappingIntent,
    /// Endpoints (36 bytes): A CIEXYZTriple Object that defines the CIE
    /// chromaticity x, y, and z coordinates of the three colors that
    /// correspond to the RGB endpoints for the logical color space associated
    /// with the bitmap. If the ColorSpaceType field does not specify
    /// LCS_CALIBRATED_RGB, this field MUST be ignored.
    pub endpoints: crate::wmf::parser::CIEXYZTriple,
    /// GammaRed (4 bytes): A 32-bit fixed point value that defines the toned
    /// response curve for red. If the ColorSpaceType field does not specify
    /// LCS_CALIBRATED_RGB, this field MUST be ignored.
    pub gamma_red: u32,
    /// A 32-bit fixed point value that defines the toned response curve for
    /// green. If the ColorSpaceType field does not specify LCS_CALIBRATED_RGB,
    /// this field MUST be ignored.
    pub gamma_green: u32,
    /// GammaBlue (4 bytes): A 32-bit fixed point value that defines the toned
    /// response curve for blue. If the ColorSpaceType field does not specify
    /// LCS_CALIBRATED_RGB, this field MUST be ignored.
    pub gamma_blue: u32,
    /// Filename (520 bytes): An optional, null-terminated Unicode UTF16-LE
    /// character string, which specifies the name of a file that contains a
    /// color profile. If a file name is specified, and the ColorSpaceType
    /// field is set to LCS_CALIBRATED_RGB, the other fields of this structure
    /// SHOULD be ignored.
    pub filename: Option<String>,
}

impl LogColorSpaceW {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub fn parse<R: crate::wmf::Read>(
        buf: &mut R,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let (
            (signature, signature_bytes),
            (version, version_bytes),
            (size, size_bytes),
            (color_space_type, color_space_type_bytes),
            (intent, intent_bytes),
            (endpoints, endpoints_bytes),
            (gamma_red, gamma_red_bytes),
            (gamma_green, gamma_green_bytes),
            (gamma_blue, gamma_blue_bytes),
        ) = (
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
            crate::wmf::parser::LogicalColorSpace::parse(buf)?,
            crate::wmf::parser::GamutMappingIntent::parse(buf)?,
            crate::wmf::parser::CIEXYZTriple::parse(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
        );
        let mut consumed_bytes = signature_bytes
            + version_bytes
            + size_bytes
            + color_space_type_bytes
            + intent_bytes
            + endpoints_bytes
            + gamma_red_bytes
            + gamma_green_bytes
            + gamma_blue_bytes;

        let filename = if size as usize - consumed_bytes >= 520 {
            let (bytes, filename_bytes) = crate::wmf::parser::read_variable(buf, 520)?;
            consumed_bytes += filename_bytes;
            let len = bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len());

            Some(crate::wmf::parser::objects::structure::utf16le_bytes_to_string(&bytes[..len])?)
        } else {
            None
        };

        if signature != 0x50534F43 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: "The signature field must be 0x50534F43".to_owned(),
            });
        }

        if version != 0x00000400 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: "The version field must be 0x00000400".to_owned(),
            });
        }

        Ok((
            Self {
                signature,
                version,
                size,
                color_space_type,
                intent,
                endpoints,
                gamma_red,
                gamma_green,
                gamma_blue,
                filename,
            },
            consumed_bytes,
        ))
    }
}
