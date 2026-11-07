//! 흐름을 밀지 않는 글상자의 현재 글줄과 두 안 여백이 소유하는 최소 상자.

use crate::model::shape::{ShapeObject, TextWrap};
use crate::renderer::composer::{
    compose_paragraph_in_context, paragraph_flow_end, recalculate_section_vpos, reflow_line_segs,
    ParagraphBox,
};
use crate::renderer::style_resolver::ResolvedStyleSet;
use crate::renderer::{hwpunit_to_px, px_to_hwpunit};

pub(super) struct PreparedTextBox {
    pub(super) shape: ShapeObject,
    pub(super) height: f64,
}

/// 내용이 선언 상자를 넘으면 현재 문단 프레임으로 높이와 그릴 글줄을 함께 만든다.
/// 원본 도형·저장 줄은 변경하지 않는다. 자리차지·인라인·변환된 상자는 각 흐름과
/// 아핀 프레임의 소유를 유지하며, 이 경로는 흐름 없는 가로 글상자만 담당한다.
pub(super) fn prepare(
    shape: &ShapeObject,
    width: f64,
    height: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<PreparedTextBox> {
    let common = shape.common();
    let text_box = shape.drawing()?.text_box.as_ref()?;
    if common.treat_as_char
        || common.size_protect
        || !matches!(
            common.text_wrap,
            TextWrap::InFrontOfText | TextWrap::BehindText
        )
        || text_box.list_attr & 0x07 != 0
        || text_box.paint_clip.is_some()
        || text_box
            .paragraphs
            .iter()
            .any(|para| !para.controls.is_empty())
    {
        return None;
    }
    let horizontal_margin = i32::from(text_box.margin_left) + i32::from(text_box.margin_right);
    let vertical_margin = i32::from(text_box.margin_top) + i32::from(text_box.margin_bottom);
    let inner_width = width - hwpunit_to_px(horizontal_margin, dpi);
    if !inner_width.is_finite() || inner_width <= 0.0 || !height.is_finite() {
        return None;
    }
    // 온전한 저장 글줄이 현재 글자 높이·안 여백과 함께 들어가면 높이
    // 확장의 이유가 없다. 가로 재조판의 추정 폭으로 새 줄을 만든 뒤 그
    // 높이를 원본 넘침이라고 판정하면 PDF의 위치 지정 낱말까지 분해된다.
    let origin = super::textbox_vpos_origin_hu(common, false).filter(|origin| {
        text_box
            .paragraphs
            .iter()
            .find_map(|para| para.line_segs.first())
            .is_some_and(|line| line.vertical_pos >= *origin)
    });
    let stored_content_fits = text_box.paragraphs.iter().all(|para| {
        if para.line_segs.is_empty()
            || para.stored_text_partition_is_dirty()
            || para.cell_format_vpos_dirty
        {
            return false;
        }
        let composed = compose_paragraph_in_context(para, styles);
        composed.lines.len() == para.line_segs.len()
            && composed
                .lines
                .iter()
                .zip(&para.line_segs)
                .all(|(line, source)| {
                    let font_height = line
                        .runs
                        .iter()
                        .filter_map(|run| styles.char_styles.get(run.char_style_id as usize))
                        .map(|style| px_to_hwpunit(style.font_size, dpi))
                        .max()
                        .unwrap_or(0);
                    source.line_height > 0
                        && super::normalize_textbox_vpos_hu(source.vertical_pos, origin)
                            .saturating_add(source.line_height.max(font_height))
                            .saturating_add(vertical_margin)
                            <= px_to_hwpunit(height, dpi)
                })
    });
    if stored_content_fits {
        return None;
    }
    let mut paragraphs = text_box.paragraphs.clone();
    for index in 0..paragraphs.len() {
        let old_end = paragraph_flow_end(&paragraphs[index]);
        let style = styles
            .para_styles
            .get(paragraphs[index].para_shape_id as usize);
        let frame = ParagraphBox::content_for_style(inner_width, style, dpi);
        if !frame.is_usable() {
            return None;
        }
        reflow_line_segs(&mut paragraphs[index], frame, styles, dpi);
        if paragraph_flow_end(&paragraphs[index]) != old_end {
            recalculate_section_vpos(&mut paragraphs, index, None, old_end, styles, dpi, false);
        }
    }
    // 마지막 글줄 뒤의 줄간격은 상자 내용이 아니다. 앞선 줄간격·문단 간격은
    // 공통 재조판의 vpos에 이미 포함되어 있다. 빈 글줄도 발행된 상자로 센다.
    // 저장 줄이 쪽 좌표를 소유하면 paint와 같은 원점을 뺀다. 쪽에서 잰
    // 오프셋을 상자 내용 높이로 더하거나 뒤쪽 줄만 재기저화하지 않는다.
    let origin = super::textbox_vpos_origin_hu(common, false).filter(|origin| {
        paragraphs
            .iter()
            .find_map(|paragraph| paragraph.line_segs.first())
            .map(|line| line.vertical_pos >= *origin)
            .unwrap_or(false)
    });
    let content_end = paragraphs
        .iter()
        .flat_map(|para| &para.line_segs)
        .map(|line| {
            super::normalize_textbox_vpos_hu(line.vertical_pos, origin)
                .saturating_add(line.line_height)
        })
        .max()?;
    let required = content_end.checked_add(vertical_margin)?;
    if required <= px_to_hwpunit(height, dpi) {
        return None;
    }
    let mut prepared = shape.clone();
    prepared.drawing_mut()?.text_box.as_mut()?.paragraphs = paragraphs;
    Some(PreparedTextBox {
        shape: prepared,
        height: hwpunit_to_px(required, dpi),
    })
}
