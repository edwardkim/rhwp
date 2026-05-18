/// The META_SETMAPPERFLAGS Record defines the algorithm that the font mapper
/// uses when it maps logical fonts to physical fonts.
#[derive(Clone, Debug)]
pub struct META_SETMAPPERFLAGS {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_SETMAPPERFLAGS.
    pub record_function: u16,
    /// MapperValues (4 bytes): A 32-bit unsigned integer that defines whether
    /// the font mapper attempts to match a font aspect ratio to the current
    /// device aspect ratio. If bit zero is set, the mapper selects only
    /// matching fonts.
    pub mapper_values: u32,
}

impl META_SETMAPPERFLAGS {
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
            crate::wmf::parser::RecordType::META_SETMAPPERFLAGS,
        )?;

        let (mapper_values, mapper_values_bytes) = crate::wmf::parser::read_u32_from_le_bytes(buf)?;
        record_size.consume(mapper_values_bytes);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            mapper_values,
        })
    }
}
