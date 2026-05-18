/// The LogBrush Object defines the style, color, and pattern of a brush. This
/// object is used only in the META_CREATEBRUSHINDIRECT Record to create a Brush
/// Object.
#[derive(Clone, Debug)]
pub enum LogBrush {
    DIBPattern,
    DIBPatternPT,
    Hatched {
        color_ref: crate::wmf::parser::ColorRef,
        brush_hatch: crate::wmf::parser::HatchStyle,
    },
    Pattern,
    Solid {
        color_ref: crate::wmf::parser::ColorRef,
    },
    Null,
}

impl LogBrush {
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
            crate::wmf::parser::BrushStyle::BS_DIBPATTERN => {
                let (_, c) = crate::wmf::parser::read::<R, 6>(buf)?;
                consumed_bytes += c;

                Self::DIBPattern
            }
            crate::wmf::parser::BrushStyle::BS_DIBPATTERNPT => {
                let (_, c) = crate::wmf::parser::read::<R, 6>(buf)?;
                consumed_bytes += c;

                Self::DIBPatternPT
            }
            crate::wmf::parser::BrushStyle::BS_HATCHED => {
                let ((color_ref, color_ref_bytes), (brush_hatch, brush_hatch_bytes)) = (
                    crate::wmf::parser::ColorRef::parse(buf)?,
                    crate::wmf::parser::HatchStyle::parse(buf)?,
                );
                consumed_bytes += color_ref_bytes + brush_hatch_bytes;

                Self::Hatched {
                    color_ref,
                    brush_hatch,
                }
            }
            crate::wmf::parser::BrushStyle::BS_PATTERN => {
                let (_, c) = crate::wmf::parser::read::<R, 6>(buf)?;
                consumed_bytes += c;

                Self::Pattern
            }
            crate::wmf::parser::BrushStyle::BS_SOLID => {
                let ((color_ref, color_ref_bytes), (_, c)) = (
                    crate::wmf::parser::ColorRef::parse(buf)?,
                    crate::wmf::parser::read::<R, 2>(buf)?,
                );
                consumed_bytes += color_ref_bytes + c;

                Self::Solid { color_ref }
            }
            crate::wmf::parser::BrushStyle::BS_NULL => {
                let (_, c) = crate::wmf::parser::read::<R, 6>(buf)?;
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
