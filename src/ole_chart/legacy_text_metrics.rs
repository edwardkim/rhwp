//! VtTextBlock 캐시와 현재 선언 글꼴의 물리 좌표계를 연결한다.

use crate::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
use crate::renderer::TextStyle;

use super::legacy_presentation::{Font, Text};

pub(super) struct TextMeasure {
    pub width: f64,
    pub height: f64,
    pub cache_unit: f64,
}

pub(super) fn cached_size(text: &Text) -> Option<[f64; 2]> {
    let metrics = &text.metrics;
    if !metrics.cache_valid
        || metrics.extent.iter().any(|value| *value < 0)
        || metrics.margin < 0
        || metrics.orientation != 0
        || metrics.horizontal > 2
        || metrics.vertical != 2
    {
        return None;
    }
    let width = f64::from(metrics.rect[2]) - f64::from(metrics.rect[0]);
    let height = f64::from(metrics.rect[3]) - f64::from(metrics.rect[1]);
    (width > 0.0 && height > 0.0).then_some([width, height])
}

/// 폭을 정수 px로 반올림하기 전에 캐시 단위를 복원한다.
/// 저장 장치의 글자 종횡비를 보존하고 선언 포인트 크기와 같은 배율을 쓴다.
pub(super) fn measure(text: &Text, font: &Font, dpi: f64) -> Option<TextMeasure> {
    cached_size(text)?;
    let style = TextStyle {
        font_family: font.family.clone(),
        font_size: font.size * dpi / 72.0,
        font_space_em: crate::renderer::style_resolver::declared_space_advance_em(
            &font.family,
            false,
            false,
        ),
        ..TextStyle::default()
    };
    let width = *EmbeddedTextMeasurer
        .compute_char_positions(&text.value, &style)
        .last()?;
    let cache_unit = width / f64::from(text.metrics.extent[0]);
    let height = f64::from(text.metrics.extent[1]) * cache_unit;
    (cache_unit.is_finite() && cache_unit > 0.0 && height > 0.0).then_some(TextMeasure {
        width,
        height,
        cache_unit,
    })
}
