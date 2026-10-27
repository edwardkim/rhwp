//! 확인된 결측 기호는 측정에서 선택한 실제 공급 프로그램으로 그린다.
use super::TextStyle;

/// 원 글꼴의 hit, 명시적 대체 설정, HFT 전진 계약을 다시 추측하지 않는다.
/// 글줄 측정의 최종 결정이 공급자를 선택한 낱글자만 paint에 전달한다.
pub(crate) fn family(style: &TextStyle, text: &str) -> Option<&'static str> {
    let mut chars = text.chars();
    let ch = chars.next()?;
    if chars.next().is_some() || !matches!(ch, '\u{2024}' | '\u{2027}') {
        return None;
    }
    let decision = super::layout::char_width_decision(&[ch], &[1], 0, style, true);
    if !matches!(
        decision.width_source,
        "metricGlyphFallback" | "declaredGlyphFallbackMetric"
    ) {
        return None;
    }
    decision.metric.map(|selected| selected.metric.name)
}

/// Canvas의 크기·장평·첨자·굵기 계약은 기존 공통 descriptor를 재사용한다.
pub(crate) fn canvas_descriptor(style: &TextStyle, text: &str) -> Option<String> {
    let family = family(style, text)?;
    let mut paint_style = style.clone();
    paint_style.font_family = family.to_owned();
    Some(
        super::canvas_text_font::CanvasTextFont::for_positioned_text(&paint_style, 0.0)
            .descriptor()
            .to_owned(),
    )
}
