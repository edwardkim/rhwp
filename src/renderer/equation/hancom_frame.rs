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

/// 원본 수식 face의 합·곱 프레임 (전체 높이, 기준선, 연산자 상자 높이, 한계값 글줄 높이).
///
/// #8927 첨부 한컴 2020 실측에서 한계값이 없는 `{sum}`의 높이는 32개 글자 크기 모두
/// 1.35em을 0.75px 격자로 맞춘 값이고, 기준선은 글줄과 같은 em 상자 축에 놓인다.
/// 한계값은 첨자 글자 크기를 px로 맞춘 글줄 하나씩을 위·아래에 더한다
/// (`sum`·`prod` 한계값 17건의 높이·기준선 일치).
pub(super) fn limit_operator(
    fs: f64,
    script_fs: f64,
    dpi: f64,
    has_sup: bool,
    has_sub: bool,
) -> (f64, f64, f64, f64) {
    let px = dpi / 96.0;
    let grid = 0.75 * px;
    let op_height = (fs * 1.35 / grid).round() * grid;
    let limit_height = (script_fs / px).round() * px;
    let sup_height = if has_sup { limit_height } else { 0.0 };
    let sub_height = if has_sub { limit_height } else { 0.0 };
    (
        sup_height + op_height + sub_height,
        sup_height + super::font_frame_metrics::centered_line_ascent(op_height, fs),
        op_height,
        limit_height,
    )
}

/// 합·곱 상자가 위 프레임을 실제로 지니면 그 프레임. 다른 표시 계약의 큰 연산자는 제외한다.
fn limit_operator_box(value: &LayoutBox, fs: f64, dpi: f64) -> Option<(f64, f64)> {
    let LayoutKind::BigOp {
        symbol, sub, sup, ..
    } = &value.kind
    else {
        return None;
    };
    if !matches!(symbol.as_str(), "∑" | "∏") {
        return None;
    }
    let script_fs = [sub, sup]
        .into_iter()
        .flatten()
        .map(|limit| limit.font_size)
        .next()
        .unwrap_or(fs * 0.68);
    let (height, baseline, _, _) = limit_operator(fs, script_fs, dpi, sup.is_some(), sub.is_some());
    let same = |a: f64, b: f64| (a - b).abs() <= f64::EPSILON * fs.max(1.0) * 64.0;
    (same(value.height, height) && same(value.baseline, baseline)).then_some((height, baseline))
}

pub(super) fn metrics(value: &LayoutBox, fs: f64, dpi: f64) -> Option<(f64, f64)> {
    if let Some(frame) = limit_operator_box(value, fs, dpi) {
        return Some(frame);
    }
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
            let mut frames = Vec::with_capacity(items.len());
            for child in items {
                frames.push(metrics(child, fs, dpi)?);
            }
            Some(super::font_frame_metrics::combine_row_frames(&frames))
        }
        _ => super::logical_font_frame::metrics_at_dpi(value, fs, dpi),
    }
}
