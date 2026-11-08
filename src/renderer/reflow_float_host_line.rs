//! 재조판하는 부동 개체의 잉크 없는 호스트 줄 점유.

use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::model::shape::VertAlign;
use crate::model::table::Table;

use super::composer::ComposedParagraph;
use super::float_placement::{is_non_flow_decoration, is_para_topbottom_float, signed_hwpunit};
use super::style_resolver::{ResolvedParaStyle, ResolvedStyleSet};

/// 저장 줄이 없는 부동 개체 앵커도 문단의 빈 글자 줄은 소유한다.
/// 파서가 만든 LineSeg는 어울림 안내 줄의 저장 증거가 아니다. 실제 저장된
/// 줄은 기존 사다리/안내 줄 경로에 남기고, 재조판 앵커의 글꼴·줄간격만 공유한다.
/// 개체의 페인트 크기는 이 글자 줄의 높이를 대신하지 않는다.
pub(crate) fn neutral_anchor_metrics(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    para_style: Option<&ResolvedParaStyle>,
    hwp3_legacy_caps: bool,
    dpi: f64,
) -> Option<(f64, f64, f64)> {
    // NoLS 표 host의 줄과 글꼴 선택은 새 base의 공통 결과를 그대로 소비한다.
    // 일반 부동 앵커의 첫 글자 모양으로 재계산하면 layout의 최대 글꼴 선택과 달라진다.
    if super::para_is_unstored_empty_overlay_table_host(para) {
        return super::unstored_overlay_table_host_line_metrics(para, styles, para_style);
    }
    if !para.text.trim().is_empty()
        || para.controls.is_empty()
        || !super::empty_host_controls_are_flow_neutral(para)
        || !super::para_has_no_stored_line_segs(para)
    {
        return None;
    }
    let shape = para
        .char_shape_id_at(0)
        .or_else(|| para.char_shapes.first().map(|shape| shape.char_shape_id))?;
    let char_style = styles.char_styles.get(shape as usize)?;
    let font_size = char_style.font_size;
    if font_size <= 0.0 {
        return None;
    }
    if hwp3_legacy_caps
        && (font_size > super::hwpunit_to_px(1000, dpi) + 0.1
            || (!char_style.bold && font_size < super::hwpunit_to_px(800, dpi) - 0.1))
    {
        return None;
    }
    let spacing_type = para_style.map_or(crate::model::style::LineSpacingType::Percent, |style| {
        style.line_spacing_type
    });
    let spacing = para_style.map_or(160.0, |style| style.line_spacing);
    let (height, leading) =
        super::corrected_line_metrics(0.0, 0.0, font_size, spacing_type, spacing);
    Some((height, leading, font_size))
}

/// 양수 첫줄 들여쓰기는 텍스트 없이도 줄 안쪽을 차지한다.
/// 왼쪽 여백과 내어쓰기는 줄 밖의 기하이며, 빈 개체 줄을 만들지 않는다.
/// 자리차지 밴드 위에 줄 높이가 들어가면 그 줄은 표 뒤 꼬리가 아니다.
/// 공백을 포함한 실제 텍스트는 별도 문단 조각이 이미 소유한다. 표 꼬리에
/// 그 줄을 더하지 않는다. 저장 줄·글앞/글뒤 장식 외의 다른 개체도 기존 줄/밴드 경로를 유지한다.
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
            ) && !is_non_flow_decoration(control)
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
    // 글앞·글뒤 장식을 함께 가진 빈 문단은 표만 가진 개체 앵커가 아니라 자기 빈 줄을 갖는다.
    // 음수 오프셋 표는 host 원점 위로 올라가 재조판 표 상자 계약(오프셋 ≥ 0) 밖이다.
    let owns_line = signed_hwpunit(table.common.vertical_offset) >= 0
        && para.controls.iter().any(is_non_flow_decoration);
    if style.indent <= 0.0 && !owns_line {
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
