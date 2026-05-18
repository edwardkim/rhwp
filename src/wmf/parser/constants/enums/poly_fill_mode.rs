/// The PolyFillMode Enumeration specifies the method used for filling a
/// polygon.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u16)]
pub enum PolyFillMode {
    /// Selects alternate mode (fills the area between odd-numbered and
    /// even-numbered polygon sides on each scan line).
    ALTERNATE = 0x0001,
    /// Selects winding mode (fills any region with a nonzero winding value).
    WINDING = 0x0002,
}

crate::wmf::parser::constants::impl_parser!(PolyFillMode, u16);
