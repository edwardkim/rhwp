//! 저장된 다단 첫 쪽의 공통 시작 밴드. 문단 앞 간격과 별도로 소유한다.

use crate::model::{
    paragraph::{LineSeg, Paragraph},
    provenance::LayoutCompatibilityProfile,
};
use crate::renderer::{
    hwpunit_to_px,
    pagination::{ColumnContent, PageContent},
    style_resolver::ResolvedStyleSet,
};

/// 첫 줄이 원점보다 아래에 있고 다음 저장 줄이 같은 사다리를 닫으면
/// 그 작은 시작 밴드는 첫 문단을 뺀 좌표가 아니라 물리 쪽의 여백이다.
pub(crate) fn resolve(
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    profile: LayoutCompatibilityProfile,
    column_count: usize,
    dpi: f64,
) -> Option<f64> {
    if column_count < 2
        || profile.session_edited()
        || profile.hwp3_layout()
        || profile.hwp3_native_layout()
        || !(profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
        || !dpi.is_finite()
        || dpi <= 0.0
    {
        return None;
    }
    let first = paragraphs.first()?;
    let next = paragraphs.get(1)?;
    let [line] = first.line_segs.as_slice() else {
        return None;
    };
    let successor = next.line_segs.first()?;
    let valid = |p: &Paragraph, line: &LineSeg| {
        !p.stored_text_partition_is_dirty()
            && !p.cell_format_vpos_dirty
            && line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            && line.tag & LineSeg::TAG_FIRST_SEGMENT != 0
            && line.segment_width > 0
            && line.line_height > 0
            && line.line_spacing >= 0
    };
    if !valid(first, line)
        || !valid(next, successor)
        || super::has_paragraph_float_graphic(first)
        || super::has_paragraph_float_graphic(next)
        || first.controls.iter().any(control_owns_flow)
        || line.vertical_pos <= 0
        || line.vertical_pos >= line.line_height
    {
        return None;
    }
    let first_style = styles.para_styles.get(first.para_shape_id as usize)?;
    let next_style = styles.para_styles.get(next.para_shape_id as usize)?;
    // 명시적 문단 앞 간격은 기존 저장 간격 계약이 처리한다. 같은 폭의
    // 저장 줄이 여백 없이 이어지는 경우만 쪽 공통 밴드로 인증한다.
    if first_style.spacing_before != 0.0
        || first_style.spacing_after != 0.0
        || next_style.spacing_before != 0.0
        || i64::from(line.vertical_pos) + i64::from(line.line_height) + i64::from(line.line_spacing)
            != i64::from(successor.vertical_pos)
        || i64::from(line.column_start) + i64::from(line.segment_width)
            != i64::from(successor.column_start) + i64::from(successor.segment_width)
    {
        return None;
    }
    Some(hwpunit_to_px(line.vertical_pos, dpi))
}

/// 용지·쪽에 고정되고 겹침을 허용하는 비인라인 배경은 글줄 흐름을
/// 소유하지 않는다. 보이는 도형이 있다는 이유로 저장 줄 원점을 버리지 않는다.
fn control_owns_flow(control: &crate::model::control::Control) -> bool {
    use crate::model::{control::Control, shape::VertRelTo};
    let common = match control {
        Control::Table(table) => &table.common,
        Control::Shape(shape) => shape.common(),
        Control::Picture(picture) => &picture.common,
        _ => return false,
    };
    common.treat_as_char
        || common.flow_with_text
        || !common.allow_overlap
        || !matches!(common.vert_rel_to, VertRelTo::Paper | VertRelTo::Page)
}

/// 첫 물리 쪽의 같은 다단 존만 시작 밴드를 공유한다. 다음 쪽·미주·
/// 앞 존 뒤에서 시작한 다단에는 첫 구역 줄의 원점을 재사용하지 않는다.
pub(crate) fn for_paint(
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    profile: LayoutCompatibilityProfile,
    column_count: usize,
    column: &ColumnContent,
    page: &PageContent,
    dpi: f64,
) -> f64 {
    if column.endnote_flow
        || column.zone_y_offset != 0.0
        || !page.column_contents.iter().any(|c| {
            !c.endnote_flow
                && c.zone_y_offset == 0.0
                && c.items.iter().any(|item| {
                    matches!(
                        item,
                        crate::renderer::pagination::PageItem::FullParagraph { para_index: 0 }
                    )
                })
        })
    {
        return 0.0;
    }
    resolve(paragraphs, styles, profile, column_count, dpi).unwrap_or(0.0)
}
