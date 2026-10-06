//! 미주 호출별 입력·캐리. 페이지 상태와 별도로 소유한다.

use crate::renderer::typeset::notes::endnotes::profile::EnSsotLevel;

/// [#2064] `compute_endnote_metrics` 의 호출-시점 입력 묶음 — 라운드 5 클로저의 캡처를
/// 명시화한 것 (두 호출부 사이 값 변화를 보존하기 위해 호출부마다 신선하게 구성).
#[derive(Clone, Copy)]
pub(in crate::renderer::typeset) struct EnMetricsVars {
    pub(in crate::renderer::typeset) available: f64,
    pub(in crate::renderer::typeset) cap_large_separator_stale_forward_vpos: bool,
    pub(in crate::renderer::typeset) col_count: u16,
    pub(in crate::renderer::typeset) compact_endnote_separator_profile: bool,
    pub(in crate::renderer::typeset) current_column_has_tac_picture_only: bool,
    pub(in crate::renderer::typeset) current_height_for_metrics: f64,
    pub(in crate::renderer::typeset) dpi: f64,
    pub(in crate::renderer::typeset) en_para_idx: usize,
    pub(in crate::renderer::typeset) h4f: f64,
    pub(in crate::renderer::typeset) has_treat_as_char_picture_shape: bool,
    pub(in crate::renderer::typeset) has_visible_endnote_separator: bool,
    pub(in crate::renderer::typeset) internal_vpos_rewind: bool,
    pub(in crate::renderer::typeset) large_separator_block: bool,
    pub(in crate::renderer::typeset) large_vpos_jump_at_column_top: bool,
    pub(in crate::renderer::typeset) line_advances_sum: f64,
    pub(in crate::renderer::typeset) local_vpos_rewind: bool,
    pub(in crate::renderer::typeset) local_vpos_rewind_crosses_prev_content: bool,
    pub(in crate::renderer::typeset) min_vpos_rewind_height: f64,
    pub(in crate::renderer::typeset) new_endnote_between_notes_px: Option<f64>,
    pub(in crate::renderer::typeset) no_separator_new_note_head_fits_current_column: bool,
    pub(in crate::renderer::typeset) ssot_debug: bool,
    pub(in crate::renderer::typeset) ssot_level: EnSsotLevel,
    pub(in crate::renderer::typeset) this_bottom_offset: Option<i32>,
    pub(in crate::renderer::typeset) this_first_offset: Option<i32>,
    pub(in crate::renderer::typeset) tot: f64,
    pub(in crate::renderer::typeset) trailing_ls_px: f64,
}

/// [#2026] 미주-간 흐름 캐리 상태 (#1904 라운드 1 이연 설계 — EndnoteFlowState).
/// caller(참조 루프)가 값 왕복으로 유지하고, 꼬리에서 prev/current 스왑을 수행한다.
#[derive(Clone, Copy)]
pub(in crate::renderer::typeset) struct EndnoteFlowState {
    pub(in crate::renderer::typeset) vpos_offset: i32,
    pub(in crate::renderer::typeset) prev_en_bottom_vpos: Option<i32>,
    pub(in crate::renderer::typeset) prev_en_content_bottom_vpos: Option<i32>,
    pub(in crate::renderer::typeset) emitted_endnote_count: usize,
    pub(in crate::renderer::typeset) last_render_endnote_para_local_idx: Option<usize>,
    pub(in crate::renderer::typeset) cleared_single_line_internal_rewind_split: bool,
    pub(in crate::renderer::typeset) current_endnote_had_inline_object_vpos_overestimate: bool,
}

/// [#2026] en_para 루프의 미주-당 읽기 플래그/스칼라 묶음.
#[derive(Clone, Copy)]
pub(in crate::renderer::typeset) struct EndnoteEmitVars {
    pub(in crate::renderer::typeset) boundary_prev_endnote_had_vpos_rewind: bool,
    pub(in crate::renderer::typeset) endnote_has_vpos_rewind: bool,
    pub(in crate::renderer::typeset) continued_endnote_tail_before_new_note: bool,
    pub(in crate::renderer::typeset) default_nonzero_between_note_tail_candidate: bool,
    pub(in crate::renderer::typeset) default_question_group_title_tail: bool,
    pub(in crate::renderer::typeset) compact_endnote_separator_profile: bool,
    pub(in crate::renderer::typeset) prev_endnote_had_inline_object_vpos_overestimate: bool,
    pub(in crate::renderer::typeset) endnote_start: i32,
}

/// [#2026] 미주-당 프리앰블(구분자 방출·플래그 산출)의 흐름 캐리 — caller 로컬과 값 왕복.
#[derive(Clone, Copy)]
pub(in crate::renderer::typeset) struct EndnotePrepCarry {
    pub(in crate::renderer::typeset) vpos_offset: i32,
    pub(in crate::renderer::typeset) prev_en_bottom_vpos: Option<i32>,
    pub(in crate::renderer::typeset) prev_en_content_bottom_vpos: Option<i32>,
    pub(in crate::renderer::typeset) prev_endnote_had_vpos_rewind: bool,
    pub(in crate::renderer::typeset) emitted_endnote_separator: bool,
    pub(in crate::renderer::typeset) current_endnote_had_inline_object_vpos_overestimate: bool,
}
