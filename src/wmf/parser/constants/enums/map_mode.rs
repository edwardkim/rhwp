/// The MapMode Enumeration defines how logical units are mapped to physical
/// units; that is, assuming that the origins in both the logical and physical
/// coordinate systems are at the same point on the drawing surface, what is the
/// physical coordinate (x',y') that corresponds to logical coordinate (x,y).
///
/// For example, suppose the mapping mode is MM_TEXT. Given the following
/// definition of that mapping mode, and an origin (0,0) at the top left corner
/// of the drawing surface, logical coordinate (4,5) would map to physical
/// coordinate (4,5) in pixels.
///
/// Now suppose the mapping mode is MM_LOENGLISH, with the same origin as the
/// previous example. Given the following definition of that mapping mode,
/// logical coordinate (4,-5) would map to physical coordinate (0.04,0.05) in
/// inches.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u16)]
pub enum MapMode {
    /// Each logical unit is mapped to one device pixel. Positive x is to the
    /// right; positive y is down.
    MM_TEXT = 0x0001,
    /// Each logical unit is mapped to 0.1 millimeter. Positive x is to the
    /// right; positive y is up.
    MM_LOMETRIC = 0x0002,
    /// Each logical unit is mapped to 0.01 millimeter. Positive x is to the
    /// right; positive y is up.
    MM_HIMETRIC = 0x0003,
    /// Each logical unit is mapped to 0.01 inch. Positive x is to the right;
    /// positive y is up.
    MM_LOENGLISH = 0x0004,
    /// Each logical unit is mapped to 0.001 inch. Positive x is to the right;
    /// positive y is up.
    MM_HIENGLISH = 0x0005,
    /// Each logical unit is mapped to one twentieth (1/20) of a point. In
    /// printing, a point is 1/72 of an inch; therefore, 1/20 of a point is
    /// 1/1440 of an inch. This unit is also known as a "twip". Positive x is
    /// to the right; positive y is up.
    MM_TWIPS = 0x0006,
    /// Logical units are mapped to arbitrary device units with equally scaled
    /// axes; that is, one unit along the x-axis is equal to one unit along the
    /// y-axis. The META_SETWINDOWEXT and META_SETVIEWPORTEXT records specify
    /// the units and the orientation of the axes.
    ///
    /// The processing application SHOULD make adjustments as necessary to
    /// ensure the x and y units remain the same size. For example, when the
    /// window extent is set, the viewport SHOULD be adjusted to keep the units
    /// isotropic.
    MM_ISOTROPIC = 0x0007,
    /// Logical units are mapped to arbitrary units with arbitrarily scaled
    /// axes.
    MM_ANISOTROPIC = 0x0008,
}

crate::wmf::parser::constants::impl_parser!(MapMode, u16);
