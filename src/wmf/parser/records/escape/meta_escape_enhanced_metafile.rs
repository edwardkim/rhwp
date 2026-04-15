impl crate::wmf::parser::META_ESCAPE {
    pub(super) fn parse_as_META_ESCAPE_ENHANCED_METAFILE<R: crate::wmf::Read>(
        buf: &mut R,
        mut record_size: crate::wmf::parser::RecordSize,
        record_function: u16,
    ) -> Result<Self, crate::wmf::parser::ParseError> {
        let (
            (byte_count, byte_count_bytes),
            (comment_identifier, comment_identifier_bytes),
            (comment_type, comment_type_bytes),
            (version, version_bytes),
            (checksum, checksum_bytes),
            (flags, flags_bytes),
            (comment_record_count, comment_record_count_bytes),
            (current_record_size, current_record_size_bytes),
            (remaining_bytes, remaining_bytes_bytes),
            (enhanced_metafile_data_size, enhanced_metafile_data_size_bytes),
        ) = (
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
        );
        record_size.consume(
            byte_count_bytes
                + comment_identifier_bytes
                + comment_type_bytes
                + version_bytes
                + checksum_bytes
                + flags_bytes
                + comment_record_count_bytes
                + current_record_size_bytes
                + remaining_bytes_bytes
                + enhanced_metafile_data_size_bytes,
        );

        let expected_byte_count = enhanced_metafile_data_size + 34;

        if u32::from(byte_count) != expected_byte_count {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: format!(
                    "The byte_count `{byte_count:#010X}` field must be same \
                     as `{expected_byte_count:#010X}`",
                ),
            });
        }

        if comment_identifier != 0x43464D57 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: format!(
                    "The comment_identifier `{comment_identifier:#010X}` \
                     field must be `0x43464D57`",
                ),
            });
        }

        if comment_type != 0x00000001 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: format!(
                    "The comment_type `{comment_type:#010X}` field must be \
                     `0x00000001`",
                ),
            });
        }

        if version != 0x00010000 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: format!("The version `{version:#010X}` field must be `0x00010000`"),
            });
        }

        if flags != 0x00000000 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: format!("The flags `{version:#010X}` field must be `0x00000000`",),
            });
        }

        if current_record_size > 8192 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: format!(
                    "The current_record_size `{current_record_size}` field \
                     must be less than or equal to `8192`",
                ),
            });
        }

        let (enhanced_metafile_data, c) =
            crate::wmf::parser::read_variable(buf, enhanced_metafile_data_size as usize)?;
        record_size.consume(c);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self::META_ESCAPE_ENHANCED_METAFILE {
            record_size,
            record_function,
            byte_count,
            comment_identifier,
            comment_type,
            version,
            checksum,
            flags,
            comment_record_count,
            current_record_size,
            remaining_bytes,
            enhanced_metafile_data_size,
            enhanced_metafile_data,
        })
    }
}
