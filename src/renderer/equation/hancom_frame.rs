//! 실제 글리프 전진과 구분한 한컴 수식의 공통 구조 프레임.
//!
//! 2024 정상 저장 b6550187-1ade-439e-bbef-94f7f39c7dd2와 독립 Print
//! 64235c43-aae6-4268-8ac2-2be5d8352581의 두 face·네 크기를 대조했다.
//! 높이·기준선은 같고 글리프·전진폭은 다르므로 구조 메트릭만 공유한다.

use super::layout::{LayoutBox, LayoutKind};
use super::symbols::DecoKind;

fn quantized_frame(height: f64, baseline: f64) -> (f64, f64) {
    let percent = (baseline / height * 100.0).round();
    (height, height * percent / 100.0)
}

pub(super) fn fraction(
    numer: &LayoutBox,
    denom: &LayoutBox,
    fs: f64,
    dpi: f64,
) -> Option<(f64, f64)> {
    let line_height = super::font_frame_metrics::line_at_dpi(fs, dpi)?.0;
    let (height, baseline) = super::font_frame_metrics::fraction_at_dpi(fs, dpi)?;
    let above = (metrics(numer, fs, dpi)?.0 - line_height).max(0.0);
    let below = (metrics(denom, fs, dpi)?.0 - line_height).max(0.0);
    Some(quantized_frame(height + above + below, baseline + above))
}

pub(super) fn metrics(value: &LayoutBox, fs: f64, dpi: f64) -> Option<(f64, f64)> {
    match &value.kind {
        LayoutKind::Decoration {
            kind: DecoKind::Hat,
            body,
            ..
        } => {
            let line_height = super::font_frame_metrics::line_at_dpi(fs, dpi)?.0;
            let accent_height = super::font_frame_metrics::accent_at_dpi(fs, dpi, false)?.0;
            let (height, baseline) = metrics(body, fs, dpi)?;
            let band = accent_height - line_height;
            Some(quantized_frame(height + band, baseline + band))
        }
        LayoutKind::Fraction { numer, denom, .. } => fraction(numer, denom, fs, dpi),
        LayoutKind::Subscript { .. } | LayoutKind::SubSup { .. } => {
            let (height, baseline) = super::logical_font_frame::metrics_at_dpi(value, fs, dpi)?;
            // 저장 프레임은 기준선을 전체 높이의 정수 백분율로 기록한다.
            // 분수 뒤 첨자의 점유를 늘린 뒤에도 같은 계약을 적용한다.
            Some(quantized_frame(height, baseline))
        }
        LayoutKind::Color { body, .. } | LayoutKind::FontStyle { body, .. } => {
            metrics(body, fs, dpi)
        }
        LayoutKind::Row(items) if super::logical_font_frame::horizontal_row(value, items) => {
            if super::logical_font_frame::font_line(value, fs) {
                return super::logical_font_frame::metrics_at_dpi(value, fs, dpi);
            }
            let mut ascent = 0.0_f64;
            let mut descent = 0.0_f64;
            for child in items {
                let (height, baseline) = metrics(child, fs, dpi)?;
                ascent = ascent.max(baseline);
                descent = descent.max(height - baseline);
            }
            Some((ascent + descent, ascent))
        }
        _ => super::logical_font_frame::metrics_at_dpi(value, fs, dpi),
    }
}
