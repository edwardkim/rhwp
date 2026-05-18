use crate::wmf::imports::*;

/// The META_EXTTEXTOUT Record outputs text by using the font, background color,
/// and text color that are defined in the playback device context. Optionally,
/// dimensions can be provided for clipping, opaquing, or both.
#[derive(Clone, Debug)]
pub struct META_EXTTEXTOUT {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_EXTTEXTOUT.
    pub record_function: u16,
    /// Y (2 bytes): A 16-bit signed integer that defines the y-coordinate, in
    /// logical units, where the text string is to be located.
    pub y: i16,
    /// X (2 bytes): A 16-bit signed integer that defines the x-coordinate, in
    /// logical units, where the text string is to be located.
    pub x: i16,
    /// StringLength (2 bytes): A 16-bit signed integer that defines the length
    /// of the string.
    pub string_length: i16,
    /// fwOpts (2 bytes): A 16-bit unsigned integer that defines the use of the
    /// application-defined rectangle. This member can be a combination of one
    /// or more values in the ExtTextOutOptions Flags.
    pub fw_opts: BTreeSet<crate::wmf::parser::ExtTextOutOptions>,
    /// Rectangle (8 bytes): An optional 8-byte Rect Object. When either
    /// ETO_CLIPPED, ETO_OPAQUE, or both are specified, the rectangle defines
    /// the dimensions, in logical coordinates, used for clipping, opaquing, or
    /// both. When neither ETO_CLIPPED nor ETO_OPAQUE is specified, the
    /// coordinates in Rectangle are ignored.
    pub rectangle: Option<crate::wmf::parser::Rect>,
    /// String (variable): A variable-length string that specifies the text to
    /// be drawn. The string does not need to be null-terminated, because
    /// StringLength specifies the length of the string. If the length is odd,
    /// an extra byte is placed after it so that the following member (optional
    /// Dx) is aligned on a 16-bit boundary. The string will be decoded based
    /// on the font object currently selected into the playback device context.
    /// If a font matching the font object’s specification is not found, the
    /// decoding is undefined. If a matching font is found that matches the
    /// charset specified in the font object, the string should be decoded with
    /// the codepages in the following table.
    pub string: Vec<u8>,
    /// Dx (variable): An optional array of 16-bit signed integers that
    /// indicate the distance between origins of adjacent character cells. For
    /// example, Dx[i] logical units separate the origins of character cell i
    /// and character cell i + 1. If this field is present, there MUST be the
    /// same number of values as there are characters in the string.
    pub dx: Vec<i16>,
}

impl META_EXTTEXTOUT {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        fields(
            %record_size,
            record_function = %format!("{record_function:#06X}"),
        ),
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub fn parse<R: crate::wmf::Read>(
        buf: &mut R,
        mut record_size: crate::wmf::parser::RecordSize,
        record_function: u16,
    ) -> Result<Self, crate::wmf::parser::ParseError> {
        crate::wmf::parser::records::check_lower_byte_matches(
            record_function,
            crate::wmf::parser::RecordType::META_EXTTEXTOUT,
        )?;

        #[rustfmt::skip]
        let (
            (y, y_bytes),
            (x, x_bytes),
            (string_length, string_length_bytes),
        ) = (
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
        );
        record_size.consume(y_bytes + x_bytes + string_length_bytes);

        let fw_opts = {
            let (value, c) = crate::wmf::parser::read_u16_from_le_bytes(buf)?;
            record_size.consume(c);

            let mut fw_opts = BTreeSet::new();

            for v in [
                crate::wmf::parser::ExtTextOutOptions::ETO_OPAQUE,
                crate::wmf::parser::ExtTextOutOptions::ETO_CLIPPED,
                crate::wmf::parser::ExtTextOutOptions::ETO_GLYPH_INDEX,
                crate::wmf::parser::ExtTextOutOptions::ETO_RTLREADING,
                crate::wmf::parser::ExtTextOutOptions::ETO_NUMERICSLOCAL,
                crate::wmf::parser::ExtTextOutOptions::ETO_NUMERICSLATIN,
                crate::wmf::parser::ExtTextOutOptions::ETO_PDY,
            ] {
                if value & (v as u16) == v as u16 {
                    fw_opts.insert(v);
                }
            }

            fw_opts
        };

        let rectangle = if fw_opts.contains(&crate::wmf::parser::ExtTextOutOptions::ETO_OPAQUE)
            || fw_opts.contains(&crate::wmf::parser::ExtTextOutOptions::ETO_CLIPPED)
        {
            let (v, c) = crate::wmf::parser::Rect::parse(buf)?;
            record_size.consume(c);

            Some(v)
        } else {
            None
        };

        let (string, string_bytes) =
            crate::wmf::parser::read_variable(buf, string_length as usize)?;
        record_size.consume(string_bytes);

        // ignore odd bytes
        if string_length % 2 != 0 {
            let _ = crate::wmf::parser::read::<R, 1>(buf)?;
            record_size.consume(1);
        }

        let mut dx = vec![];

        if record_size.remaining() {
            dx.reserve_exact(string_length as usize);

            for _ in 0..string_length {
                let (v, c) = crate::wmf::parser::read_i16_from_le_bytes(buf)?;

                record_size.consume(c);
                dx.push(v);
            }
        }

        let (_, c) = crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;
        record_size.consume(c);

        Ok(Self {
            record_size,
            record_function,
            y,
            x,
            string_length,
            fw_opts,
            rectangle,
            string,
            dx,
        })
    }

    /// Converts the string to UTF-8 using the specified character set.
    ///
    /// # Arguments
    ///
    /// - `charset` - The character set to use for conversion.
    ///
    /// # Returns
    ///
    /// A UTF-8 string, or `ParseError` if decoding fails.
    pub fn into_utf8(
        &self,
        charset: crate::wmf::parser::CharacterSet,
    ) -> Result<String, crate::wmf::parser::ParseError> {
        crate::wmf::parser::bytes_into_utf8(&self.string, charset)
    }
}
