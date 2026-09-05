//! Lazy-base rounding shared with the integration boundary tests.

pub(crate) fn resolve_lazy_base(
    prev_vpos_end: i32,
    y_delta_hu: i32,
    trailing_ls_hu: i32,
    trimmed_spacing_before_px: f64,
) -> i32 {
    let normalize = |base: i32| {
        if trimmed_spacing_before_px > 0.5 && (-16..0).contains(&base) {
            0
        } else {
            base
        }
    };

    // Normalize the corrected base BEFORE choosing the fallback. Otherwise
    // 16560 - (14962 + 1600) = -2 becomes the positive fallback 1598.
    let corrected = normalize(prev_vpos_end - (y_delta_hu + trailing_ls_hu));
    let base = if corrected >= 0 {
        corrected
    } else {
        prev_vpos_end - y_delta_hu
    };
    // Preserve the existing rounding rule for the fallback itself as well.
    normalize(base)
}
