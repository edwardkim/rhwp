/// The META_DELETEOBJECT Record deletes an object, including the Bitmap16
/// Object , Brush Object, DeviceIndependentBitmap Object, Font Object, Palette
/// Object, Pen Object, and Region Object. After the object is deleted, its
/// index in the WMF Object Table is no longer valid but is available to be
/// reused.
#[derive(Clone, Debug)]
pub struct META_DELETEOBJECT {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// record type. The lower byte MUST match the lower byte of the RecordType
    /// Enumeration table value META_DELETEOBJECT.
    pub record_function: u16,
    /// ObjectIndex (2 bytes): A 16-bit unsigned integer used to index into the
    /// WMF Object Table to get the object to be deleted.
    pub object_index: u16,
}

impl META_DELETEOBJECT {
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
            crate::wmf::parser::RecordType::META_DELETEOBJECT,
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
