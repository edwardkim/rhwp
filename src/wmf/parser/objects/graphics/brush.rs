/// The Brush Object defines the style, color, and pattern of a brush. Brush
/// Objects are created by the META_CREATEBRUSHINDIRECT, META_CREATEPATTERNBRUSH
/// and META_DIBCREATEPATTERNBRUSH records.
#[derive(Clone, Debug)]
pub enum Brush {
    DIBPatternPT {
        color_usage: crate::wmf::parser::ColorUsage,
        brush_hatch: crate::wmf::parser::DeviceIndependentBitmap,
    },
    Hatched {
        color_ref: crate::wmf::parser::ColorRef,
        brush_hatch: crate::wmf::parser::HatchStyle,
    },
    Pattern {
        brush_hatch: crate::wmf::parser::Bitmap16,
    },
    Solid {
        color_ref: crate::wmf::parser::ColorRef,
    },
    Null,
}

impl Brush {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        err(level = tracing::Level::ERROR, Display),
    ))]
    pub fn parse<R: crate::wmf::Read>(
        buf: &mut R,
    ) -> Result<(Self, usize), crate::wmf::parser::ParseError> {
        let (style, mut consumed_bytes) = crate::wmf::parser::BrushStyle::parse(buf)?;
        let v = match style {
            crate::wmf::parser::BrushStyle::BS_DIBPATTERNPT => {
                use crate::wmf::parser::DeviceIndependentBitmap;

                let (color_usage, c) = crate::wmf::parser::ColorUsage::parse(buf)?;
                consumed_bytes += c;

                let (brush_hatch, c) =
                    DeviceIndependentBitmap::parse_with_color_usage(buf, color_usage)?;
                consumed_bytes += c;

                Self::DIBPatternPT {
                    color_usage,
                    brush_hatch,
                }
            }
            crate::wmf::parser::BrushStyle::BS_HATCHED => {
                let (color_ref, c) = crate::wmf::parser::ColorRef::parse(buf)?;
                consumed_bytes += c;

                let (brush_hatch, c) = crate::wmf::parser::HatchStyle::parse(buf)?;
                consumed_bytes += c;

                Self::Hatched {
                    color_ref,
                    brush_hatch,
                }
            }
            crate::wmf::parser::BrushStyle::BS_PATTERN => {
                // SHOULD be ignored.
                let (_, c) = crate::wmf::parser::read::<R, 4>(buf)?;
                consumed_bytes += c;

                let (brush_hatch, c) = crate::wmf::parser::Bitmap16::parse(buf)?;
                consumed_bytes += c;

                Self::Pattern { brush_hatch }
            }
            crate::wmf::parser::BrushStyle::BS_SOLID => {
                let (color_ref, c) = crate::wmf::parser::ColorRef::parse(buf)?;
                consumed_bytes += c;

                Self::Solid { color_ref }
            }
            crate::wmf::parser::BrushStyle::BS_NULL => {
                // SHOULD be ignored.
                let (_, c) = crate::wmf::parser::read::<R, 4>(buf)?;
                consumed_bytes += c;

                Self::Null
            }
            v => {
                return Err(crate::wmf::parser::ParseError::NotSupported {
                    cause: format!("BrushStyle {v:?}"),
                });
            }
        };

        Ok((v, consumed_bytes))
    }
}
