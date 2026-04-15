/// The FloodFill Enumeration specifies the type of fill operation to be
/// performed.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u16)]
pub enum FloodFill {
    /// The fill area is bounded by the color specified by the Color member.
    /// This style is identical to the filling performed by the META_FLOODFILL
    /// Record.
    FLOODFILLBORDER = 0x0000,
    /// The fill area is bounded by the color that is specified by the Color
    /// member. Filling continues outward in all directions as long as the
    /// color is encountered. This style is useful for filling areas with
    /// multicolored boundaries.
    FLOODFILLSURFACE = 0x0001,
}

crate::wmf::parser::constants::impl_parser!(FloodFill, u16);
