/// The Pen Object specifies the style, width, and color of a pen.
#[derive(Clone, Debug)]
pub struct Pen {
    /// PenStyle (2 bytes): A 16-bit unsigned integer that specifies the pen
    /// style. The value MUST be defined from the PenStyle Enumeration table.
    pub style: PenStyleSubsection,
    /// Width (4 bytes): A 32-bit PointS Object that specifies a point for the
    /// object dimensions. The x-coordinate is the pen width. The y-coordinate
    /// is ignored.
    pub width: crate::wmf::parser::PointS,
    /// ColorRef (4 bytes): A 32-bit ColorRef Object that specifies the pen
    /// color value.
    pub color_ref: crate::wmf::parser::ColorRef,
}

impl Pen {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub fn parse<R: crate::wmf::Read>(
        buf: &mut R,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let ((style, style_bytes), (width, width_bytes), (color_ref, color_ref_bytes)) = (
            PenStyleSubsection::parse(buf)?,
            crate::wmf::parser::PointS::parse(buf)?,
            crate::wmf::parser::ColorRef::parse(buf)?,
        );

        Ok((
            Self {
                style,
                width,
                color_ref,
            },
            style_bytes + width_bytes + color_ref_bytes,
        ))
    }
}

#[derive(Clone, Debug)]
pub struct PenStyleSubsection {
    pub end_cap: crate::wmf::parser::PenStyle,
    pub line_join: crate::wmf::parser::PenStyle,
    pub style: crate::wmf::parser::PenStyle,
    pub typ: crate::wmf::parser::PenStyle,
}

impl PenStyleSubsection {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub fn parse<R: crate::wmf::Read>(
        buf: &mut R,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let (style_u16, style_bytes) = crate::wmf::parser::read_u16_from_le_bytes(buf)?;

        Ok((
            Self {
                // NOTE: use `PS_SOLID` as `PS_COSMETIC`.
                end_cap: Self::end_cap(style_u16),
                line_join: Self::line_join(style_u16),
                style: Self::style(style_u16),
                typ: crate::wmf::parser::PenStyle::PS_SOLID,
            },
            style_bytes,
        ))
    }

    fn end_cap(v: u16) -> crate::wmf::parser::PenStyle {
        const MASK: u16 = 0x0F00;

        for s in crate::wmf::parser::PenStyle::end_cap() {
            if v & MASK == s as u16 {
                return s;
            }
        }

        crate::wmf::parser::PenStyle::PS_ENDCAP_FLAT
    }

    pub fn line_join(v: u16) -> crate::wmf::parser::PenStyle {
        const MASK: u16 = 0xF000;

        for s in crate::wmf::parser::PenStyle::line_join() {
            if v & MASK == s as u16 {
                return s;
            }
        }

        crate::wmf::parser::PenStyle::PS_JOIN_MITER
    }

    pub fn style(v: u16) -> crate::wmf::parser::PenStyle {
        const MASK: u16 = 0x000F;

        for s in crate::wmf::parser::PenStyle::style() {
            if v & MASK == s as u16 {
                return s;
            }
        }

        crate::wmf::parser::PenStyle::PS_SOLID
    }
}
