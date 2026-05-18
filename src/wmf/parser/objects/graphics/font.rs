use crate::wmf::imports::*;

/// The Font object specifies the attributes of a logical font.
#[derive(Clone, Debug)]
pub struct Font {
    /// Height (2 bytes): A 16-bit signed integer that specifies the height, in
    /// logical units, of the font's character cell. The character height is
    /// computed as the character cell height minus the internal leading.
    ///
    /// For all height comparisons, the font mapper SHOULD find the largest
    /// physical font that does not exceed the requested size.
    ///
    /// All Windows versions: mapping the logical font size to the available
    /// physical fonts occurs the first time the logical font needs to be used
    /// in a drawing operation. For the MM_TEXT mapping mode, the following
    /// formula can be used to compute the height of a font with a specified
    /// point size.
    ///
    /// ```text
    /// Height = -MulDiv(PointSize, GetDeviceCaps(hDC, LOGPIXELSY), 72);
    /// ```
    pub height: i16,
    /// Width (2 bytes): A 16-bit signed integer that defines the average
    /// width, in logical units, of characters in the font. If Width is 0x0000,
    /// the aspect ratio of the device SHOULD be matched against the
    /// digitization aspect ratio of the available fonts to find the closest
    /// match, determined by the absolute value of the difference.
    pub width: i16,
    /// Escapement (2 bytes): A 16-bit signed integer that defines the angle,
    /// in tenths of degrees, between the escapement vector and the x-axis of
    /// the device. The escapement vector is parallel to the base line of a row
    /// of text.
    pub escapement: i16,
    /// Orientation (2 bytes): A 16-bit signed integer that defines the angle,
    /// in tenths of degrees, between each character's base line and the x-axis
    /// of the device.
    pub orientation: i16,
    /// Weight (2 bytes): A 16-bit signed integer that defines the weight of
    /// the font in the range 0 through 1000. For example, 400 is normal and
    /// 700 is bold. If this value is 0x0000, a default weight SHOULD be used.
    pub weight: i16,
    /// Italic (1 byte): A 8-bit Boolean value that specifies the italic
    /// attribute of the font.
    pub italic: bool,
    /// Underline (1 byte): An 8-bit Boolean value that specifies the underline
    /// attribute of the font.
    pub underline: bool,
    /// StrikeOut (1 byte): An 8-bit Boolean value that specifies the strikeout
    /// attribute of the font.
    pub strike_out: bool,
    /// CharSet (1 byte): An 8-bit unsigned integer that defines the character
    /// set. It SHOULD be set to a value in the CharacterSet Enumeration.
    ///
    /// The DEFAULT_CHARSET value MAY be used to allow the name and size of a
    /// font to fully describe the logical font. If the specified font name
    /// does not exist, a font in another character set MAY be substituted. The
    /// DEFAULT_CHARSET value is set to a value based on the current system
    /// locale. For example, when the system locale is United States, it is set
    /// to ANSI_CHARSET.
    ///
    /// If a typeface name in the FaceName field is specified, the CharSet
    /// value MUST match the character set of that typeface.
    pub charset: crate::wmf::parser::CharacterSet,
    /// OutPrecision (1 byte): An 8-bit unsigned integer that defines the
    /// output precision. The output precision defines how closely the output
    /// matches the requested font height, width, character orientation,
    /// escapement, pitch, and font type. It MUST be one of the values from the
    /// OutPrecision Enumeration.
    ///
    /// Applications can use the OUT_DEVICE_PRECIS, OUT_RASTER_PRECIS,
    /// OUT_TT_PRECIS, and OUT_PS_ONLY_PRECIS values to control how the font
    /// mapper selects a font when the operating system contains more than one
    /// font with a specified name. For example, if an operating system
    /// contains a font named "Symbol" in raster and TrueType forms, specifying
    /// OUT_TT_PRECIS forces the font mapper to select the TrueType version.
    /// Specifying OUT_TT_ONLY_PRECIS forces the font mapper to select a
    /// TrueType font, even if it substitutes a TrueType font of another name.
    pub out_precision: crate::wmf::parser::OutPrecision,
    /// ClipPrecision (1 byte): An 8-bit unsigned integer that defines the
    /// clipping precision. The clipping precision defines how to clip
    /// characters that are partially outside the clipping region. It MUST be a
    /// combination of one or more of the bit settings in the ClipPrecision
    /// Flags.
    pub clip_precision: crate::wmf::parser::ClipPrecision,
    /// Quality (1 byte): An 8-bit unsigned integer that defines the output
    /// quality. The output quality defines how carefully to attempt to match
    /// the logical font attributes to those of an actual physical font. It
    /// MUST be one of the values in the FontQuality Enumeration.
    pub quality: crate::wmf::parser::FontQuality,
    /// PitchAndFamily (1 byte): A PitchAndFamily Object that defines the pitch
    /// and the family of the font. Font families specify the look of fonts in
    /// a general way and are intended for specifying fonts when the exact
    /// typeface wanted is not available.
    pub pitch_and_family: crate::wmf::parser::PitchAndFamily,
    /// Facename (32 bytes): A null-terminated string of up to 32 8-bit Latin-1
    /// [ISO/IEC-8859-1] ANSI characters that specifies the typeface name of
    /// the font. Any characters following the terminating null are ignored.
    pub facename: String,
    /// Fallback facename: The facename interpreted according to the
    /// specified charset. This is extra field to help with font matching.
    pub fallback_facename: Vec<String>,
}

