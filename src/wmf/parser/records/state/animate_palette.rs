/// The META_ANIMATEPALETTE Record redefines entries in the logical palette that
/// is defined in the playback device context with the specified Palette Object.
///
/// The logical palette that is specified by the Palette Object in this record
/// is the source of the palette changes, and the logical palette that is
/// currently selected into the playback device context is the destination.
/// Entries in the destination palette with the PC_RESERVED PaletteEntryFlag
/// Enumeration set SHOULD be modified by this record, and entries with that
/// flag clear SHOULD NOT be modified. If none of the entries in the destination
/// palette have the PC_RESERVED flag set, then this record SHOULD have no
/// effect.
#[derive(Clone, Debug)]
pub struct META_ANIMATEPALETTE {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_ANIMATEPALETTE.
    pub record_function: u16,
    /// Palette (variable): A variable-sized Palette Object that specifies a
    /// logical palette.
    pub palette: crate::wmf::parser::Palette,
}

impl META_ANIMATEPALETTE {
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
            crate::wmf::parser::RecordType::META_ANIMATEPALETTE,
        )?;

        let (palette, palette_bytes) = crate::wmf::parser::Palette::parse(buf)?;
        record_size.consume(palette_bytes);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            palette,
        })
    }
}
