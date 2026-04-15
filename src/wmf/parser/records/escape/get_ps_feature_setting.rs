impl crate::wmf::parser::META_ESCAPE {
    pub(in crate::wmf::parser::records::escape) fn parse_as_GET_PS_FEATURESETTING<
        R: crate::wmf::Read,
    >(
        buf: &mut R,
        mut record_size: crate::wmf::parser::RecordSize,
        record_function: u16,
    ) -> Result<Self, crate::wmf::parser::ParseError> {
        let ((byte_count, byte_count_bytes), (feature, feature_bytes)) = (
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::PostScriptFeatureSetting::parse(buf)?,
        );
        record_size.consume(byte_count_bytes + feature_bytes);

        if byte_count != 0x0004 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: format!("The byte_count `{byte_count:#06X}` field must be `0x0004`",),
            });
        }

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self::GET_PS_FEATURESETTING {
            record_size,
            record_function,
            byte_count,
            feature,
        })
    }
}
