/// ClipPrecision Flags specify clipping precision, which defines how to clip
/// characters that are partially outside a clipping region. These flags can be
/// combined to specify multiple options.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u8)]
pub enum ClipPrecision {
    /// Specifies that default clipping MUST be used.
    CLIP_DEFAULT_PRECIS = 0x00000000,
    /// This value SHOULD NOT be used.
    CLIP_CHARACTER_PRECIS = 0x00000001,
    /// This value MAY be returned when enumerating rasterized, TrueType and
    /// vector fonts. (Windows NT 3.1, Windows NT 3.5, Windows NT 3.51, Windows
    /// NT 4.0, Windows 2000, and Windows XP: This value is always returned
    /// when enumerating fonts.)
    CLIP_STROKE_PRECIS = 0x00000002,
    /// This value is used to control font rotation, as follows:
    ///
    /// - If set, the rotation for all fonts SHOULD be determined by the
    ///   orientation of the coordinate system; that is, whether the
    ///   orientation is left-handed or right-handed.
    ///
    /// - If clear, device fonts SHOULD rotate counterclockwise, but the
    ///   rotation of other fonts SHOULD be determined by the orientation of
    ///   the coordinate system.
    CLIP_LH_ANGLES = 0x00000010,
    /// This value SHOULD NOT be used. (This value is supported in the
    /// following Windows versions: Windows NT 3.1, Windows NT 3.5, Windows NT
    /// 3.51, Windows 95, Windows NT 4.0, Windows 98, Windows Millennium
    /// Edition, Windows 2000, Windows XP, and Windows Server 2003.)
    CLIP_TT_ALWAYS = 0x00000020,
    /// This value specifies that font association SHOULD be turned off. (This
    /// value is supported in the following Windows versions: Windows NT 3.1,
    /// Windows NT 3.5, Windows NT 3.51, and Windows NT 4.0. Font association
    /// is turned off in Windows 2000, Windows XP, and Windows Server 2003.)
    CLIP_DFA_DISABLE = 0x00000040,
    /// This value specifies that font embedding MUST be used to render
    /// document content; embedded fonts are read-only.
    CLIP_EMBEDDED = 0x00000080,
}

crate::wmf::parser::constants::impl_parser!(ClipPrecision, u8);
