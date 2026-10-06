//! 본문 각주 측정과 표 큐 최소 높이의 서로 다른 계약.

use crate::renderer::typeset::{compose_paragraph, hwpunit_to_px, Footnote};

/// 각주의 실제 렌더 높이. 일반 paginator의 빠른 저장 LineSeg 추정과 달리
/// `layout_footnote_area`와 같은 composed line/line-spacing 규칙을 쓴다. 긴 URL은
/// renderer에서 두 줄 이상으로 재래핑되므로, 저장 LineSeg 한 줄만 예약하면 본문과
/// FootnoteArea가 겹친다. 문단 자체가 없는 malformed 각주는 실제 layout처럼 0을
/// 반환하고, 비어 있는 문단만 layout의 400HU 안내 줄을 예약한다.
pub(in crate::renderer::typeset) fn composed_footnote_content_height(
    footnote: &Footnote,
    dpi: f64,
) -> f64 {
    let total_lines: usize = footnote
        .paragraphs
        .iter()
        .map(|para| {
            crate::renderer::composer::compose_paragraph(para)
                .lines
                .len()
                .max(1)
        })
        .sum();
    if total_lines == 0 {
        return 0.0;
    }

    let mut height = 0.0;
    let mut line_index = 0usize;
    for para in &footnote.paragraphs {
        let composed = crate::renderer::composer::compose_paragraph(para);
        if composed.lines.is_empty() {
            height += hwpunit_to_px(400, dpi);
            line_index += 1;
            continue;
        }
        for line in &composed.lines {
            height += hwpunit_to_px(line.line_height, dpi);
            line_index += 1;
            if line_index < total_lines {
                height += hwpunit_to_px(line.line_spacing, dpi);
            }
        }
    }
    height
}

/// 표 셀 fragment queue의 기존 최소 예약값은 400HU다. Body 각주의 exact metric과
/// 섞지 않고 queue 경로에서만 이 하한을 유지한다.
pub(in crate::renderer::typeset) fn queued_table_footnote_content_height(
    footnote: &Footnote,
    dpi: f64,
) -> f64 {
    composed_footnote_content_height(footnote, dpi).max(hwpunit_to_px(400, dpi))
}
