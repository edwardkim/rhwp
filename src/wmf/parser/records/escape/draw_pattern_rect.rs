impl crate::wmf::parser::META_ESCAPE {
    pub(in crate::wmf::parser::records::escape) fn parse_as_DRAWPATTERNRECT<R: crate::wmf::Read>(
        buf: &mut R,
        mut record_size: crate::wmf::parser::RecordSize,
        record_function: u16,
    ) -> Result<Self, crate::wmf::parser::ParseError> {
        let (
            (byte_count, byte_count_bytes),
            (position, position_bytes),
            (size, size_bytes),
            (style, style_bytes),
            (pattern, pattern_bytes),
        ) = (
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::PointL::parse(buf)?,
            crate::wmf::parser::PointL::parse(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
        );
        record_size
            .consume(byte_count_bytes + position_bytes + size_bytes + style_bytes + pattern_bytes);

        if byte_count != 0x0014 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: format!("The byte_count `{byte_count:#06X}` field must be `0x0014`",),
            });
        }

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self::DRAWPATTERNRECT {
            record_size,
            record_function,
            byte_count,
            position,
            size,
            style,
            pattern,
        })
    }
}
