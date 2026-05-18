/// The META_SETTEXTALIGN Record defines text-alignment values in the playback
/// device context.
#[derive(Clone, Debug)]
pub struct META_SETTEXTALIGN {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_SETTEXTALIGN.
    pub record_function: u16,
    /// TextAlignmentMode (2 bytes): A 16-bit unsigned integer that defines
    /// text alignment. This value MUST be a combination of one or more
    /// TextAlignmentMode Flags for text with a horizontal baseline, and
    /// VerticalTextAlignmentMode Flags for text with a vertical baseline.
    pub text_alignment_mode: u16,
    /// Reserved (2 bytes): An optional 16-bit field that MUST be ignored.
    /// (Windows NT 3.1, Windows NT 3.5, and Windows NT 3.51 implementations do
    /// not write this field to WMF metafiles.)
    pub reserved: Option<[u8; 2]>,
}

impl META_SETTEXTALIGN {
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
            crate::wmf::parser::RecordType::META_SETTEXTALIGN,
        )?;

        let (text_alignment_mode, text_alignment_mode_bytes) =
            crate::wmf::parser::read_u16_from_le_bytes(buf)?;
        record_size.consume(text_alignment_mode_bytes);

        let reserved = if record_size.byte_count() > 8 {
            let (v, c) = crate::wmf::parser::read::<R, 2>(buf)?;

            record_size.consume(c);
            Some(v)
        } else {
            None
        };

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            text_alignment_mode,
            reserved,
        })
    }
}
