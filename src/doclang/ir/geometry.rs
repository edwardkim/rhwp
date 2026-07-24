/// Spatial geometry of a floating or inline object, in HWPUNIT.
///
/// 1 inch = 7200 HWPUNIT.  These values come from `CommonObjAttr` in rhwp and
/// are preserved in the Semantic IR for use in the DocLang v2 `<location>`
/// element.  They are not emitted in v1 output but are stored so that the
/// adapter does not have to be rewritten when v2 support is added.
#[derive(Debug, Clone, PartialEq)]
pub struct Geometry {
    /// Object width in HWPUNIT.
    pub width: i32,
    /// Object height in HWPUNIT.
    pub height: i32,
    /// Horizontal offset from the reference point, in HWPUNIT.
    pub h_offset: i32,
    /// Vertical offset from the reference point, in HWPUNIT.
    pub v_offset: i32,
    /// When `true` the object is anchored inline with the text flow
    /// (`treat_as_char` in HWP terminology); when `false` it floats.
    pub treat_as_char: bool,
}
