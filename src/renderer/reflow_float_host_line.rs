//! 재조판하는 자리차지 표의 잉크 없는 호스트 줄 점유.

use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::model::shape::VertAlign;
use crate::model::table::Table;

use super::composer::ComposedParagraph;
use super::float_placement::{is_para_topbottom_float, signed_hwpunit};
use super::style_resolver::ResolvedStyleSet;

/// 양수 첫줄 들여쓰기는 텍스트 없이도 줄 안쪽을 차지한다.
/// 왼쪽 여백과 내어쓰기는 줄 밖의 기하이며, 빈 개체 줄을 만들지 않는다.
/// 자리차지 밴드 위에 줄 높이가 들어가면 그 줄은 표 뒤 꼬리가 아니다.
/// 공백을 포함한 실제 텍스트는 별도 문단 조각이 이미 소유한다. 표 꼬리에
/// 그 줄을 더하지 않는다. 저장 줄·여러 개체도 기존 줄/밴드 경로를 유지한다.
pub(crate) fn trailing_advance(
    para: &Paragraph,
    table: &Table,
    composed: Option<&ComposedParagraph>,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<f64> {
    if !super::para_has_no_stored_line_segs(para)
        || !is_para_topbottom_float(&table.common)
        || !table.common.flow_with_text
        || table.common.vert_align != VertAlign::Top
        || !para.text.is_empty()
        || para.controls.iter().any(|control| {
            !matches!(
                control,
                Control::Table(_) | Control::SectionDef(_) | Control::ColumnDef(_)
            )
        })
        || para
            .controls
            .iter()
            .filter(|control| matches!(control, Control::Table(_)))
            .count()
            != 1
    {
        return None;
    }
    let style_id = composed.map_or(para.para_shape_id, |paragraph| paragraph.para_style_id);
    let style = styles.para_styles.get(style_id as usize)?;
    if style.indent <= 0.0 {
        return Some(0.0);
    }
    // 종단 글자 모양이 빈 줄 상자를 소유한다. 개체 제어 문자의
    // 모양은 빈 종단 run보다 큰 글꼴을 우연히 가지고 있어도 줄 높이를 바꾸지 않는다.
    let end_shape = para
        .char_shapes
        .iter()
        .find(|shape| shape.start_pos == para.char_count.saturating_sub(1))
        .map(|shape| shape.char_shape_id)
        .or_else(|| para.char_shape_id_at(0))
        .or_else(|| para.char_shapes.last().map(|shape| shape.char_shape_id))?;
    let font_size = styles.char_styles.get(end_shape as usize)?.font_size;
    if font_size <= 0.0 {
        return None;
    }
    let (height, leading) = super::corrected_line_metrics(
        0.0,
        0.0,
        font_size,
        style.line_spacing_type,
        style.line_spacing,
    );
    let band_top = super::hwpunit_to_px(signed_hwpunit(table.common.vertical_offset), dpi);
    Some(if height <= band_top {
        0.0
    } else {
        height + leading
    })
}
