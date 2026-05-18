/// The META_SETBKMODE Record defines the background raster operation mix mode
/// in the playback device context. The background mix mode is the mode for
/// combining pens, text, hatched brushes, and interiors of filled objects with
/// background colors on the output surface.
#[derive(Clone, Debug)]
pub struct META_SETBKMODE {
    /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the number
    /// of WORD structures, defined in [MS-DTYP] section 2.2.61, in the WMF
    /// record.
    pub record_size: crate::wmf::parser::RecordSize,
    /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines this
    /// WMF record type. The lower byte MUST match the lower byte of the
    /// RecordType Enumeration table value META_SETBKMODE.
    pub record_function: u16,
    /// BkMode (2 bytes): A 16-bit unsigned integer that defines background mix
    /// mode. This MUST be one of the values in the MixMode Enumeration.
    pub bk_mode: crate::wmf::parser::MixMode,
    /// Reserved (2 bytes): An optional 16-bit field that MUST be ignored.
    /// (Windows NT 3.1, Windows NT 3.5, and Windows NT 3.51 implementations do
    /// not write this field to WMF metafiles.)
    pub reserved: Option<[u8; 2]>,
}

impl META_SETBKMODE {
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
            crate::wmf::parser::RecordType::META_SETBKMODE,
        )?;

        let (bk_mode, bk_mode_bytes) = crate::wmf::parser::MixMode::parse(buf)?;
        record_size.consume(bk_mode_bytes);

        let reserved = if record_size.byte_count() > 8 {
            let (v, c) = crate::wmf::parser::read::<R, 2>(buf)?;

            record_size.consume(c);
            Some(v)
        } else {
            None
        };

        crate::wmf::parser::records::consume_remaining_bytes(buf, record_size)?;

        Ok(Self {
            record_size,
            record_function,
            bk_mode,
            reserved,
        })
    }
}
