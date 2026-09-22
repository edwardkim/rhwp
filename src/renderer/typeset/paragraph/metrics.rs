//! 문단 구성 결과와 기존 흐름 높이 조회.
//!
//! fit 높이·전체 높이·줄 전진량은 서로 다른 의미를 유지한다. 원본 IR이나
//! 페이지 상태를 변경하지 않는다. 기존 환경 변수 기반 진단 출력은 유지한다.

use super::super::para_near_rowbreak_table;
use super::stored_lines::stored_ladder_encodes_spacing_before;
use crate::model::{paragraph::Paragraph, provenance::LayoutCompatibilityProfile};

/// 문단 format() 결과: 문단의 실제 렌더링 높이 정보
#[derive(Debug, Clone)]
pub(in crate::renderer::typeset) struct FormattedParagraph {
    /// Measured remaining inline space on the composed tail line. None means
    /// that this path has no reliable width result (not an implicit overflow).
    pub(in crate::renderer::typeset) tail_line_remaining_width: Option<f64>,
    /// frame이 실제로 재조판한 줄만 보존한다. Some이면 source 줄로 되돌아가지 않는다.
    pub(in crate::renderer::typeset) computed_host_lines:
        Option<Vec<crate::renderer::float_placement::ParagraphHostLine>>,
    /// 총 높이 (spacing 포함)
    pub(in crate::renderer::typeset) total_height: f64,
    /// 줄별 콘텐츠 높이 (line_height만)
    pub(in crate::renderer::typeset) line_heights: Vec<f64>,
    /// 줄별 줄간격 (line_spacing)
    pub(in crate::renderer::typeset) line_spacings: Vec<f64>,
    /// spacing_before
    pub(in crate::renderer::typeset) spacing_before: f64,
    /// spacing_after
    pub(in crate::renderer::typeset) spacing_after: f64,
    /// trailing line_spacing을 제외한 판단용 높이
    pub(in crate::renderer::typeset) height_for_fit: f64,
    /// `total_height` 에 섞어 넣은 TAC 표 바깥 여백(세로). 저장 사다리 지문처럼
    /// **생성기가 쓴 값과 대조하는** 계산에서는 이 몫을 도로 빼야 한다.
    pub(in crate::renderer::typeset) tac_outer_margin_v_px: f64,
}

impl FormattedParagraph {
    /// 특정 줄의 advance 높이 (콘텐츠 + 줄간격)
    ///
    /// Issue #3780: 연속 페이지 재배치에서 기록된 줄 인덱스가 새 레이아웃 줄 수를
    /// 넘는 off-by-one(len 31, index 31 실측 패닉)이 들어올 수 있다 — 존재하지
    /// 않는 줄의 advance 는 0.0 으로 방어해 렌더를 지속한다.
    #[inline]
    pub(in crate::renderer::typeset) fn line_advance(&self, line_idx: usize) -> f64 {
        if line_idx >= self.line_count() {
            return 0.0;
        }
        self.line_heights[line_idx] + self.line_spacings[line_idx]
    }

    /// 두 벡터가 함께 인덱싱되는 자리의 안전 상한 (구성은 zip 이라 보통 같다).
    #[inline]
    pub(in crate::renderer::typeset) fn line_count(&self) -> usize {
        self.line_heights.len().min(self.line_spacings.len())
    }

    /// 줄 범위의 advance 합계 (Issue #3780 — 범위를 실제 줄 수로 클램프)
    pub(in crate::renderer::typeset) fn line_advances_sum(
        &self,
        range: std::ops::Range<usize>,
    ) -> f64 {
        let end = range.end.min(self.line_count());
        let start = range.start.min(end);
        (start..end)
            .map(|i| self.line_heights[i] + self.line_spacings[i])
            .sum()
    }

