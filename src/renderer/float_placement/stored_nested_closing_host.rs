//! 선행 저장 줄 끝과 실제 개체 바깥 상자로 종료 호스트의 소속을 인증한다.

use super::stored_nested_before_host::original_lines;
use super::{hwpunit_to_px, is_para_topbottom_float};
use crate::model::{control::Control, paragraph::Paragraph, table::Table};

/// 실제 바깥 상자가 선행 줄 끝부터 호스트 줄 시작까지를 정확히 닫으면
/// 호스트 줄은 표 뒤에 있다. 표 앞에 호스트 문단의 앞 간격을 다시 놓지 않는다.
pub(crate) fn stored_nested_host_closes_after_previous_line(
    paragraphs: &[Paragraph],
    para_index: usize,
    child: &Table,
    body_height: f64,
    dpi: f64,
) -> bool {
    let Some(para) = paragraphs.get(para_index) else {
        return false;
    };
    let Some(previous) = para_index.checked_sub(1).and_then(|i| paragraphs.get(i)) else {
        return false;
    };
    let [host] = para.line_segs.as_slice() else {
        return false;
    };
    if !dpi.is_finite()
        || dpi <= 0.0
        || !body_height.is_finite()
        || !para.text.trim().is_empty()
        || !matches!(para.controls.as_slice(), [Control::Table(_)])
        || !previous.controls.is_empty()
        || !paragraphs[..=para_index].iter().all(original_lines)
        || child.common.treat_as_char
        || !is_para_topbottom_float(&child.common)
        || !child.common.flow_with_text
        || child.common.vertical_offset != 0
        || child.caption.is_some()
        || child.common.height == 0
        || child.common.height > i32::MAX as u32
    {
        return false;
    }
    let lines: Vec<_> = paragraphs[..=para_index]
        .iter()
        .flat_map(|p| &p.line_segs)
        .collect();
    if lines
        .windows(2)
        .any(|p| p[1].vertical_pos < p[0].vertical_pos)
    {
        return false;
    }
    let Some(line) = previous.line_segs.last() else {
        return false;
    };
    let Some(end) = line
        .vertical_pos
        .checked_add(line.line_height)
        .and_then(|end| end.checked_add(line.line_spacing))
    else {
        return false;
    };
    let Some(gap) = host.vertical_pos.checked_sub(end).filter(|gap| *gap > 0) else {
        return false;
    };
    let declared = hwpunit_to_px(child.common.height as i32, dpi);
    let outer = body_height
        + hwpunit_to_px(
            i32::from(child.outer_margin_top) + i32::from(child.outer_margin_bottom),
            dpi,
        );
    let tolerance = hwpunit_to_px(2, dpi);
    (body_height - declared).abs() <= tolerance
        && (hwpunit_to_px(gap, dpi) - outer).abs() <= tolerance
}
