impl crate::wmf::parser::META_ESCAPE {
    pub(in crate::wmf::parser::records::escape) fn parse_as_STARTDOC<R: crate::wmf::Read>(
        buf: &mut R,
        mut record_size: crate::wmf::parser::RecordSize,
        record_function: u16,
    ) -> Result<Self, crate::wmf::parser::ParseError> {
        let (byte_count, byte_count_bytes) = crate::wmf::parser::read_u16_from_le_bytes(buf)?;
        record_size.consume(byte_count_bytes);

        if byte_count >= 260 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: format!(
                    "The byte_count `{byte_count}` field must be less than \
                     `260`",
                ),
            });
        }

        let (doc_name, c) = crate::wmf::parser::read_variable(buf, byte_count as usize)?;
        record_size.consume(c);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self::STARTDOC {
            record_size,
            record_function,
            byte_count,
            doc_name,
        })
    }
}
