/// The Layout Enumeration defines options for controlling the direction in
/// which text and graphics are drawn. (Windows NT 3.1, Windows NT 3.5, Windows
/// NT 3.51, Windows 95, Windows NT 4.0, Windows 98, and Windows Millennium
/// Edition: This functionality is not supported.)
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u16)]
pub enum Layout {
    /// Sets the default horizontal layout to be left-to-right.
    LAYOUT_LTR = 0x0000,
    /// Sets the default horizontal layout to be right-to-left. Switching to
    /// this layout SHOULD cause the mapping mode in the playback device
    /// context to become MM_ISOTROPIC.
    LAYOUT_RTL = 0x0001,
    /// Disables mirroring of bitmaps that are drawn by META_BITBLT Record and
    /// META_STRETCHBLT Record operations, when the layout is right-to-left.
    LAYOUT_BITMAPORIENTATIONPRESERVED = 0x0008,
}

crate::wmf::parser::constants::impl_parser!(Layout, u16);