impl Font {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub fn parse<R: crate::wmf::Read>(
        buf: &mut R,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let (
            (height, height_bytes),
            (width, width_bytes),
            (escapement, escapement_bytes),
            (orientation, orientation_bytes),
            (weight, weight_bytes),
            (italic, italic_bytes),
            (underline, underline_bytes),
            (strike_out, strike_out_bytes),
            (charset, charset_bytes),
            (out_precision, out_precision_bytes),
            (clip_precision, clip_precision_bytes),
            (quality, quality_bytes),
            (pitch_and_family, pitch_and_family_bytes),
        ) = (
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            {
                let (v, c) = crate::wmf::parser::read_u8_from_le_bytes(buf)?;
                (v == 0x01, c)
            },
            {
                let (v, c) = crate::wmf::parser::read_u8_from_le_bytes(buf)?;
                (v == 0x01, c)
            },
            {
                let (v, c) = crate::wmf::parser::read_u8_from_le_bytes(buf)?;
                (v == 0x01, c)
            },
            crate::wmf::parser::CharacterSet::parse(buf)?,
            crate::wmf::parser::OutPrecision::parse(buf)?,
            crate::wmf::parser::ClipPrecision::parse(buf)?,
            crate::wmf::parser::FontQuality::parse(buf)?,
            crate::wmf::parser::PitchAndFamily::parse(buf)?,
        );
        let mut consumed_bytes = height_bytes
            + width_bytes
            + escapement_bytes
            + orientation_bytes
            + weight_bytes
            + italic_bytes
            + underline_bytes
            + strike_out_bytes
            + charset_bytes
            + out_precision_bytes
            + clip_precision_bytes
            + quality_bytes
            + pitch_and_family_bytes;

        let (facename, facename_as_charset) = {
            let mut bytes = vec![0; 32];
            let c = buf.read(&mut bytes).map_err(|err| {
                crate::wmf::parser::ParseError::UnexpectedPattern {
                    cause: format!("{err:?}"),
                }
            })?;
            consumed_bytes += c;

            // Find the position of the first null byte (0)
            let len = bytes.iter().position(|&c| c == 0).unwrap_or(32);

            // Convert bytes to UTF-8 string from Latin-1
            let as_latin1 = crate::wmf::parser::bytes_into_utf8(
                &bytes[..len],
                crate::wmf::parser::CharacterSet::ANSI_CHARSET,
            )?;

            // Convert bytes to UTF-8 string from specified charset
            let as_charset = crate::wmf::parser::bytes_into_utf8(&bytes[..len], charset)?;

            // WMF spec: facename 은 Latin-1 ANSI character 만 허용. 그러나 한컴
            // 등 multi-byte charset (HANGUL_CHARSET, SHIFTJIS_CHARSET 등) WMF 는
            // CP949/SJIS 등 binary 그대로 facename 에 넣음 — spec 위반이나 실제
            // 한컴 WMF 의 일반적 패턴. charset 이 ANSI 가 아니면 charset 으로
            // 해석한 결과를 primary 로 사용.
            // (sample16 paragraph 394 WMF 의 facename "굴림체" 가 Latin-1 으로
            // 해석되어 "±¼¸²Ã¼" 깨진 글자로 표시되던 회귀.)
            if charset != crate::wmf::parser::CharacterSet::ANSI_CHARSET {
                (as_charset, as_latin1)
            } else {
                (as_latin1, as_charset)
            }
        };

        let mut fallback_facename = Vec::new();

        if facename != facename_as_charset {
            fallback_facename.push(facename_as_charset);
        }

        // if facename is `Symbol`, set charset `SYMBOL_CHARSET`.
        let charset = if facename.to_ascii_lowercase().contains("symbol")
            || fallback_facename
                .iter()
                .any(|f| f.to_ascii_lowercase().contains("symbol"))
        {
            let symbol = "Symbol".to_string();
            if facename != symbol && !fallback_facename.contains(&symbol) {
                fallback_facename.push(symbol);
            }

            crate::wmf::parser::CharacterSet::SYMBOL_CHARSET
        } else {
            charset
        };

        Ok((
            Self {
                height,
                width,
                escapement,
                orientation,
                weight,
                italic,
                underline,
                strike_out,
                charset,
                out_precision,
                clip_precision,
                quality,
                pitch_and_family,
                facename,
                fallback_facename,
            },
            consumed_bytes,
        ))
    }
}