    /// [#6753] `flow_advance_height` 가 실제로 트림한 것 중 **`spacing_before` 몫**(px).
    ///
    /// 트림 자체는 "저장 사다리가 이미 담았고 vpos-snap 이 좌표를 복원한다"는 전제 위에
    /// 서지만, **lazy 기준 역산**은 그 복원 이전의 sequential y 를 쓴다. 그래서 역산에만
    /// 이 값을 되돌려 준다(전진량은 그대로 둔다).
    pub(in crate::renderer::typeset) fn flow_trimmed_spacing_before(
        &self,
        para: &Paragraph,
        col_count: u16,
        allow_spacing_before_only: bool,
        ladder_dirty: bool,
        lazy_base: bool,
    ) -> f64 {
        let advance = self.flow_advance_height(
            para,
            col_count,
            allow_spacing_before_only,
            ladder_dirty,
            lazy_base,
        );
        if advance + 0.5 >= self.total_height {
            return 0.0; // 트림이 발동하지 않았다.
        }
        // `flow_advance_height` 의 `sb_trim` 과 같은 판정 — sa 만 트림된 경우는 0.
        let sa_trim = self.spacing_after > 0.5;
        let sb_trim =
            allow_spacing_before_only && self.spacing_before > 0.5 && !(lazy_base && !sa_trim);
        if sb_trim {
            self.spacing_before
        } else {
            0.0
        }
    }

    pub(in crate::renderer::typeset) fn flow_advance_height(
        &self,
        para: &Paragraph,
        col_count: u16,
        allow_spacing_before_only: bool,
        ladder_dirty: bool,
        lazy_base: bool,
    ) -> f64 {
        // [#6970] 다단에서도 **합성(reflow) lineseg 문단은 트림하지 않는다** — 아래
        // `#2279 ①` 이 단단 경로에 건 가드와 같은 이유다. 트림은 "저장 ladder 가 spacing 을
        // 이미 반영하고 vpos-snap 이 좌표를 복원한다"는 전제 위에 서는데, 합성 문단에는 그
        // ladder 가 없어 트림분이 흐름에서 그냥 소실된다.
        //
        // 저장 `LINE_SEG` 가 없는 2단 문서에서 그 소실이 쌓여 단 채움 회계가 무너진다 —
        // `synth_no_ls_square_wrap.hwp` 실측: 문단 151개에서 Σ 741.6px(문단당 6~10px)를
        // 덜 세고, 단 0 은 `usedHeight 708.9 ≤ 가용 718.1` 로 "아직 남았다"고 판단해 계속
        // 담는다. 실제로 담은 항목 합은 1000.4px 라 282px 초과이고, 넘친 내용이 다음 단으로
        // 가지 않고 그 자리에 그려진다(off-canvas 20 · overflow 16).
        //
        // 다단에서 `height_for_fit` 을 쓰는 본래 이유(#391: trailing_ls 인플레이션이 단을
        // 조기 종료시킨다)는 **저장 ladder 가 있는 문단**에 대한 것이므로 그대로 둔다.
        let has_authoritative_seg_for_multicolumn = para.line_segs.iter().any(|seg| {
            seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        });
        if col_count > 1 && has_authoritative_seg_for_multicolumn {
            return self.height_for_fit;
        }
        // [#2279 ①] spacing 트림은 **비합성(authoritative) 저장 lineseg** 문단에만.
        // 저장 ladder 가 spacing 을 이미 반영하고 vpos-snap 이 좌표를 복원하는
        // 전제의 트림이므로, 합성(reflow) lineseg 문단은 ladder 가 없어 트림하면
        // sb·ls 가 흐름에서 그냥 소실된다 (한글 fresh 는 가산 — 기계생성 결재
        // 문서 −1쪽 계열, DIAG_ADV 실측 문단당 4~33px).
        // [#2279 ①-2] dirty(합성 혼합) ladder 구간도 동일 — #2243 전방-스냅만
        // 허용되어 트림분이 복원되지 않으므로 full advance 를 쓴다
        // (36398700 pi6..9 구간 −60px 폐합, 한글 재저장 anchor 실측).
        let has_authoritative_seg = para.line_segs.iter().any(|seg| {
            seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        });
        // [#2279 ①-4] lazy-base(page_base 미확립) 사다리에서는 sb-형 트림의
        // 복원(스냅)이 성립하지 않는다 — sb-형만 차단, sa-형 트림은 유지
        // (36398700 pi31/35 −13.5px 미복원 실측 vs issue_1853 캡션 문서의
        // sa-형 트림은 정상 복원되어 전면 차단 시 +1쪽 과다 반증).
        let sa_trim = self.spacing_after > 0.5;
        let sb_trim =
            allow_spacing_before_only && self.spacing_before > 0.5 && !(lazy_base && !sa_trim);
        if std::env::var("RHWP_DIAG_LAZYBLK").is_ok()
            && allow_spacing_before_only
            && self.spacing_before > 0.5
            && lazy_base
            && !sa_trim
        {
            eprintln!(
                "DIAG_LAZYBLK sb={:.1} total={:.1} h4f={:.1}",
                self.spacing_before, self.total_height, self.height_for_fit
            );
        }
        if para.controls.is_empty()
            && has_authoritative_seg
            && !ladder_dirty
            && (sa_trim || sb_trim)
            && self.height_for_fit > 0.0
            && self.height_for_fit + 0.5 < self.total_height
        {
            return self.height_for_fit.min(self.total_height);
        }
        self.total_height
    }
}

