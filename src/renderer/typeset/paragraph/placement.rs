//! 전체 문단의 항목 순서와 분할 문단의 항목/전진 높이를 판정한다. 상태 변경은 하지 않는다.

use super::metrics::FormattedParagraph;
use super::split::SplitBoundary;
use crate::model::{control::Control, paragraph::Paragraph};
use crate::renderer::float_placement::empty_offset_float_deferred_text_ladder_hu;
use crate::renderer::pagination::PageItem;

/// 기존 컷으로 만든 항목과 그 항목의 흐름 전진량. state는 다시 측정하지 않고 적용한다.
pub(in crate::renderer::typeset) struct ParagraphFragment {
    pub item: PageItem,
    pub height: f64,
}

/// None이면 아직 아무 항목도 배치하지 않았으며 다음 단/쪽에서 같은 시작 줄을 재시도한다.
/// fit에 쓴 cumulative와 실제 전진 높이를 혼합하지 않는다.
#[allow(clippy::too_many_arguments)]
pub(in crate::renderer::typeset) fn plan_fragment(
    fmt: &FormattedParagraph,
    para_idx: usize,
    cursor_line: usize,
    line_count: usize,
    sp_b: f64,
    avail_for_lines: f64,
    boundary: SplitBoundary,
    used_saved_tail_vpos_fit: bool,
    current_items: &[PageItem],
) -> Option<ParagraphFragment> {
    let SplitBoundary {
        end_line,
        cumulative,
    } = boundary;
    let part_line_height = fmt.line_advances_sum(cursor_line..end_line);
    let part_sp_after = if end_line >= line_count {
        fmt.spacing_after
    } else {
        0.0
    };
    let part_height = sp_b + part_line_height + part_sp_after;

    let item = if cursor_line == 0 && end_line >= line_count {
        // 전체가 배치됨 — overflow 재확인
        let prev_is_table = current_items.last().map_or(false, |item| {
            matches!(item, PageItem::Table { .. } | PageItem::PartialTable { .. })
        });
        let overflow_threshold = if prev_is_table {
            let trailing_ls = fmt
                .line_spacings
                .get(end_line.saturating_sub(1))
                .copied()
                .unwrap_or(0.0);
            cumulative - trailing_ls
        } else {
            cumulative
        };
        if overflow_threshold > avail_for_lines
            && !current_items.is_empty()
            && !used_saved_tail_vpos_fit
        {
            return None;
        }
        PageItem::FullParagraph {
            para_index: para_idx,
        }
    } else {
        PageItem::PartialParagraph {
            para_index: para_idx,
            start_line: cursor_line,
            end_line,
        }
    };
    Some(ParagraphFragment {
        item,
        height: part_height,
    })
}

/// 빈 host의 float 뒤 본문이 표 위 공간을 먼저 채우는 기존 순서를 판정한다.
pub(in crate::renderer::typeset) fn defer_preceding_float(
    current_items: &[PageItem],
    paragraphs: &[Paragraph],
    para_idx: usize,
    para: &Paragraph,
) -> bool {
    matches!(
        current_items.last(),
        Some(PageItem::Table {
            para_index: host_para_idx,
            control_index,
        }) if *host_para_idx + 1 == para_idx
            && paragraphs
                .get(*host_para_idx)
                .and_then(|host| host.controls.get(*control_index).map(|control| (host, control)))
                .is_some_and(|(host, control)| matches!(control, Control::Table(table)
                    if empty_offset_float_deferred_text_ladder_hu(host, table, para).is_some()))
    )
}
