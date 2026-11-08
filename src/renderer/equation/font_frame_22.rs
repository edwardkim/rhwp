//! 22pt 기본 줄·명시적 공백/정체·단일 위첨자의 독립 정상 저장/Print 프레임.
//! job 6ccdd36d-13a5-4758-99e0-91bc4a33487c, 2a652b98-863c-4397-97a0-d19ea108ad1e.
//! 다른 크기·prime·미측정 복합 구조를 이 값으로 추정하지 않는다.
use super::layout::{LayoutBox, LayoutKind};
use super::logical_font_frame::{contains_prime, font_line, horizontal_row};
use super::symbols::FontStyleKind;

pub(super) fn frame(value: &LayoutBox, fs: f64, dpi: f64) -> Option<(f64, f64)> {
    if contains_prime(value) {
        return None;
    }
    let measured = |hu: i32, percent: f64| {
        let height = crate::renderer::hwpunit_to_px(hu, dpi);
        Some((height, height * percent / 100.0))
    };
    match &value.kind {
        LayoutKind::Color { body, .. } => frame(body, fs, dpi),
        LayoutKind::Space(_) => Some((fs, fs * 0.86)),
        LayoutKind::FontStyle {
            style: FontStyleKind::Roman,
            body,
        } if font_line(body, fs) => Some((fs, fs * 0.86)),
        LayoutKind::FontStyle {
            style: FontStyleKind::Italic,
            body,
        }
        | LayoutKind::Paren { body, .. } => frame(body, fs, dpi),
        LayoutKind::Text(text) if font_line(value, fs) && !super::text_has_cjk(text) => {
            measured(2175, 86.0)
        }
        LayoutKind::Number(_)
        | LayoutKind::FontGlyph { .. }
        | LayoutKind::HancomGlyphs(_)
        | LayoutKind::Symbol(_)
            if font_line(value, fs) =>
        {
            measured(2175, 86.0)
        }
        LayoutKind::MathSymbol(text) if text == "∠" && font_line(value, fs) => {
            measured(2175, 86.0)
        }
        LayoutKind::Superscript { base, sup }
            if font_line(base, fs)
                && font_line(sup, sup.font_size)
                && !contains_prime(base)
                && !contains_prime(sup)
                && (sup.font_size - fs * 0.68).abs() <= f64::EPSILON * fs * 4.0 =>
        {
            super::font_frame_metrics::superscript_at_dpi(fs, dpi)
        }
        LayoutKind::Row(items) if horizontal_row(value, items) => {
            let mut ascent = 0.0_f64;
            let mut descent = 0.0_f64;
            for child in items {
                let (height, baseline) = frame(child, fs, dpi)?;
                ascent = ascent.max(baseline);
                descent = descent.max(height - baseline);
            }
            Some((ascent + descent, ascent))
        }
        _ => None,
    }
}
