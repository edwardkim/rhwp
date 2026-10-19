//! 어울림 그림 control이 소유한 유효한 저장 줄의 문단 상대 원점.

use crate::model::shape::{HorzRelTo, TextWrap, VertAlign, VertRelTo};
use crate::model::{control::Control, paragraph::LineSeg, paragraph::Paragraph};
use crate::renderer::{composer, layout_frame::ParagraphBox, style_resolver::ResolvedStyleSet};

/// 좁아진 줄은 배제 영역의 결과이며 그림 control의 소유를 증명하지 않는다.
/// 원본 control 위치와 전폭 줄 구성 결과를 같은 UTF-16 축으로 해소한다.
/// 어울림이 밀어낸 저장 줄을 원래 앵커로 다시 쓰지 않는다.
pub(crate) fn stored_picture_anchor_delta(
    paragraph: &Paragraph,
    control_index: usize,
    column_width: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<f64> {
    let Control::Picture(picture) = paragraph.controls.get(control_index)? else {
        return None;
    };
    let debug = std::env::var_os("RHWP_PICTURE_ANCHOR_DEBUG").is_some();
    if debug {
        eprintln!(
            "PICTURE_ANCHOR_INPUT id={} ci={} rows={} dirty={} raw={:?} offset={} width={} height={}",
            picture.common.instance_id,
            control_index,
            paragraph.line_segs.len(),
            paragraph.stored_text_partition_is_dirty(),
            paragraph.control_utf16_positions().get(control_index),
            picture.common.vertical_offset,
            picture.common.width,
            picture.common.height,
        );
    }
    if picture.common.treat_as_char
        || picture.common.text_wrap != TextWrap::Square
        || picture.common.vert_rel_to != VertRelTo::Para
        || paragraph.stored_text_partition_is_dirty()
        || paragraph.line_segs.iter().any(|line| {
            line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0 || line.line_height <= 0
        })
        || paragraph.line_segs.windows(2).any(|pair| {
            pair[1].vertical_pos < pair[0].vertical_pos || pair[1].text_start < pair[0].text_start
        })
    {
        return None;
    }
    if paragraph.line_segs.is_empty() {
        return None;
    }
    // 같은 저장 줄의 수식 메트릭이 갱신됐다면 본문과 같은 물리 줄 원점을 쓴다.
    // 이 producer는 저장 줄의 구성만 읽으며 개체 배제 영역 계산을 호출하지 않는다.
    // 줄 소속이 달라진 경우에는 투영하지 않아 원본 control의 소유를 유지한다.
    let projected = composer::stored_row_projection::project_equation_rows(
        std::slice::from_ref(paragraph),
        styles,
    );
    let paragraph = projected
        .as_ref()
        .and_then(|rows| rows.first())
        .unwrap_or(paragraph);
    // 단일 그림이 만든 저장 차선의 시작을 같은 원본 control과 연결한다.
    // marker가 이미 어울림 때문에 뒤 줄로 밀린 경우에도 배제 상자의 최초 원점을
    // marker의 뒤 텍스트 줄로 덮지 않는다.
    if let Some(anchor) =
        stored_exclusion_anchor(paragraph, control_index, column_width, styles, dpi)
    {
        if debug {
            eprintln!("PICTURE_ANCHOR_SAVED ci={control_index} anchor={anchor:.4}");
        }
        return Some(anchor);
    }
    let raw_start = *paragraph.control_utf16_positions().get(control_index)?;
    let paragraph_box = ParagraphBox::body_for_style(
        column_width,
        styles.para_styles.get(paragraph.para_shape_id as usize),
        dpi,
    );
    if !paragraph_box.is_usable() {
        return None;
    }
    let mut frame = paragraph_box.frame(0);
    // 이 조회는 저장 차선 폭을 재사용하지 않고 전폭 frame에 직접 채운다.
    // 유효한 저장 줄의 들여쓰기 의도까지 지우지 않는다.
    let rows = composer::layout_picture_anchor_in_frame(paragraph, &mut frame, styles, dpi)?;
    let first = rows.first()?;
    let anchor = rows.iter().rfind(|row| row.text_start <= raw_start)?;
    if debug {
        eprintln!(
            "PICTURE_ANCHOR_FULL_WIDTH ci={} raw={} rows={} owner_start={} owner_vpos={} first_vpos={}",
            control_index,
            raw_start,
            rows.len(),
            anchor.text_start,
            anchor.vertical_pos,
            first.vertical_pos,
        );
    }
    Some(crate::renderer::hwpunit_to_px(
        anchor.vertical_pos.checked_sub(first.vertical_pos)?,
        dpi,
    ))
}

/// 원본 그림·offset·가로 차선과 세로 점유가 유일한 앵커를 입증할 때만 수용한다.
/// 위치 겹침으로 control 소유를 추측하지 않고 이미 식별한 control의 후보만 대조한다.
fn stored_exclusion_anchor(
    paragraph: &Paragraph,
    control_index: usize,
    column_width: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<f64> {
    use super::{picture_flow_frame_size_hu, stored_control_line_indices, ObjectPlacementFrame};
    use crate::renderer::{hwpunit_to_px, page_layout::LayoutRect};
    let Control::Picture(picture) = paragraph.controls.get(control_index)? else {
        return None;
    };
    if picture.caption.is_some()
        || !matches!(
            picture.common.horz_rel_to,
            HorzRelTo::Column | HorzRelTo::Para
        )
        || !matches!(
            picture.common.vert_align,
            VertAlign::Top | VertAlign::Inside
        )
    {
        return None;
    }
    let square_floats = paragraph
        .controls
        .iter()
        .filter(|control| {
            let common = match control {
                Control::Picture(value) => &value.common,
                Control::Table(value) => &value.common,
                Control::Shape(value) => value.common(),
                _ => return false,
            };
            !common.treat_as_char && common.text_wrap == TextWrap::Square
        })
        .count();
    if square_floats != 1 {
        return None;
    }
    let owner = *stored_control_line_indices(paragraph)?.get(control_index)?;
    let style = styles.para_styles.get(paragraph.para_shape_id as usize);
    let left = style.map_or(0.0, |value| value.margin_left);
    let right = style.map_or(0.0, |value| value.margin_right);
    let column = LayoutRect {
        x: 0.0,
        y: 0.0,
        width: column_width,
        height: 0.0,
    };
    let container = LayoutRect {
        x: left,
        width: (column_width - left - right).max(0.0),
        ..column
    };
    let frame = ObjectPlacementFrame {
        container: &container,
        column: &column,
        body: &column,
        paper: &column,
        paragraph_y: 0.0,
        alignment: style.map_or(crate::model::style::Alignment::Left, |value| {
            value.alignment
        }),
        dpi,
    };
    let (width, height) = picture_flow_frame_size_hu(picture);
    let width = hwpunit_to_px(
        width + i32::from(picture.common.margin.left) + i32::from(picture.common.margin.right),
        dpi,
    );
    let height = hwpunit_to_px(
        height + i32::from(picture.common.margin.top) + i32::from(picture.common.margin.bottom),
        dpi,
    );
    if width <= 0.0 || height <= 0.0 {
        return None;
    }
    let (x, offset) = frame.position(&picture.common, width, height);
    // 저장 정밀도 한 HWPUNIT만 허용한다. 화면 px나 문서별 위치를 고정하지 않는다.
    let quantum = hwpunit_to_px(1, dpi);
    let beside = |line: &LineSeg| {
        let start = hwpunit_to_px(line.column_start, dpi);
        let end = start + hwpunit_to_px(line.segment_width as i32, dpi);
        line.segment_width > 0
            && line.text_height > 0
            && ((end - x).abs() <= quantum || (start - x - width).abs() <= quantum)
    };
    let first_beside = paragraph.line_segs.iter().position(beside)?;
    let base = paragraph.line_segs.first()?.vertical_pos;
    let row_top = |line: &LineSeg| hwpunit_to_px(line.vertical_pos - base, dpi);
    let overlaps = |line: &LineSeg, top: f64| {
        row_top(line) < top + height && top < row_top(line) + hwpunit_to_px(line.text_height, dpi)
    };
    let mut candidates = (0..=owner).filter_map(|index| {
        let anchor = row_top(paragraph.line_segs.get(index)?);
        let top = anchor + offset;
        if !overlaps(&paragraph.line_segs[first_beside], top) {
            return None;
        }
        if paragraph.line_segs[..first_beside].iter().any(|line| {
            let start = hwpunit_to_px(line.column_start, dpi);
            let end = start + hwpunit_to_px(line.segment_width as i32, dpi);
            start < x + width && x < end && overlaps(line, top)
        }) {
            return None;
        }
        Some(anchor)
    });
    let anchor = candidates.next()?;
    candidates.next().is_none().then_some(anchor)
}
