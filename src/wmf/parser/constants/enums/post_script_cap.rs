/// The PostScriptCap Enumeration defines line-ending types for use with a
/// PostScript printer driver.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(i32)]
pub enum PostScriptCap {
    /// Specifies that the line-ending style has not been set and that a
    /// default style can be used. (In Windows implementations, the PostScript
    /// printer driver uses a default line join style of PostScriptFlatCap.)
    PostScriptNotSet = -2,
    /// Specifies that the line ends at the last point. The end is squared off.
    PostScriptFlatCap = 0,
    /// Specifies a circular cap. The center of the circle is the last point in
    /// the line. The diameter of the circle is the same as the line width;
    /// that is, the thickness of the line.
    PostScriptRoundCap = 1,
    /// Specifies a square cap. The center of the square is the last point in
    /// the line. The height and width of the square are the same as the line
    /// width; that is, the thickness of the line.
    PostScriptSquareCap = 2,
}

crate::wmf::parser::constants::impl_parser!(PostScriptCap, i32);
