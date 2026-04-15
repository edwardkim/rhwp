impl crate::wmf::parser::META_ESCAPE {
    pub(in crate::wmf::parser::records::escape) fn parse_as_GETCOLORTABLE<R: crate::wmf::Read>(
        buf: &mut R,
        mut record_size: crate::wmf::parser::RecordSize,
        record_function: u16,
    ) -> Result<Self, crate::wmf::parser::ParseError> {
        let ((byte_count, byte_count_bytes), (start, start_bytes)) = (
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
        );
        record_size.consume(byte_count_bytes + start_bytes);
        let (_, c) = crate::wmf::parser::read_variable(buf, start as usize)?;
        record_size.consume(c);

        let (color_table_buffer, c) =
            crate::wmf::parser::read_variable(buf, (byte_count - start) as usize)?;
        record_size.consume(c);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self::GETCOLORTABLE {
            record_size,
            record_function,
            byte_count,
            start,
            color_table_buffer,
        })
    }
}
