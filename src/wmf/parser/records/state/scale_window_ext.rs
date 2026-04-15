/// The META_SCALEWINDOWEXT Record scales the horizontal and vertical extents of
/// the output window that is defined in the playback device context by using
/// the ratios formed by specified multiplicands and divisors.
#[derive(Clone, Debug)]
pub struct META_SCALEWINDOWEXT {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_SCALEWINDOWEXT.
    pub record_function: u16,
    /// yDenom (2 bytes): A 16-bit signed integer that defines the amount by
    /// which to divide the result of multiplying the current y-extent by the
    /// value of the yNum member.
    pub y_denom: i16,
    /// yNum (2 bytes): A 16-bit signed integer that defines the amount by
    /// which to multiply the current y-extent.
    pub y_num: i16,
    /// xDenom (2 bytes): A 16-bit signed integer that defines the amount by
    /// which to divide the result of multiplying the current x-extent by the
    /// value of the xNum member.
    pub x_denom: i16,
    /// xNum (2 bytes): A 16-bit signed integer that defines the amount by
    /// which to multiply the current x-extent.
    pub x_num: i16,
}

impl META_SCALEWINDOWEXT {
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
            crate::wmf::parser::RecordType::META_SCALEWINDOWEXT,
        )?;

        let (
            (y_denom, y_denom_bytes),
            (y_num, y_num_bytes),
            (x_denom, x_denom_bytes),
            (x_num, x_num_bytes),
        ) = (
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
            crate::wmf::parser::read_i16_from_le_bytes(buf)?,
        );
        record_size.consume(y_denom_bytes + y_num_bytes + x_denom_bytes + x_num_bytes);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            y_denom,
            y_num,
            x_denom,
            x_num,
        })
    }
}
