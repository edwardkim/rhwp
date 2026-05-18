/// The HatchStyle Enumeration specifies the hatch pattern.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u16)]
pub enum HatchStyle {
    /// A horizontal hatch.
    HS_HORIZONTAL = 0x0000,
    /// A vertical hatch.
    HS_VERTICAL = 0x0001,
    /// A 45-degree downward, left-to-right hatch.
    HS_FDIAGONAL = 0x0002,
    /// A 45-degree upward, left-to-right hatch.
    HS_BDIAGONAL = 0x0003,
    /// A horizontal and vertical cross-hatch.
    HS_CROSS = 0x0004,
    /// A 45-degree crosshatch.
    HS_DIAGCROSS = 0x0005,
}

crate::wmf::parser::constants::impl_parser!(HatchStyle, u16);
