//! 행 스캔의 불변 입력과 컷·누적 높이 결과.

/// [#2085] 표 행-스캔 분할점 캐리 (값 왕복). split_end_cut 은 move.
pub(in crate::renderer::typeset) struct BlockTableRowScan {
    pub(in crate::renderer::typeset) consumed: f64,
    pub(in crate::renderer::typeset) end_row: usize,
    pub(in crate::renderer::typeset) split_block_start: Option<usize>,
    pub(in crate::renderer::typeset) split_end_cut: Vec<usize>,
    pub(in crate::renderer::typeset) split_end_limit: f64,
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
    pub(in crate::renderer::typeset) start_row_height_override: Option<f64>,
}
