//! 행 스캔의 불변 입력과 컷·누적 높이 결과.

/// [#2085] 표 행-스캔 분할점 캐리 (값 왕복). split_end_cut 은 move.
pub(in crate::renderer::typeset) struct BlockTableRowScan {
    pub(in crate::renderer::typeset) consumed: f64,
    /// 독립 저장 프레임이 측정 행의 예약보다 추가로 소유한 물리 공간.
    /// 일반 컷의 실제 상자와 내용의 차이는 원래 측정 행의 추가 띠가 아니다.
    pub(in crate::renderer::typeset) source_frame_band: f64,
    /// 일반 컷을 수용할 때 확정한 빈 상자 소유. 추정 행의 추가 높이가 아니다.
    pub(in crate::renderer::typeset) plain_cut_owns_blank_band: bool,
    /// 일반 행 스캔에서 실제 수용한 마지막 완전 행의 색인과 높이.
    pub(in crate::renderer::typeset) accepted_end_row_box: Option<(usize, f64)>,
    /// 일반 스캔의 첫 행이 실제 예약한 물리 상자. 재스캔도 같은 결과를 넘긴다.
    pub(in crate::renderer::typeset) accepted_start_row_box: Option<(usize, f64)>,
    /// 마지막 미완료 일반 행이 실제 예약한 상자. 완전 행 원장과 구분한다.
    pub(in crate::renderer::typeset) accepted_partial_row_box: Option<(usize, f64)>,
    pub(in crate::renderer::typeset) end_row: usize,
    pub(in crate::renderer::typeset) split_block_start: Option<usize>,
    pub(in crate::renderer::typeset) split_end_cut: Vec<usize>,
    pub(in crate::renderer::typeset) split_end_limit: f64,
    /// 끝 블록 컷이 실제로 예약한 높이. 유닛 전진량과 구별한다.
    pub(in crate::renderer::typeset) split_block_physical_height: Option<f64>,
    /// 마지막으로 포함한 완전 행을 현재 조각의 남은 물리 높이로 압축할 때의
    /// 렌더 전용 높이 상한. continuation은 끝행의 full cut으로 빈 tail을
    /// 소비한 뒤 다음 행으로 전진한다.
    pub(in crate::renderer::typeset) end_row_height_override: Option<f64>,
}

/// [#2085] 표 행-스캔의 조각-스코프 읽기 스칼라 묶음.
#[derive(Clone, Copy)]
pub(in crate::renderer::typeset) struct BlockRowScanVars {
    pub(in crate::renderer::typeset) cursor_row: usize,
    pub(in crate::renderer::typeset) row_count: usize,
    pub(in crate::renderer::typeset) cs: f64,
    pub(in crate::renderer::typeset) can_intra_split: bool,
    pub(in crate::renderer::typeset) is_continuation: bool,
    pub(in crate::renderer::typeset) avail_for_rows: f64,
    pub(in crate::renderer::typeset) reflow_frame_avail_for_rows: Option<f64>,
    /// 각주 refit이 필요한 표에서는 병합 칸의 빈 쪽 끝 띠를 미리 소유하지 않는다.
    pub(in crate::renderer::typeset) has_table_footnotes: bool,
    pub(in crate::renderer::typeset) header_overhead: f64,
    pub(in crate::renderer::typeset) landscape_rowbreak_bleed: bool,
    pub(in crate::renderer::typeset) landscape_whole_row_tolerance: f64,
    pub(in crate::renderer::typeset) landscape_short_row_tolerance: f64,
    pub(in crate::renderer::typeset) landscape_short_row_max_height: f64,
    pub(in crate::renderer::typeset) strict_painted_bottom_fit: bool,
    pub(in crate::renderer::typeset) source_first_fragment_overflow_allowance: f64,
    /// 저장된 첫 조각 프레임이 가장 가깝게 소유하는 행 끝. 프레임 여유로
    /// whole-row를 수용할 때 이 끝을 넘지 않도록 제한한다.
    pub(in crate::renderer::typeset) source_first_fragment_row_end: Option<usize>,
    pub(in crate::renderer::typeset) start_cut_is_block: bool,
    pub(in crate::renderer::typeset) start_row_height_override: Option<f64>,
    pub(in crate::renderer::typeset) source_terminal_suffix_start: Option<usize>,
    /// 첫 조각에서 0행의 보호 rowspan 묶음을 0행 뒤에서 자른다(이월 게이트의 결정).
    pub(in crate::renderer::typeset) opening_block_cuts_after_first_row: bool,
    /// 저장 근거가 «내용은 이 조각, 남은 빈 띠는 다음 조각»으로 증언한 행.
    pub(in crate::renderer::typeset) source_carried_band_row: Option<usize>,
    /// 비끝 조각 상자를 닫을 때 `avail_for_rows` 밖에서 더 빼야 하는 표 바깥 아래 여백
    /// (선언 여백 − 이 조각 예산이 이미 예약한 몫).
    pub(in crate::renderer::typeset) unreserved_outer_bottom: f64,
}
