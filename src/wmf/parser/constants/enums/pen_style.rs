use crate::wmf::imports::*;

/// The 16-bit PenStyle Enumeration is used to specify different types of pens
/// that can be used in graphics operations.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u16)]
pub enum PenStyle {
    /// This value 0x0000 has multiple meanings:
    /// - The pen is cosmetic `PS_COSMETIC = 0x0000`.
    /// - Line end caps are round `PS_ENDCAP_ROUND = 0x0000`.
    /// - Line joins are round `PS_JOIN_ROUND = 0x0000`.
    /// - The pen is solid `PS_SOLID = 0x0000`.
    PS_SOLID = 0x0000,
    /// The pen is dashed.
    PS_DASH = 0x0001,
    /// The pen is dotted.
    PS_DOT = 0x0002,
    /// The pen has alternating dashes and dots.
    PS_DASHDOT = 0x0003,
    /// The pen has dashes and double dots.
    PS_DASHDOTDOT = 0x0004,
    /// The pen is invisible.
    PS_NULL = 0x0005,
    /// The pen is solid. When this pen is used in any drawing record that
    /// takes a bounding rectangle, the dimensions of the figure are shrunk so
    /// that it fits entirely in the bounding rectangle, taking into account
    /// the width of the pen.
    PS_INSIDEFRAME = 0x0006,
    /// The pen uses a styling array supplied by the user.
    PS_USERSTYLE = 0x0007,
    /// The pen sets every other pixel (this style is applicable only for
    /// cosmetic pens).
    PS_ALTERNATE = 0x0008,
    /// Line end caps are square.
    PS_ENDCAP_SQUARE = 0x0100,
    /// Line end caps are flat.
    PS_ENDCAP_FLAT = 0x0200,
    /// Line joins are beveled.
    PS_JOIN_BEVEL = 0x1000,
    /// Line joins are mitered when they are within the current limit set by
    /// the SETMITERLIMIT Record. A join is beveled when it would exceed the
    /// limit.
    PS_JOIN_MITER = 0x2000,
}

crate::wmf::parser::constants::impl_parser!(PenStyle, u16);

impl PenStyle {
    pub fn end_cap() -> BTreeSet<Self> {
        BTreeSet::from_iter([
            // NOTE: use `PS_SOLID` as `PS_ENDCAP_ROUND`.
            Self::PS_SOLID,
            Self::PS_ENDCAP_SQUARE,
            Self::PS_ENDCAP_FLAT,
        ])
    }

    pub fn line_join() -> BTreeSet<Self> {
        BTreeSet::from_iter([
            // NOTE: use `PS_SOLID` as `PS_JOIN_ROUND`.
            Self::PS_SOLID,
            Self::PS_JOIN_BEVEL,
            Self::PS_JOIN_MITER,
        ])
    }

    pub fn style() -> BTreeSet<Self> {
        BTreeSet::from_iter([
            Self::PS_SOLID,
            Self::PS_DASH,
            Self::PS_DOT,
            Self::PS_DASHDOT,
            Self::PS_DASHDOTDOT,
            Self::PS_NULL,
            Self::PS_INSIDEFRAME,
            Self::PS_USERSTYLE,
            Self::PS_ALTERNATE,
        ])
    }
}
