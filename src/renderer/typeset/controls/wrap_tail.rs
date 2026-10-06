//! 어울림 접두 줄/전폭 꼬리와 매칭 실패 뒤 저장 쪽 경계의 읽기 전용 판단.
//! 가용 높이 조회·포맷·밴드 종료·쪽 전환은 조정자에 남긴다.

use super::wrap_match::WrapBand;
use crate::model::paragraph::Paragraph;
use crate::renderer::page_layout::PageLayoutInfo;
use crate::renderer::typeset::para_is_page_bottom_fixed_table_anchor;
use crate::renderer::typeset::paragraph::metrics::FormattedParagraph;

pub(in crate::renderer::typeset) struct WrapPrefix {
    pub len: usize,
    suffix_is_full_width: bool,
}

/// 기존 저장 줄 순서와 단 너비 조회 위치를 보존한다.
pub(in crate::renderer::typeset) fn classify_prefix(
    para: &Paragraph,
    band: WrapBand,
    layout: &PageLayoutInfo,
) -> WrapPrefix {
    let wrap_prefix_len = para
        .line_segs
        .iter()
        .take_while(|seg| seg.column_start == band.cs && seg.segment_width as i32 == band.sw)
        .count();
    // [#4650 · #4599 ⑩] 종전에는 전폭 꼬리 '정확히 한 줄'만 분리했으나, 반폭
    // Square 표 옆 문단이 여러 전폭 꼬리 줄을 갖는 형상(156714641 p1
    // pi13: 표 옆 prefix 4줄 + 전폭 꼬리 5줄)이 일반 배치로 떨어져
    // prefix 가 표 하단 아래(952.9)로 밀렸다 — 한글 2022 캐시 PDF 는
    // 표 옆 747.9. 꼬리 전 줄이 전폭이고 저장 seg 와 조판 줄이 1:1 인
    // 경우로 확장한다(#4090 의 안정 형상 판별은 유지).
    let suffix_is_full_width = para.line_segs[wrap_prefix_len..].iter().all(|seg| {
        seg.column_start == 0
            && (seg.segment_width as i32 - layout.column_width_hu()).abs() <= 3_000
    }) && wrap_prefix_len < para.line_segs.len();
    WrapPrefix {
        len: wrap_prefix_len,
        suffix_is_full_width,
    }
}

impl WrapPrefix {
    /// 포맷 이후 기존 안정 형상 조건을 만족할 때만 꼬리의 전진량을 계산한다.
    pub(in crate::renderer::typeset) fn suffix_height(
        &self,
        para: &Paragraph,
        formatted: &FormattedParagraph,
        is_empty_para: bool,
    ) -> Option<f64> {
        let wrap_prefix_len = self.len;
        let suffix_is_full_width = self.suffix_is_full_width;
        let can_split_prefix = !is_empty_para
            && wrap_prefix_len > 0
            && wrap_prefix_len < para.line_segs.len()
            && suffix_is_full_width
            && formatted.line_count() == para.line_segs.len();
        if can_split_prefix {
            let suffix_height = formatted
                .line_advances_sum(wrap_prefix_len..formatted.line_count())
                + formatted.spacing_after;
            return Some(suffix_height);
        }
        None
    }
}

/// 밴드 종료 후에만 호출한다. 가용 높이는 조회하지 않는다.
pub(in crate::renderer::typeset) fn mismatch_starts_new_page(
    para: &Paragraph,
    paragraphs: &[Paragraph],
    para_idx: usize,
    has_items: bool,
    col_count: u16,
) -> bool {
    if para_idx > 0 && has_items {
        let prev_para = &paragraphs[para_idx - 1];
        let curr_first_vpos = para.line_segs.first().map(|s| s.vertical_pos);
        let prev_last_vpos = prev_para.line_segs.last().map(|s| s.vertical_pos);
        if let (Some(cv), Some(pv)) = (curr_first_vpos, prev_last_vpos) {
            let trigger = if col_count > 1 {
                cv < pv && pv > 5000
            } else {
                // [#2098] 쪽-하단 고정 틀 앵커(vpos=0 절대배치)는 흐름 리셋 신호가 아니다.
                cv == 0 && pv > 5000 && !para_is_page_bottom_fixed_table_anchor(para)
            };
            if trigger {
                return true;
            }
        }
    }
    false
}
