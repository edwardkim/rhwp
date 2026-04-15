/// The ColorUsage Enumeration specifies whether a color table exists in a
/// device-independent bitmap (DIB) and how to interpret its values.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u16)]
pub enum ColorUsage {
    /// The color table contains RGB values specified by RGBQuad Objects
    DIB_RGB_COLORS = 0x0000,
    /// The color table contains 16-bit indices into the current logical
    /// palette in the playback device context.
    DIB_PAL_COLORS = 0x0001,
    /// No color table exists. The pixels in the DIB are indices into the
    /// current logical palette in the playback device context.
    /// A DIB is specified by a DeviceIndependentBitmap Object.
    DIB_PAL_INDICES = 0x0002,
}

crate::wmf::parser::constants::impl_parser!(ColorUsage, u16);
