//! Skia migration: effects must be implemented explicitly by glyph consumers.

use rhwp::paint::PaintTextStyle;
use rhwp::renderer::TextStyle;

#[test]
fn effects_require_explicit_replay_but_preserve_simple_glyph_geometry() {
    let mut style = PaintTextStyle::from(&TextStyle {
        font_size: 12.0,
        ..Default::default()
    });
    assert!(style.is_fill_only_glyph_replay());
    let mut zero_size = style.clone();
    zero_size.font_size = 0.0;
    assert!(!zero_size.is_simple_glyph_run_replay());
    style.outline_type = 1;
    style.shadow_type = 1;
    style.emboss = true;
    assert!(style.is_simple_glyph_run_replay());
    assert!(!style.is_fill_only_glyph_replay());
    style.shadow_offset_x = f64::INFINITY;
    assert!(!style.is_simple_glyph_run_replay());
    style.shadow_offset_x = 1.0;
    style.shade_color = 0x112233;
    assert!(!style.is_simple_glyph_run_replay());
}
