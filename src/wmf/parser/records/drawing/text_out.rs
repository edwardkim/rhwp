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
    ///
    /// [#7130] 여기에는 글자 `string_length` 바이트만 담는다. 홀수 길이일 때
    /// 뒤따르는 WORD 정렬용 채움 1바이트는 읽고 버린다 — 그 바이트는 0 으로
    /// 정해져 있지 않아서 남겨 두면 글자로 그려진다.
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
        // [#7130] `String` 필드가 차지하는 자리는 짝수로 올림한 크기지만, 글자는
        // 앞 `string_length` 바이트뿐이다(MS-WMF §2.3.5.6 — `StringLength` 는 글자
        // 수이고, 홀수면 뒤 1바이트는 WORD 정렬용 채움이다). 종전에는 올림한 크기를
        // 통째로 `string` 에 담아 채움 바이트까지 `into_utf8` 이 글자로 바꿨다.
        // 채움 바이트는 0 으로 정해져 있지 않다 — 한/글이 내보낸 OLE 회로도에서는
        // `Emitter` 뒤 `0x6F`, `VCC`·`10V` 뒤 `0x31`, `COM`·`GND` 뒤 `0x5E`·`0x4B`
        // 처럼 버퍼에 남아 있던 값이 그대로 들어와 라벨 끝에 글자가 하나 더 붙었다.
        // 같은 저장소의 `META_EXTTEXTOUT` 은 처음부터 이 방식이다.
        let (string, string_bytes) =
            crate::wmf::parser::read_variable(buf, string_length as usize)?;
        record_size.consume(string_bytes);

        // ignore odd bytes
        if string_length % 2 != 0 {
            let _ = crate::wmf::parser::read::<R, 1>(buf)?;
            record_size.consume(1);
        }

        let ((y_start, y_start_bytes), (x_start, x_start_bytes)) = (
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
        );
        record_size.consume(y_start_bytes + x_start_bytes);

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
