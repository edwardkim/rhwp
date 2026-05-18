use crate::wmf::imports::*;

/// The META_HEADER Record is the first record in a standard (nonplaceable) WMF
/// metafile.
#[derive(Clone, Debug)]
pub struct META_HEADER {
    /// Type (2 bytes): A 16-bit unsigned integer that defines the type of
    /// metafile. It MUST be a value in the MetafileType Enumeration.
    pub typ: crate::wmf::parser::MetafileType,
    /// HeaderSize (2 bytes): A 16-bit unsigned integer that defines the number
    /// of 16-bit WORD structures, defined in [MS-DTYP] section 2.2.61, in the
    /// header.
    pub header_size: u16,
    /// Version (2 bytes): A 16-bit unsigned integer that defines the metafile
    /// version. It MUST be a value in the MetafileVersion Enumeration.
    /// (Metafiles created by Windows contain the value METAVERSION300.)
    pub version: crate::wmf::parser::MetafileVersion,
    /// SizeLow (2 bytes): A 16-bit unsigned integer that defines the low-order
    /// word of the number of 16-bit WORD structures in the entire metafile.
    pub size_low: u16,
    /// SizeHigh (2 bytes): A 16-bit unsigned integer that defines the
    /// high-order word of the number of 16-bit WORD structures in the entire
    /// metafile.
    pub size_high: u16,
    /// NumberOfObjects (2 bytes): A 16-bit unsigned integer that specifies the
    /// number of graphics objects that are defined in the entire metafile.
    /// These objects include brushes, pens, and the other objects specified in
    /// section 2.2.1.
    pub number_of_objects: u16,
    /// MaxRecord (4 bytes): A 32-bit unsigned integer that specifies the size
    /// of the largest record used in the metafile (in 16-bit elements).
    pub max_record: u32,
    /// NumberOfMembers (2 bytes): A 16-bit unsigned integer that is not used.
    /// It SHOULD be 0x0000.
    pub number_of_members: u16,
}

impl META_HEADER {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        fields(key = %format!("{key:#010X}")),
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub(in crate::wmf::parser::records::control) fn parse<R: crate::wmf::Read>(
        buf: &mut R,
        key: u32,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let bytes: [u8; 4] = key.to_le_bytes();
        let mut b = &bytes[..];
        let ((typ, mut consumed_bytes), (header_size, _)) = (
            crate::wmf::parser::MetafileType::parse(&mut b)?,
            crate::wmf::parser::read_u16_from_le_bytes(&mut b)?,
        );

        let (
            (version, version_bytes),
            (size_low, size_low_bytes),
            (size_high, size_high_bytes),
            (number_of_objects, number_of_objects_bytes),
            (max_record, max_record_bytes),
            (number_of_members, number_of_members_bytes),
        ) = (
            crate::wmf::parser::MetafileVersion::parse(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
        );
        consumed_bytes += version_bytes
            + size_low_bytes
            + size_high_bytes
            + number_of_objects_bytes
            + max_record_bytes
            + number_of_members_bytes;

        if number_of_members != 0x0000 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: "The number_of_members field should be `0x0000`".to_owned(),
            });
        }

        Ok((
            Self {
                typ,
                header_size,
                version,
                size_low,
                size_high,
                number_of_objects,
                max_record,
                number_of_members,
            },
            consumed_bytes,
        ))
    }
}