/// 흐름 누적에 사용하는 독립 관측값. 전체 fit 높이와 섞지 않는다.
pub(in crate::renderer::typeset) struct ParagraphFlowHints {
    pub body_bottom_vpos: Option<i32>,
    pub trim_spacing_before_for_flow: bool,
    pub trimmed_sb_gate: f64,
}

pub(in crate::renderer::typeset) fn flow_hints(
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    para_idx: usize,
    profile: LayoutCompatibilityProfile,
    dpi: f64,
) -> ParagraphFlowHints {
    // fits: 문단 전체가 현재 공간에 들어가는가?
    // [Task #359] fit 판정은 height_for_fit (trailing_ls 제외) 으로,
    // 누적은 total_height (full) 로 분리. 각 항목별 trailing_ls 가
    // 누적에서 빠지면 N items 누적 시 N × trailing_ls 만큼 drift 발생
    // (k-water-rfp p3 case: 36 items × 평균 ~9px = ~311px LAYOUT_OVERFLOW).
    // trailing_ls 는 페이지 마지막 항목의 fit 판정에만 의미가 있음
    // (페이지 끝에는 다음 줄이 없으니 line_spacing 미적용).
    // [Task #1082] 본문 para 의 bottom offset vpos — 미주 vpos-delta 시드용.
    let body_bottom_vpos: Option<i32> = para.line_segs.last().map(|s| {
        s.vertical_pos
            .saturating_add(s.line_height)
            .saturating_add(s.line_spacing)
    });
    // HWP3-origin 변환본은 spacing_before 누적을 보존해야 dump-pages 요약과
    // 실제 한컴 줄 흐름이 유지된다(#1116).
    let trim_spacing_before_for_flow = !profile.hwp3_layout()
        && !para_near_rowbreak_table(paragraphs, para_idx)
        // [#5801] 저장 사다리가 문단 위 간격을 안 담았으면 트림의 전제가 깨진다 —
        // 트림하면 쪽 채움을 문단마다 sb 만큼 짧게 센다.
        && stored_ladder_encodes_spacing_before(
            paragraphs,
            para_idx,
            fmt.spacing_before,
            dpi,
        );
    // [#6753] 트림된 `sb` 되돌리기는 **저장 사다리가 권위인 네이티브 HWP5** 에 한정한다.
    //
    // HWPX 의 `vpos` 리셋은 writer-local 재시작일 수 있어 별도 기계(`#5801` 의 HWPX 전용
    // dirty 철회 · `#6063` · `hwpx_saved_reset_fragment_matches_current_flow`)가 따로 다룬다.
    // 전 포맷에 켠 판은 `samples/` 전수에서 `issue1880_*.hwpx` 2건을 악화시켰다
    // (5쪽 넘침 1 → 4, 최대 +82.65px). 같은 판에서 HWP5 문서는 3건 전부 개선이었다.
    let trimmed_sb_gate = if profile.hwp5_stored_pagination_layout() {
        1.0
    } else {
        0.0
    };

    ParagraphFlowHints {
        body_bottom_vpos,
        trim_spacing_before_for_flow,
        trimmed_sb_gate,
    }
}
