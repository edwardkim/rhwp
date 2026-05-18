use crate::wmf::imports::*;

/// The Palette Object specifies the colors in a logical palette.
#[derive(Clone, Debug)]
pub struct Palette {
    /// Start (2 bytes): A 16-bit unsigned integer that defines the offset into
    /// the Palette Object when used with the META_SETPALENTRIES and
    /// META_ANIMATEPALETTE record types. When used with META_CREATEPALETTE
    /// record type, it MUST be 0x0300.
    pub start: u16,
    /// NumberOfEntries (2 bytes): A 16-bit unsigned integer that defines the
    /// number of objects in aPaletteEntries.
    pub number_of_entries: u16,
    /// aPaletteEntries (variable): An array of NumberOfEntries 32-bit
    /// PaletteEntry Objects.
    pub a_palette_entries: Vec<crate::wmf::parser::PaletteEntry>,
}

impl Palette {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub fn parse<R: crate::wmf::Read>(
        buf: &mut R,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let ((start, start_bytes), (number_of_entries, number_of_entries_bytes)) = (
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
        );

        let mut consumed_bytes = start_bytes + number_of_entries_bytes;
        let mut a_palette_entries = Vec::with_capacity(number_of_entries as usize);

        for _ in 0..number_of_entries {
            let (v, c) = crate::wmf::parser::PaletteEntry::parse(buf)?;

            consumed_bytes += c;
            a_palette_entries.push(v);
        }

        Ok((
            Self {
                start,
                number_of_entries,
                a_palette_entries,
            },
            consumed_bytes,
        ))
    }
}
