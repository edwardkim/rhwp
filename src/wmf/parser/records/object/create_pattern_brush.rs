use crate::wmf::imports::*;

/// The META_CREATEPATTERNBRUSH Record creates a brush object with a pattern
/// specified by a bitmap. (The META_CREATEPATTERNBRUSH record type is
/// deprecated. No version of Windows emits this record. Playback implementation
/// of this record in Windows is intended solely for compatibility purposes so
/// that Windows metafiles containing this record can be rendered.)
#[derive(Clone)]
pub struct META_CREATEPATTERNBRUSH {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_CREATEPATTERNBRUSH.
    pub record_function: u16,
    /// Bitmap16 (14 bytes): A partial Bitmap16 Object, which defines
    /// parameters for the bitmap that specifies the pattern for the brush.
    pub bitmap16: crate::wmf::parser::Bitmap16,
    /// Reserved (18 bytes): This field MUST be ignored.
    pub reserved: [u8; 18],
    /// Pattern (variable): A variable-length array of bytes that defines the
    /// bitmap pixel data that composes the brush pattern. The length of this
    /// field, in bytes, can be computed from bitmap parameters as follows.
    ///
    /// ```text
    /// (((Width * BitsPixel + 15) >> 4) << 1) * Height
    /// ```
    pub pattern: Vec<u8>,
}

impl core::fmt::Debug for META_CREATEPATTERNBRUSH {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("META_CREATEPATTERNBRUSH")
            .field("record_size", &self.record_size)
            .field("record_function", &self.record_function)
            .field("bitmap16", &self.bitmap16)
            .field("reserved", &self.reserved)
            .field("pattern", &format!("[u8; {}]", self.pattern.len()))
            .finish()
    }
}

impl META_CREATEPATTERNBRUSH {
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
            crate::wmf::parser::RecordType::META_CREATEPATTERNBRUSH,
        )?;

        let (bitmap16, bitmap16_bytes) = crate::wmf::parser::Bitmap16::parse_without_bits(buf)?;
        let (_, ignored_bytes) = crate::wmf::parser::read_variable(buf, 14 - bitmap16_bytes)?;
        let (reserved, reserved_bytes) = crate::wmf::parser::read::<R, 18>(buf)?;
        record_size.consume(bitmap16_bytes + ignored_bytes + reserved_bytes);

        let (pattern, pattern_bytes) =
            crate::wmf::parser::read_variable(buf, bitmap16.calc_length())?;
        record_size.consume(pattern_bytes);

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            bitmap16,
            reserved,
            pattern,
        })
    }

    pub fn create_brush(&self) -> crate::wmf::parser::Brush {
        let mut brush_hatch = self.bitmap16.clone();
        brush_hatch.bits.clone_from(&self.pattern);

        crate::wmf::parser::Brush::Pattern { brush_hatch }
    }
}
