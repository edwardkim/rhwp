/// The META_RESTOREDC Record restores the playback device context from a
/// previously saved device context.
#[derive(Clone, Debug)]
pub struct META_RESTOREDC {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_RESTOREDC.
    pub record_function: u16,
    /// nSavedDC (2 bytes): A 16-bit signed integer that defines the saved
    /// state to be restored. If this member is positive, nSavedDC represents a
    /// specific instance of the state to be restored. If this member is
    /// negative, nSavedDC represents an instance relative to the current
    /// state.
    pub n_saved_dc: i16,
}

impl META_RESTOREDC {
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
            crate::wmf::parser::RecordType::META_RESTOREDC,
        )?;

        let (n_saved_dc, n_saved_dc_bytes) = crate::wmf::parser::read_i16_from_le_bytes(buf)?;
        record_size.consume(n_saved_dc_bytes);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            n_saved_dc,
        })
    }
}
