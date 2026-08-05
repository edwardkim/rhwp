/// The FontQuality Enumeration specifies how closely the attributes of the
/// logical font match those of the physical font when rendering text.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u8)]
pub enum FontQuality {
    /// Specifies that the character quality of the font does not matter, so
    /// DRAFT_QUALITY can be used.
    DEFAULT_QUALITY = 0x00,
    /// Specifies that the character quality of the font is less important than
    /// the matching of logical attribuetes. For rasterized fonts, scaling
    /// SHOULD be enabled, which means that more font sizes are available.
    DRAFT_QUALITY = 0x01,
    /// Specifies that the character quality of the font is more important than
    /// the matching of logical attributes. For rasterized fonts, scaling
    /// SHOULD be disabled, and the font closest in size SHOULD be chosen.
    PROOF_QUALITY = 0x02,
    /// Specifies that anti-aliasing SHOULD NOT be used when rendering text.
    /// (Windows NT 3.1, Windows NT 3.5, and Windows NT 3.51: Anti-aliasing is
    /// not supported.)
    NONANTIALIASED_QUALITY = 0x03,
    /// Specifies that anti-aliasing SHOULD be used when rendering text, if the
    /// font supports it. (Windows NT 3.1, Windows NT 3.5, and Windows NT 3.51:
    /// Anti-aliasing is not supported.)
    ANTIALIASED_QUALITY = 0x04,
    /// Specifies that ClearType anti-aliasing SHOULD be used when rendering
    /// text, if the font supports it. (Windows NT 3.1, Windows NT 3.5, Windows
    /// NT 3.51, Windows 95, Windows NT 4.0, Windows 98, Windows Millennium
    /// Edition, and Windows 2000: ClearType is not supported.)
    CLEARTYPE_QUALITY = 0x05,
    /// GDI `CLEARTYPE_NATURAL_QUALITY`. MS-WMF 열거에는 없지만 Windows LOGFONT
    /// 가 정의하는 실존 값으로, 실문서 WMF 가 이 값을 담아 온다 (#4063 — 10k
    /// 스윕에서 이 값 거부가 그림 전체 변환 실패로 번진 사례 다수).
    CLEARTYPE_NATURAL_QUALITY = 0x06,
}

crate::wmf::parser::constants::impl_parser!(FontQuality, u8);
