use crate::wmf::imports::*;

/// The META_TEXTOUT Record outputs a character string at the specified location
/// by using the font, background color, and text color that are defined in the
/// playback device context.
#[derive(Clone, Debug)]
pub struct META_TEXTOUT {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_TEXTOUT.
    pub record_function: u16,
    /// StringLength (2 bytes): A 16-bit signed integer that defines the length
    /// of the string, in bytes, pointed to by String.
    pub string_length: i16,
    /// String (variable): The size of this field MUST be a multiple of two. If
    /// StringLength is an odd number, then this field MUST be of a size
    /// greater than or equal to StringLength + 1. A variable-length string
    /// that specifies the text to be drawn. The string does not need to be
    /// null-terminated, because StringLength specifies the length of the
    /// string. The string is written at the location specified by the XStart
    /// and YStart fields.
    pub string: Vec<u8>,
    /// YStart (2 bytes): A 16-bit signed integer that defines the vertical
    /// (y-axis) coordinate, in logical units, of the point where drawing is to
    /// start.
    pub y_start: i16,
    /// XStart (2 bytes): A 16-bit signed integer that defines the horizontal
    /// (x-axis) coordinate, in logical units, of the point where drawing is to
    /// start.
    pub x_start: i16,
}

impl META_TEXTOUT {
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
            crate::wmf::parser::RecordType::META_TEXTOUT,
        )?;

        let (string_length, string_length_bytes) = crate::wmf::parser::read_i16_from_le_bytes(buf)?;
        record_size.consume(string_length_bytes);

        // `string_length` 는 i16 라 손상된 WMF 가 음수를 담을 수 있다. 음수를
        // `as usize` 로 넓히면 usize::MAX 근처가 되어 `read_variable`(내부
        // `vec![0u8; len]`)가 capacity overflow 로 패닉한다(-1 → 18446744073709551615).
        // #3875 가 POLYLINE/POLYGON 에 넣은 가드와 같은 클래스다.
        if string_length < 0 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: format!("The string_length field `{string_length}` must not be negative"),
            });
        }
        // usize 로 넓힌 뒤 홀수 보정을 더한다 — i16 에서 `32767 + 1` 은 오버플로라
        // (debug 패닉 / release wrap→음수→huge) 넓힌 뒤 더해야 안전하다.
        let string_len = string_length as usize + (string_length as usize % 2);

        let ((string, string_bytes), (y_start, y_start_bytes), (x_start, x_start_bytes)) = (
            crate::wmf::parser::read_variable(buf, string_len as usize)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
        );
        record_size.consume(string_bytes + y_start_bytes + x_start_bytes);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            string_length,
            string,
            y_start,
            x_start,
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
