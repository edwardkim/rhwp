/// The META_CREATEFONTINDIRECT Record creates a Font Object.
#[derive(Clone, Debug)]
pub struct META_CREATEFONTINDIRECT {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_CREATEFONTINDIRECT.
    pub record_function: u16,
    /// Font (variable): Font Object data that defines the font to create.
    pub font: crate::wmf::parser::Font,
}

impl META_CREATEFONTINDIRECT {
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
            crate::wmf::parser::RecordType::META_CREATEFONTINDIRECT,
        )?;

        let (b, c) = crate::wmf::parser::read_variable(buf, record_size.remaining_bytes())?;
        record_size.consume(c);

        let mut buffer = &b[..];
        let (font, _) = crate::wmf::parser::Font::parse(&mut buffer)?;

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            font,
        })
    }
}
