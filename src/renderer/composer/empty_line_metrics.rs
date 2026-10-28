//! 빈 줄도 활성 글자모양의 줄 상자를 가진다. 오래된 빈 셀 캐시와 구분한다.
use super::{frame_geometry::FrameGeometry, ComposedParagraph, ResolvedStyleSet};
use crate::model::paragraph::Paragraph;
use crate::model::style::LineSpacingType;
use crate::renderer::{hwpunit_to_px, px_to_hwpunit, DEFAULT_DPI};

pub(super) fn refresh(
    composed: &mut ComposedParagraph,
    source: &Paragraph,
    styles: &ResolvedStyleSet,
) {
    if !source.text.is_empty() || !source.controls.is_empty() {
        return;
    }
    let ([stored], [line]) = (source.line_segs.as_slice(), composed.lines.as_mut_slice()) else {
        return;
    };
    // 다중 안내 줄·0높이 숨김·본문 압축 줄은 이 빈 em 상자의 계약이 아니다.
    if stored.line_height <= 0
        || stored.text_height != stored.line_height
        || stored.baseline_distance <= 0
        || stored.baseline_distance > stored.text_height
    {
        return;
    }
    let Some(para_style) = styles.para_styles.get(source.para_shape_id as usize) else {
        return;
    };
    if para_style.line_spacing_type != LineSpacingType::Percent {
        return;
    }
    let Some(font_size) = source
        .char_shapes
        .first()
        .and_then(|shape| styles.char_styles.get(shape.char_shape_id as usize))
        .map(|style| style.font_size)
        .filter(|size| size.is_finite() && *size > 0.0)
    else {
        return;
    };
    let dpi = styles.resolved_dpi.unwrap_or(DEFAULT_DPI);
    let font_hu = px_to_hwpunit(font_size, dpi);
    if stored.text_height >= font_hu {
        return;
    }
    let (height, spacing) = crate::renderer::corrected_line_metrics_for_source(
        hwpunit_to_px(stored.line_height, dpi),
        hwpunit_to_px(stored.text_height, dpi),
        hwpunit_to_px(stored.line_spacing, dpi),
        font_size,
        para_style.line_spacing_type,
        para_style.line_spacing,
        true,
        false,
    );
    // 유효한 저장 text_height의 수용 범위는 일반 줄 메트릭과 같다.
    // 작은 빈 em을 오래된 캐시로 오인해 정상 저장 행을 재성장시키지 않는다.
    if px_to_hwpunit(height, dpi) == stored.line_height {
        return;
    }
    let mut resolved = stored.clone();
    resolved.line_height = px_to_hwpunit(height, dpi);
    resolved.text_height = font_hu;
    resolved.baseline_distance = (f64::from(stored.baseline_distance) * f64::from(font_hu)
        / f64::from(stored.text_height))
    .round() as i32;
    resolved.line_spacing = px_to_hwpunit(spacing, dpi);
    line.line_height = resolved.line_height;
    line.baseline_distance = resolved.baseline_distance;
    line.line_spacing = resolved.line_spacing;
    // 원본은 보존한다. 측정과 실제 셀 배치는 이 프레임의 같은 줄 메트릭을 소비한다.
    composed.frame_geometry = Some(std::sync::Arc::new(FrameGeometry {
        line_segs: vec![resolved],
        space_metrics: vec![(stored.text_start, source.line_space_metric(0))],
        inline_table_heights: Vec::new(),
        stored_partition: false,
    }));
}
