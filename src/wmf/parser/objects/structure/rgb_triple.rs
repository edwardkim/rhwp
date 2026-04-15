/// The RGBTriple Object defines the pixel color values in an uncompressed DIB
/// Object.
#[derive(Clone, Debug)]
pub struct RGBTriple {
    /// Red (1 byte): An 8-bit unsigned integer that defines the relative
    /// intensity of red.
    pub red: u8,
    /// Green (1 byte): An 8-bit unsigned integer that defines the relative
    /// intensity of green.
    pub green: u8,
    /// Blue (1 byte): An 8-bit unsigned integer that defines the relative
    /// intensity of blue.
    pub blue: u8,
}

impl RGBTriple {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub fn parse<R: crate::wmf::Read>(
        buf: &mut R,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let ((red, red_bytes), (green, green_bytes), (blue, blue_bytes)) = (
            crate::wmf::parser::read_u8_from_le_bytes(buf)?,
            crate::wmf::parser::read_u8_from_le_bytes(buf)?,
            crate::wmf::parser::read_u8_from_le_bytes(buf)?,
        );

        Ok((
            Self { red, green, blue },
            red_bytes + green_bytes + blue_bytes,
        ))
    }
}
