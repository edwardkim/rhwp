/// The META_SELECTOBJECT Record specifies a graphics object for the playback
/// device context.
///
/// The new object replaces the previous object of the same type, unless if the
/// previous object is a palette object. If the previous object is a Palette
/// Object, then the META_SELECTPALETTE Record MUST be used instead of the
/// META_SELECTOBJECT Record, because the META_SELECTOBJECT Record does not
/// support replacing the Palette Object type.
#[derive(Clone, Debug)]
pub struct META_SELECTOBJECT {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// record type. The lower byte MUST match the lower byte of the RecordType
    /// Enumeration table value META_SELECTOBJECT.
    pub record_function: u16,
    /// ObjectIndex (2 bytes): A 16-bit unsigned integer used to index into the
    /// WMF Object Table to get the object to be selected.
    pub object_index: u16,
}

impl META_SELECTOBJECT {
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
            crate::wmf::parser::RecordType::META_SELECTOBJECT,
        )?;

        let (object_index, object_index_bytes) = crate::wmf::parser::read_u16_from_le_bytes(buf)?;
        record_size.consume(object_index_bytes);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            object_index,
        })
    }
}
