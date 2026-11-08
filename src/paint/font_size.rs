//! Numeric size contract shared by glyph producers and replay admission.

pub(crate) const MAX_GLYPH_FONT_SIZE_PX: f64 = 4096.0;

/// Match the six-decimal `sizePx` field written by the layer JSON exporter.
pub(crate) fn glyph_font_size_is_portable(size_px: f64) -> bool {
    size_px.is_finite()
        && size_px > 0.0
        && size_px <= MAX_GLYPH_FONT_SIZE_PX
        && format!("{size_px:.6}")
            .parse::<f64>()
            .is_ok_and(|wire| wire > 0.0)
}
