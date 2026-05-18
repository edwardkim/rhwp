/// The RectL Object defines a rectangle.
///
/// A rectangle defined with a RectL Object is filled up to— but not
/// including—the right column and bottom row of pixels.
#[derive(Clone, Debug)]
pub struct RectL {
    /// Left (4 bytes): A 32-bit signed integer that defines the x coordinate,
    /// in logical coordinates, of the upper-left corner of the rectangle.
    pub left: i32,
    /// Top (4 bytes): A 32-bit signed integer that defines the y coordinate,
    /// in logical coordinates, of the upper-left corner of the rectangle.
    pub top: i32,
    /// Right (4 bytes): A 32-bit signed integer that defines the x coordinate,
    /// in logical coordinates, of the lower-right corner of the rectangle.
    pub right: i32,
    /// Bottom (4 bytes): A 32-bit signed integer that defines y coordinate, in
    /// logical coordinates, of the lower-right corner of the rectangle.
    pub bottom: i32,
}

impl RectL {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub fn parse<R: crate::wmf::Read>(
        buf: &mut R,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let ((left, left_bytes), (top, top_bytes), (right, right_bytes), (bottom, bottom_bytes)) = (
            crate::wmf::parser::read_i32_from_le_bytes(buf)?,
            crate::wmf::parser::read_i32_from_le_bytes(buf)?,
            crate::wmf::parser::read_i32_from_le_bytes(buf)?,
            crate::wmf::parser::read_i32_from_le_bytes(buf)?,
        );

        Ok((
            Self {
                left,
                top,
                right,
                bottom,
            },
            left_bytes + top_bytes + right_bytes + bottom_bytes,
        ))
    }
}
