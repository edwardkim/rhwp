/// The PostScriptJoin Enumeration defines line-joining capabilities for use
/// with a PostScript printer driver.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(i32)]
pub enum PostScriptJoin {
    /// Specifies that the line-joining style has not been set and that a
    /// default style can be used. (In Windows implementations, the PostScript
    /// printer driver uses a default line join style of PostScriptMiterJoin.)
    PostScriptNotSet = -2,
    /// Specifies a mitered join, which produces a sharp or clipped corner.
    PostScriptMiterJoin = 0,
    /// Specifies a circular join, which produces a smooth, circular arc
    /// between the lines.
    PostScriptRoundJoin = 1,
    /// Specifies a beveled join, which produces a diagonal corner.
    PostScriptBevelJoin = 2,
}

crate::wmf::parser::constants::impl_parser!(PostScriptJoin, i32);
