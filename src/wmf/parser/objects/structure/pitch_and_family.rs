/// The PitchAndFamily Object specifies the pitch and family properties of a
/// Font Object. Pitch refers to the width of the characters, and family refers
/// to the general appearance of a font.
#[derive(Clone, Debug)]
pub struct PitchAndFamily {
    /// Family (4 bits): A property of a font that describes its general
    /// appearance. This MUST be a value in the FamilyFont Enumeration.
    pub family: crate::wmf::parser::FamilyFont,
    /// Pitch (2 bits): A property of a font that describes the pitch, of the
    /// characters. This MUST be a value in the PitchFont Enumeration.
    pub pitch: crate::wmf::parser::PitchFont,
}

impl PitchAndFamily {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub fn parse<R: crate::wmf::Read>(
        buf: &mut R,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let (byte, consumed_bytes) = crate::wmf::parser::read_u8_from_le_bytes(buf)?;

        let family = byte >> 4;
        let Some(family) = crate::wmf::parser::FamilyFont::from_repr(byte >> 4) else {
            return Err(crate::wmf::parser::ParseError::UnexpectedEnumValue {
                cause: format!("unexpected value as FamilyFont: {family:#04X}"),
            });
        };

        let pitch = byte & 0b00000011;
        let Some(pitch) = crate::wmf::parser::PitchFont::from_repr(pitch) else {
            return Err(crate::wmf::parser::ParseError::UnexpectedEnumValue {
                cause: format!("unexpected value as PitchFont: {pitch:#04X}"),
            });
        };

        Ok((Self { family, pitch }, consumed_bytes))
    }
}
