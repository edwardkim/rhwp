/// The META_INVERTREGION Record draws a region in which the colors are
/// inverted.
#[derive(Clone, Debug)]
pub struct META_INVERTREGION {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_INVERTREGION.
    pub record_function: u16,
    /// Region (2 bytes): A 16-bit unsigned integer used to index into the WMF
    /// Object Table to get the region to be inverted.
    pub region: u16,
}

impl META_INVERTREGION {
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
            crate::wmf::parser::RecordType::META_INVERTREGION,
        )?;

        let (region, region_bytes) = crate::wmf::parser::read_u16_from_le_bytes(buf)?;
        record_size.consume(region_bytes);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            region,
        })
    }
}
