/// The META_CREATEPENINDIRECT Record creates a Pen Object.
#[derive(Clone, Debug)]
pub struct META_CREATEPENINDIRECT {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_CREATEPENINDIRECT.
    pub record_function: u16,
    /// Pen (10 bytes): Pen Object data that defines the pen to create.
    pub pen: crate::wmf::parser::Pen,
}

impl META_CREATEPENINDIRECT {
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
            crate::wmf::parser::RecordType::META_CREATEPENINDIRECT,
        )?;

        let (pen, pen_bytes) = crate::wmf::parser::Pen::parse(buf)?;
        record_size.consume(pen_bytes);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            pen,
        })
    }
}
