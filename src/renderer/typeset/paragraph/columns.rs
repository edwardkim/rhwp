//! 다단 문단의 저장 경계 조회와 확정 조각 계산. 상태 쓰기와 단 전환은 소유하지 않는다.

use super::metrics::FormattedParagraph;
use super::placement::ParagraphFragment;
use crate::model::paragraph::Paragraph;
use crate::renderer::pagination::PageItem;

pub(super) fn detect_breaks(
    para: &Paragraph,
    col_count: u16,
    current_column: u16,
    current_endnote_flow: bool,
    body_height_px: f64,
    dpi: f64,
) -> Vec<usize> {
    // 다단 레이아웃에서 문단 내 단 경계 감지
    // [Task #459] on_first_multicolumn_page 가드 제거: 다단 구역이 여러 페이지에 걸칠 때
    // 후속 페이지에서도 LINE_SEG vpos-reset 으로 인코딩된 단 경계를 인식해야 함.
    // [Task #2320] current_column == 0 한정 해제: 비-0 단(마지막 단 포함)에서 시작하는
    // 문단의 문단 내 vpos 되감김은 단/쪽 경계 인코딩이다 — 마지막 단이면 "다음 쪽
    // 단 0 으로 계속" (treatise p2 pi=29 라인1 되감김 67050→4926, 한컴 2022 PDF 정합).
    // 가드 2종: ①미주 흐름은 한 단 안에서도 vpos 가 크게 되감기므로 제외
    // (ColumnContent.endnote_flow 계약) ②되감김 목표가 단 상단 근방일 때만 인정
    // (detect_near_top_rewind_breaks — 페이지 중간 되감김은 어울림 흐름 잔재).
    if col_count > 1 && current_column == 0 {
        detect_column_breaks_in_paragraph(para)
    } else if col_count > 1 && current_column > 0 && !current_endnote_flow {
        detect_near_top_rewind_breaks(para, body_height_px, dpi)
    } else {
        vec![0]
    }
}

/// 범위가 구성 줄 수 밖이면 기존 루프를 종료한다. 별도의 fit 판단을 추가하지 않는다.
pub(super) fn plan_fragment(
    para_idx: usize,
    fmt: &FormattedParagraph,
    break_start: usize,
    break_end: usize,
    line_count: usize,
) -> Option<ParagraphFragment> {
    if break_start >= line_count || break_end > line_count {
        return None;
    }
    let part_height = fmt.line_advances_sum(break_start..break_end);
    let item = if break_start == 0 && break_end >= line_count {
        PageItem::FullParagraph {
            para_index: para_idx,
        }
    } else {
        PageItem::PartialParagraph {
            para_index: para_idx,
            start_line: break_start,
            end_line: break_end,
        }
    };
    Some(ParagraphFragment {
        item,
        height: part_height,
    })
}

/// [Task #2320] 비-0 단 시작 문단의 쪽/단 경계 되감김 감지.
///
/// 마지막 단에서 시작하는 문단의 문단 내 되감김은 "다음 쪽 단 0 으로 계속"의
/// 쪽 경계 인코딩이다 — 단, **되감김 목표가 단 상단 근방**일 때만 인정한다
/// (treatise pi=29: 4926HU ≈ 본문 높이 7%). 목표가 페이지 중간에 머무는
/// 되감김(143E 신문 스크랩 pi=9: ≈40%)은 경계가 아니라 개체 어울림 흐름의
/// 잔재이므로 제외한다. 단 0 시작 경로(`detect_column_breaks_in_paragraph`,
/// 임계값 없는 any-decrease)는 건드리지 않는다.
fn detect_near_top_rewind_breaks(para: &Paragraph, body_height_px: f64, dpi: f64) -> Vec<usize> {
    const NEAR_TOP_RATIO: f64 = 0.15;
    let mut breaks = vec![0usize];
    if para.line_segs.len() <= 1 {
        return breaks;
    }
    for i in 1..para.line_segs.len() {
        let prev = &para.line_segs[i - 1];
        let curr = &para.line_segs[i];
        if curr.vertical_pos < prev.vertical_pos
            && crate::renderer::hwpunit_to_px(curr.vertical_pos, dpi)
                <= body_height_px * NEAR_TOP_RATIO
        {
            breaks.push(i);
        }
    }
    breaks
}

/// 다단 레이아웃에서 문단 내 단 경계를 감지한다.
fn detect_column_breaks_in_paragraph(para: &Paragraph) -> Vec<usize> {
    let mut breaks = vec![0usize];
    if para.line_segs.len() <= 1 {
        return breaks;
    }
    for i in 1..para.line_segs.len() {
        if para.line_segs[i].vertical_pos < para.line_segs[i - 1].vertical_pos {
            breaks.push(i);
        }
    }
    breaks
}
