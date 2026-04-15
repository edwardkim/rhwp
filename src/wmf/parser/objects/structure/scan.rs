use crate::wmf::imports::*;

/// The Scan Object specifies a collection of scanlines.
#[derive(Clone, Debug)]
#[allow(clippy::struct_field_names)]
pub struct Scan {
    /// Count (2 bytes): A 16-bit unsigned integer that specifies the number of
    /// horizontal (x-axis) coordinates in the ScanLines array. This value MUST
    /// be a multiple of 2, since left and right endpoints are required to
    /// specify each scanline.
    pub count: u16,
    /// Top (2 bytes): A 16-bit unsigned integer that defines the vertical
    /// (y-axis) coordinate, in logical units, of the top scanline.
    pub top: u16,
    /// Bottom (2 bytes): A 16-bit unsigned integer that defines the vertical
    /// (y-axis) coordinate, in logical units, of the bottom scanline.
    pub bottom: u16,
    /// ScanLines (variable): An array of scanlines, each specified by left and
    /// right horizontal (x-axis) coordinates of its endpoints.
    pub scan_lines: Vec<ScanLine>,
    /// Count2 (2 bytes): A 16-bit unsigned integer that MUST be the same as
    /// the value of the Count field; it is present to allow upward travel in
    /// the structure.
    pub count2: u16,
}

impl Scan {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub fn parse<R: crate::wmf::Read>(
        buf: &mut R,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let ((count, count_bytes), (top, top_bytes), (bottom, bottom_bytes)) = (
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
        );
        let mut consumed_bytes = count_bytes + top_bytes + bottom_bytes;

        if count % 2 != 0 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: format!("The count field `{count}` must be even value"),
            });
        }

        let line_count = count as usize / 2;
        let mut scan_lines = Vec::with_capacity(line_count);

        for _ in 0..line_count {
            let (v, c) = ScanLine::parse(buf)?;

            consumed_bytes += c;
            scan_lines.push(v);
        }

        let (count2, c) = crate::wmf::parser::read_u16_from_le_bytes(buf)?;
        consumed_bytes += c;

        if count != count2 {
            return Err(crate::wmf::parser::ParseError::UnexpectedPattern {
                cause: format!(
                    "The count field `{count}` and count2 field `{count2}` \
                     must have same value"
                ),
            });
        }

        Ok((
            Self {
                count,
                top,
                bottom,
                scan_lines,
                count2,
            },
            consumed_bytes,
        ))
    }
}

#[derive(Clone, Debug)]
pub struct ScanLine {
    /// Left (2 bytes): A 16-bit unsigned integer that defines the horizontal
    /// (x-axis) coordinate, in logical units, of the left endpoint of the
    /// scanline.
    pub left: u16,
    /// Right (2 bytes): A 16-bit unsigned integer that defines the horizontal
    /// (x-axis) coordinate, in logical units, of the right endpoint of the
    /// scanline.
    pub right: u16,
}

impl ScanLine {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub fn parse<R: crate::wmf::Read>(
        buf: &mut R,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let ((left, left_bytes), (right, right_bytes)) = (
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
            crate::wmf::parser::read_u16_from_le_bytes(buf)?,
        );

        Ok((Self { left, right }, left_bytes + right_bytes))
    }
}
