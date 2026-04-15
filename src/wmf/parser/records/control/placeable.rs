/// The META_PLACEABLE Record is the first record in a placeable WMF metafile,
/// which is an extension to the WMF metafile format. (Windows NT 3.1, Windows
/// NT 3.5, Windows NT 3.51, and Windows 95: This feature is not supported.) The
/// information in this extension allows the specification of the placement and
/// size of the target image, which makes it adaptable to different output
/// devices.
///
/// The META_PLACEABLE MUST be the first record of the metafile, located
/// immediately before the META_HEADER Record
#[derive(Clone, Debug)]
pub struct META_PLACEABLE {
    /// Key (4 bytes): Identification value that indicates the presence of a
    /// placeable metafile header. This value MUST be 0x9AC6CDD7.
    pub key: u32,
    /// HWmf (2 bytes): The resource handle to the metafile, when the metafile
    /// is in memory. When the metafile is on disk, this field MUST contain
    /// 0x0000. This attribute of the metafile is specified in the Type field
    /// of the META_HEADER Record.
    pub hwmf: u16,
    /// BoundingBox (8 bytes): The rectangle in the playback context (or simply
    /// the destination rectangle), measured in logical units, for displaying
    /// the metafile. The size of a logical unit is specified by the Inch
    /// field.
    pub bounding_box: crate::wmf::parser::Rect,
    /// Inch (2 bytes): The number of logical units per inch used to represent
    /// the image. This value can be used to scale an image.
    ///
    /// By convention, an image is considered to be recorded at 1440 logical
    /// units (twips) per inch. Thus, a value of 720 specifies that the image
    /// SHOULD be rendered at twice its normal size, and a value of 2880
    /// specifies that the image SHOULD be rendered at half its normal size.
    pub inch: u16,
    /// Reserved (4 bytes): A field that is not used and MUST be set to
    /// 0x00000000.
    pub reserved: u32,
    /// Checksum (2 bytes): A checksum for the previous 10 16-bit values in the
    /// header. This value can be used to determine whether the metafile has
    /// become corrupted. The value is calculated by initializing the checksum
    /// to zero and then XORing it one at a time with the 10 16-bit values in
    /// the header.
    pub checksum: [u8; 2],
}

impl META_PLACEABLE {
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
        let (
            (hwmf, hwmf_bytes),
            (bounding_box, bounding_box_bytes),
            (inch, inch_bytes),
            (reserved, reserved_bytes),
            (checksum, checksum_bytes),
        ) = (
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::Rect::parse(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u32_from_le_bytes(buf)?,
            crate::wmf::parser::read::<R, 2>(buf)?,
        );
        let consumed_bytes =
            hwmf_bytes + bounding_box_bytes + inch_bytes + reserved_bytes + checksum_bytes;

        Ok((
            Self {
                key,
                hwmf,
                bounding_box,
                inch,
                reserved,
                checksum,
            },
            consumed_bytes,
        ))
    }
}
